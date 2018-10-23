package cn.jy.lazydict;

import android.Manifest;
import android.animation.ObjectAnimator;
import android.app.Activity;
import android.content.Context;
import android.content.pm.PackageManager;
import android.graphics.Bitmap;
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
import android.os.Message;
import android.support.annotation.NonNull;
import android.support.v4.app.ActivityCompat;
import android.support.v4.content.ContextCompat;
import android.util.Log;
import android.util.Size;
import android.view.Surface;
import android.view.SurfaceView;
import android.webkit.WebView;
import android.widget.FrameLayout;
import android.widget.ImageView;
import android.widget.TextView;
import android.widget.Toast;

import com.googlecode.tesseract.android.TessBaseAPI;

import java.io.File;
import java.io.IOException;
import java.nio.ByteBuffer;
import java.util.Arrays;
import java.util.Collections;
import java.util.Comparator;

import static com.googlecode.tesseract.android.TessBaseAPI.PageSegMode.PSM_SINGLE_WORD;
//新华字典数据库 https://github.com/pwxcoo/chinese-xinhua
//摄像头 https://www.cnblogs.com/haibindev/p/8408598.html

public class MainActivity extends Activity implements ImageReader.OnImageAvailableListener {
    static final String TAG = MainActivity.class.getSimpleName();

    native int[] renderPreview(ByteBuffer y, ByteBuffer u, ByteBuffer v, int width, int height, int y_row_stride, int uv_row_stride, int uv_pixel_stride, int sensor_orientation);
    native boolean setPreviewSurface(Surface surface);
    native void onTextRecognized(long time, String text);

    static TessBaseAPI tessBaseAPI;
    boolean recognizing = false;

    static {
        //去广告！！！！！！！ 去广告 到淘宝店铺购买密钥！！
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
    private TextView tv_status;
    private FrameLayout fl_status;
    private WebView loader;
    private ImageView iv_bitmap;

    private long step = System.currentTimeMillis();

    @Override
    public void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        Log.d(TAG, "<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<STEP_onCreate "+(System.currentTimeMillis()-step));
        setContentView(R.layout.activity_main);
        preview_surface = findViewById(R.id.preview_surface);
        fl_root = findViewById(R.id.fl_root);
        tv_status = findViewById(R.id.tv_status);
        loader = findViewById(R.id.loader);
        fl_status = findViewById(R.id.fl_status);
        iv_bitmap = findViewById(R.id.iv_bitmap);
        loader.loadUrl("file:///android_asset/loading.html");
    }

