package cn.jy.lazydict;

public class RustApp {
    static {
        System.loadLibrary("lazy_dict");
    }

    /**
     * YUV420SP转colors
     * @param data
     * @param width
     * @param height
     * @param cameraOrientation
     * @return
     */
    public static native int[] decodeYUV420SP(byte[] data, int width, int height, int cameraOrientation);
}
