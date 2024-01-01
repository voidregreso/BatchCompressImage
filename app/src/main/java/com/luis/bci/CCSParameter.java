package com.luis.bci;

public class CCSParameter {
    public boolean keep_metadata;
    public int jpeg_qu;
    public ChromaSubsampling subsamp_mode;
    public int png_qu;
    public boolean png_force_zopfli;
    public int webp_qu;
    public boolean optm;
    public int width;
    public int height;

    public enum ChromaSubsampling {
        CS444,
        CS422,
        CS420,
        CS411,
        Auto
    }

    public CCSParameter(boolean keep_metadata, int jpeg_qu, ChromaSubsampling subsamp_mode, int png_qu,
            boolean png_force_zopfli, int webp_qu, boolean optm, int width, int height) {
        this.keep_metadata = keep_metadata;
        this.jpeg_qu = jpeg_qu;
        this.subsamp_mode = subsamp_mode;
        this.png_qu = png_qu;
        this.png_force_zopfli = png_force_zopfli;
        this.webp_qu = webp_qu;
        this.optm = optm;
        this.width = width;
        this.height = height;
    }

}