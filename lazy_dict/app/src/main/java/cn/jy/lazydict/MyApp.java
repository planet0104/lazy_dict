package cn.jy.lazydict;

import android.app.Activity;
import android.app.Application;
import android.content.pm.ApplicationInfo;
import android.util.Log;

import com.googlecode.tesseract.android.TessBaseAPI;

import java.io.File;

public class MyApp extends Application {
    public static final String SPF_KEY_LAST_OPEN = "lastOpen";
    public static final String SPF_KEY_SHOW_DAY = "showDay";

    private static TessBaseAPI tessBaseAPI;
    static final String TAG = MyApp.class.getSimpleName();
    @Override
    public void onCreate() {
        super.onCreate();
    }

    public static synchronized TessBaseAPI getTessApi(Activity activity){
        File tessDataDir = new File(activity.getFilesDir(), "tessdata");
        try{
            if(!tessDataDir.exists()){
                if(FileUtils.unpackZip(activity.getAssets().open("tessdata.zip"), activity.getFilesDir(), null)){
                    Log.d(TAG, "tessdata解压成功");
                }else{
                    Log.e(TAG, "tessdata解压失败");
                    return null;
                }
            }else{
                Log.e(TAG, "tessdata已经存在");
            }
        }catch (Exception e){
            e.printStackTrace();
            return null;
        }

        //初始化 TessBaseAPI
        boolean tessInit;
        if(tessBaseAPI == null){
            tessBaseAPI = new TessBaseAPI();
            Log.d(TAG, "版本:"+tessBaseAPI.getVersion());
            tessInit = tessBaseAPI.init(activity.getFilesDir().getAbsolutePath(), "chi_sim");
        }else{
            tessInit = true;
        }
        if(tessInit){
            return tessBaseAPI;
        }else{
            return null;
        }
    }

    public static boolean isTessBaseAPIInitialized() {
        return tessBaseAPI!=null;
    }
}
