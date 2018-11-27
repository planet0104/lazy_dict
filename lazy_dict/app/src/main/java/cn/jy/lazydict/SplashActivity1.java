package cn.jy.lazydict;

import android.app.Activity;
import android.os.Build;
import android.os.Bundle;
import android.os.Handler;
import android.os.Message;
import android.util.Log;
import android.view.ViewGroup;
import android.view.Window;
import android.view.WindowManager;
import android.widget.RelativeLayout;
import android.widget.Toast;

import java.text.SimpleDateFormat;
import java.util.Date;

import cds.sdg.sdf.AdManager;
import cds.sdg.sdf.nm.cm.ErrorCode;
import cds.sdg.sdf.nm.sp.SplashViewSettings;
import cds.sdg.sdf.nm.sp.SpotListener;
import cds.sdg.sdf.nm.sp.SpotManager;
import cds.sdg.sdf.nm.sp.SpotRequestListener;

import static cn.jy.lazydict.Toolkit.MSG_TESS_INIT_SUCCESS;

public class SplashActivity1 extends Activity{
    static final String TAG = "SplashActivity1";

    private PermissionHelper mPermissionHelper;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        // 设置全屏
        getWindow().setFlags(WindowManager.LayoutParams.FLAG_FULLSCREEN, WindowManager.LayoutParams.FLAG_FULLSCREEN);
        // 移除标题栏
        requestWindowFeature(Window.FEATURE_NO_TITLE);

