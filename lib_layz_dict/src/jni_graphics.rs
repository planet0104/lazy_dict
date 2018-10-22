#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::os::raw::{c_int, c_void, c_uint};
use jni::sys::{JNIEnv, jclass, jobject};
use jni;
use jni::objects::{JStaticMethodID, JClass};
//Bitmap作为

const ANDROID_BITMAP_FORMAT_NONE:i32 = 0;
const ANDROID_BITMAP_FORMAT_RGBA_8888:i32 = 1;
const ANDROID_BITMAP_FORMAT_RGB_565:i32   = 4;
const ANDROID_BITMAP_FORMAT_RGBA_4444:i32 = 7;
const ANDROID_BITMAP_FORMAT_A_8:i32       = 8;

pub fn get_format_name(format: i32) -> String{
	String::from(match format{
		ANDROID_BITMAP_FORMAT_NONE => "None",
		ANDROID_BITMAP_FORMAT_RGBA_8888 => "RGBA_8888",
		ANDROID_BITMAP_FORMAT_RGB_565 => "RGB_565",
		ANDROID_BITMAP_FORMAT_RGBA_4444 => "RGBA_4444",
		ANDROID_BITMAP_FORMAT_A_8 => "FORMAT_A_8",
		_ => "未知格式"
	})
}

#[repr(C)]
pub struct AndroidBitmapInfo {
	pub width: c_uint,
	pub height: c_uint,
	pub stride: c_uint,
	pub format: c_int,
	pub flags: c_uint, // 0 for now
}

#[link(name = "jnigraphics")]
#[allow(non_snake_case)]
extern "C" {
	///给定一个java位图对象，为它填写AndroidBitmap结构。如果调用失败，将忽略info参数
	pub fn AndroidBitmap_getInfo(env: *mut JNIEnv, jbitmap: jobject, info: *mut AndroidBitmapInfo) -> c_int;

  	///给定一个java位图对象，尝试锁定像素地址。 锁定将确保像素的内存在unlockPixels调用之前不会移动，并确保如果像素先前已被清除，则它们将被恢复。
  	///如果此调用成功，则必须通过调用AndroidBitmap_unlockPixels来平衡，之后不再使用像素的地址。
  	///如果成功，* addrPtr将被设置为像素地址。 如果调用失败，将忽略addrPtr。
	pub fn AndroidBitmap_lockPixels(env: *mut JNIEnv, jbitmap: jobject, addrPtr: *mut *mut c_void) -> c_int;

	///调用此方法可以平衡对AndroidBitmap_lockPixels的成功调用
	pub fn AndroidBitmap_unlockPixels(env: *mut JNIEnv, jbitmap: jobject) -> c_int;
}

pub fn unlock_bitmap(env: &jni::JNIEnv, bitmap: jobject){
	let ret = unsafe{ AndroidBitmap_unlockPixels(env.get_native_interface(), bitmap) };
	trace!("AndroidBitmap_unlockPixels:{}", ret);
}

pub fn lock_bitmap<'a>(env: &jni::JNIEnv, bitmap: jobject) -> Result<(AndroidBitmapInfo, &'a mut [u8]), String>{
	let mut info = AndroidBitmapInfo{
		width: 0,
		height: 0,
		stride: 0,
		format: 0,
		flags: 0,
	};
	
	let ret = unsafe{AndroidBitmap_getInfo(env.get_native_interface(), bitmap, &mut info)};
	if ret<0{
		return Result::Err(format!("AndroidBitmap_getInfo调用失败! {}", ret));
	}
  	trace!("图片 {}x{} format={}", info.width, info.height, info.format);

	let mut pixels = 0 as *mut c_void;
    let ret = unsafe{ AndroidBitmap_lockPixels(env.get_native_interface(), bitmap, &mut pixels) };
    if ret<0{
      return Result::Err(format!("AndroidBitmap_lockPixels! {}", ret));
    }
    let pixels = unsafe{ ::std::slice::from_raw_parts_mut(pixels as *mut u8, (info.width*info.height*2) as usize)};
	Ok((info, pixels))
}

pub fn create_java_bitmap(env: jni::JNIEnv) -> Result<jobject, jni::errors::Error>{
	//1) Get the static method id of createBitmap(int width, int height, Bitmap.Config config):
	let java_bitmap_class:JClass = env.find_class("android/graphics/Bitmap")?;
    let method_id:JStaticMethodID = env.get_static_method_id(java_bitmap_class, "createBitmap", "(IILandroid/graphics/Bitmap$Config;)Landroid/graphics/Bitmap;")?;

	//2) Creating enum for Bitmap.Config with given value:
	const wchar_t config_name[] = L"ARGB_8888";
	jstring j_config_name = env.NewString((const jchar*)config_name, wcslen(config_name));
	jclass bcfg_class = env.FindClass("android/graphics/Bitmap$Config");
	jobject java_bitmap_config = env.CallStaticObjectMethod(bcfg_class, env.GetStaticMethodID(bcfg_class, "valueOf", "(Ljava/lang/String;)Landroid/graphics/Bitmap$Config;"), j_config_name);

	Ok(())
}