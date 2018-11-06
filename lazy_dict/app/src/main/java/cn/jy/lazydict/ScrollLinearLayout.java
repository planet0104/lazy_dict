package cn.jy.lazydict;

import android.content.Context;
import android.util.AttributeSet;
import android.view.ViewGroup;
import android.view.animation.Interpolator;
import android.widget.LinearLayout;
import android.widget.Scroller;
/**
 * 可滑动的LinearLayout
 * @author JiaYe 2014-7-24
 *
 */
public class ScrollLinearLayout extends LinearLayout {  
	static final String TAG = "ScrollLinearLayout"; 
	
	private boolean mFinished = true;

	public interface OnScrollFinishedListener{
		public void onScrollFinished(ViewGroup scrollView);
	}

	private Scroller mScroller;
	private OnScrollFinishedListener mOnScrollFinishedListener;

	public void setOnScrollFinishedListener(
			OnScrollFinishedListener mOnScrollFinishedListener) {
		this.mOnScrollFinishedListener = mOnScrollFinishedListener;
	}
	public void removeOnScrollFinishedListener(){
	    mOnScrollFinishedListener = null;
	}
	
	public boolean isFinished() {
		return mFinished;
	}

	public ScrollLinearLayout(Context context){
		super(context);
		mScroller = new Scroller(context);
	}

	public ScrollLinearLayout(Context context, AttributeSet attrs) {  
		super(context, attrs);  
		mScroller = new Scroller(context);  
	}
	
	public void setInterpolator(Interpolator interpolator){
	    mScroller = new Scroller(getContext(), interpolator);
	}

	/**
	 * 调用此方法滚动到目标位置  
	 * @param fx
	 * @param fy
	 */
	public void smoothScrollTo(int fx, int fy, int duration) {  
		int dx = fx - getScrollX();  
		int dy = fy - getScrollY();  
		smoothScrollBy(dx, dy, duration);  
	}

	/**
	 * 调用此方法设置滚动的相对偏移
	 * @param dx
	 * @param dy
	 */
	public void smoothScrollBy(int dx, int dy, int duration) {  
		//设置mScroller的滚动偏移量
		mFinished = false;
		mScroller.startScroll(getScrollX(), getScrollY(), dx, dy, duration);
		invalidate();//这里必须调用invalidate()才能保证computeScroll()会被调用，否则不一定会刷新界面，看不到滚动效果  
	}

	@Override  
	public void computeScroll() {
		try{
			//先判断mScroller滚动是否完成
			if (mScroller.computeScrollOffset()) {

				//这里调用View的scrollTo()完成实际的滚动
				scrollTo(mScroller.getCurrX(), mScroller.getCurrY());
				//必须调用该方法，否则不一定能看到滚动效果
				postInvalidate();

				if(mScroller.isFinished()){
					mFinished = true;
					if(mOnScrollFinishedListener != null){
						mOnScrollFinishedListener.onScrollFinished(this);
					}
				}
			}
		}catch (Exception e){

		}
		try{
			super.computeScroll();
		}catch (Exception e){

		}
	}
	
	public Scroller getScroller() {
		return mScroller;
	}
}  