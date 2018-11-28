package cn.jy.lazydict;

import android.content.Context;
import android.content.Intent;
import android.os.Build;
import android.os.Bundle;
import android.os.Handler;
import android.os.Message;
import android.util.Log;
import android.view.ViewGroup;
import android.view.Window;
import android.view.WindowManager;
import android.widget.RelativeLayout;
import android.widget.Toast;

import cds.sdg.sdf.AdManager;
import cds.sdg.sdf.nm.cm.ErrorCode;
import cds.sdg.sdf.nm.sp.SplashViewSettings;
import cds.sdg.sdf.nm.sp.SpotListener;
import cds.sdg.sdf.nm.sp.SpotManager;
import cds.sdg.sdf.nm.sp.SpotRequestListener;
import cds.sdg.sdf.onlineconfig.OnlineConfigCallBack;

import static cn.jy.lazydict.Toolkit.MSG_TESS_INIT_SUCCESS;

public class SplashActivity extends BaseActivity {
	static final String TAG = "SplashActivity";
	Context mContext;

	private PermissionHelper mPermissionHelper;
	private Handler mHander = new Handler();
	
	@Override
	protected void onCreate(Bundle savedInstanceState) {
		mContext = this;
		super.onCreate(savedInstanceState);
		mContext = this;
		// 设置全屏
		getWindow().setFlags(WindowManager.LayoutParams.FLAG_FULLSCREEN, WindowManager.LayoutParams.FLAG_FULLSCREEN);
		// 移除标题栏
		requestWindowFeature(Window.FEATURE_NO_TITLE);
		setContentView(R.layout.activity_splash);
		
		// 当系统为6.0以上时，需要申请权限
		mPermissionHelper = new PermissionHelper(this);
		mPermissionHelper.setOnApplyPermissionListener(new PermissionHelper.OnApplyPermissionListener() {
			@Override
			public void onAfterApplyAllPermission() {
				Log.i(TAG, "All of requested permissions has been granted, so run app logic.");
				runApp();
			}
		});
		if (Build.VERSION.SDK_INT < 23) {
			// 如果系统版本低于23，直接跑应用的逻辑
			Log.d(TAG, "The api level of system is lower than 23, so run app logic directly.");
			runApp();
		} else {
			// 如果权限全部申请了，那就直接跑应用逻辑
			if (mPermissionHelper.isAllRequestedPermissionGranted()) {
				Log.d(TAG, "All of requested permissions has been granted, so run app logic directly.");
				runApp();
			} else {
				// 如果还有权限为申请，而且系统版本大于23，执行申请权限逻辑
				Log.i(TAG, "Some of requested permissions hasn't been granted, so apply permissions first.");
				mPermissionHelper.applyPermissions();
			}
		}
	}
	
	@Override
	public void onRequestPermissionsResult(int requestCode, String[] permissions, int[] grantResults) {
		super.onRequestPermissionsResult(requestCode, permissions, grantResults);
		mPermissionHelper.onRequestPermissionsResult(requestCode, permissions, grantResults);
	}
	
	@Override
	protected void onActivityResult(int requestCode, int resultCode, Intent data) {
		super.onActivityResult(requestCode, resultCode, data);
		mPermissionHelper.onActivityResult(requestCode, resultCode, data);
	}

	private void runApp() {
		//初始化SDK
		AdManager.getInstance(this).init("88000ee36d1eefd2", "711a3e0ee8f99d09", BuildConfig.DEBUG);
		initJieBa();
		//初始化
		Toolkit.initTessTwo(this, new Handler(new Handler.Callback() {
			@Override
			public boolean handleMessage(Message message) {
				if(message.what == MSG_TESS_INIT_SUCCESS){
				}else{
					Toast.makeText(SplashActivity.this, "初始化失败!", Toast.LENGTH_LONG).show();
				}
				return false;
			}
		}));

		//读取是否显示gg
		AdManager.getInstance(this).asyncGetOnlineConfig("load", new OnlineConfigCallBack() {
			@Override
			public void onGetOnlineConfigSuccessful(String key, String value) {
				if(value.equals("1")){
					initSdk();
				}else{
					delayStart();
				}
			}

			@Override
			public void onGetOnlineConfigFailed(String key) {
				delayStart();
			}
		});
	}

