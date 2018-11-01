package cn.jy.lazydict;

import android.app.Activity;
import android.graphics.Rect;
import android.hardware.Camera;
import android.util.Log;
import android.view.Surface;
import android.view.SurfaceView;

import java.util.ArrayList;
import java.util.List;

public class CameraUtils {
    static final String TAG = CameraUtils.class.getSimpleName();

    /**
     * 连续自动对焦
     * @param camera
     */
    public static void setContinuallyAutoFocus(Camera camera){
        if(camera!=null){
            Camera.Parameters params = camera.getParameters();
            params.setFocusMode(Camera.Parameters.FOCUS_MODE_CONTINUOUS_PICTURE);
            camera.setParameters(params);
        }
    }

    /**
     * 区域聚焦
     * @param camera
     * @param rect
     * @param callback
     */
    public static void focusOnRect(Camera camera, Rect rect, Camera.AutoFocusCallback callback) {
        if (camera != null) {
            Camera.Parameters parameters = camera.getParameters(); // 先获取当前相机的参数配置对象
            parameters.setFocusMode(Camera.Parameters.FOCUS_MODE_AUTO); // 设置聚焦模式
            if (parameters.getMaxNumFocusAreas() > 0) {
                List<Camera.Area> focusAreas = new ArrayList<>();
                focusAreas.add(new Camera.Area(rect, 1000));
                parameters.setFocusAreas(focusAreas);
            }
            camera.cancelAutoFocus(); // 先要取消掉进程中所有的聚焦功能
            camera.setParameters(parameters); // 一定要记得把相应参数设置给相机
            camera.autoFocus(callback);
        }else{
            callback.onAutoFocus(false, camera);
        }
    }

    /**
     * 点击聚焦
     * @param camera
     * @param surfaceView
     * @param x
     * @param y
     * @param callback
     */
    public static void focusOnTouch(Camera camera, SurfaceView surfaceView, int x, int y, Camera.AutoFocusCallback callback) {
        Rect rect = new Rect(x - 100, y - 100, x + 100, y + 100);
        int left = rect.left * 2000 / surfaceView.getWidth() - 1000;
        int top = rect.top * 2000 / surfaceView.getHeight() - 1000;
        int right = rect.right * 2000 / surfaceView.getWidth() - 1000;
        int bottom = rect.bottom * 2000 / surfaceView.getHeight() - 1000;
        // 如果超出了(-1000,1000)到(1000, 1000)的范围，则会导致相机崩溃
        left = left < -1000 ? -1000 : left;
        top = top < -1000 ? -1000 : top;
        right = right > 1000 ? 1000 : right;
        bottom = bottom > 1000 ? 1000 : bottom;
        focusOnRect(camera, new Rect(left, top, right, bottom), callback);
    }

    /**
     * 根据预览View大小设置最合适相机预览大小
     * @param previewWidth
     * @param previewHeight
     */
    public static int[] setPreviewSize(Camera camera, int degrees, int previewWidth , int previewHeight){
        Camera.Parameters parameters = camera.getParameters();
        List<Camera.Size> sizes = parameters.getSupportedPreviewSizes();
        Camera.Size choose = null;
        for(Camera.Size size: sizes){
            Log.d(TAG, size.width+","+size.height);
            int max = Math.max(previewHeight, previewWidth);
            if(size.width<=max && size.height<=max){
                choose = size;
                break;
            }
        }
        if(choose == null){
            choose = sizes.get(0);
        }
        if(degrees==0 || degrees==180){
            parameters.setPreviewSize(choose.width, choose.height);
            return new int[]{choose.width, choose.height};
        }else{
            parameters.setPreviewSize(choose.height, choose.width);
            return new int[]{choose.height, choose.width};
        }
    }

    /**
     * 设置相机方向
     * @param activity
     * @param cameraId
     * @param camera
     * @return degrees
     */
    public static int setCameraDisplayOrientation(Activity activity, int cameraId, Camera camera) {
        android.hardware.Camera.CameraInfo info =
                new android.hardware.Camera.CameraInfo();
        android.hardware.Camera.getCameraInfo(cameraId, info);
        int rotation = activity.getWindowManager().getDefaultDisplay()
                .getRotation();
        int degrees = 0;
        switch (rotation) {
            case Surface.ROTATION_0: degrees = 0; break;
            case Surface.ROTATION_90: degrees = 90; break;
            case Surface.ROTATION_180: degrees = 180; break;
            case Surface.ROTATION_270: degrees = 270; break;
        }

        int result;
        if (info.facing == Camera.CameraInfo.CAMERA_FACING_FRONT) {
            result = (info.orientation + degrees) % 360;
            result = (360 - result) % 360;  // compensate the mirror
        } else {  // back-facing
            result = (info.orientation - degrees + 360) % 360;
        }
        camera.setDisplayOrientation(result);
        Log.d(TAG, "degrees="+degrees);
        return degrees;
    }
}
