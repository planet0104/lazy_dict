package cn.jy.lazydict;

import android.app.Activity;
import android.hardware.Camera;
import android.util.Log;
import android.view.Surface;

import java.util.List;

public class CameraUtils {
    static final String TAG = CameraUtils.class.getSimpleName();

    /**
     * 连续自动对焦
     * @param camera
     */
    public static void setContinuallyAutoFocus(Camera camera){
        Camera.Parameters params = camera.getParameters();
        params.setFocusMode(Camera.Parameters.FOCUS_MODE_CONTINUOUS_PICTURE);
        camera.setParameters(params);
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
