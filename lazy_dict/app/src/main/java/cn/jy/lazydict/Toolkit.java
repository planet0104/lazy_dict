package cn.jy.lazydict;

import android.app.Activity;
import android.content.Context;
import android.graphics.Bitmap;
import android.graphics.Rect;
import android.graphics.RectF;
import android.os.Handler;
import android.os.Message;
import android.util.Log;
import android.view.View;
import android.view.ViewGroup;

import com.googlecode.tesseract.android.ResultIterator;
import com.googlecode.tesseract.android.TessBaseAPI;

import java.io.File;
import java.io.IOException;
import java.util.Arrays;

public class Toolkit {
    static final String TAG = Toolkit.class.getSimpleName();
    static {
        System.loadLibrary("lazy_dict");
    }

    /**
     * YUV420SP转Bitmap
     * @param data
     * @param width
     * @param height
     * @param cameraOrientation
     * @return
     */
    public static native Bitmap decodeYUV420SP(byte[] data, int width, int height, int cameraOrientation) throws Exception;

    /**
     * 根据坐标选择一个文字块
     * @param tg
     * @param x
     * @param y
     * @return
     * @throws Exception
     */
    public static native RectF getCharacterRect(ThresholdGray tg, int x, int y) throws Exception;

    /**
     * 计算阈值和灰度图
     * @param bitmap
     * @return
     * @throws Exception
     */
    public static native ThresholdGray calcThreshold(Bitmap bitmap) throws Exception;

    /**
     * 二值化
     * @param bitmap
     * @return
     * @throws Exception
     */
    public static native void binary(Bitmap bitmap) throws Exception;

    public static native RectF[] split(Bitmap bitmap) throws Exception;

    public static native String[] jiebaCut(String text) throws Exception;

    public static Rect getLocationInParent(View view, ViewGroup parent){
        int[] loc = new int[2];
        view.getLocationInWindow(loc);
        int[] locP = new int[2];
        parent.getLocationInWindow(locP);
        int left = loc[0];
        int top = loc[1]-locP[1];
        return new Rect(left, top, left+view.getMeasuredWidth(), top+view.getMeasuredHeight());
    }

    public static final int MSG_TESS_RECOGNIZE_ERROR = 0;
    public static final int MSG_TESS_RECOGNIZE_COMPLETE = 1;
    public static final int MSG_TESS_RECOGNIZE_LINE = 2;
    public static final int MSG_TESS_RECOGNIZE_START = 3;

    public static final int MSG_TESS_INIT_SUCCESS = 10;
    public static final int MSG_TESS_INIT_ERROR = 11;

