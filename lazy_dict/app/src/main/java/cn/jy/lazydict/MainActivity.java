package cn.jy.lazydict;

import android.Manifest;
import android.content.pm.PackageManager;
import android.graphics.Bitmap;
import android.graphics.BitmapFactory;
import android.graphics.ImageFormat;
import android.hardware.camera2.CameraAccessException;
import android.hardware.camera2.CameraCaptureSession;
import android.hardware.camera2.CameraDevice;
import android.hardware.camera2.CameraManager;
import android.hardware.camera2.CaptureRequest;
import android.media.Image;
import android.media.ImageReader;
import android.os.Bundle;
import android.os.Handler;
import android.support.annotation.NonNull;
import android.support.v4.app.ActivityCompat;
import android.support.v4.content.ContextCompat;
import android.util.Log;
import android.widget.TextView;
import android.widget.Toast;

import org.liballeg.android.AllegroActivity;

import java.nio.ByteBuffer;
import java.util.Collections;
//新华字典数据库 https://github.com/pwxcoo/chinese-xinhua
//摄像头 https://www.cnblogs.com/haibindev/p/8408598.html

public class MainActivity extends AllegroActivity implements ImageReader.OnImageAvailableListener {
    static final String TAG = MainActivity.class.getSimpleName();

    static {
        System.loadLibrary("allegro");
        System.loadLibrary("allegro_primitives");
        System.loadLibrary("allegro_image");
        System.loadLibrary("allegro_font");
        System.loadLibrary("allegro_ttf");
        System.loadLibrary("allegro_audio");
        System.loadLibrary("allegro_acodec");
        System.loadLibrary("allegro_color");
    }
    public MainActivity() {
        super("liblazy_dict.so");
    }

    private CameraManager cameraManager;
    private CameraDevice cameraDevice;
    private ImageReader imageReader;
    private CameraCaptureSession cameraCaptureSession;
    private Handler backgroundHandler = new Handler();

    @Override
    public void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        TextView textView =new TextView(this);
        textView.setText("hello!!!");
        setContentView(textView);
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
//        cameraManager = (CameraManager) getSystemService(Context.CAMERA_SERVICE);
//        if (ActivityCompat.checkSelfPermission(this, Manifest.permission.CAMERA) == PackageManager.PERMISSION_GRANTED) {
//            try {
//                cameraManager.openCamera(LENS_FACING_BACK + "", stateCallback, backgroundHandler);
//            } catch (CameraAccessException e) {
//                e.printStackTrace();
//                toast("相机开启失败");
//            }
//        }
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

    native void send(ByteBuffer y, ByteBuffer u, ByteBuffer v, int width, int height);
    native void sendRgb(ByteBuffer buffer, int width, int height);

    @Override
    public void onImageAvailable(ImageReader imageReader) {
        Image image = imageReader.acquireNextImage();
        if (image != null) {
            //绘制预览图片
            try{
                //send(image.getPlanes()[0].getBuffer(), image.getPlanes()[1].getBuffer(), image.getPlanes()[2].getBuffer(), image.getWidth(), image.getHeight());
                ByteBuffer buffer = image.getPlanes()[0].getBuffer();
                byte[] bytes = new byte[buffer.remaining()];
                buffer.get(bytes);
                //由缓冲区存入字节数组
                Bitmap bitmap = BitmapFactory.decodeByteArray(bytes, 0, bytes.length);
                int picw = bitmap.getWidth();
                int pich = bitmap.getHeight();
                int[] pix = new int[picw * pich];
                bitmap.getPixels(pix, 0, picw, 0, 0, picw, pich);
                Log.d(TAG, "hehetotal="+picw*pich*3);
                ByteBuffer new_buf = ByteBuffer.allocate(picw*pich*3);
                for (int y = 0; y < pich; y++){
                    for (int x = 0; x < picw; x++)
                    {
                        int index = y * picw + x;
                        int r = (pix[index] >> 16) & 0xff;     //bitwise shifting
                        int g = (pix[index] >> 8) & 0xff;
                        int b = pix[index] & 0xff;
                        new_buf.put((byte) r);
                        new_buf.put((byte) g);
                        new_buf.put((byte) b);
                    }}
                Log.d(TAG,"nheheew_buf="+new_buf.array().length);
                sendRgb(new_buf, picw, pich);
                Log.d(TAG,"sendRgb OK."+new_buf.array().length);
            }catch (Throwable t){
                t.printStackTrace();
            }
            //必须close，否则无法继续收到图片
            image.close();
        }
    }

    private void requestPreview() throws CameraAccessException {
        imageReader = ImageReader.newInstance(640, 480, ImageFormat.JPEG, /*maxImages*/2);
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

    private void toast(String s){
        Toast.makeText(this, s, Toast.LENGTH_LONG).show();
        Log.d(TAG, s);
    }
}
