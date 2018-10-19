#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
// #![allow(non_upper_case_globals)]

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
pub const WINDOW_FORMAT_RGB_888: i32 = 3;
pub const WINDOW_FORMAT_RGB_565: i32 = 4;

pub const ANATIVEWINDOW_TRANSFORM_IDENTITY: i32            = 0x00;
pub const ANATIVEWINDOW_TRANSFORM_MIRROR_HORIZONTAL: i32   = 0x01;
pub const ANATIVEWINDOW_TRANSFORM_MIRROR_VERTICAL: i32     = 0x02;
pub const ANATIVEWINDOW_TRANSFORM_ROTATE_90: i32           = 0x04;

pub const ANATIVEWINDOW_TRANSFORM_ROTATE_180: i32          = ANATIVEWINDOW_TRANSFORM_MIRROR_HORIZONTAL |
                                                  ANATIVEWINDOW_TRANSFORM_MIRROR_VERTICAL;
pub const ANATIVEWINDOW_TRANSFORM_ROTATE_270: i32          = ANATIVEWINDOW_TRANSFORM_ROTATE_180 |
                                                  ANATIVEWINDOW_TRANSFORM_ROTATE_90;

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