    private void requestCameraPermission() {
        Log.d(TAG, "<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<STEP_requestCameraPermission "+(System.currentTimeMillis()-step));
        tv_status.setText(R.string.status_camera);
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
            Log.d(TAG, "<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<STEP_onOpened "+(System.currentTimeMillis()-step));
            MainActivity.this.cameraDevice = cameraDevice;
            //启动预览
            try {
                tv_status.setText(R.string.status_preview);
                requestPreview();
            } catch (CameraAccessException e) {
                e.printStackTrace();
                tv_status.setText(R.string.error_preview);
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
        Log.d(TAG, "<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<STEP_initCamera "+(System.currentTimeMillis()-step));
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
        Log.d(TAG, "<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<STEP_onRequestPermissionsResult "+(System.currentTimeMillis()-step));
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

    //long lastTime = System.currentTimeMillis();

    @Override
    public void onImageAvailable(ImageReader imageReader) {

        //long duration = System.currentTimeMillis()-lastTime;
        //lastTime = System.currentTimeMillis();
        //经过测试, 这里帧率最高为30左右, 如果手机性能差，过高的分辨率帧率可能达不到30帧
        //Log.d(TAG, "帧时间="+duration+"ms");

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
                            surface_height = screen_height;//是去掉虚边
                        }
                        //Log.d(TAG, "renderPreview="+ret[0]+"x"+ret[1]+" surface_height="+surface_height);
                        if(preview_surface.getMeasuredHeight() != surface_height
                                || preview_surface.getMeasuredWidth() != surface_width){
                            FrameLayout.LayoutParams flp = (FrameLayout.LayoutParams) preview_surface.getLayoutParams();
                            flp.height = surface_height;
                            flp.width = surface_width;
                            preview_surface.setLayoutParams(flp);
                        }
                        if(fl_status.getAlpha()==1.0){
                            ObjectAnimator fadeOut = ObjectAnimator.ofFloat(fl_status, "alpha",  1f, .0f);
                            fadeOut.setDuration(500);
                            fadeOut.start();
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
        Log.d(TAG, "<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<STEP_requestPreview "+(System.currentTimeMillis()-step));
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
            //Log.d(TAG, "预览大小Size="+s.toString());
        }
        sensor_orientation = characteristics.get(CameraCharacteristics.SENSOR_ORIENTATION);
        Log.d(TAG, "预览大小:"+minSize.toString()+" ORIENTATION="+sensor_orientation);
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
                Log.d(TAG,"预览配置完成");
                if(cameraDevice == null){
                    tv_status.setText(R.string.status_camera_close);
                    return;
                }
                MainActivity.this.cameraCaptureSession = cameraCaptureSession;
                builder.set(CaptureRequest.CONTROL_AF_MODE, CaptureRequest.CONTROL_AF_MODE_CONTINUOUS_PICTURE);
                CaptureRequest request = builder.build();
                try {
                    cameraCaptureSession.setRepeatingRequest(request, null, backgroundHandler);
                } catch (CameraAccessException e) {
                    e.printStackTrace();
                    tv_status.setText(R.string.error_preview);
                }
            }

            @Override
            public void onConfigureFailed(@NonNull CameraCaptureSession cameraCaptureSession) {
                Log.d(TAG, "预览会话创建失败");
                tv_status.setText(R.string.error_preview);
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
        Log.d(TAG, "<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<STEP_onResume "+(System.currentTimeMillis()-step));
        super.onResume();
        fl_status.setAlpha(1.0f);
        //初始化tess-two文件
        //init_tess_two();
        //开始预览
        loader.postDelayed(new Runnable() {
            @Override
            public void run() {
                init_tess_two();
            }
        }, 100);
    }

    Handler progressHandler = new Handler(new Handler.Callback() {
        @Override
        public boolean handleMessage(Message msg) {
            tv_status.setText("正在初始化 "+msg.what+"%");
            return false;
        }
    });

    private void init_tess_two(){
        //将tessdata文件夹解压到files文件夹
        new Thread(new Runnable() {
            @Override
            public void run() {
                boolean success = false;
                try {
                    File tessDataDir = new File(getFilesDir(), "tessdata");
                    if(!tessDataDir.exists()){
                        if(FileUtils.unpackZip(getAssets().open("tessdata.zip"), getFilesDir(), progressHandler)){
                            Log.d(TAG, "tessdata解压成功");
                            success = true;
                        }else{
                            Log.e(TAG, "tessdata解压失败");
                            tv_status.setText(R.string.error_init);
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
                                requestCameraPermission();
                            }else{
                                tv_status.setText("Tess初始化失败!");
                            }
                        }
                    });
                }
            }
        }).start();
    }

    /**
     * @param imageData byte representation of the image
     * @param width width image width
     * @param height height image height
     * @param bpp 每个像素字节
     * @param bpl 每行字节
     * @return
     */
    public void getText(final byte[][] imageData, final int[] width, final int height, final int bpp, final int bpl){
        if(recognizing) return;
        recognizing = true;
        Thread t = new Thread(new Runnable() {
            @Override
            public void run() {
                //Log.d(TAG, "识别->getText>>>>>>>>>>>> byte.len()="+imageData.length+" "+width+"x"+height+" bpp="+bpp+" bpl="+bpl);
                //Log.d(TAG, "识别->版本:"+tessBaseAPI.getVersion());
//                final Bitmap bmp = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888);
//                try{
//                    ByteBuffer buffer = ByteBuffer.wrap(imageData);
//                    bmp.copyPixelsFromBuffer(buffer);
//                    runOnUiThread(new Runnable() {
//                        @Override
//                        public void run() {
//                            iv_bitmap.setImageBitmap(bmp);
//                        }
//                    });
//                }catch (Throwable t){
//                    t.printStackTrace();
//                }
                final long time = System.currentTimeMillis();
                Log.d(TAG, "识别->开始调用>");
                //tess配置说明 https://www.jianshu.com/p/c4a11241b557
                //PSM_SINGLE_WORD 单独的字
                tessBaseAPI.setPageSegMode(PSM_SINGLE_WORD);
                for (int i=0; i<imageData.length; i++){
                    tessBaseAPI.setImage(imageData[i], width[i], height, bpp, bpl);
                    final String text = tessBaseAPI.getUTF8Text();
                    //        ResultIterator resultIterator = tessBaseAPI.getResultIterator();
                    //        int level = TessBaseAPI.PageIteratorLevel.RIL_SYMBOL;
                    //        do{
                    //            Log.d(TAG, resultIterator.getUTF8Text(level)+"-"+resultIterator.confidence(level));
                    //        }while(resultIterator.next(level));
                    //        resultIterator.delete();
                    runOnUiThread(new Runnable() {
                        @Override
                        public void run() {
                            onTextRecognized(time, text);
                            Log.d(TAG, "识别结果:"+time+">>>"+text);
                            recognizing = false;
                        }
                    });
                }
                Log.d(TAG, "识别耗时:"+(System.currentTimeMillis()-time)+"ms");
            }
        });
        //t.setPriority(Thread.MAX_PRIORITY);
        t.start();
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
