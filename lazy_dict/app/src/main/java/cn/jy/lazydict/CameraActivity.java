package cn.jy.lazydict;

import android.annotation.SuppressLint;
import android.app.Activity;
import android.app.AlertDialog;
import android.content.Context;
import android.content.DialogInterface;
import android.graphics.Bitmap;
import android.graphics.ImageFormat;
import android.graphics.Rect;
import android.graphics.RectF;
import android.hardware.Camera;
import android.os.Bundle;
import android.support.annotation.Nullable;
import android.util.Log;
import android.view.MotionEvent;
import android.view.SurfaceHolder;
import android.view.SurfaceView;
import android.view.View;
import android.widget.FrameLayout;
import android.widget.ImageButton;
import android.widget.ImageView;
import android.widget.Toast;

import com.googlecode.tesseract.android.ResultIterator;
import com.googlecode.tesseract.android.TessBaseAPI;

import java.io.File;
import java.io.IOException;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

public class CameraActivity extends Activity{
    static final String TAG = CameraActivity.class.getSimpleName();
    ImageView iv_test;

    //--------- 预览相关 ------------
    FrameLayout fl_preview;
    SurfaceView surface_view;
    SurfaceHolder surface_holder;
    ImageButton btn_capture;

    //------ 截图相关 -------------
    FrameLayout fl_capture;
    ImageView iv_capture;
    ImageButton btn_preview;

    //所有用户选择的Rect
    List<RectF> allRect = new ArrayList<>();
    Bitmap capture;
    Bitmap drawCache;
    Camera camera;
    byte[] previewFrame = null;
    boolean isPreview = true;

    static TessBaseAPI tessBaseAPI;

    @SuppressLint("ClickableViewAccessibility")
    @Override
    protected void onCreate(@Nullable Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_camera);
        iv_test = findViewById(R.id.iv_test);
        iv_capture = findViewById(R.id.iv_capture);
        fl_capture = findViewById(R.id.fl_capture);
        iv_capture.setDrawingCacheEnabled(true);

