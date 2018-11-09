package cn.jy.lazydict;

import android.Manifest;
import android.animation.ObjectAnimator;
import android.animation.ValueAnimator;
import android.annotation.SuppressLint;
import android.app.Activity;
import android.app.AlertDialog;
import android.content.Context;
import android.content.DialogInterface;
import android.content.Intent;
import android.content.pm.PackageInfo;
import android.content.pm.PackageManager;
import android.graphics.Bitmap;
import android.graphics.ImageFormat;
import android.graphics.Rect;
import android.graphics.drawable.AnimationDrawable;
import android.hardware.Camera;
import android.os.Build;
import android.os.Bundle;
import android.os.Handler;
import android.os.Message;
import android.support.annotation.Nullable;
import android.support.annotation.RequiresApi;
import android.support.v4.app.ActivityCompat;
import android.support.v4.content.ContextCompat;
import android.util.Log;
import android.view.MotionEvent;
import android.view.SurfaceHolder;
import android.view.SurfaceView;
import android.view.View;
import android.view.ViewGroup;
import android.view.animation.LinearInterpolator;
import android.webkit.WebResourceRequest;
import android.webkit.WebView;
import android.webkit.WebViewClient;
import android.widget.FrameLayout;
import android.widget.ImageButton;
import android.widget.ImageView;
import android.widget.LinearLayout;
import android.widget.TextView;
import android.widget.Toast;

import com.googlecode.tesseract.android.TessBaseAPI;

import cds.sdg.sdf.nm.sp.SpotManager;

import static cn.jy.lazydict.Toolkit.MSG_TESS_RECOGNIZE_LINE;
import static cn.jy.lazydict.Toolkit.loadText;

public class CameraActivity extends Activity{
    //icon
    //https://www.iconfinder.com/icons/1055094/check_select_icon
    static final String TAG = CameraActivity.class.getSimpleName();
    ImageView iv_test;
    ImageView iv_switch_red;
    ImageView iv_noise;
    private AnimationDrawable animDot;
    private AnimationDrawable animDotSlow;
    AlertDialog messageDialog;

    //--------- 预览相关 ------------
    FrameLayout fl_preview;
    SurfaceView surface_view;
    SurfaceHolder surface_holder;
    ImageButton btn_capture;
    LinearLayout ll_btn_capture;

    //------ 截图相关 -------------
    FrameLayout fl_capture;
    ImageView iv_capture;
    ImageButton btn_preview;
    ImageView iv_area;
    ScrollLinearLayout sl_clip_rect;
    ImageView iv_switch_red_2;
    FrameLayout ll_mean;
    WebView wv_mean;
    /**
     * 搜索动画
     */
    ImageView iv_find;

    Bitmap capture;
    Bitmap drawCache;
    Camera camera;
    byte[] previewFrame = null;
    boolean isPreview = true;
    int tvRectTop = 0;
    Handler tessHandler;
    ValueAnimator findAnimator = ValueAnimator.ofInt(0, 360);

    LinearLayout ll_lines;

    TextView tv_help;
    LinearLayout ll_menu;
    TextView tv_about;
    TextView tv_up_search;
    /**
     * 要识别的区域
     */
    Bitmap bitmapRect;

    static TessBaseAPI tessBaseAPI;

    @RequiresApi(api = Build.VERSION_CODES.LOLLIPOP)
    @SuppressLint("ClickableViewAccessibility")
    @Override
    protected void onCreate(@Nullable Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_camera);
        ll_lines = findViewById(R.id.ll_lines);
        ll_mean = findViewById(R.id.ll_mean);
        tv_help = findViewById(R.id.tv_help);
        ll_menu = findViewById(R.id.ll_menu);
        tv_about = findViewById(R.id.tv_about);
        ll_btn_capture = findViewById(R.id.ll_btn_capture);
        tv_up_search = findViewById(R.id.tv_up_search);
        iv_switch_red_2 = findViewById(R.id.iv_switch_red_2);
        animDot = (AnimationDrawable) getDrawable(R.drawable.anim_dot);
        animDotSlow = (AnimationDrawable)getDrawable(R.drawable.anim_dot_slow);
        iv_switch_red = findViewById(R.id.iv_switch_red);
        sl_clip_rect = findViewById(R.id.sl_clip_rect);
        iv_switch_red.setImageDrawable(animDot);
        iv_noise = findViewById(R.id.iv_noise);
        iv_test = findViewById(R.id.iv_test);
        iv_capture = findViewById(R.id.iv_capture);
        fl_capture = findViewById(R.id.fl_capture);
        iv_capture.setDrawingCacheEnabled(true);
        iv_area = findViewById(R.id.iv_area);
        iv_find = findViewById(R.id.iv_find);
        wv_mean = findViewById(R.id.wv_mean);
        wv_mean.setWebViewClient(new WebViewClient() {
            public void onPageFinished(WebView view, String url) {
                wv_mean.postDelayed(new Runnable() {
                    @Override
                    public void run() {
                        wv_mean.scrollTo(0,0);
                    }
                }, 100);
            }

            @Override
            public boolean shouldOverrideUrlLoading(WebView view, WebResourceRequest request) {
                if(request.getUrl().toString().contains("up.search")){
                    if(bitmapRect==null){
                        showMessageDialog("图片有误，请重新拍照识别。", false);
                    }else{
                        //高级搜索
                        ll_mean.setVisibility(View.GONE);
                        ll_lines.removeAllViews();
                        Toolkit.upSearch(CameraActivity.this, bitmapRect, tessHandler);
                    }
                    return true;
                }
                Intent intent = new Intent();
                intent.setAction("android.intent.action.VIEW");
                intent.setData(request.getUrl());
                startActivity(intent);
                return true;
            }
        });

