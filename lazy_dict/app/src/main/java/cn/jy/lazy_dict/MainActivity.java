package cn.jy.lazy_dict;

import org.liballeg.android.AllegroActivity;

public class MainActivity extends AllegroActivity {
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
}
