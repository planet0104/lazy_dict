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
import android.webkit.WebView;

import com.baidu.ocr.sdk.OCR;
import com.baidu.ocr.sdk.OnResultListener;
import com.baidu.ocr.sdk.exception.OCRError;
import com.baidu.ocr.sdk.model.AccessToken;
import com.baidu.ocr.sdk.model.GeneralBasicParams;
import com.baidu.ocr.sdk.model.GeneralResult;
import com.baidu.ocr.sdk.model.WordSimple;
import com.googlecode.tesseract.android.ResultIterator;
import com.googlecode.tesseract.android.TessBaseAPI;

import org.jsoup.Jsoup;
import org.jsoup.nodes.Document;
import org.jsoup.nodes.TextNode;
import org.jsoup.select.Elements;

import java.io.File;
import java.io.FileOutputStream;
import java.util.Arrays;

import okhttp3.OkHttpClient;
import okhttp3.Request;
import okhttp3.Response;

public class Toolkit {
    static final String TAG = Toolkit.class.getSimpleName();
    static {
        System.loadLibrary("lazy_dict");
    }

    interface  DecodeCB{
        void success(Bitmap bitmap);
        void error(Throwable t);
    }

    /**
     * YUV420SP转Bitmap
     * @param data
     * @param width
     * @param height
     * @param cameraOrientation
     * @return
     */
    public static native Bitmap decodeYUV420SP(Context activity, byte[] data, int width, int height, int cameraOrientation) throws Exception;

    /**
     * yuv420转bitmap
     * @param activity
     * @param data
     * @param width
     * @param height
     * @param cameraOrientation
     * @param decodeCB
     */
    public static void decodeYUV420SP(Context activity, byte[] data, int width, int height, int cameraOrientation, DecodeCB decodeCB){
        try{
            decodeCB.success(decodeYUV420SP(activity, data, width, height, cameraOrientation));
        }catch (Throwable e){
            e.printStackTrace();
            decodeCB.error(e);
        }
    }


    /**
     * 二值化
     * @param bitmap
     * @return
     * @throws Exception
     */
    public static native void binary(Context activity, Bitmap bitmap) throws Exception;

    /**
     * 分割图片行
     * @param bitmap
     * @return
     * @throws Exception
     */
    public static native RectF[] split(Context activity, Bitmap bitmap) throws Exception;

    /**
     * 结巴分词
     * @param text
     * @return
     * @throws Exception
     */
    public static native String[] jiebaCut(Context activity, String text) throws Exception;

    /**
     * 汉字转拼音
     * @param text
     * @return
     * @throws Exception
     */
    public static native String[] pinyin(Context activity, String text) throws Exception;

    /**
     * 查询汉字释义
     * @param word
     * @return
     * @throws Exception
     */
    public static native Word search(Activity activity, String word) throws Exception;

    /**
     * 检查百度百科是否存在此词条
     * @param word
     * @param handler
     */
    public static void checkBaiKe(final String word, final Handler handler){
        new Thread(new Runnable() {
            @Override
            public void run() {
                Message msg = Message.obtain();
                msg.what = MSG_BAIKE_SEARCH_RESULT;
                String[] result = new String[2];
                msg.obj = null;
                String url = "https://baike.baidu.com/item/"+word;
                result[0] = url;
                try{
                    OkHttpClient client = new OkHttpClient();
                    Request request = new Request.Builder()
                            .url("https://baike.baidu.com/item/"+word)
                            .build();
                    Response response = client.newCall(request).execute();
                    if(response.code() == 200){
                        String doc = response.body().string();
                        if(doc != null){
                            Document document = Jsoup.parse(doc);
                            Elements elements = document.select("div.lemma-summary");
                            if(elements!=null && elements.size()>0){
                                //替换所有A标签
                                Elements alist = elements.select("a");
                                for(org.jsoup.nodes.Element e : alist){
                                    e.replaceWith(new TextNode(e.text()));
                                }
                                result[1] = elements.html()+"<a href=\""+url+"\">查看<i><b>百度百科</b></i>解释</a>";
                                msg.obj = result;
                            }
                        }
                    }
                }catch (Throwable e){
                    e.printStackTrace();
                }
                handler.sendMessage(msg);
            }
        }).start();
    }

    /**
     * 查询词语意思
     * @param words
     * @return
     * @throws Exception
     */
    public static native String searchWords(Activity activity, String words) throws Exception;

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

    public static final int MSG_BAIKE_SEARCH_RESULT = 20;
    public static final int MSG_SPLIT_SEARCH = 30;

