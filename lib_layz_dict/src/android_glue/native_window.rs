#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::os::raw::c_void;
use std::os::raw::c_float;
use std::os::raw::c_double;
use std::os::raw::c_char;
use std::os::raw::c_schar;
use std::os::raw::c_uchar;
use std::os::raw::c_int;
use std::os::raw::c_short;
use std::os::raw::c_ushort;
use std::os::raw::c_longlong;
use jni::sys::{jobject, JNIEnv, JavaVM};
use libc;
use std::os::raw;

/**
 * asset_manager.h
 */
pub type AAssetManager = raw::c_void;

/**
 * native_window.h
 */
pub type ANativeWindow = raw::c_void;

/**
 * native_activity.h
 */
pub type ANativeActivity = raw::c_void;

#[repr(C)]
pub struct ANativeActivity {
    pub callbacks: *mut ANativeActivityCallbacks,
    pub vm: *mut JavaVM,
    pub env: *mut JNIEnv,
    pub clazz: jobject,
    pub internalDataPath: *const libc::c_char,
    pub externalDataPath: *const libc::c_char,
    pub sdkVersion: libc::int32_t,
    pub instance: *mut libc::c_void,
    pub assetManager: *mut AAssetManager,
    pub obbPath: *const libc::c_char,
}

#[repr(C)]
pub struct ANativeActivityCallbacks {
    pub onStart: extern fn(*mut ANativeActivity),
    pub onResume: extern fn(*mut ANativeActivity),
    pub onSaveInstanceState: extern fn(*mut ANativeActivity, *mut libc::size_t),
    pub onPause: extern fn(*mut ANativeActivity),
    pub onStop: extern fn(*mut ANativeActivity),
    pub onDestroy: extern fn(*mut ANativeActivity),
    pub onWindowFocusChanged: extern fn(*mut ANativeActivity, libc::c_int),
    pub onNativeWindowCreated: extern fn(*mut ANativeActivity, *const ANativeWindow),
    pub onNativeWindowResized: extern fn(*mut ANativeActivity, *const ANativeWindow),
    pub onNativeWindowRedrawNeeded: extern fn(*mut ANativeActivity, *const ANativeWindow),
    pub onNativeWindowDestroyed: extern fn(*mut ANativeActivity, *const ANativeWindow),
    pub onInputQueueCreated: extern fn(*mut ANativeActivity, *mut AInputQueue),
    pub onInputQueueDestroyed: extern fn(*mut ANativeActivity, *mut AInputQueue),
    pub onContentRectChanged: extern fn(*mut ANativeActivity, *const ARect),
    pub onConfigurationChanged: extern fn(*mut ANativeActivity),
    pub onLowMemory: extern fn(*mut ANativeActivity),
}