        // 当系统为6.0以上时，需要申请权限
        mPermissionHelper = new PermissionHelper(this);
        mPermissionHelper.setOnApplyPermissionListener(new PermissionHelper.OnApplyPermissionListener() {
            @Override
            public void onAfterApplyAllPermission() {
                Log.i(TAG, "All of requested permissions has been granted, so run app logic.");
                runApp();
            }
        });
        if (Build.VERSION.SDK_INT < 23) {
            // 如果系统版本低于23，直接跑应用的逻辑
            Log.d(TAG, "The api level of system is lower than 23, so run app logic directly.");
            runApp();
        } else {
            // 如果权限全部申请了，那就直接跑应用逻辑
            if (mPermissionHelper.isAllRequestedPermissionGranted()) {
                Log.d(TAG, "All of requested permissions has been granted, so run app logic directly.");
                runApp();
            } else {
                // 如果还有权限为申请，而且系统版本大于23，执行申请权限逻辑
                Log.i(TAG, "Some of requested permissions hasn't been granted, so apply permissions first.");
                mPermissionHelper.applyPermissions();
            }
        }
    }

    /**
     * 预加载广告
     */
    private void preloadAd() {
        // 注意：不必每次展示插播广告前都请求，只需在应用启动时请求一次
        SpotManager.getInstance(this).requestSpot(new SpotRequestListener() {
            @Override
            public void onRequestSuccess() {
                Log.d(TAG, "请求插屏广告成功");
            }

            @Override
            public void onRequestFailed(int errorCode) {
                Log.e(TAG, "请求插屏广告失败，errorCode"+errorCode);
                switch (errorCode) {
                    case ErrorCode.NON_NETWORK:
                        showShortToast("网络异常");
                        break;
                    case ErrorCode.NON_AD:
                        Log.d(TAG, "暂无插屏广告");
                        break;
                    default:
                        Log.d(TAG, "请稍后再试");
                        break;
                }
            }
        });
    }

    private void runApp(){
        AdManager.getInstance(this).init("88000ee36d1eefd2", "711a3e0ee8f99d09", true);
        SpotManager.getInstance(this).requestSpot(new SpotRequestListener() {
            @Override
            public void onRequestSuccess() {
                Log.d(TAG, "开屏广告预加载成功.");
            }

            @Override
            public void onRequestFailed(int i) {
                Log.e(TAG, "开屏广告预加载失败."+i);
            }
        });
        setupSplashAd();

        try {
            Toolkit.jiebaCut(this, "字");
        } catch (Exception e) {
            Log.d(TAG, "获取到 出错！"+e.getStackTrace());
            e.printStackTrace();
        }

        //初始化
        Toolkit.initTessTwo(this, new Handler(new Handler.Callback() {
            @Override
            public boolean handleMessage(Message message) {
                if(message.what == MSG_TESS_INIT_SUCCESS){
                }else{
                    Toast.makeText(SplashActivity1.this, "初始化失败!", Toast.LENGTH_LONG).show();
                }
                return false;
            }
        }));
        initJieBa();
    }

    /**
     * 设置开屏广告
     */
    private void setupSplashAd() {
        // 创建开屏容器
        final RelativeLayout splashLayout = findViewById(R.id.rl_splash);
        RelativeLayout.LayoutParams params =
                new RelativeLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT);
        params.addRule(RelativeLayout.ABOVE, R.id.view_divider);

        // 对开屏进行设置
        SplashViewSettings splashViewSettings = new SplashViewSettings();
        //		// 设置是否展示失败自动跳转，默认自动跳转
        //		splashViewSettings.setAutoJumpToTargetWhenShowFailed(false);
        // 设置跳转的窗口类
        splashViewSettings.setTargetClass(CameraActivity.class);
        // 设置开屏的容器
        splashViewSettings.setSplashViewContainer(splashLayout);

        // 展示开屏广告
        SpotManager.getInstance(this)
                .showSplash(this, splashViewSettings, new SpotListener() {

                    @Override
                    public void onShowSuccess() {
                        Log.d(TAG, "开屏展示成功");
                    }

                    @Override
                    public void onShowFailed(int errorCode) {
                        Log.d(TAG, "开屏展示失败");
                        switch (errorCode) {
                            case ErrorCode.NON_NETWORK:
                                Log.e(TAG, "网络异常");
                                break;
                            case ErrorCode.NON_AD:
                                Log.e(TAG, "暂无开屏广告");
                                break;
                            case ErrorCode.RESOURCE_NOT_READY:
                                Log.e(TAG, "开屏资源还没准备好");
                                break;
                            case ErrorCode.SHOW_INTERVAL_LIMITED:
                                Log.e(TAG, "开屏展示间隔限制");
                                break;
                            case ErrorCode.WIDGET_NOT_IN_VISIBILITY_STATE:
                                Log.e(TAG, "开屏控件处在不可见状态");
                                break;
                            default:
                                Log.e(TAG, "errorCode:"+errorCode);
                                break;
                        }
                    }

                    @Override
                    public void onSpotClosed() {
                        Log.d(TAG,"开屏被关闭");
                    }

                    @Override
                    public void onSpotClicked(boolean isWebPage) {
                        Log.d(TAG,"开屏被点击");
                        Log.d(TAG, "是否是网页广告？ "+(isWebPage ? "是" : "不是"));
                    }
                });
    }

    private void initJieBa(){
        //初始化jieba
        new Thread(new Runnable() {
            @Override
            public void run() {
                try {
                    Toolkit.jiebaCut(SplashActivity1.this,"字");
                } catch (Exception e) {
                    runOnUiThread(new Runnable() {
                        @Override
                        public void run() {
                            Toast.makeText(SplashActivity1.this, "初始化失败!", Toast.LENGTH_LONG).show();
                            System.exit(0);
                        }
                    });
                    e.printStackTrace();
                }
            }
        }).start();
    }

    private void showAdd(){
        //        try {
//            PackageManager packageManager = getApplicationContext().getPackageManager();
//            PackageInfo packageInfo = packageManager.getPackageInfo(this.getPackageName(), 0);
//            //应用装时间
//            long firstInstallTime = packageInfo.firstInstallTime;
//            long installTime = System.currentTimeMillis()-firstInstallTime;
//            if(installTime>1000*60*60*24*3){
//                //3天以后弹出广告
//                //弹出广告每天显示一次
//                String showDay = SharedPreferencesHelper.getString(this, MyApp.SPF_KEY_SHOW_DAY);
//                String today = SimpleDateFormat.getDateInstance().format(new Date());
//                if(!today.equals(showDay)){
//                    //显示广告
//                    adViewBanner.setVisibility(View.VISIBLE);
//                    adViewBanner.post(new Runnable() {
//                        @Override
//                        public void run() {
//                            AdRequest adRequest = new AdRequest.Builder()
//                                    .addTestDevice("B12B1D2E164FE99C3E11BF3FD7640FD8")
//                                    .build();
//                            adViewBanner.loadAd(adRequest);
//                        }
//                    });
//                    SharedPreferencesHelper.saveString(this, MyApp.SPF_KEY_SHOW_DAY, today);
//                }
//            }
//        } catch (PackageManager.NameNotFoundException e) {
//            e.printStackTrace();
//        }
        //3天以后弹出广告
        //弹出广告每天显示一次
        String showDay = SharedPreferencesHelper.getString(this, MyApp.SPF_KEY_SHOW_DAY);
        SharedPreferencesHelper.saveString(this, MyApp.SPF_KEY_SHOW_DAY, null);//测试！！！！
        final String today = SimpleDateFormat.getDateInstance().format(new Date());
//        if(!today.equals(showDay)){
//            ll_ad_view_banner.setVisibility(View.VISIBLE);
//            ll_ad_view_banner.post(new Runnable() {
//                @Override
//                public void run() {
//                    //显示广告
//                    adViewBanner.post(new Runnable() {
//                        @Override
//                        public void run() {
//                            AdRequest adRequest = new AdRequest.Builder()
//                                    .addTestDevice("B12B1D2E164FE99C3E11BF3FD7640FD8")
//                                    .build();
//                            adViewBanner.loadAd(adRequest);
//                        }
//                    });
//                    SharedPreferencesHelper.saveString(CameraActivity.this, MyApp.SPF_KEY_SHOW_DAY, today);
//                }
//            });
//        }
    }

    /**
     * 展示短时Toast
     *
     * @param format
     * @param args
     */
    protected void showShortToast(String format, Object... args) {
        showToast(Toast.LENGTH_SHORT, format, args);
    }

    /**
     * 展示Toast
     *
     * @param duration
     * @param format
     * @param args
     */
    private void showToast(int duration, String format, Object... args) {
        Toast.makeText(this, String.format(format, args), duration).show();
    }
}
