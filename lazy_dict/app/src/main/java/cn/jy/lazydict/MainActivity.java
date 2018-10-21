package cn.jy.lazydict;

import android.Manifest;
import android.app.Activity;
import android.content.Context;
import android.content.pm.PackageManager;
import android.graphics.ImageFormat;
import android.graphics.PixelFormat;
import android.hardware.camera2.CameraAccessException;
import android.hardware.camera2.CameraCaptureSession;
import android.hardware.camera2.CameraCharacteristics;
import android.hardware.camera2.CameraDevice;
import android.hardware.camera2.CameraManager;
import android.hardware.camera2.CaptureRequest;
import android.hardware.camera2.params.StreamConfigurationMap;
import android.media.Image;
import android.media.ImageReader;
import android.os.Bundle;
import android.os.Handler;
import android.support.annotation.NonNull;
import android.support.v4.app.ActivityCompat;
import android.support.v4.content.ContextCompat;
import android.util.Log;
import android.util.Size;
import android.view.Surface;
import android.view.SurfaceView;
import android.widget.FrameLayout;
import android.widget.Toast;

import java.nio.ByteBuffer;
import java.util.Arrays;
import java.util.Collections;
import java.util.Comparator;
//新华字典数据库 https://github.com/pwxcoo/chinese-xinhua
//摄像头 https://www.cnblogs.com/haibindev/p/8408598.html

public class MainActivity extends Activity implements ImageReader.OnImageAvailableListener {
    static final String TAG = MainActivity.class.getSimpleName();

    native int[] renderPreview(ByteBuffer y, ByteBuffer u, ByteBuffer v, int width, int height, int y_row_stride, int uv_row_stride, int uv_pixel_stride, int sensor_orientation);
    native boolean setPreviewSurface(Surface surface);

    static {
        System.loadLibrary("SDL2");
        System.loadLibrary("lazy_dict");
    }

    private int sensor_orientation = 270;

    private CameraManager cameraManager;
    private CameraDevice cameraDevice;
    private CameraCaptureSession cameraCaptureSession;
    private Handler backgroundHandler = new Handler();

    private FrameLayout fl_root;
    private SurfaceView preview_surface;

    final String CAMERA_ID = CameraCharacteristics.LENS_FACING_FRONT+"";