        tv_help.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                if(ll_menu.getAlpha() == 0){
                    showMenu();
                }else{
                    hideMenu();
                }
            }
        });

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
                if(iv_find.getVisibility() == View.VISIBLE){
                    return;
                }
                ll_lines.removeAllViews();
                ll_mean.setVisibility(View.GONE);
                //移动电视到中间
                Rect tvRect = Toolkit.getLocationInParent(sl_clip_rect.getChildAt(0), sl_clip_rect);
                tvRectTop = tvRect.top;
                sl_clip_rect.setOnScrollFinishedListener(new ScrollLinearLayout.OnScrollFinishedListener() {
                    @Override
                    public void onScrollFinished(ViewGroup scrollView) {
                        sl_clip_rect.setOnScrollFinishedListener(null);
                        if(camera == null) initCamera();
                    }
                });
                sl_clip_rect.smoothScrollTo(0, 0, 800);
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

        animDot.start();

        //处理识别的结果
        tessHandler = new Handler(new Handler.Callback() {
            @Override
            public boolean handleMessage(Message msg) {
                switch (msg.what){
                    case Toolkit.MSG_TESS_RECOGNIZE_START:
                        iv_switch_red_2.setImageDrawable(animDot);
                        animDot.start();
                        iv_find.setVisibility(View.VISIBLE);
                        iv_find.post(new Runnable() {
                            @Override
                            public void run() {
                                findAnimator.setDuration(1000);
                                findAnimator.setRepeatCount(-1);
                                findAnimator.setInterpolator(new LinearInterpolator());
                                //半径
                                final int r = Toolkit.dip2px(CameraActivity.this, 15);
                                final int w = iv_find.getMeasuredWidth();
                                final int h = iv_find.getMeasuredHeight();
                                final int x0 = ((FrameLayout.LayoutParams)iv_find.getLayoutParams()).leftMargin+w/2;
                                final int y0 = ((FrameLayout.LayoutParams)iv_find.getLayoutParams()).topMargin+h/2;
                                findAnimator.addUpdateListener(new ValueAnimator.AnimatorUpdateListener() {
                                    @Override
                                    public void onAnimationUpdate(ValueAnimator animation) {
                                        int theta = (int)animation.getAnimatedValue();
                                        int x1 = (int) (x0 + r * Math.cos(theta * 3.14/180));
                                        int y1 = (int) (y0 + r * Math.sin(theta * 3.14/180));
                                        iv_find.layout(x1-w/2, y1-h/2,x1+w/2,y1+h/2);
                                    }
                                });
                                findAnimator.start();
                            }
                        });
                        break;
                    case Toolkit.MSG_TESS_RECOGNIZE_ERROR:
                        Exception e = (Exception) msg.obj;
                        showMessageDialog(e.getMessage(), false);
                        break;
                    case Toolkit.MSG_TESS_RECOGNIZE_COMPLETE:
                        animDot.stop();
                        findAnimator.end();
                        iv_find.setVisibility(View.GONE);
                        iv_switch_red_2.setBackgroundResource(R.drawable.dot_red);
                        if(ll_lines.getChildCount()==0){
                            showMessageDialog("没有找到文字", false);
                        }
                        Log.d(TAG, "识别完毕.");
                        break;
                    case MSG_TESS_RECOGNIZE_LINE:
                        String[] result = (String[]) msg.obj;
                        if(result==null || result.length==0){
                            return true;
                        }
                        int pos = msg.arg1;//在第几个位置插入

                        //背景变色
                        int tvbg;
                        if(ll_lines.getTag() == null){
                            tvbg = R.drawable.text_line_red_selector;
                            ll_lines.setTag("");
                        }else{
                            tvbg = R.drawable.text_line_blue_selector;
                            ll_lines.setTag(null);
                        }
                        for(String text : result){
                            //检查是否有相同的FlowLayout存在
                            boolean contains = false;
                            for (int l=0; l<ll_lines.getChildCount(); l++){
                                String oldText = (String) ll_lines.getChildAt(l).getTag();
                                if(oldText.equals(text)){
                                    contains = true;
                                    break;
                                }
                            }
                            if (contains) {
                                continue;
                            }

                            if(text==null || text.trim().length()==0) continue;
                            //每一行
                            FlowLayout line_layout = new FlowLayout(CameraActivity.this);
                            LinearLayout.LayoutParams llp = new LinearLayout.LayoutParams(LinearLayout.LayoutParams.WRAP_CONTENT, LinearLayout.LayoutParams.MATCH_PARENT);
                            llp.topMargin = Toolkit.dip2px(CameraActivity.this, 8);
                            //line_layout.setGravity(Gravity.CENTER);
                            line_layout.setLayoutParams(llp);
                            line_layout.setBackgroundResource(tvbg);
                            String[] pinyin = null;
                            try{ pinyin= Toolkit.pinyin(text); }catch (Exception e1){e1.printStackTrace();}
                            char[] chars = text.toCharArray();
                            for(int i=0; i<text.length(); i++){
                                PinYinTextView tv = new PinYinTextView(CameraActivity.this);
                                if(text.length()==1){
                                    tv.getPinyinView().setMinWidth(Toolkit.dip2px(CameraActivity.this, 32));
                                }
                                tv.setText(chars[i]+"");
                                if(pinyin.length>i){
                                    tv.setPinyin(pinyin[i]);
                                }else{
                                    tv.setPinyin("");
                                }
                                line_layout.addView(tv);
                            }
                            line_layout.setTag(text);
                            line_layout.setOnClickListener(new View.OnClickListener() {
                                @Override
                                public void onClick(View v) {
                                    //切换选中状态
                                    for(int i=0; i<ll_lines.getChildCount(); i++){
                                        ll_lines.getChildAt(i).setSelected(false);
                                    }
                                    v.setSelected(true);

                                    final String text = (String) v.getTag();
                                    if(text.length()==1){
                                        //查字
                                        Word word = null;
                                        try{ word = Toolkit.search(text); }catch (Exception e){ e.printStackTrace(); }
                                        if(word == null){
                                            Toolkit.loadText(wv_mean, "正在网络上搜索...");
                                            Toolkit.checkBaiKe(text, tessHandler);
                                            //wv_mean
                                        }else{
                                            Toolkit.loadText(wv_mean, word.toString());
                                        }
                                    }else{
                                        //查词
                                        String mean = null;
                                        try{ mean = Toolkit.searchWords(text); }catch (Exception e){ e.printStackTrace(); }
                                        if(mean == null){
                                            Toolkit.loadText(wv_mean, "正在网络上搜索...");
                                            Toolkit.checkBaiKe(text, tessHandler);
                                        }else{
                                            Toolkit.loadText(wv_mean, mean);
                                        }
                                    }
                                }
                            });
                            if(pos != -1){
                                ll_lines.addView(line_layout, pos, llp);
                            }else{
                                ll_lines.addView(line_layout, llp);
                            }
                        }
                        if(ll_mean.getVisibility()==View.GONE && ll_lines.getChildCount()>0){
                            ll_mean.setVisibility(View.VISIBLE);
                            pos = pos==-1?0:pos;
                            ll_lines.getChildAt(pos).performClick();
                        }
                        break;

                    case Toolkit.MSG_TESS_INIT_ERROR:
                        showMessageDialog("初始化失败!", true);
                        break;
                    case Toolkit.MSG_TESS_INIT_SUCCESS:
                        //启动相机
                        tessBaseAPI = (TessBaseAPI) msg.obj;
                        ll_btn_capture.setVisibility(View.VISIBLE);
                        requestCameraPermission();
                        break;
                    case Toolkit.MSG_BAIKE_SEARCH_RESULT:
                        if(msg.obj==null){
                            Toolkit.loadText(wv_mean, "未找到解释");
                            //拆分
                            ViewGroup line_layout = null;
                            int i;
                            for(i=0; i<ll_lines.getChildCount(); i++){
                                line_layout  = (ViewGroup) ll_lines.getChildAt(i);
                                if(line_layout.isSelected() && line_layout.getChildCount()>1){
                                    break;
                                }
                            }
                            if(line_layout!=null){
                                ll_mean.setVisibility(View.GONE);
                                ll_lines.removeView(line_layout);
                                String text = (String) line_layout.getTag();
                                String[] line = new String[text.length()];
                                char[] chars = text.toCharArray();
                                int ci = chars.length;
                                for(char c : chars){
                                    ci -= 1;
                                    line[ci] = c+"";
                                }
                                //返回一行的分词结果
                                msg = Message.obtain();
                                msg.what = MSG_TESS_RECOGNIZE_LINE;
                                msg.obj = line;
                                msg.arg1 = i;//在第几个位置插入
                                tessHandler.sendMessage(msg);
                            }
                        }else{
                            String[] res = (String[]) msg.obj;
                            Toolkit.loadText(wv_mean, res[1]);
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

        //初始化jieba
        new Thread(new Runnable() {
            @Override
            public void run() {
                try {
                    Toolkit.jiebaCut("字");
                } catch (Exception e) {
                    e.printStackTrace();
                }
            }
        }).start();

        tv_about.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                hideMenu();
                PackageManager manager;
                String version = "1.0";
                PackageInfo info = null;
                manager = getPackageManager();
                try {
                    info = manager.getPackageInfo(getPackageName(), 0);
                    version = info.versionName+"."+info.versionCode;
                    //info.packageName;
                    //info.signatures;
                } catch (PackageManager.NameNotFoundException e) {
                    e.printStackTrace();
                }
                String about = "<h1>懒人字典</h1>版本:"+version+"<br/>作者: planet2@qq.com<br/>微信:<br/><img src=\"file:///android_asset/wx.png\" /><br/><b><i>非本软件使用问题请勿扰</i></b>";
                loadText(wv_mean, about);
            }
        });

        tv_up_search.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                hideMenu();
                String about = "<h3>高级搜索</h1><b>如果文字识别有误，或者无法识别请使用此功能。</b><br/><i>高级搜索</i>使用免费版百度文字识别API，由于每天使用次数有限，请优先使用普通搜索！<br/> <a href=\"http://up.search\">点击这里<i>开始高级搜索</i></a>";
                loadText(wv_mean, about);
            }
        });
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
        ll_mean.setVisibility(View.GONE);
        iv_area.setImageResource(R.color.transparent);
        ll_lines.removeAllViews();
        drawCache = null;
        iv_capture.destroyDrawingCache();
        capture = null;
        fl_capture.setVisibility(View.GONE);
        fl_preview.setVisibility(View.VISIBLE);
        isPreview = true;
        animDot.stop();
        iv_switch_red.setImageDrawable(animDotSlow);
        animDotSlow.start();
        iv_noise.setVisibility(View.GONE);
        iv_noise.setVisibility(View.GONE);
    }

    /**
     * 切换到截屏视图
     */
    private void changeStateCapture(){
        Log.d(TAG, "changeStateCapture");
        fl_capture.setVisibility(View.VISIBLE);
        fl_preview.setVisibility(View.GONE);
        isPreview = false;
        animDotSlow.stop();
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
                iv_area.post(new Runnable() {
                    @Override
                    public void run() {
                        Rect visibleRect = Toolkit.getLocationInParent(iv_area, fl_preview);
                        bitmapRect = Bitmap.createBitmap(iv_capture.getDrawingCache(), visibleRect.left, visibleRect.top, visibleRect.width(), visibleRect.height());
                        iv_area.setImageBitmap(bitmapRect);
                        //移动电视到顶部
                        Rect tvRect = Toolkit.getLocationInParent(sl_clip_rect.getChildAt(0), sl_clip_rect);
                        tvRectTop = tvRect.top+Toolkit.dip2px(CameraActivity.this, 40);
                        sl_clip_rect.smoothScrollTo(0, tvRectTop, 800);
                        Toolkit.tessRecognize(tessBaseAPI, bitmapRect, tessHandler);
                        //iv_test.setImageBitmap(rect);
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
        if(messageDialog==null){
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

    private void showMessageDialog(String errorMsg, final boolean isError){
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

    private void showMenu(){
        //显示
        ObjectAnimator objectAnimator = ObjectAnimator.ofFloat(ll_menu, "alpha", 0f, 1f);
        objectAnimator.setDuration(400);
        objectAnimator.start();
    }

    private void hideMenu(){
        //隐藏
        ObjectAnimator objectAnimator = ObjectAnimator.ofFloat(ll_menu, "alpha", 1f, 0f);
        objectAnimator.setDuration(400);
        objectAnimator.start();
    }

    @Override
    protected void onDestroy() {
        SpotManager.getInstance(this).onAppExit();
        super.onDestroy();
    }
}
