package cn.jy.lazydict;

import android.app.Activity;
import android.os.Bundle;
import android.support.annotation.Nullable;
import android.widget.FrameLayout;

import cds.sdg.sdf.AdManager;
import cds.sdg.sdf.nm.sp.SplashViewSettings;
import cds.sdg.sdf.nm.sp.SpotManager;
import cds.sdg.sdf.nm.sp.SpotRequestListener;

public class StartActivity extends Activity {
    FrameLayout ll_root;
    @Override
    protected void onCreate(@Nullable Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_start);
        ll_root = findViewById(R.id.ll_root);

        //有米广告发布ID： 88000ee36d1eefd2
        //有米广告应用密钥：711a3e0ee8f99d09
        AdManager.getInstance(this).init("88000ee36d1eefd2", "711a3e0ee8f99d09", BuildConfig.DEBUG);
        //预加载开屏广告
        SpotManager.getInstance(this).requestSpot(new SpotRequestListener() {
            @Override
            public void onRequestSuccess() {

            }

            @Override
            public void onRequestFailed(int i) {

            }
        });

        SplashViewSettings splashViewSettings = new SplashViewSettings();
        //设置展示失败是否自动跳转至设定的窗口 默认自动跳转
        splashViewSettings.setAutoJumpToTargetWhenShowFailed(false);
        //设置结束开屏后跳转的窗口
        splashViewSettings.setTargetClass(CameraActivity.class);
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();
        SpotManager.getInstance(this).onDestroy();
    }
}
