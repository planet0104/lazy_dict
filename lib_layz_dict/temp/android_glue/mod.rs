// pub mod native_window;

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
use std::os::raw::c_long;
use std::mem::size_of;

/**
 * asset_manager.h
 */
pub type AAssetManager = c_void;

/**
 * native_window.h
 */
pub type ANativeWindow = c_void;

/**
 * native_activity.h
 */
pub type ANativeActivity = c_void;

pub type AInputEvent = c_void;
pub type AConfiguration = c_void;
pub type ALooper = c_void;
pub type AInputQueue = ();  // FIXME: wrong
//pub type ARect = ();  // FIXME: wrong
#[repr(C)]
pub struct ARect {
    pub left: i32, pub top: i32, pub right: i32, pub bottom: i32
}

pub type pthread_t = c_long;
pub type pthread_cond_t = c_long;
pub type pthread_mutex_t = c_long;
pub type pthread_mutexattr_t = c_long;
pub type pthread_attr_t = c_void;       // FIXME: wrong

extern {
    fn pipe(_: *mut c_int) -> c_int;
    fn dup2(fildes: c_int, fildes2: c_int) -> c_int;
    fn read(fd: c_int, buf: *mut c_void, count: usize) -> isize;
    fn pthread_create(_: *mut pthread_t, _: *const pthread_attr_t,
                      _: extern fn(*mut c_void) -> *mut c_void, _: *mut c_void) -> c_int;
    fn pthread_detach(thread: pthread_t) -> c_int;
}

#[repr(C)]
pub struct android_poll_source {
    pub id: i32,      // can be LOOPER_ID_MAIN, LOOPER_ID_INPUT or LOOPER_ID_USER
    pub app: *mut android_app,
    pub process: extern fn(*mut android_app, *mut android_poll_source),
}

/*
 * android_native_app_glue.h
 * 这是线程应用程序的标准粘合代码的接口。 
 * 在此模型中，应用程序的代码在其自己的线程中运行，与进程的主线程分开。 
 * 这个线程不需要与Java VM相关联，尽管它需要使JNI调用任何Java对象。
 */
#[repr(C)]
pub struct android_app {
    //应用程序可以在此处放置指向其自身状态对象的指针。
    pub userData: *mut c_void,
    pub onAppCmd: extern fn(*mut android_app, i32),
    pub onInputEvent: extern fn(*mut android_app, *const AInputEvent) -> i32,
    pub activity: *const ANativeActivity,
    pub config: *const AConfiguration,
    pub savedState: *mut c_void,
    pub savedStateSize: usize,
    pub looper: *mut ALooper,
    pub inputQueue: *const AInputQueue,
    pub window: *const ANativeWindow,
    pub contentRect: ARect,
    pub activityState: c_int,
    pub destroyRequested: c_int,
    mutex: pthread_mutex_t,
    cond: pthread_cond_t,
    msgread: c_int,
    msgwrite: c_int,
    thread: pthread_t,
    cmdPollSource: android_poll_source,
    inputPollSource: android_poll_source,
    running: c_int,
    stateSaved: c_int,
    destroyed: c_int,
    redrawNeeded: c_int,
    pendingInputQueue: *mut AInputQueue,
    pendingWindow: *mut ANativeWindow,
    pendingContentRect: ARect,
}

pub const LOOPER_ID_MAIN: i32 = 1;
pub const LOOPER_ID_INPUT: i32 = 1;
pub const LOOPER_ID_USER: i32 = 1;

pub const APP_CMD_INPUT_CHANGED: i32 = 0;
pub const APP_CMD_INIT_WINDOW: i32 = 1;
pub const APP_CMD_TERM_WINDOW: i32 = 2;
pub const APP_CMD_WINDOW_RESIZED: i32 = 3;
pub const APP_CMD_WINDOW_REDRAW_NEEDED: i32 = 4;
pub const APP_CMD_CONTENT_RECT_CHANGED: i32 = 5;
pub const APP_CMD_GAINED_FOCUS: i32 = 6;
pub const APP_CMD_LOST_FOCUS: i32 = 7;
pub const APP_CMD_CONFIG_CHANGED: i32 = 8;
pub const APP_CMD_LOW_MEMORY: i32 = 9;
pub const APP_CMD_START: i32 = 10;
pub const APP_CMD_RESUME: i32 = 11;
pub const APP_CMD_SAVE_STATE: i32 = 12;
pub const APP_CMD_PAUSE: i32 = 13;
pub const APP_CMD_STOP: i32 = 14;
pub const APP_CMD_DESTROY: i32 = 15;

/**
 * Call when ALooper_pollAll() returns LOOPER_ID_MAIN, reading the next
 * app command message.
 */
fn android_app_read_cmd(android_app: *mut android_app) -> int8_t{
    let mut cmd:int8_t = 0;
    if (read(android_app.msgread, &cmd, size_of(cmd)) == size_of(cmd)) {
        switch (cmd) {
            case APP_CMD_SAVE_STATE:
                free_saved_state(android_app);
                break;
        }
        return cmd;
    } else {
        LOGE("No data on command pipe!");
    }
    return -1;
}

/**
 * Call with the command returned by android_app_read_cmd() to do the
 * initial pre-processing of the given command.  You can perform your own
 * actions for the command after calling this function.
 */
void android_app_pre_exec_cmd(struct android_app* android_app, int8_t cmd);

/**
 * Call with the command returned by android_app_read_cmd() to do the
 * final post-processing of the given command.  You must have done your own
 * actions for the command before calling this function.
 */
void android_app_post_exec_cmd(struct android_app* android_app, int8_t cmd);