	private void delayStart(){
		mHander.postDelayed(new Runnable() {
			@Override
			public void run() {
				startActivity(new Intent(SplashActivity.this, CameraActivity.class));
				finish();
			}
		}, 1000);
	}

	private void initSdk(){
		preloadAd();
		setupSplashAd();
	}

	private void initJieBa(){
		//初始化jieba
		new Thread(new Runnable() {
			@Override
			public void run() {
				try {
					Toolkit.jiebaCut(SplashActivity.this,"字");
				} catch (Exception e) {
					runOnUiThread(new Runnable() {
						@Override
						public void run() {
							Toast.makeText(SplashActivity.this, "初始化失败!", Toast.LENGTH_LONG).show();
							System.exit(0);
						}
					});
					e.printStackTrace();
				}
			}
		}).start();
	}
	
	/**
	 * 预加载gg
	 */
	private void preloadAd() {
		// 注意：不必每次展示插播gg前都请求，只需在应用启动时请求一次
		SpotManager.getInstance(mContext).requestSpot(new SpotRequestListener() {
			@Override
			public void onRequestSuccess() {
				logInfo("请求gg成功");
				//				// 应用安装后首次展示kp会因为本地没有数据而跳过
				//              // 如果开发者需要在首次也能展示kp，可以在请求gg成功之前展示应用的logo，请求成功后再加载kp
				//				setupSplashAd();
			}
			
			@Override
			public void onRequestFailed(int errorCode) {
				logError("请求cpgg失败，errorCode: %s", errorCode);
				switch (errorCode) {
				case ErrorCode.NON_NETWORK:
					showShortToast("网络异常");
					break;
				case ErrorCode.NON_AD:
//					showShortToast("暂无cpgg");
					break;
				default:
//					showShortToast("请稍后再试");
					break;
				}
			}
		});
	}
	
	/**
	 * 设置kpgg
	 */
	private void setupSplashAd() {
		// 创建kp容器
		final RelativeLayout splashLayout = findViewById(R.id.rl_splash);
		RelativeLayout.LayoutParams params =
				new RelativeLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT);
		params.addRule(RelativeLayout.ABOVE, R.id.view_divider);
		
		// 对kp进行设置
		SplashViewSettings splashViewSettings = new SplashViewSettings();
		//		// 设置是否展示失败自动跳转，默认自动跳转
		//		splashViewSettings.setAutoJumpToTargetWhenShowFailed(false);
		// 设置跳转的窗口类
		splashViewSettings.setTargetClass(CameraActivity.class);
		// 设置kp的容器
		splashViewSettings.setSplashViewContainer(splashLayout);

		// 展示kpgg
		SpotManager.getInstance(mContext)
		                    .showSplash(mContext, splashViewSettings, new SpotListener() {
			
			                    @Override
			                    public void onShowSuccess() {
				                    logInfo("kp展示成功");
			                    }
			
			                    @Override
			                    public void onShowFailed(int errorCode) {
				                    logError("kp展示失败");
				                    switch (errorCode) {
				                    case ErrorCode.NON_NETWORK:
					                    logError("网络异常");
					                    break;
				                    case ErrorCode.NON_AD:
					                    logError("暂无kpgg");
					                    break;
				                    case ErrorCode.RESOURCE_NOT_READY:
					                    logError("kp资源还没准备好");
					                    break;
				                    case ErrorCode.SHOW_INTERVAL_LIMITED:
					                    logError("kp展示间隔限制");
					                    break;
				                    case ErrorCode.WIDGET_NOT_IN_VISIBILITY_STATE:
					                    logError("kp控件处在不可见状态");
					                    break;
				                    default:
					                    logError("errorCode: %d", errorCode);
					                    break;
				                    }
			                    }
			
			                    @Override
			                    public void onSpotClosed() {
				                    logDebug("kp被关闭");
			                    }
			
			                    @Override
			                    public void onSpotClicked(boolean isWebPage) {
				                    logDebug("kp被点击");
				                    logInfo("是否是网页gg？%s", isWebPage ? "是" : "不是");
			                    }
		                    });
	}
	
	@Override
	protected void onDestroy() {
		super.onDestroy();
		// kp展示界面的 onDestroy() 回调方法中调用
		SpotManager.getInstance(mContext).onDestroy();
	}
}
