package cn.jy.lazydict;

import android.app.Activity;
import android.content.Context;
import android.content.Intent;
import android.content.pm.ApplicationInfo;
import android.content.pm.PackageInfo;
import android.content.pm.PackageManager;
import android.content.pm.Signature;
import android.os.Bundle;
import android.os.Handler;
import android.os.Message;
import android.util.Log;
import android.widget.Toast;

import com.google.android.gms.ads.AdRequest;
import com.google.android.gms.ads.AdView;
import com.google.android.gms.ads.MobileAds;

import java.text.SimpleDateFormat;
import java.util.Date;

import static cn.jy.lazydict.Toolkit.MSG_TESS_INIT_SUCCESS;

public class SplashActivity extends Activity{
    static final String TAG = "SplashActivity1";
    private AdView mAdView0;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        //--------------- 加固 ----------------------
//        if((getApplicationInfo().flags&=ApplicationInfo.FLAG_DEBUGGABLE) !=0){
//            finish();
//            return;
//        }
        //-------------------------------------------
        super.onCreate(savedInstanceState);

        Log.d(TAG, "获取到 getApplicationInfo().flags="+getApplicationInfo().flags);
        Log.d(TAG, "获取到 ApplicationInfo.FLAG_DEBUGGABLE="+ApplicationInfo.FLAG_DEBUGGABLE);

        try {
            Toolkit.jiebaCut(this, "字");
        } catch (Exception e) {
            Log.d(TAG, "获取到 出错！"+e.getStackTrace());
            e.printStackTrace();
        }

        //        if((getApplicationInfo().flags&=ApplicationInfo.FLAG_DEBUGGABLE) !=0){
//            Log.d(TAG, "禁止调试!");
//            showMessageDialog("禁止调试", true);
//            return;
//        }
        int pid = android.os.Process.myPid();
        Log.d("哈哈", "status pid="+pid);
        //初始化广告
        MobileAds.initialize(this, BuildConfig.AD_APP_ID);
        setContentView(R.layout.activity_splash);
        mAdView0 = findViewById(R.id.adView0);

        //初始化
        Toolkit.initTessTwo(this, new Handler(new Handler.Callback() {
            @Override
            public boolean handleMessage(Message message) {
                if(message.what == MSG_TESS_INIT_SUCCESS){
                }else{
                    Toast.makeText(SplashActivity.this, "初始化失败!", Toast.LENGTH_LONG).show();
                }
                return false;
            }
        }));
        initJieBa();
        mAdView0.loadAd(getAdRequest());

        //至少显示5s
        mAdView0.postDelayed(new Runnable() {
            @Override
            public void run() {
                startActivity(new Intent(SplashActivity.this, CameraActivity.class));
                finish();
            }
        }, 5000);
    }

    private AdRequest getAdRequest(){
        AdRequest.Builder builder = new AdRequest.Builder();
        if(BuildConfig.DEBUG){
            builder.addTestDevice("B12B1D2E164FE99C3E11BF3FD7640FD8");
        }
        return builder.build();
    }

    private void initJieBa(){
        //初始化jieba
        new Thread(new Runnable() {
            @Override
            public void run() {
                try {
                    Toolkit.jiebaCut(SplashActivity.this,"字");
                } catch (Exception e) {
                    runOnUiThread(new Runnable() {
                        @Override
                        public void run() {
                            Toast.makeText(SplashActivity.this, "初始化失败!", Toast.LENGTH_LONG).show();
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
}
