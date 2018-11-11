package cn.jy.lazydict;

import android.content.Context;
import android.content.SharedPreferences;
import android.content.SharedPreferences.Editor;

import java.util.Map;

/**
 * SharedPreferencesHelper.java 提供存储本地数据的方法
 * @author planet
 *
 */
public class SharedPreferencesHelper {
	public static final String SPM_FILE_NAME = "app_data";

	/**
	 * 保存布尔值到SharedPreferences
	 */
	public static void saveBoolean(Context context, String key,boolean b) {
		Editor editor = context.getSharedPreferences(SPM_FILE_NAME, 0).edit();
		editor.putBoolean(key, b);
		editor.commit();
	}

	/**
	 * 保存浮点数到SharedPreferences
	 */
	public static void saveFloat(Context context, String key,float f) {
		Editor editor = context.getSharedPreferences(SPM_FILE_NAME, 0).edit();
		editor.putFloat(key, f);
		editor.commit();
	}

	/**
	 * 保存整数到SharedPreferences
	 */
	public static void saveInt(Context context, String key, int i) {
		Editor editor = context.getSharedPreferences(SPM_FILE_NAME, 0).edit();
		editor.putInt(key, i);
		editor.commit();
	}

	/**
	 * 保存长整数到SharedPreferences
	 * */
	public static void saveLong(Context context, String key, long l) {
		Editor editor = context.getSharedPreferences(SPM_FILE_NAME, 0).edit();
		editor.putLong(key, l);
		editor.commit();
	}

	/**
	 * 保存字符串到SharedPreferences
	 * */
	public static void saveString(Context context, String key, String s) {
		Editor editor = context.getSharedPreferences(SPM_FILE_NAME, 0).edit();
		editor.putString(key, s);
		editor.commit();
	}

	/**
	 * 删除
	 * */
	public static void remove(Context context, String[] key) {
		if(key == null) return;
		Editor editor = context.getSharedPreferences(SPM_FILE_NAME, 0).edit();
		for(String k : key){
			editor.remove(k);
		}
		editor.commit();
	}

	/**
	 * 删除
	 * */
	public static void remove(Context context, String key) {
		if(key == null) return;
		Editor editor = context.getSharedPreferences(SPM_FILE_NAME, 0).edit();
		editor.remove(key);
		editor.commit();
	}

	/**
	 * 从SharedPreferences读取布尔值
	 * */
	public static boolean getBoolean(Context context, String key) {
		return context.getSharedPreferences(SPM_FILE_NAME, 0).getBoolean(key, false);
	}

	/**
	 * 从SharedPreferences读取浮点数
	 * */
	public static float getFloat(Context context, String key) {
		return context.getSharedPreferences(SPM_FILE_NAME, 0).getFloat(key, 0);
	}

	/**
	 * 从SharedPreferences获取整数
	 * */
	public static int getInt(Context context, String key) {
		return context.getSharedPreferences(SPM_FILE_NAME, 0).getInt(key, -1);
	}

	/**
	 * 从SharedPreferences读取长整数
	 * */
	public static long getLong(Context context, String key) {
		return context.getSharedPreferences(SPM_FILE_NAME, 0).getLong(key, -1);
	}

	/**
	 * 从SharedPreferences读取字符串
	 * */
	public static String getString(Context context, String key) {
		return context.getSharedPreferences(SPM_FILE_NAME, 0).getString(key, null);
	}
	
	/**
	 * 从SharedPreferences读取字符串
	 * */
	public static String getString(Context context, String key, String defautValue) {
		return context.getSharedPreferences(SPM_FILE_NAME, 0).getString(key, defautValue);
	}

	/**
	 * 从SharedPreferences读取配置文件的所有数据
	 * */
	public static Map<String, ?> getMap(Context context) {
		return context.getSharedPreferences(SPM_FILE_NAME, 0).getAll();
	}

	/**
	 * 检查对应的值是否存在
	 * */
	public static boolean contain(Context context, String key) {
		SharedPreferences sp = context.getSharedPreferences(SPM_FILE_NAME,
				Context.MODE_PRIVATE);
		return sp.contains(key);
	}
}
