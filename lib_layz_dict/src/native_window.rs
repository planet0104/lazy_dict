use jni::sys::{jobject, JNIEnv};
use std::os::raw::{c_void};
//
//       android/rect.h
//
#[repr(C)]
pub struct ARect {
     pub left:              i32,
     pub top:               i32,
     pub right:             i32,
     pub bottom:                i32,
}

//  android/native_window_jni.h
//  android/native_window.h

pub type ANativeWindow = c_void;
#[repr(C)]
pub struct ANativeWindow_Buffer {
     pub width:             i32,
     pub height:                i32,
     pub stride:                i32,
     pub format:                i32,
     pub bits:              *mut c_void,
     pub reserved:              [u32; 5],
}

pub const WINDOW_FORMAT_RGBA_8888: i32 = 1;
pub const WINDOW_FORMAT_RGBX_8888: i32 = 2;
pub const WINDOW_FORMAT_RGB_565: i32 = 4;

#[link(name = "nativewindow")]
#[allow(non_snake_case)]
extern "C" {
	pub fn ANativeWindow_acquire(window: *mut ANativeWindow);
    pub fn ANativeWindow_getFormat(window: *mut ANativeWindow) -> i32;
    pub fn ANativeWindow_getHeight(window: *mut ANativeWindow) -> i32;
    pub fn ANativeWindow_getWidth(window: *mut ANativeWindow) -> i32;
    pub fn ANativeWindow_lock(window: *mut ANativeWindow, outBuffer: *mut ANativeWindow_Buffer, inOutDirtyBounds: *mut ARect) -> i32;
    pub fn ANativeWindow_release(window: *mut ANativeWindow);
    pub fn ANativeWindow_setBuffersGeometry(window: *mut ANativeWindow, width: i32, height: i32, format: i32) -> i32;
    pub fn ANativeWindow_unlockAndPost(window: *mut ANativeWindow) -> i32;
    pub fn ANativeWindow_fromSurface(env: *mut JNIEnv, surface: jobject) -> *mut ANativeWindow;
}