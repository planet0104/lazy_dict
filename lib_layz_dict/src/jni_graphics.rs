use std::os::raw::{c_int, c_void, c_uint};
use jni::sys::{JNIEnv, jobject};
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
