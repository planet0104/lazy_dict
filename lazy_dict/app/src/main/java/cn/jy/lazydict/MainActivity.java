package cn.jy.lazydict;

import android.Manifest;
import android.app.NativeActivity;
import android.content.Context;
import android.content.pm.PackageManager;
import android.graphics.ImageFormat;
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
import android.widget.Toast;

import java.nio.ByteBuffer;
import java.util.Arrays;
import java.util.Collections;
import java.util.Comparator;
import java.util.List;

import static android.hardware.camera2.CameraMetadata.LENS_FACING_BACK;
//新华字典数据库 https://github.com/pwxcoo/chinese-xinhua
//摄像头 https://www.cnblogs.com/haibindev/p/8408598.html

public class MainActivity extends NativeActivity implements ImageReader.OnImageAvailableListener {
    static final String TAG = MainActivity.class.getSimpleName();

    native void send(ByteBuffer y, ByteBuffer u, ByteBuffer v, int width, int height);
    private native void sendRgb(byte[] bytes, int width, int height, int rowStride);

    static {
        System.loadLibrary("SDL2");
        System.loadLibrary("lazy_dict");
    }

    private CameraManager cameraManager;
    private CameraDevice cameraDevice;
    private ImageReader imageReader;
    private CameraCaptureSession cameraCaptureSession;
    private Handler backgroundHandler = new Handler();

    @Override
    public void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
//        new Handler().postDelayed(new Runnable() {
//            @Override
//            public void run() {
//                try {
//                    BitmapFactory.Options options = new BitmapFactory.Options();
//                    options.inPreferredConfig = Bitmap.Config.RGB_565;
//                    Bitmap bitmap = BitmapFactory.decodeStream(getAssets().open("rust.png"), null, options);
//                    Log.d(TAG,"new_buf="+bitmap.getByteCount());
//                    sendRgb(bitmap, bitmap.getRowBytes());
//                    Log.d(TAG,"sendRgb OK.");
//                } catch (Throwable e) {
//                    e.printStackTrace();
//                    return;
//                }
//            }
//        }, 2000);
        requestCameraPermission();
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
                cameraManager.openCamera(LENS_FACING_BACK + "", stateCallback, backgroundHandler);
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

    @Override
    public void onImageAvailable(ImageReader imageReader) {
        Image image = imageReader.acquireNextImage();
        if (image != null) {
            //绘制预览图片
            try{
                send(image.getPlanes()[0].getBuffer(), image.getPlanes()[1].getBuffer(), image.getPlanes()[2].getBuffer(), image.getWidth(), image.getHeight());
//                Image.Plane plane = image.getPlanes()[0];
//                Log.i(TAG, "plane.getBuffer().array().length="+plane.getBuffer().array().length+" getRowStride="+plane.getRowStride());
//                sendRgb(plane.getBuffer().array(), image.getWidth(), image.getHeight(), plane.getRowStride());
                //byte[] bytes = new byte[buffer.remaining()];
                //buffer.get(bytes);

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
                = cameraManager.getCameraCharacteristics(LENS_FACING_BACK+"");
        // 获取摄像头支持的配置属性
        StreamConfigurationMap map = characteristics.get(
                CameraCharacteristics.SCALER_STREAM_CONFIGURATION_MAP);
        List<Size> size = Arrays.asList(map.getOutputSizes(ImageFormat.YUV_420_888));
        for (Size s : size){
            Log.d(TAG, "预览大小: "+s.toString());
        }
        // 获取摄像头支持的最大尺寸
        Size minSize = Collections.min(size, new CompareSizesByArea());
        imageReader = ImageReader.newInstance(minSize.getWidth(), minSize.getHeight(), ImageFormat.YUV_420_888, /*maxImages*/2);
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

    private void toast(String s){
        Toast.makeText(this, s, Toast.LENGTH_LONG).show();
        Log.d(TAG, s);
    }
}
