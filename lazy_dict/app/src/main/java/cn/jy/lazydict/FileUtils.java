package cn.jy.lazydict;

import android.content.Context;

import java.io.BufferedInputStream;
import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.zip.ZipEntry;
import java.util.zip.ZipInputStream;

public class FileUtils {

    public static boolean unpackZip(InputStream zipFile, File path, android.os.Handler handler){
        ZipInputStream zis;
        try {
            String filename;
            zis = new ZipInputStream(new BufferedInputStream(zipFile));
            int totalBytes = 42096342;
            int current = 0;
            ZipEntry ze;
            byte[] buffer = new byte[1024];
            int count;

            while ((ze = zis.getNextEntry()) != null) {
                //写入文件
                filename = ze.getName();

                //如果不存在则需要创建目录，否则会生成异常...
                if (ze.isDirectory()) {
                    File fmd = new File(path, filename);
                    fmd.mkdirs();
                    continue;
                }

                FileOutputStream fout = new FileOutputStream(new File(path, filename));

                //阅读拉链和书写
                while ((count = zis.read(buffer)) != -1) {
                    fout.write(buffer, 0, count);
                    current += count;
                    if(handler!=null){
                        handler.sendEmptyMessage((int)(((float)current/(float)totalBytes) * 100));
                    }
                }

                fout.close();
                zis.closeEntry();
            }

            zis.close();
        } catch(IOException e){
            e.printStackTrace();
            return false;
        }
        return true;
    }

    public static void copyAssets(Context context, String from, File to) throws IOException{
        IOException exception = null;
        OutputStream outputStream = null;
        InputStream inputStream = null;
        try {
            outputStream = new FileOutputStream(to);
            inputStream = context.getAssets().open(from);
            int read = 0;
            byte[] bytes = new byte[1024];
            while ((read = inputStream.read(bytes)) != -1) {
                outputStream.write(bytes, 0, read);
            }
        }catch(IOException e){
            exception = e;
        } finally{
            if (inputStream != null) {
                try {
                    inputStream.close();
                } catch (IOException e) {
                    exception = e;
                }
            }
            if (outputStream != null) {
                try {
                    //outputStream.flush();
                    outputStream.close();
                } catch (IOException e) {
                    exception = e;
                }
            }
        }
        if(exception!=null){
            throw exception;
        }
    }
}
