package cn.jy.lazydict;

import android.Manifest;
import android.animation.Animator;
import android.animation.ValueAnimator;
import android.annotation.SuppressLint;
import android.app.Activity;
import android.app.AlertDialog;
import android.content.Context;
import android.content.DialogInterface;
import android.content.pm.PackageManager;
import android.graphics.Bitmap;
import android.graphics.ImageFormat;
import android.hardware.Camera;
import android.os.Build;
import android.os.Bundle;
import android.os.Handler;
import android.os.Message;
import android.support.annotation.Nullable;
import android.support.annotation.RequiresApi;
import android.support.v4.app.ActivityCompat;
import android.support.v4.content.ContextCompat;
import android.text.Html;
import android.util.Log;
import android.view.MotionEvent;
import android.view.SurfaceHolder;
import android.view.SurfaceView;
import android.view.View;
import android.view.animation.AccelerateDecelerateInterpolator;
import android.webkit.WebView;
import android.widget.Button;
import android.widget.FrameLayout;
import android.widget.ImageButton;
import android.widget.ImageView;
import android.widget.LinearLayout;
import android.widget.TextView;
import android.widget.Toast;

import com.googlecode.tesseract.android.TessBaseAPI;

public class CameraActivity extends Activity{
    //icon
    //https://www.iconfinder.com/icons/1055094/check_select_icon
    static final String TAG = CameraActivity.class.getSimpleName();
    ImageView iv_test;
    AlertDialog messageDialog;
    TextView tv_scan_tip;

    //--------- 预览相关 ------------
    FrameLayout fl_preview;
    SurfaceView surface_view;
    SurfaceHolder surface_holder;
    ImageButton btn_capture;

    //------ 截图相关 -------------
    View v_scan_line;
    FrameLayout fl_capture;
    ImageView iv_capture;
    ImageButton btn_preview;
    TextView tv_mean;
    FrameLayout ll_mean;
    WebView wv_mean;
    Button btn_up_search;
    FrameLayout fl_scan_area;
    LinearLayout ll_mask;
    Bitmap capture;
    Bitmap drawCache;
    Camera camera;
    byte[] previewFrame = null;
    boolean isPreview = true;
    Handler tessHandler;
    ValueAnimator findAnimator;

    FrameLayout fl_capture_bottom;

    LinearLayout ll_lines;

    static TessBaseAPI tessBaseAPI;

    @RequiresApi(api = Build.VERSION_CODES.LOLLIPOP)
    @SuppressLint("ClickableViewAccessibility")
    @Override
    protected void onCreate(@Nullable Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        getWindow().setStatusBarColor(0xff303030);
        setContentView(R.layout.activity_camera);
        tv_scan_tip = findViewById(R.id.tv_scan_tip);
        fl_capture_bottom = findViewById(R.id.fl_capture_bottom);
        ll_lines = findViewById(R.id.ll_lines);
        tv_mean = findViewById(R.id.tv_mean);
        ll_mean = findViewById(R.id.ll_mean);
        ll_mask = findViewById(R.id.ll_mask);
        v_scan_line = findViewById(R.id.v_scan_line);
        btn_up_search = findViewById(R.id.btn_up_search);
        iv_test = findViewById(R.id.iv_test);
        iv_capture = findViewById(R.id.iv_capture);
        fl_capture = findViewById(R.id.fl_capture);
        iv_capture.setDrawingCacheEnabled(true);
        wv_mean = findViewById(R.id.wv_mean);
        fl_scan_area = findViewById(R.id.fl_scan_area);
        wv_mean.getSettings().setJavaScriptEnabled(true);

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
                //识别中不处理
                if(iv_capture.getTag() != null){
                    return;
                }
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

        //处理识别的结果
        tessHandler = new Handler(new Handler.Callback() {
            @Override
            public boolean handleMessage(Message msg) {
                switch (msg.what){
                    case Toolkit.MSG_TESS_RECOGNIZE_START:
                        //开始搜索，执行动画
                        //tv_scan_tip.setText(R.string.info_scanning);
                        v_scan_line.post(new Runnable() {
                            @Override
                            public void run() {
                                int r = fl_scan_area.getMeasuredHeight();
                                findAnimator = ValueAnimator.ofInt(-r, r);
                                findAnimator.setDuration(2000);
                                findAnimator.setRepeatCount(-1);
                                findAnimator.setInterpolator(new AccelerateDecelerateInterpolator());
                                final int w = v_scan_line.getMeasuredWidth();
                                final int h = v_scan_line.getMeasuredHeight();
                                findAnimator.addListener(new Animator.AnimatorListener() {
                                    @Override
                                    public void onAnimationStart(Animator animation) {
                                        v_scan_line.setVisibility(View.VISIBLE);
                                    }
                                    @Override
                                    public void onAnimationEnd(Animator animation) {}
                                    @Override
                                    public void onAnimationCancel(Animator animation) {}
                                    @Override
                                    public void onAnimationRepeat(Animator animation) {}
                                });

                                findAnimator.addUpdateListener(new ValueAnimator.AnimatorUpdateListener() {
                                    @Override
                                    public void onAnimationUpdate(ValueAnimator animation) {
                                        int val = (int)animation.getAnimatedValue();
                                        v_scan_line.layout(0, val, w,val+h);
                                    }
                                });
                                findAnimator.start();
                            }
                        });
                        break;
                    case Toolkit.MSG_TESS_RECOGNIZE_ERROR:
                        Exception e = (Exception) msg.obj;
                        Log.d(TAG, "MSG_TESS_RECOGNIZE_ERROR!!");
                        if(e != null)
                        showMessageDialog(e.getMessage(), false);
                        //继续往下走
                    case Toolkit.MSG_TESS_RECOGNIZE_COMPLETE:
                        findAnimator.end();
                        iv_capture.setTag(null);
                        if(ll_lines.getChildCount()==0){
                            showMessageDialog("没有找到文字", false);
                        }
                        Log.d(TAG, "识别完毕.");
                        break;

                    case Toolkit.MSG_TESS_INIT_ERROR:
                        showMessageDialog("初始化失败!", true);
                        break;
                    case Toolkit.MSG_TESS_INIT_SUCCESS:
                        //启动相机
                        tessBaseAPI = (TessBaseAPI) msg.obj;
                        requestCameraPermission();
                        break;
                    case Toolkit.MSG_BAIKE_SEARCH_RESULT:
                        if(msg.obj==null){
                            tv_mean.setText("未找到解释");
                        }else{
                            String[] res = (String[]) msg.obj;
                            tv_mean.setText(Html.fromHtml(res[1]));
                        }
                        break;
                }
                return false;
            }
        });

        ll_lines.addOnLayoutChangeListener(new View.OnLayoutChangeListener() {
            @Override
            public void onLayoutChange(View v, int left, int top, int right, int bottom, int oldLeft, int oldTop, int oldRight, int oldBottom) {

            }
        });

//        btn_up_search.setOnClickListener(new View.OnClickListener() {
//            @Override
//            public void onClick(View v) {
//                Rect visibleRect = Toolkit.getLocationInParent(iv_area, fl_preview);
//                Bitmap rect = Bitmap.createBitmap(iv_capture.getDrawingCache(), visibleRect.left, visibleRect.top, visibleRect.width(), visibleRect.height());
//                Toolkit.upSearch(CameraActivity.this, rect, tessHandler);
//            }
//        });
    }