    /**
     * 识别图片文字
     * @param tessBaseAPI
     * @param bitmap
     * @param handler
     */
    public static void tessRecognize(final TessBaseAPI tessBaseAPI, final Bitmap bitmap, final android.os.Handler handler){
        Thread thread = new Thread(new Runnable() {
            @Override
            public void run() {
                try {
                    RectF[] splitRect = Toolkit.split(bitmap);
                    //识别
                    tessBaseAPI.setPageSegMode(TessBaseAPI.PageSegMode.PSM_SINGLE_LINE);

                    Message msg = Message.obtain();
                    msg.what = MSG_TESS_RECOGNIZE_START;
                    handler.sendMessage(msg);

                    for(RectF lineRect : splitRect){
                        if(lineRect.height()<=10 || lineRect.width()<=10){
                            //宽高小于10像素的忽略
                            continue;
                        }
                        long t = System.currentTimeMillis();
                        Bitmap rb = Bitmap.createBitmap(bitmap, (int)lineRect.left, (int)lineRect.top, (int)(lineRect.right-lineRect.left), (int)(lineRect.bottom-lineRect.top));
                        Toolkit.binary(rb);
                        tessBaseAPI.setImage(rb);
                        String line = "";
                        String _text = tessBaseAPI.getUTF8Text();
                        //------------------------------------------------------------------
                        ResultIterator resultIterator = tessBaseAPI.getResultIterator();
                        int level = TessBaseAPI.PageIteratorLevel.RIL_SYMBOL;
                        do{
                            //提取准确率高于80%的字符
                            if(resultIterator.confidence(level)>80.0){
                                String ts = resultIterator.getUTF8Text(level);
                                char[] chars = ts.toCharArray();
                                for(char ch: chars){
                                    if(Toolkit.isChinese(ch)){
                                        line += ch;
                                    }
                                }
                            }
                        }while(resultIterator.next(level));
                        resultIterator.delete();
                        //------------------------------------------------------------------
                        Log.d(TAG, "耗时:"+(System.currentTimeMillis()-t)+"毫秒 text="+line);
                        long ft = System.currentTimeMillis();
                        String[] words = Toolkit.jiebaCut(line);
                        Log.d(TAG, "分词结果:"+Arrays.toString(words)+" 耗时:"+(System.currentTimeMillis()-ft)+"ms");
                        //返回一行的分词结果
                        msg = Message.obtain();
                        msg.what = MSG_TESS_RECOGNIZE_LINE;
                        msg.obj = words;
                        handler.sendMessage(msg);
                    }

                    //识别完成
                    msg = Message.obtain();
                    msg.what = MSG_TESS_RECOGNIZE_COMPLETE;
                    handler.sendMessage(msg);
                } catch (Exception e) {
                    e.printStackTrace();
                    //识别出错
                    Message msg = Message.obtain();
                    msg.what = MSG_TESS_RECOGNIZE_ERROR;
                    msg.obj = e;
                    handler.sendMessage(msg);
                }
            }
        });
        thread.start();
    }

    public static void initTessTwo(final Activity activity, final TessBaseAPI tessBaseAPI, final Handler handler){
        //将tessdata文件夹解压到files文件夹
        new Thread(new Runnable() {
            @Override
            public void run() {
                boolean success = false;
                try {
                    File tessDataDir = new File(activity.getFilesDir(), "tessdata");
                    if(!tessDataDir.exists()){
                        if(FileUtils.unpackZip(activity.getAssets().open("tessdata.zip"), activity.getFilesDir(), null)){
                            Log.d(TAG, "tessdata解压成功");
                            success = true;
                        }else{
                            Log.e(TAG, "tessdata解压失败");
                        }
                    }else{
                        success = true;
                        Log.e(TAG, "tessdata已经存在");
                    }
                } catch (IOException e) {
                    Log.e(TAG, "tessdata文件夹读取失败!");
                    e.printStackTrace();
                }
                if(success){
                    //初始化 TessBaseAPI
                    boolean tessInit;
                    TessBaseAPI baseAPI;
                    if(tessBaseAPI == null){
                        baseAPI = new TessBaseAPI();
                        Log.d(TAG, "版本:"+baseAPI.getVersion());
                        tessInit = baseAPI.init(activity.getFilesDir().getAbsolutePath(), "chi_sim");
                    }else{
                        baseAPI = tessBaseAPI;
                        tessInit = true;
                    }

                    Message msg = Message.obtain();
                    if(!tessInit){
                        msg.what = MSG_TESS_INIT_ERROR;
                        handler.sendMessage(msg);
                    }else{
                        msg.what = MSG_TESS_INIT_SUCCESS;
                        msg.obj = baseAPI;
                        handler.sendMessage(msg);
                    }
                }else{
                    Message msg = Message.obtain();
                    msg.what = MSG_TESS_INIT_ERROR;
                    handler.sendMessage(msg);
                }
            }
        }).start();
    }

    /**
     * 输入的字符是否是汉字
     * @param a char
     * @return boolean
     */
    public static boolean isChinese(char a) {
        int v = (int)a;
        return (v >=19968 && v <= 171941);
    }

    public static int dip2px(Context context, float dipValue){
        final float scale = context.getResources().getDisplayMetrics().density;
        return (int)(dipValue * scale + 0.5f);
    }
}
