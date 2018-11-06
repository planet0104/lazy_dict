package cn.jy.lazydict;

import android.content.Context;
import android.os.Build;
import android.support.annotation.Nullable;
import android.support.annotation.RequiresApi;
import android.util.AttributeSet;
import android.view.LayoutInflater;
import android.widget.LinearLayout;
import android.widget.TextView;

public class PinYinTextView extends LinearLayout {
    TextView tv_pinyin;
    TextView tv_zi;

    public PinYinTextView(Context context) {
        super(context);
        init();
    }

    public PinYinTextView(Context context, @Nullable AttributeSet attrs) {
        super(context, attrs);
        init();
    }

    public PinYinTextView(Context context, @Nullable AttributeSet attrs, int defStyleAttr) {
        super(context, attrs, defStyleAttr);
        init();
    }

    @RequiresApi(api = Build.VERSION_CODES.LOLLIPOP)
    public PinYinTextView(Context context, AttributeSet attrs, int defStyleAttr, int defStyleRes) {
        super(context, attrs, defStyleAttr, defStyleRes);
        init();
    }

    private void init() {
        LayoutInflater.from(getContext()).inflate(R.layout.v_pinyin, this, true);
        tv_pinyin = findViewById(R.id.tv_pinyin);
        tv_zi = findViewById(R.id.tv_zi);
    }

    public void setText(String text){
        tv_zi.setText(text);
    }

    public void setPinyin(String text){
        tv_pinyin.setText(text);
    }
}
