package cn.jy.lazydict;

import android.app.Dialog;
import android.graphics.Bitmap;
import android.graphics.RectF;
import android.hardware.Camera;
import android.os.Build;
import android.os.Bundle;
import android.os.Handler;
import android.os.Message;
import android.support.annotation.NonNull;
import android.util.Log;
import android.view.View;
import android.widget.ImageView;
import android.widget.ProgressBar;
import android.widget.TextView;

import com.googlecode.tesseract.android.ResultIterator;
import com.googlecode.tesseract.android.TessBaseAPI;

import java.util.Arrays;

import static cn.jy.lazydict.CameraActivity.TAG;
import static cn.jy.lazydict.CameraActivity.tessBaseAPI;
import static cn.jy.lazydict.Toolkit.MSG_DECODE_COMPLETE;
import static cn.jy.lazydict.Toolkit.MSG_TESS_RECOGNIZE_COMPLETE;
import static cn.jy.lazydict.Toolkit.MSG_TESS_RECOGNIZE_ERROR;
import static cn.jy.lazydict.Toolkit.MSG_TESS_RECOGNIZE_LINE;
import static cn.jy.lazydict.Toolkit.MSG_TESS_RECOGNIZE_START;

public class SearchDialog extends Dialog implements Handler.Callback {
    Camera.CameraInfo info; Camera.Size size; byte[] frame;

    ImageView iv_back;
    CameraActivity activity;
    Bitmap capture;
    ProgressBar pb_loading;
    private Handler handler;
    FlowLayout labels;

