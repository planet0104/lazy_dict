package cn.jy.lazydict;

import android.annotation.SuppressLint;
import android.app.Activity;
import android.app.AlertDialog;
import android.content.Context;
import android.content.DialogInterface;
import android.graphics.Bitmap;
import android.graphics.Canvas;
import android.graphics.DashPathEffect;
import android.graphics.ImageFormat;
import android.graphics.Paint;
import android.graphics.PorterDuff;
import android.graphics.PorterDuffXfermode;
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

import com.googlecode.tesseract.android.TessBaseAPI;

import java.io.File;
import java.io.IOException;
import java.util.ArrayList;
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
    ImageView iv_mask;
    ImageView iv_capture;
    ImageButton btn_preview;

    //所有用户选择的Rect
    List<RectF> allRect = new ArrayList<>();
    Bitmap mask;
    Bitmap capture;
    Bitmap drawCache;
    ThresholdGray tg;
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
        iv_mask = findViewById(R.id.iv_mask);
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

        iv_mask.setOnTouchListener(new View.OnTouchListener() {
            @Override
            public boolean onTouch(View v, MotionEvent event) {
                switch (event.getAction()){
                    case MotionEvent.ACTION_DOWN:
                        addRect(event.getX(), event.getY());
                        break;
                }
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
        iv_mask.setImageBitmap(null);
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
                Toolkit.binary(capture);
                iv_capture.setImageBitmap(capture);
                Log.d(TAG, "转换耗时:"+(System.currentTimeMillis()-t)+"ms");
                releaseCamera();//释放相机
                changeStateCapture();//切换到截图状态


                //识别
                t = System.currentTimeMillis();
                tessBaseAPI.setPageSegMode();
                tessBaseAPI.setImage(capture);
                final String text = tessBaseAPI.getUTF8Text();
                Log.d(TAG, "识别文字耗时:"+(System.currentTimeMillis()-t)+"毫秒 text="+text);

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
                                initCamera();
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

    private void addRect(float x, float y){
        try {
            long t = System.currentTimeMillis();
            if(drawCache == null){
                drawCache =iv_capture.getDrawingCache();
                Log.d(TAG, "开始调用calcThreshold...");
                tg = Toolkit.calcThreshold(drawCache);
                Log.d(TAG, "calcThreshold调用完毕...  tg="+tg.toString());
            }
//            int[] colors = new int[drawCache.getWidth()*drawCache.getHeight()];
//            for(int i=0; i<tg.grays.length; i++){
//                colors[i] = Color.argb(255, 0xFF&tg.grays[i], 0xFF&tg.grays[i], 0xFF&tg.grays[i]);
//                //Log.d(TAG, "tg.grays[i]="+(0xFF&tg.grays[i]));
//                //colors[i] = Color.RED;
//            }
//            Bitmap b = Bitmap.createBitmap(colors, 0, drawCache.getWidth(), drawCache.getWidth(), drawCache.getHeight(), Bitmap.Config.ARGB_8888);
//            iv_test.setImageBitmap(b);
            RectF rect = Toolkit.getCharacterRect(tg, (int)x, (int)y);
            Log.d(TAG, "x="+x+" y="+y+" bitmap.width="+drawCache.getWidth()+" rect="+rect.toShortString()+" 耗时:"+(System.currentTimeMillis()-t)+"ms");

            //已经存在不再添加
            for(RectF ur: allRect){
                if(ur.left == rect.left && ur.top == rect.top && ur.right == rect.right && ur.bottom == rect.bottom){
                    toast(this, "已经添加!");
                    return;
                }
            }

            allRect.add(rect);

            if(mask == null){
                mask = Bitmap.createBitmap(drawCache.getWidth(), drawCache.getHeight(), Bitmap.Config.ARGB_8888);
            }

            Canvas canvas = new Canvas(mask);
            //清空画布
            Paint paint = new Paint();
            paint.setXfermode(new PorterDuffXfermode(PorterDuff.Mode.CLEAR));
            canvas.drawPaint(paint);

            //先绘制红色半透明
            paint = new Paint();
            paint.setStyle(Paint.Style.STROKE);
            int strokeWidth = Toolkit.dip2px(this,3);
            paint.setStrokeWidth(strokeWidth);

            for(RectF ur: allRect){
                paint.setPathEffect(null);
                paint.setColor(0x7fff0000);
                canvas.drawRoundRect(ur, strokeWidth, strokeWidth, paint);
                paint.setColor(0x7fffffff);
                paint.setPathEffect(new DashPathEffect(new float[]{strokeWidth*2, strokeWidth*2}, 0));
                canvas.drawRoundRect(ur, strokeWidth, strokeWidth, paint);
            }
            iv_mask.setImageBitmap(mask);
        } catch (Exception e) {
            e.printStackTrace();
            showMessageDialog(e.getMessage(), false);
        }
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
