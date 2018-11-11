package cn.jy.lazydict;

import android.app.Activity;
import android.content.Intent;
import android.os.Bundle;
import android.os.Handler;
import android.os.Message;
import android.widget.Toast;

import com.google.android.gms.ads.AdRequest;
import com.google.android.gms.ads.AdView;
import com.google.android.gms.ads.MobileAds;

import java.text.SimpleDateFormat;
import java.util.Date;

import static cn.jy.lazydict.Toolkit.MSG_TESS_INIT_SUCCESS;
import static com.google.android.gms.ads.AdSize.LARGE_BANNER;

public class SplashActivity extends Activity{
    private long startTime = 0;
    private AdView mAdView0;
    private AdView mAdView1;
    private AdView mAdView2;
    private AdView mAdView3;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        startTime = System.currentTimeMillis();
        //初始化广告
        MobileAds.initialize(this, BuildConfig.AD_APP_ID);
        setContentView(R.layout.activity_splash);
        mAdView0 = findViewById(R.id.adView0);
        mAdView1 = findViewById(R.id.adView1);
        mAdView2 = findViewById(R.id.adView2);
        mAdView3 = findViewById(R.id.adView3);
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
        mAdView1.loadAd(getAdRequest());
        mAdView2.loadAd(getAdRequest());
        mAdView3.loadAd(getAdRequest());

        //清空计时器
        SharedPreferencesHelper.saveLong(this, "SPF_KEY_LAST_OPEN", System.currentTimeMillis());

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
                    Toolkit.jiebaCut("字");
                } catch (Exception e) {
                    Toast.makeText(SplashActivity.this, "初始化失败!", Toast.LENGTH_LONG).show();
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