    public SearchDialog(@NonNull CameraActivity context, final Camera.CameraInfo info, final Camera.Size size, final byte[] frame) {
        super(context, R.style.Dialog_Fullscreen);
        activity = context;
        this.frame = frame;
        this.info = info;
        this.size = size;
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.LOLLIPOP) {
            try{
                getWindow().setStatusBarColor(0xff303030);
            }catch (Exception e){e.printStackTrace();}
        }
        setContentView(R.layout.dialog_search);
        pb_loading = findViewById(R.id.pb_loading);
        handler = new Handler(this);
        labels = findViewById(R.id.labels);
        iv_back = findViewById(R.id.iv_back);
        iv_back.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                dismiss();
            }
        });
        decode();
    }

    private void decode(){
        activity.tessHandler.sendEmptyMessage(MSG_TESS_RECOGNIZE_START);
        new Thread(new Runnable() {
            @Override
            public void run() {try {
                capture = Toolkit.decodeYUV420SP(frame, size.width, size.height, info.orientation);
                Message msg = Message.obtain();
                msg.what = MSG_DECODE_COMPLETE;
                handler.sendMessage(msg);
            } catch (Exception e) {
                e.printStackTrace();
                handler.sendEmptyMessage(MSG_TESS_RECOGNIZE_ERROR);
            }}}).start();
    }

    private void searchLocal(){
        new Thread(new Runnable() {
            @Override
            public void run() {try {
                int[] loc = new int[2];
                activity.ll_mask.getLocationInWindow(loc);
                int[] loc2 = new int[2];
                activity.fl_scan_area.getLocationInWindow(loc2);
                int left = loc2[0];
                int top = loc2[1]-loc[1];
                Bitmap rect = Bitmap.createBitmap(activity.iv_capture.getDrawingCache(), left, top, activity.fl_scan_area.getMeasuredWidth(), activity.fl_scan_area.getMeasuredHeight());
//                        Rect scanRect = Toolkit.getLocationInParent(fl_scan_area, fl_preview);
                //Toolkit.upSearch(CameraActivity.this, rect, tessHandler);
                RectF[] splitRect = Toolkit.split(rect);
                //识别
                tessBaseAPI.setPageSegMode(TessBaseAPI.PageSegMode.PSM_SINGLE_LINE);
                for(RectF lineRect : splitRect){
                    if(lineRect.height()<=10 || lineRect.width()<=10){
                        //宽高小于10像素的忽略
                        continue;
                    }
                    long t = System.currentTimeMillis();
                    Bitmap rb = Bitmap.createBitmap(rect, (int)lineRect.left, (int)lineRect.top, (int)(lineRect.right-lineRect.left), (int)(lineRect.bottom-lineRect.top));
                    Toolkit.binary(rb);
                    tessBaseAPI.setImage(rb);
                    String line = "";
                    String _text = tessBaseAPI.getUTF8Text();
                    //------------------------------------------------------------------
                    ResultIterator resultIterator = tessBaseAPI.getResultIterator();
                    int level = TessBaseAPI.PageIteratorLevel.RIL_SYMBOL;
                    do{
                        //提取准确率高于80%的字符
                        if(resultIterator.confidence(level)>80.0){
                            String ts = resultIterator.getUTF8Text(level);
                            char[] chars = ts.toCharArray();
                            for(char ch: chars){
                                if(Toolkit.isChinese(ch)){
                                    line += ch;
                                }
                            }
                        }
                    }while(resultIterator.next(level));
                    resultIterator.delete();
                    //------------------------------------------------------------------
                    Log.d(TAG, "耗时:"+(System.currentTimeMillis()-t)+"毫秒 text="+line);
                    long ft = System.currentTimeMillis();
                    String[] words = Toolkit.jiebaCut(line);
                    Log.d(TAG, "分词结果:"+Arrays.toString(words)+" 耗时:"+(System.currentTimeMillis()-ft)+"ms");
                    if(words!=null && words.length>0){
                        //返回一行的分词结果
                        Message msg = Message.obtain();
                        msg.what = MSG_TESS_RECOGNIZE_LINE;
                        msg.obj = words;
                        handler.sendMessage(msg);
                    }
                }

                //识别完成
                Message msg = Message.obtain();
                msg.what = MSG_TESS_RECOGNIZE_COMPLETE;
                handler.sendMessage(msg);
            }catch (Exception e){e.printStackTrace();
                handler.sendEmptyMessage(MSG_TESS_RECOGNIZE_ERROR);}}
        }).start();
    }

    @Override
    public boolean handleMessage(Message msg) {
        switch (msg.what){
            case Toolkit.MSG_TESS_RECOGNIZE_ERROR:
                activity.showMessageDialog("识别出错!", false);
                break;
            case Toolkit.MSG_DECODE_COMPLETE:
                activity.iv_capture.setImageBitmap(capture);
                searchLocal();
                break;
            case Toolkit.MSG_TESS_RECOGNIZE_LINE:
                String[] result = (String[]) msg.obj;
                for(String text : result){
                    TextView tv = new TextView(getContext());
                    tv.setText(text);
                    labels.addView(tv);
//                    LinearLayout line_layout = new LinearLayout(CameraActivity.this);
//                    LinearLayout.LayoutParams llp = new LinearLayout.LayoutParams(LinearLayout.LayoutParams.WRAP_CONTENT, LinearLayout.LayoutParams.WRAP_CONTENT);
//                    llp.topMargin = Toolkit.dip2px(CameraActivity.this, 10);
//                    line_layout.setGravity(Gravity.CENTER);
//                    line_layout.setLayoutParams(llp);
//                    line_layout.setOrientation(LinearLayout.HORIZONTAL);
//                    line_layout.setBackgroundResource(tvbg);
//                    String[] pinyin = null;
//                    try{ pinyin= Toolkit.pinyin(text); }catch (Exception e1){e1.printStackTrace();}
//                    char[] chars = text.toCharArray();
//                    for(int i=0; i<text.length(); i++){
//                        PinYinTextView tv = new PinYinTextView(CameraActivity.this);
//                        if(text.length()==1){
//                            tv.getPinyinView().setMinWidth(Toolkit.dip2px(CameraActivity.this, 32));
//                        }
//                        tv.setText(chars[i]+"");
//                        if(pinyin.length>i){
//                            tv.setPinyin(pinyin[i]);
//                        }else{
//                            tv.setPinyin("");
//                        }
//                        line_layout.addView(tv);
//                    }
//                    line_layout.setTag(text);
//                    line_layout.setOnClickListener(new View.OnClickListener() {
//                        @Override
//                        public void onClick(View v) {
//                            wv_mean.setVisibility(View.GONE);
//                            //切换选中状态
//                            for(int i=0; i<ll_lines.getChildCount(); i++){
//                                ll_lines.getChildAt(i).setSelected(false);
//                            }
//                            v.setSelected(true);
//
//                            final String text = (String) v.getTag();
//                            if(text.length()==1){
//                                //查字
//                                Word word = null;
//                                try{ word = Toolkit.search(text); }catch (Exception e){ e.printStackTrace(); }
//                                if(word == null){
//                                    tv_mean.setText("正在网络上搜索...");
//                                    Toolkit.checkBaiKe(text, tessHandler);
//                                    //wv_mean
//                                }else{
//                                    tv_mean.setText(Html.fromHtml(word.toString()));
//                                }
//                            }else{
//                                //查词
//                                String mean = null;
//                                try{ mean = Toolkit.searchWords(text); }catch (Exception e){ e.printStackTrace(); }
//                                if(mean == null){
//                                    tv_mean.setText("正在网络上搜索...");
//                                    Toolkit.checkBaiKe(text, tessHandler);
//                                }else{
//                                    tv_mean.setText(mean);
//                                }
//                            }
//                        }
//                    });
//                    ll_lines.addView(line_layout, llp);
//                    if(ll_mean.getVisibility()==View.GONE){
//                        ll_mean.setVisibility(View.VISIBLE);
//                        ll_lines.getChildAt(0).performClick();
//                    }
                }
                break;
        }
        return false;
    }

    @Override
    public void dismiss() {
        super.dismiss();
        Toolkit.initTessTwo(activity, tessBaseAPI, activity.tessHandler);
    }

    @Override
    public void onBackPressed() {
        super.onBackPressed();
        dismiss();
    }
}
