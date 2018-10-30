package cn.jy.lazydict;

import android.content.Context;
import android.graphics.Bitmap;
import android.util.AttributeSet;
import android.util.Log;
import android.view.MotionEvent;

public class CaptureView extends android.support.v7.widget.AppCompatImageView {
    static final String TAG = CaptureView.class.getSimpleName();

    public CaptureView(Context context) {
        super(context);
        init();
    }

    public CaptureView(Context context, AttributeSet attrs) {
        super(context, attrs);
        init();
    }

    public CaptureView(Context context, AttributeSet attrs, int defStyleAttr) {
        super(context, attrs, defStyleAttr);
        init();
    }

    private void init(){
        setDrawingCacheEnabled(true);
    }

    @Override
    public boolean onTouchEvent(MotionEvent event) {
        if(event.getAction() == MotionEvent.ACTION_DOWN){
            addCharacter(event.getX(), event.getY());
        }

        return super.onTouchEvent(event);
    }

    private void addCharacter(float x, float y){
        Bitmap bitmap = getDrawingCache();
        Log.d(TAG, "x="+x+" y="+y+" bitmap.width="+bitmap.getWidth());
    }
}