    /**
     * 识别图片文字
     * @param activity
     * @param bitmap
     * @param handler
     */
    public static void tessRecognize(final Activity activity, final Bitmap bitmap, final android.os.Handler handler){
        Thread thread = new Thread(new Runnable() {
            @Override
            public void run() {
                try {
                    RectF[] splitRect = Toolkit.split(activity, bitmap);
                    //识别
                    MyApp.getTessApi(activity).setPageSegMode(TessBaseAPI.PageSegMode.PSM_SINGLE_LINE);

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
                        Toolkit.binary(activity, rb);
                        MyApp.getTessApi(activity).setImage(rb);
                        String line = "";
                        String _text = MyApp.getTessApi(activity).getUTF8Text();
                        //------------------------------------------------------------------
                        ResultIterator resultIterator = MyApp.getTessApi(activity).getResultIterator();
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
                        String[] words = Toolkit.jiebaCut(activity, line);
                        Log.d(TAG, "分词结果:"+Arrays.toString(words)+" 耗时:"+(System.currentTimeMillis()-ft)+"ms");
                        //返回一行的分词结果
                        msg = Message.obtain();
                        msg.what = MSG_TESS_RECOGNIZE_LINE;
                        msg.arg1 = -1;
                        msg.obj = words;
                        handler.sendMessage(msg);
                    }

                    //识别完成
                    msg = Message.obtain();
                    msg.what = MSG_TESS_RECOGNIZE_COMPLETE;
                    handler.sendMessage(msg);
                } catch (Throwable e) {
                    e.printStackTrace();
                    //识别出错
                    Message msg = Message.obtain();
                    msg.what = MSG_TESS_RECOGNIZE_ERROR;
                    msg.obj = new Exception("识别出错");
                    handler.sendMessage(msg);
                }
            }
        });
        thread.start();
    }

    public static void initTessTwo(final Activity activity, final Handler handler){
        //将tessdata文件夹解压到files文件夹
        new Thread(new Runnable() {
            @Override
            public void run() {
                Message msg = Message.obtain();
                if(MyApp.getTessApi(activity)!=null){
                    msg.what = MSG_TESS_INIT_SUCCESS;
                }else{
                    msg.what = MSG_TESS_INIT_ERROR;
                }
                if(handler!=null)
                handler.sendMessage(msg);
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
    public static final int MSG_UP_SEARCH_RESULT = 30;
    public static void upSearch(final Activity activity, Bitmap bitmap, final Handler handler){
        final File tmp = new File(activity.getCacheDir(), "cap.jpg");
        try{
            bitmap.compress(Bitmap.CompressFormat.JPEG, 100, new FileOutputStream(tmp));
        }catch (Throwable e){
            e.printStackTrace();
            //识别出错
            Message msg = Message.obtain();
            msg.what = MSG_TESS_RECOGNIZE_ERROR;
            msg.obj = e;
            handler.sendMessage(msg);
            return;
        }
        if(!tmp.exists()){
            //识别出错
            Message msg = Message.obtain();
            msg.what = MSG_TESS_RECOGNIZE_ERROR;
            msg.obj = new Exception("图片压缩失败!");
            handler.sendMessage(msg);
            return;
        }
        final Runnable r = new Runnable() {
            @Override
            public void run() {
                Log.d(TAG, "文字识别Run.");
                // 通用文字识别参数设置
                GeneralBasicParams param = new GeneralBasicParams();
                param.setImageFile(tmp);

                // 调用通用文字识别服务
                OCR.getInstance(activity).recognizeGeneralBasic(param, new OnResultListener<GeneralResult>() {
                    @Override
                    public void onResult(GeneralResult result) {
                        // 调用成功，返回GeneralResult对象
                        for (WordSimple wordSimple : result.getWordList()) {
                            try{
                                String w = wordSimple.getWords();
                                if(w==null) continue;
                                StringBuilder sb = new StringBuilder();
                                for(char c : w.toCharArray()){
                                    if(Toolkit.isChinese(c)){
                                        sb.append(c);
                                    }
                                }
                                String[] words = Toolkit.jiebaCut(activity, sb.toString());
                                Log.d(TAG, "分词结果:"+Arrays.toString(words));
                                //返回一行的分词结果
                                Message msg = Message.obtain();
                                msg.what = MSG_TESS_RECOGNIZE_LINE;
                                msg.arg1 = -1;
                                msg.obj = words;
                                handler.sendMessage(msg);
                            }catch (Throwable e){e.printStackTrace();}
                        }
                        //识别完成
                        Log.d(TAG, "识别完成！！！！！！");
                        Message msg = Message.obtain();
                        msg.what = MSG_TESS_RECOGNIZE_COMPLETE;
                        handler.sendMessage(msg);
                    }
                    @Override
                    public void onError(OCRError error) {
                        error.printStackTrace();
                        // 调用失败，返回OCRError对象
                        //识别出错
                        Message msg = Message.obtain();
                        msg.what = MSG_TESS_RECOGNIZE_ERROR;
                        msg.obj = new Exception(error.getMessage());
                        handler.sendMessage(msg);
                    }
                });
            }
        };

        Message msg = Message.obtain();
        msg.what = MSG_TESS_RECOGNIZE_START;
        handler.sendMessage(msg);

        if(OCR.getInstance(activity).getAccessToken() == null || OCR.getInstance(activity).getAccessToken().hasExpired()){
            OCR.getInstance(activity).initAccessToken(new OnResultListener<AccessToken>() {
                @Override
                public void onResult(AccessToken result) {
                    r.run();
                }
                @Override
                public void onError(OCRError error) {
                    error.printStackTrace();
                    // 调用失败，返回OCRError子类SDKError对象
                    //识别出错
                    Message msg = Message.obtain();
                    msg.what = MSG_TESS_RECOGNIZE_ERROR;
                    msg.obj = new Exception(error.getMessage());
                    handler.sendMessage(msg);
                }
            }, activity.getApplicationContext());
        }else{
            r.run();
        }
    }

    public static  void loadText(final WebView wv, String text){
        String html = "<!DOCTYPE HTML>\n" +
                "<html>\n" +
                "<body style=\"color:#4f5d73\">\n" +
                "-" +
                "</body>\n" +
                "</html>";
        html = html.replace("-", text);
        final String finalHtml = html;
        wv.post(new Runnable() {
            @Override
            public void run() {
                wv.loadDataWithBaseURL("", finalHtml, "text/html", "utf-8", null);
            }
        });
    }
}