    private void showSurfaceViewAndStart(){
        surface_view.setVisibility(View.VISIBLE);
        surface_view.post(new Runnable() {
            @Override
            public void run() {
                initCamera();
            }
        });
    }

    //申请权限
    private void requestCameraPermission() {
        if (ContextCompat.checkSelfPermission(this, Manifest.permission.CAMERA) != PackageManager.PERMISSION_GRANTED) {
            ActivityCompat.requestPermissions(this, new String[]{Manifest.permission.CAMERA}, 1);
        } else {
            showSurfaceViewAndStart();
        }
    }

    @Override
    public void onRequestPermissionsResult(int requestCode, String[] permissions, int[] grantResults) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults);
        if (requestCode == 1) {
            if (grantResults[0] == PackageManager.PERMISSION_GRANTED) {
                showSurfaceViewAndStart();
            } else {
                showMessageDialog("无法启动相机，请先允许相机访问权限", true);
            }
        }
    }

    /**
     * 切换到预览视图
     */
    private void changeStatePreview(){
        Log.d(TAG, "changeStatePreview");
        //清空数据
        fl_capture_bottom.setVisibility(View.VISIBLE);
        tv_scan_tip.setText(R.string.info_preview);
        ll_mean.setVisibility(View.GONE);
        iv_capture.setImageResource(android.R.color.transparent);
        iv_capture.setTag(null);
        ll_lines.removeAllViews();
        drawCache = null;
        iv_capture.destroyDrawingCache();
        capture = null;
        //fl_capture.setVisibility(View.GONE);
        //fl_preview.setVisibility(View.VISIBLE);
        btn_preview.setVisibility(View.GONE);
        btn_capture.setClickable(true);
        btn_capture.setVisibility(View.VISIBLE);
        isPreview = true;
    }

    /**
     * 切换到截屏视图
     */
    private void changeStateCapture(){
        Log.d(TAG, "changeStateCapture");
        //fl_capture.setVisibility(View.VISIBLE);
        //fl_preview.setVisibility(View.GONE);
        btn_capture.setClickable(false);
        btn_preview.setVisibility(View.VISIBLE);
        isPreview = false;
    }

    private void capture(){
        Log.d(TAG, "capture.");
        if(previewFrame!=null && camera!=null){
            android.hardware.Camera.CameraInfo info =
                    new android.hardware.Camera.CameraInfo();
            android.hardware.Camera.getCameraInfo(Camera.CameraInfo.CAMERA_FACING_BACK, info);
            Camera.Size size = camera.getParameters().getPreviewSize();
            byte[] frame = previewFrame;
            releaseCamera();//释放相机
            fl_capture_bottom.setVisibility(View.GONE);
            new SearchDialog(this, info, size, frame).show();
        }else{
            showMessageDialog("相机出错!", false);
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
        if(fl_capture_bottom.getVisibility()==View.VISIBLE){
            surface_view.post(new Runnable() {
                @Override
                public void run() {
                    if(camera==null && isPreview){
                        //initCamera();
                        Toolkit.initTessTwo(CameraActivity.this, tessBaseAPI, tessHandler);
                    }
                }
            });
        }
    }

    public static void toast(Context context, String s){
        Toast.makeText(context, s, Toast.LENGTH_SHORT).show();
    }

    void showMessageDialog(String errorMsg, final boolean isError){
        AlertDialog.Builder builder = new AlertDialog.Builder(this);
        builder.setMessage(errorMsg);
        builder.setTitle("程序错误");
        builder.setPositiveButton("确定", new DialogInterface.OnClickListener() {
            @Override
            public void onClick(DialogInterface dialog, int which) {
                messageDialog = null;
                dialog.dismiss();
                if(isError)
                    finish();
            }
        });
        messageDialog = builder.create();
        messageDialog.setOnDismissListener(new DialogInterface.OnDismissListener() {
            @Override
            public void onDismiss(DialogInterface dialog) {
                messageDialog = null;
            }
        });
        messageDialog.show();
    }
}
