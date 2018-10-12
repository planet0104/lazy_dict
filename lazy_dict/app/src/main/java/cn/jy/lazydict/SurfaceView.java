package cn.jy.lazydict;

import android.content.Context;
import android.graphics.Bitmap;
import android.graphics.Canvas;
import android.support.annotation.Nullable;
import android.util.AttributeSet;
import android.view.View;

class SurfaceView extends View {
    public SurfaceView(Context context) {
        super(context);
    }

    public SurfaceView(Context context, @Nullable AttributeSet attrs) {
        super(context, attrs);
    }

    public SurfaceView(Context context, @Nullable AttributeSet attrs, int defStyleAttr) {
        super(context, attrs, defStyleAttr);
    }

    public SurfaceView(Context context, @Nullable AttributeSet attrs, int defStyleAttr, int defStyleRes) {
        super(context, attrs, defStyleAttr, defStyleRes);
    }

    private static native void drawFrame(Bitmap bitmap);

    private Bitmap bmp;

    @Override protected void onDraw(Canvas canvas) {
        if (bmp==null || bmp.getWidth() != canvas.getWidth() || bmp.getHeight() != canvas.getHeight()) {
            bmp = Bitmap.createBitmap(canvas.getWidth(), canvas.getHeight(), Bitmap.Config.RGB_565);
        }
        long t = System.currentTimeMillis();
        drawFrame(bmp);
        System.out.println("绘图耗时:"+(System.currentTimeMillis()-t)+"ms");
        canvas.drawBitmap(bmp, 0, 0, null);
        //invalidate(); /* force redreaw */
    }
}