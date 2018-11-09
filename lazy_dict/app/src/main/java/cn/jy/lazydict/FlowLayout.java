package cn.jy.lazydict;

import android.content.Context;
import android.util.AttributeSet;
import android.view.View;
import android.view.ViewGroup;

import java.util.ArrayList;
import java.util.Collections;
import java.util.Iterator;
import java.util.List;

public class FlowLayout extends ViewGroup {
	public static final int DEFAULT_HORIZONTAL_SPACING = 5;
	public static final int DEFAULT_VERTICAL_SPACING = 5;
	private int horizontalSpacing;
	private int verticalSpacing;
	private List<RowMeasurement> currentRows = Collections.emptyList();

	public FlowLayout(Context context) {
		super(context);
		init();
	}

	public FlowLayout(Context context, AttributeSet attrs) {
		super(context, attrs);
		init();
	}

	public FlowLayout(Context context, AttributeSet attrs, int defStyleAttr) {
		super(context, attrs, defStyleAttr);
		init();
	}

	private void init(){
		horizontalSpacing = dp2px(0);
		verticalSpacing = dp2px(0);
	}

	@Override
	protected void onMeasure(int widthMeasureSpec, int heightMeasureSpec) {
		final int widthMode = MeasureSpec.getMode(widthMeasureSpec);
		final int heightMode = MeasureSpec.getMode(heightMeasureSpec);
		final int maxInternalWidth = MeasureSpec.getSize(widthMeasureSpec) - getHorizontalPadding();
		final int maxInternalHeight = MeasureSpec.getSize(heightMeasureSpec) - getVerticalPadding();
		List<RowMeasurement> rows = new ArrayList<RowMeasurement>();
		RowMeasurement currentRow = new RowMeasurement(maxInternalWidth, widthMode);
		rows.add(currentRow);
		for (View child : getLayoutChildren()) {
			LayoutParams childLayoutParams = child.getLayoutParams();
			int childWidthSpec = createChildMeasureSpec(childLayoutParams.width, maxInternalWidth, widthMode);
			int childHeightSpec = createChildMeasureSpec(childLayoutParams.height, maxInternalHeight, heightMode);
			child.measure(childWidthSpec, childHeightSpec);
			int childWidth = child.getMeasuredWidth();
			int childHeight = child.getMeasuredHeight();
			if (currentRow.wouldExceedMax(childWidth)) {
				currentRow = new RowMeasurement(maxInternalWidth, widthMode);
				rows.add(currentRow);
			}
			currentRow.addChildDimensions(childWidth, childHeight);
		}

		int longestRowWidth = 0;
		int totalRowHeight = 0;
		for (int index = 0; index < rows.size(); index++) {
			RowMeasurement row = rows.get(index);
			totalRowHeight += row.getHeight();
			if (index < rows.size() - 1) {
				totalRowHeight += verticalSpacing;
			}
			longestRowWidth = Math.max(longestRowWidth, row.getWidth());
		}
		setMeasuredDimension(widthMode == MeasureSpec.EXACTLY ? MeasureSpec.getSize(widthMeasureSpec) : longestRowWidth
				+ getHorizontalPadding(), heightMode == MeasureSpec.EXACTLY ? MeasureSpec.getSize(heightMeasureSpec)
				: totalRowHeight + getVerticalPadding());
		currentRows = Collections.unmodifiableList(rows);
	}

	private int createChildMeasureSpec(int childLayoutParam, int max, int parentMode) {
		int spec;
		if (childLayoutParam == LayoutParams.FILL_PARENT) {
			spec = MeasureSpec.makeMeasureSpec(max, MeasureSpec.EXACTLY);
		} else if (childLayoutParam == LayoutParams.WRAP_CONTENT) {
			spec = MeasureSpec.makeMeasureSpec(max, parentMode == MeasureSpec.UNSPECIFIED ? MeasureSpec.UNSPECIFIED
					: MeasureSpec.AT_MOST);
		} else {
			spec = MeasureSpec.makeMeasureSpec(childLayoutParam, MeasureSpec.EXACTLY);
		}
		return spec;
	}

	@Override
	protected void onLayout(boolean changed, int leftPosition, int topPosition, int rightPosition, int bottomPosition) {
		final int widthOffset = getMeasuredWidth() - getPaddingRight();
		int x = getPaddingLeft();
		int y = getPaddingTop();

		Iterator<RowMeasurement> rowIterator = currentRows.iterator();
		RowMeasurement currentRow = rowIterator.next();
		for (View child : getLayoutChildren()) {
			final int childWidth = child.getMeasuredWidth();
			final int childHeight = child.getMeasuredHeight();
			if (x + childWidth > widthOffset) {
				x = getPaddingLeft();
				y += currentRow.height + verticalSpacing;
				if (rowIterator.hasNext()) {
					currentRow = rowIterator.next();
				}
			}
			// Align the child vertically.
			int childY = y + (currentRow.height - childHeight) / 2;
			child.layout(x, childY, x + childWidth, childY + childHeight);
			x += childWidth + horizontalSpacing;
		}
	}

	private List<View> getLayoutChildren() {
		List<View> children = new ArrayList<View>();
		for (int index = 0; index < getChildCount(); index++) {
			View child = getChildAt(index);
			if (child.getVisibility() != View.GONE) {
				children.add(child);
			}
		}
		return children;
	}

	protected int getVerticalPadding() {
		return getPaddingTop() + getPaddingBottom();
	}

	protected int getHorizontalPadding() {
		return getPaddingLeft() + getPaddingRight();
	}

	private final class RowMeasurement {
		private final int maxWidth;
		private final int widthMode;
		private int width;
		private int height;

		public RowMeasurement(int maxWidth, int widthMode) {
			this.maxWidth = maxWidth;
			this.widthMode = widthMode;
		}

		public int getHeight() {
			return height;
		}

		public int getWidth() {
			return width;
		}

		public boolean wouldExceedMax(int childWidth) {
			return widthMode == MeasureSpec.UNSPECIFIED ? false : getNewWidth(childWidth) > maxWidth;
		}

		public void addChildDimensions(int childWidth, int childHeight) {
			width = getNewWidth(childWidth);
			height = Math.max(height, childHeight);
		}

		private int getNewWidth(int childWidth) {
			return width == 0 ? childWidth : width + horizontalSpacing + childWidth;
		}
	}

	int dp2px(float spValue) {
		float fontScale = getContext().getResources().getDisplayMetrics().scaledDensity;
		return (int) (spValue * fontScale + 0.5f);
	}

	public void setHorizontalSpacing(int horizontalSpacing) {
		this.horizontalSpacing = horizontalSpacing;
		invalidate();
	}

	public void setVerticalSpacing(int verticalSpacing) {
		this.verticalSpacing = verticalSpacing;
	}
}