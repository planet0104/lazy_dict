package cn.jy.lazydict;

import android.content.Context;
import android.graphics.Bitmap;
import android.graphics.Rect;
import android.graphics.RectF;
import android.view.View;
import android.view.ViewGroup;

public class Toolkit {
    static {
        System.loadLibrary("lazy_dict");
    }

    /**
     * YUV420SP转Bitmap
     * @param data
     * @param width
     * @param height
     * @param cameraOrientation
     * @return
     */
    public static native Bitmap decodeYUV420SP(byte[] data, int width, int height, int cameraOrientation) throws Exception;

    /**
     * 根据坐标选择一个文字块
     * @param tg
     * @param x
     * @param y
     * @return
     * @throws Exception
     */
    public static native RectF getCharacterRect(ThresholdGray tg, int x, int y) throws Exception;

    /**
     * 计算阈值和灰度图
     * @param bitmap
     * @return
     * @throws Exception
     */
    public static native ThresholdGray calcThreshold(Bitmap bitmap) throws Exception;

    /**
     * 二值化
     * @param bitmap
     * @return
     * @throws Exception
     */
    public static native void binary(Bitmap bitmap) throws Exception;

    public static native RectF[] split(Bitmap bitmap) throws Exception;

    public static native String[] jiebaCut(String text) throws Exception;

    public static Rect getLocationInParent(View view, ViewGroup parent){
        int[] loc = new int[2];
        view.getLocationInWindow(loc);
        int[] locP = new int[2];
        parent.getLocationInWindow(locP);
        int left = loc[0];
        int top = loc[1]-locP[1];
        return new Rect(left, top, left+view.getMeasuredWidth(), top+view.getMeasuredHeight());
    }

    /**
     * 输入的字符是否是汉字
     * @param a char
     * @return boolean
     */
    public static boolean isChinese(char a) {
        int v = (int)a;
        return (v >=19968 && v <= 171941);
    }

    public static int dip2px(Context context, float dipValue){
        final float scale = context.getResources().getDisplayMetrics().density;
        return (int)(dipValue * scale + 0.5f);
    }
}