        fl_preview = findViewById(R.id.fl_preview);
        btn_capture = findViewById(R.id.btn_capture);
        btn_preview = findViewById(R.id.btn_preview);
        surface_view = findViewById(R.id.surface_view);
        surface_holder = surface_view.getHolder();
        btn_capture.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                capture();
            }
        });

        btn_preview.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                if(camera == null) initCamera();
            }
        });

        surface_view.setOnTouchListener(new View.OnTouchListener() {
            @Override
            public boolean onTouch(View v, MotionEvent event) {
                CameraUtils.focusOnTouch(camera, surface_view, (int)event.getX(), (int)event.getY(), new Camera.AutoFocusCallback() {
                    @Override
                    public void onAutoFocus(boolean success, Camera c) {
                        //重新开始自动对焦
                        surface_view.postDelayed(new Runnable() {
                            @Override
                            public void run() {
                                CameraUtils.setContinuallyAutoFocus(camera);
                            }
                        }, 1000);
                    }
                });
                return false;
            }
        });
    }

    /**
     * 切换到预览视图
     */
    private void changeStatePreview(){
        Log.d(TAG, "changeStatePreview");
        //清空数据
        allRect.clear();
        drawCache = null;
        iv_capture.destroyDrawingCache();
        capture = null;
        fl_capture.setVisibility(View.GONE);
        fl_preview.setVisibility(View.VISIBLE);
        isPreview = true;
    }

    /**
     * 切换到截屏视图
     */
    private void changeStateCapture(){
        Log.d(TAG, "changeStateCapture");
        fl_capture.setVisibility(View.VISIBLE);
        fl_preview.setVisibility(View.GONE);
        isPreview = false;
    }

    private void capture(){
        Log.d(TAG, "capture.");
        if(previewFrame!=null && camera!=null){
            android.hardware.Camera.CameraInfo info =
                    new android.hardware.Camera.CameraInfo();
            android.hardware.Camera.getCameraInfo(Camera.CameraInfo.CAMERA_FACING_BACK, info);

            Camera.Size size = camera.getParameters().getPreviewSize();
            long t = System.currentTimeMillis();
            try {
                capture = Toolkit.decodeYUV420SP(previewFrame, size.width, size.height, info.orientation);
                iv_capture.setImageBitmap(capture);
                Log.d(TAG, "转换耗时:"+(System.currentTimeMillis()-t)+"ms");
                releaseCamera();//释放相机
                changeStateCapture();//切换到截图状态
                //识别
                tessBaseAPI.setPageSegMode(TessBaseAPI.PageSegMode.PSM_SINGLE_LINE);
                final View v = findViewById(R.id.v_area);
                v.post(new Runnable() {
                    @Override
                    public void run() {
                        Rect visibleRect = Toolkit.getLocationInParent(v, fl_preview);
                        Bitmap rect = Bitmap.createBitmap(iv_capture.getDrawingCache(), visibleRect.left, visibleRect.top, visibleRect.width(), visibleRect.height());
                        try {
                            RectF[] splitRect = Toolkit.split(rect);
                            List<String> resultArray = new ArrayList<>();
                            for(RectF lineRect : splitRect){
                                if(lineRect.height()<=10 || lineRect.width()<=10){
                                    //宽高小于10像素的忽略
                                    continue;
                                }
                                long t = System.currentTimeMillis();
                                Bitmap rb = Bitmap.createBitmap(rect, (int)lineRect.left, (int)lineRect.top, (int)(lineRect.right-lineRect.left), (int)(lineRect.bottom-lineRect.top));
                                iv_test.setImageBitmap(rb);
                                try{ Toolkit.binary(rb); } catch (Exception e){ }
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
                                String[] jieba = Toolkit.jiebaCut(line);
                                Log.d(TAG, "分词结果:"+Arrays.toString(jieba)+" 耗时:"+(System.currentTimeMillis()-ft)+"ms");
                            }
                        } catch (Exception e) {
                            e.printStackTrace();
                        }
                        iv_test.setImageBitmap(rect);
                    }
                });
            } catch (Exception e) {
                e.printStackTrace();
                showMessageDialog(e.getMessage(), false);
            }
        }
    }

    /**
     * 关闭相机
     */
    private void releaseCamera(){
        Log.d(TAG, "releaseCamera.");
        if (camera != null) {
            camera.setPreviewCallback(null);
            camera.stopPreview();
            camera.lock();
            camera.release();
            camera = null;
        }
    }

    private void init_tess_two(){
        //将tessdata文件夹解压到files文件夹
        new Thread(new Runnable() {
            @Override
            public void run() {
                boolean success = false;
                try {
                    File tessDataDir = new File(getFilesDir(), "tessdata");
                    if(!tessDataDir.exists()){
                        if(FileUtils.unpackZip(getAssets().open("tessdata.zip"), getFilesDir(), null)){
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
                    if(tessBaseAPI == null){
                        tessBaseAPI = new TessBaseAPI();
                        Log.d(TAG, "版本:"+tessBaseAPI.getVersion());
                        tessInit = tessBaseAPI.init(getFilesDir().getAbsolutePath(), "chi_sim");
                    }else{
                        tessInit = true;
                    }
                    final boolean tessInitFinal = tessInit;
                    runOnUiThread(new Runnable() {
                        @Override
                        public void run() {
                            if(tessInitFinal){
                                //启动相机
                                surface_view.setVisibility(View.VISIBLE);
                                surface_view.post(new Runnable() {
                                    @Override
                                    public void run() {
                                        initCamera();
                                    }
                                });
                            }else{
                                Log.e(TAG, "Tess初始化失败!");
                            }
                        }
                    });
                }
            }
        }).start();
    }

    /**
     * 启动相机
     */
    private void initCamera(){
        Log.d(TAG, "initCamera.");
        camera = Camera.open(Camera.CameraInfo.CAMERA_FACING_BACK);//默认开启后置
        if(camera!=null){
            try{
                CameraUtils.setCameraDisplayOrientation(this, Camera.CameraInfo.CAMERA_FACING_BACK, camera);
                Camera.Parameters parameters = camera.getParameters();
                parameters.setPreviewFormat(ImageFormat.NV21);
                CameraUtils.setContinuallyAutoFocus(camera);
                camera.setPreviewDisplay(surface_holder);
                camera.setPreviewCallback(new Camera.PreviewCallback() {
                    @Override
                    public void onPreviewFrame(byte[] data, Camera camera) {
                        previewFrame = data;
                    }
                });
                changeStatePreview();
                camera.startPreview();
            }catch (Exception e) {
                releaseCamera();
                e.printStackTrace();
                showMessageDialog(e.getMessage(), true);
            }
        }
    }

    @Override
    protected void onStop() {
        releaseCamera();
        super.onStop();
    }

    @Override
    protected void onResume() {
        super.onResume();
        surface_view.post(new Runnable() {
            @Override
            public void run() {
                if(camera==null && isPreview){
                    //initCamera();
                    init_tess_two();
                }
            }
        });
    }

    public static void toast(Context context, String s){
        Toast.makeText(context, s, Toast.LENGTH_SHORT).show();
    }

    private void showMessageDialog(String errorMsg, final boolean isError){
        AlertDialog.Builder builder = new AlertDialog.Builder(this);
        builder.setMessage(errorMsg);
        builder.setTitle("程序错误");
        builder.setPositiveButton("确定", new DialogInterface.OnClickListener() {
            @Override
            public void onClick(DialogInterface dialog, int which) {
                dialog.dismiss();
                if(isError)
                finish();
            }
        });
        AlertDialog dialog = builder.create();
        dialog.show();
    }
}
