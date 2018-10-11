package cn.jy.lazydict;

import android.app.NativeActivity;
import android.util.Log;

public class WinitActivity extends NativeActivity {
    static final String TAG = WinitActivity.class.getSimpleName();
    @Override
    protected void onResume() {
        super.onResume();
        Log.d(TAG, "onResume!!!!!!!!");
    }
}