    @Override
    public void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);
        preview_surface = findViewById(R.id.preview_surface);
        fl_root = findViewById(R.id.fl_root);
    }

    private void requestCameraPermission() {
        if (ContextCompat.checkSelfPermission(this, Manifest.permission.CAMERA) != PackageManager.PERMISSION_GRANTED) {
            ActivityCompat.requestPermissions(this, new String[]{Manifest.permission.CAMERA}, 1);
        } else {
            initCamera();
        }
    }

    private CameraDevice.StateCallback stateCallback = new CameraDevice.StateCallback() {
        /**
         * 相机打开
         * @param cameraDevice
         */
        @Override
        public void onOpened(@NonNull CameraDevice cameraDevice) {
            toast("相机已经开启");
            MainActivity.this.cameraDevice = cameraDevice;
            //启动预览
            try {
                requestPreview();
            } catch (CameraAccessException e) {
                e.printStackTrace();
                toast("相机预览失败!");
            }
        }

        /**
         * 相机断开连接
         * @param cameraDevice
         */
        @Override
        public void onDisconnected(@NonNull CameraDevice cameraDevice) {
            toast("相机断开连接");
        }

        /**
         * 相机错误
         * @param cameraDevice
         * @param i
         */
        @Override
        public void onError(@NonNull CameraDevice cameraDevice, int i) {
            toast("相机错误 "+i);
        }
    };

    private void initCamera() {
        Log.d(TAG, "初始化相机");
        cameraManager = (CameraManager) getSystemService(Context.CAMERA_SERVICE);
        if (ActivityCompat.checkSelfPermission(this, Manifest.permission.CAMERA) == PackageManager.PERMISSION_GRANTED) {
            try {
                cameraManager.openCamera(CAMERA_ID, stateCallback, backgroundHandler);
            } catch (CameraAccessException e) {
                e.printStackTrace();
                toast("相机开启失败");
            }
        }
    }

    @Override
    public void onRequestPermissionsResult(int requestCode, String[] permissions, int[] grantResults) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults);
        if (requestCode == 1) {
            if (grantResults[0] == PackageManager.PERMISSION_GRANTED) {
                Log.d(TAG, "相机权限已获取");
                initCamera();
            } else {
                toast("请在应用管理中打开“相机”访问权限！");
            }
        }
    }

    long lastTime = System.currentTimeMillis();

    @Override
    public void onImageAvailable(ImageReader imageReader) {

        long duration = System.currentTimeMillis()-lastTime;
        lastTime = System.currentTimeMillis();
        //经过测试, 这里帧率最高为30左右, 如果手机性能差，过高的分辨率帧率可能达不到30帧
        Log.d(TAG, "帧时间="+duration+"ms");

        Image image = imageReader.acquireNextImage();
        if (image != null) {
            //绘制预览图片
            try{
                Image.Plane[] plane = image.getPlanes();
                final int yRowStride = plane[0].getRowStride();
                final int uvRowStride = plane[1].getRowStride();
                final int uvPixelStride = plane[1].getPixelStride();
                int[] ret = renderPreview(plane[0].getBuffer(),
                        plane[1].getBuffer(),
                        plane[2].getBuffer(),
                        image.getWidth(),
                        image.getHeight(),
                        yRowStride,
                        uvRowStride,
                        uvPixelStride,
                        sensor_orientation);
                if(ret != null && ret.length==2){
                    if(ret[0]!=-1&&ret[1]!=-1){
                        //调整SurfaceView大小
                        int screen_width = fl_root.getMeasuredWidth();
                        int screen_height = fl_root.getMeasuredHeight();
                        int surface_height = (int)((float)screen_width*ret[1]/(float)ret[0]);
                        int surface_width = screen_width;

                        //如果高度小于屏幕高度, 调整高度
                        if(surface_height<screen_height){
                            surface_width = (int)(screen_height*(float)surface_width/(float)surface_height);
                            surface_height = screen_height+2;//+2是去掉虚边
                        }
                        //Log.d(TAG, "renderPreview="+ret[0]+"x"+ret[1]+" surface_height="+surface_height);
                        if(preview_surface.getMeasuredHeight() != surface_height
                                || preview_surface.getMeasuredWidth() != surface_width){
                            FrameLayout.LayoutParams flp = (FrameLayout.LayoutParams) preview_surface.getLayoutParams();
                            flp.height = surface_height;
                            flp.width = surface_width;
                            flp.leftMargin = -(surface_width-screen_width)/2;//设置-margin去掉左边虚边
                            flp.topMargin = -2;//去掉虚边
                            preview_surface.setLayoutParams(flp);
                        }
                    }
                }
            }catch (Throwable t){
                t.printStackTrace();
            }
            //必须close，否则无法继续收到图片
            image.close();
        }
    }

    private void requestPreview() throws CameraAccessException {

        // 获取指定摄像头的特性
        CameraCharacteristics characteristics
                = cameraManager.getCameraCharacteristics(CAMERA_ID);
        // 获取摄像头支持的配置属性
        StreamConfigurationMap map = characteristics.get(
                CameraCharacteristics.SCALER_STREAM_CONFIGURATION_MAP);
        Size[] sizes = map.getOutputSizes(ImageFormat.YUV_420_888);
        Arrays.sort(sizes, new CompareSizesByArea());
        Size minSize = sizes[0];
        for (Size s : sizes){
            if (s.getWidth()<=720&&s.getHeight()<=720){
                minSize = s;
            }
            Log.d(TAG, "预览大小Size="+s.toString());
        }
        //设置竖向显示
        sensor_orientation = characteristics.get(CameraCharacteristics.SENSOR_ORIENTATION);
        Log.d(TAG, "ORIENTATION========="+sensor_orientation);
        Log.d(TAG, "minSize========="+minSize.toString());
        preview_surface.getHolder().setFixedSize(minSize.getHeight(), minSize.getWidth());
        preview_surface.getHolder().setFormat(PixelFormat.RGB_888);
        preview_surface.post(new Runnable() {
            @Override
            public void run() {
                setPreviewSurface(preview_surface.getHolder().getSurface());
            }
        });
        ImageReader imageReader = ImageReader.newInstance(minSize.getWidth(), minSize.getHeight(), ImageFormat.YUV_420_888, /*maxImages*/2);
        imageReader.setOnImageAvailableListener(this, backgroundHandler);
        final CaptureRequest.Builder builder = cameraDevice.createCaptureRequest(CameraDevice.TEMPLATE_PREVIEW);
        builder.addTarget(imageReader.getSurface());
        cameraDevice.createCaptureSession(Collections.singletonList(imageReader.getSurface()), new CameraCaptureSession.StateCallback() {
            @Override
            public void onConfigured(@NonNull CameraCaptureSession cameraCaptureSession) {
                toast("预览配置完成");
                if(cameraDevice == null){
                    toast("相机已关闭");
                    return;
                }
                MainActivity.this.cameraCaptureSession = cameraCaptureSession;
                builder.set(CaptureRequest.CONTROL_AF_MODE, CaptureRequest.CONTROL_AF_MODE_CONTINUOUS_PICTURE);
                CaptureRequest request = builder.build();
                try {
                    cameraCaptureSession.setRepeatingRequest(request, null, backgroundHandler);
                    toast("预览请求成功.");
                } catch (CameraAccessException e) {
                    e.printStackTrace();
                    toast("预览请求失败!");
                }
            }

            @Override
            public void onConfigureFailed(@NonNull CameraCaptureSession cameraCaptureSession) {
                toast("预览会话创建失败");
            }
        }, backgroundHandler);
    }

    // 为Size定义一个比较器Comparator
    static class CompareSizesByArea implements Comparator<Size> {
        @Override
        public int compare(Size lhs, Size rhs)
        {
            // 强转为long保证不会发生溢出
            return Long.signum((long) lhs.getWidth() * lhs.getHeight() -
                    (long) rhs.getWidth() * rhs.getHeight());
        }
    }

    @Override
    protected void onResume() {
        super.onResume();
        //开始预览
        requestCameraPermission();
    }

    @Override
    protected void onPause() {
        super.onPause();
        //停止预览
        if(cameraCaptureSession != null) cameraCaptureSession.close();
        if(cameraDevice!=null) cameraDevice.close();
        cameraManager = null;
    }

    private void toast(String s){
        Toast.makeText(this, s, Toast.LENGTH_SHORT).show();
        Log.d(TAG, s);
    }
}
