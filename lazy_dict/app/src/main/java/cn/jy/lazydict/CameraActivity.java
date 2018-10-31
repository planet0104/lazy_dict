package cn.jy.lazydict;

import android.annotation.SuppressLint;
import android.app.Activity;
import android.content.Context;
import android.graphics.Bitmap;
import android.graphics.Canvas;
import android.graphics.DashPathEffect;
import android.graphics.ImageFormat;
import android.graphics.Paint;
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

import java.util.ArrayList;
import java.util.List;

public class CameraActivity extends Activity{
    static final String TAG = CameraActivity.class.getSimpleName();

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
    List<RectF> allRects = new ArrayList<>();
    Bitmap mask;
    Bitmap capture;
    Camera camera;
    byte[] previewFrame = null;

    @SuppressLint("ClickableViewAccessibility")
    @Override
    protected void onCreate(@Nullable Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_camera);

        iv_capture = findViewById(R.id.iv_capture);
        iv_mask = findViewById(R.id.iv_mask);
        fl_capture = findViewById(R.id.fl_capture);
        iv_capture.setDrawingCacheEnabled(true);

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
        fl_capture.setVisibility(View.GONE);
        fl_preview.setVisibility(View.VISIBLE);
    }

    /**
     * 切换到截屏视图
     */
    private void changeStateCapture(){
        Log.d(TAG, "changeStateCapture");
        fl_capture.setVisibility(View.VISIBLE);
        fl_preview.setVisibility(View.GONE);
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
            } catch (Exception e) {
                e.printStackTrace();
                toast(this, e.getMessage());
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
        //使用post, 解决Camera: app passed NULL surface
        surface_view.post(new Runnable() {
            @Override
            public void run() {
                if(camera==null && iv_capture.getVisibility()==View.GONE){
                    initCamera();
                }
            }
        });
    }

    public static void toast(Context context, String s){
        Toast.makeText(context, s, Toast.LENGTH_SHORT).show();
    }

    private void addRect(float x, float y){
        Bitmap bitmap = iv_capture.getDrawingCache();
        try {
            long t = System.currentTimeMillis();
            RectF rect = Toolkit.getCharacterRect(bitmap, (int)x, (int)y);
            Log.d(TAG, "x="+x+" y="+y+" bitmap.width="+bitmap.getWidth()+" rect="+rect.toShortString()+" 耗时:"+(System.currentTimeMillis()-t)+"ms");

            //已经存在不再添加
            for(RectF ur: allRects){
                if(ur.left == rect.left && ur.top == rect.top && ur.right == rect.right && ur.bottom == rect.bottom){
                    toast(this, "已经添加!");
                    return;
                }
            }

            allRects.add(rect);

            if(mask == null){
                mask = Bitmap.createBitmap(bitmap.getWidth(), bitmap.getHeight(), Bitmap.Config.ARGB_8888);
            }

            Canvas canvas = new Canvas(mask);
            Paint paint = new Paint();
            paint.setStyle(Paint.Style.STROKE);
            paint.setStrokeWidth(Toolkit.dip2px(this,5));

            for(RectF ur: allRects){
                paint.setPathEffect(null);
                paint.setColor(0x7fff0000);
                canvas.drawRoundRect(rect, Toolkit.dip2px(this,5), Toolkit.dip2px(this,5), paint);
                paint.setColor(0x7fffffff);
                paint.setPathEffect(new DashPathEffect(new float[]{Toolkit.dip2px(this,10), Toolkit.dip2px(this,10)}, 0));
                canvas.drawRoundRect(ur, Toolkit.dip2px(this,5), Toolkit.dip2px(this,5), paint);
            }
            iv_mask.setImageBitmap(mask);
        } catch (Exception e) {
            e.printStackTrace();
            CameraActivity.toast(this, e.getMessage());
        }
    }
}
