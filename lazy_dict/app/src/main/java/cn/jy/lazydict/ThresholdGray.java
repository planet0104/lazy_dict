package cn.jy.lazydict;

/**
     * 图片的阈值和灰度图
     */
    public class ThresholdGray{
        /**
         * 阈值0~255
         */
        public final int threshold;
        public final int width;
        public final int height;
        public final int bpp;
        /**
         * 像素灰度值 每个字节代表一个像素灰度值0~255
         */
        public final byte[] grays;
        public ThresholdGray(int threshold, int width, int height, int bpp, byte[] grays) {
            this.threshold = threshold;
            this.width = width;
            this.height = height;
            this.bpp = bpp;
            this.grays = grays;
        }
    }