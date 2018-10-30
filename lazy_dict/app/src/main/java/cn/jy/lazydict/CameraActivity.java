package cn.jy.lazydict;

import android.app.Activity;
import android.graphics.Bitmap;
import android.graphics.ImageFormat;
import android.hardware.Camera;
import android.os.Bundle;
import android.support.annotation.Nullable;
import android.util.Log;
import android.view.SurfaceHolder;
import android.view.SurfaceView;
import android.view.View;
import android.widget.ImageButton;
import android.widget.ImageView;

public class CameraActivity extends Activity implements SurfaceHolder.Callback{
    static final String TAG = CameraActivity.class.getSimpleName();
    SurfaceView surface_view;
    SurfaceHolder surface_holder;
    Camera camera;
    CaptureView iv_capture;
    ImageButton btn_capture;
    ImageButton btn_preview;
    public static ImageView iv_test;
    Bitmap captureBitmap;
    byte[] previewFrame = null;

    @Override
    protected void onCreate(@Nullable Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_camera);
        iv_capture = findViewById(R.id.iv_capture);
        btn_capture = findViewById(R.id.btn_capture);
        btn_preview = findViewById(R.id.btn_preview);
        surface_view = findViewById(R.id.surface_view);
        iv_test = findViewById(R.id.iv_test);
        surface_holder = surface_view.getHolder();
        surface_holder.addCallback(this);

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
    }

    private void capture(){
        Log.d(TAG, "capture.");
        if(previewFrame!=null && camera!=null){
            android.hardware.Camera.CameraInfo info =
                    new android.hardware.Camera.CameraInfo();
            android.hardware.Camera.getCameraInfo(Camera.CameraInfo.CAMERA_FACING_BACK, info);
            
            Camera.Size size = camera.getParameters().getPreviewSize();
            int[] colors = RustApp.decodeYUV420SP(previewFrame, size.width, size.height, info.orientation);
            iv_capture.setImageBitmap(Bitmap.createBitmap(colors,0, size.width, size.width, size.height, Bitmap.Config.ARGB_8888));
            btn_capture.setVisibility(View.GONE);
            iv_capture.setVisibility(View.VISIBLE);
            btn_preview.setVisibility(View.VISIBLE);
            releaseCamera();//释放相机
        }
    }

    private void releaseCamera(){
        Log.d(TAG, "releaseCamera.");
        if (camera != null) {
            surface_holder.removeCallback(this);
            camera.setPreviewCallback(null);
            camera.stopPreview();
            camera.lock();
            camera.release();
            camera = null;
        }
    }

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
                camera.startPreview();
                btn_preview.setVisibility(View.GONE);
                iv_capture.setVisibility(View.GONE);
                btn_capture.setVisibility(View.VISIBLE);
            }catch (Exception e) {
                releaseCamera();
            }
        }
    }

    @Override
    public void surfaceCreated(SurfaceHolder holder) {
        Log.d(TAG, "surfaceCreated.");
        //初始化Camera
        initCamera();
    }

    @Override
    public void surfaceChanged(SurfaceHolder holder, int format, int width, int height) {

    }

    @Override
    public void surfaceDestroyed(SurfaceHolder holder) {
        Log.d(TAG, "surfaceDestroyed.");
        //释放Camera
        releaseCamera();
    }
}
