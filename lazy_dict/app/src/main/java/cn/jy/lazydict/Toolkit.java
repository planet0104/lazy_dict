package cn.jy.lazydict;

import android.content.Context;
import android.graphics.Bitmap;
import android.graphics.RectF;

import java.util.ArrayList;
import java.util.List;

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
     * @param x
     * @param y
     * @return
     * @throws Exception
     */
    public static native ThresholdGray calcThreshold(Bitmap bitmap) throws Exception;

    /**
     * 分组
     * @param list
     * @return
     */
    public static List<List<RectF>> groupRects(List<RectF> list){
        List<List<RectF>> result = new ArrayList<>();
        while(list.size()>0){
            RectF rect = list.remove(0);
            if(result.size()>0){
                for(List<RectF> subs : result){
                    RectF first = subs.get(0);
                    RectF last = subs.get(subs.size()-1);
                    if(isClose(rect, first) || isClose(rect, last) ){
                        subs.add(rect);
                        continue;
                    }
                }
            }else{
                List<RectF> n = new ArrayList<>();
                n.add(rect);
                result.add(n);
            }
        }
        return result;
    }

    /**
     * 判断两个Rect是否距离很近(中心距离不超过width*1.2)
     * @param r1
     * @param r2
     * @return
     */
    private static boolean isClose(RectF r1, RectF r2){
        double width = r1.right-r1.left;
        double x1 = (r1.right-r1.left)/2.0;
        double y1 = (r1.bottom-r1.top)/2.0;
        double x2 = (r2.right-r2.left)/2.0;
        double y2 = (r2.bottom-r2.top)/2.0;
        double distance = Math.abs(Math.sqrt(((x1-x2)*(x1-x2)+(y1-y2)*(y1-y2))));
        return distance<width*1.2;
    }

    public static int dip2px(Context context, float dipValue){
        final float scale = context.getResources().getDisplayMetrics().density;
        return (int)(dipValue * scale + 0.5f);
    }
}
