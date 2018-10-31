package cn.jy.lazydict;

import android.content.Context;
import android.graphics.Bitmap;
import android.graphics.Canvas;
import android.graphics.DashPathEffect;
import android.graphics.Paint;
import android.graphics.RectF;
import android.util.AttributeSet;
import android.util.Log;
import android.view.MotionEvent;
import android.view.View;
import android.widget.ImageView;

public class CaptureView extends ImageView {
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
        try {
            long t = System.currentTimeMillis();
            RectF rect = Toolkit.getCharacterRect(bitmap, (int)x, (int)y);
            Log.d(TAG, "x="+x+" y="+y+" bitmap.width="+bitmap.getWidth()+" rect="+rect.toShortString()+" 耗时:"+(System.currentTimeMillis()-t)+"ms");
            Canvas canvas = new Canvas(bitmap);
            Paint paint = new Paint();
            paint.setStyle(Paint.Style.STROKE);
            paint.setStrokeWidth(dip2px(5));
            paint.setColor(0x7fff0000);
            canvas.drawRoundRect(rect, dip2px(5), dip2px(5), paint);
            paint.setColor(0x7fffffff);
            paint.setPathEffect(new DashPathEffect(new float[]{dip2px(10), dip2px(10)}, 0));
            canvas.drawRoundRect(rect, dip2px(5), dip2px(5), paint);
            setImageBitmap(bitmap);


        } catch (Exception e) {
            e.printStackTrace();
            CameraActivity.toast(getContext(), e.getMessage());
        }
    }

    public int dip2px(float dipValue){
        final float scale = getContext().getResources().getDisplayMetrics().density;
        return (int)(dipValue * scale + 0.5f);
    }
}
