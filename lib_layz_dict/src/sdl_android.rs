use jni_sys::jclass;
use log::Level;
use android_logger;
use android_logger::Filter;
use jni_sys::*;
use std::ffi::CString;
use std::cell::RefCell;
use std::os::raw::c_void;

#[cfg(target_os="android")]
extern "C" {
    //fn SDL_Android_Init(env: JNIEnv, cls: JClass);
    fn SDL_SetMainReady();
}

thread_local!{
    /* Main activity */
    static ACTIVITY:RefCell<Activity> = RefCell::new(Activity::new());
}

struct Activity{
    java_vm: Option<JavaVM>,
    env: Option<JNIEnv>,
    activity_class: Option<jclass>,
    audio_manager_class: Option<jclass>,
    controller_manager_class: Option<jclass>,
    /* method signatures */
    mid_get_native_surface: Option<jmethodID>,
    mid_set_activity_title: Option<jmethodID>,
    mid_set_window_style: Option<jmethodID>,
    mid_set_orientation: Option<jmethodID>,
    mid_get_context: Option<jmethodID>,
    mid_is_android_tv: Option<jmethodID>,
    mid_input_get_input_device_ids: Option<jmethodID>,
    mid_send_message: Option<jmethodID>,
    mid_show_text_input: Option<jmethodID>,
    mid_is_screen_keyboard_shown: Option<jmethodID>,
    mid_clipboard_set_text: Option<jmethodID>,
    mid_clipboard_get_text: Option<jmethodID>,
    mid_clipboard_has_text: Option<jmethodID>,
    mid_open_apk_expansion_input_stream: Option<jmethodID>,
    mid_get_manifest_vnvironment_variables: Option<jmethodID>,
    mid_get_display_dpi: Option<jmethodID>,
    fid_separate_mouse_and_touch: Option<jfieldID>,
}

impl Activity{
    fn new() -> Activity{
        Activity{
            env: None,
            java_vm: None,
            activity_class: None,
            audio_manager_class: None,
            controller_manager_class: None,
            mid_get_native_surface: None,
            mid_set_activity_title: None,
            mid_set_window_style: None,
            mid_set_orientation: None,
            mid_get_context: None,
            mid_is_android_tv: None,
            mid_input_get_input_device_ids: None,
            mid_send_message: None,
            mid_show_text_input: None,
            mid_is_screen_keyboard_shown: None,
            mid_clipboard_set_text: None,
            mid_clipboard_get_text: None,
            mid_clipboard_has_text: None,
            mid_open_apk_expansion_input_stream: None,
            mid_get_manifest_vnvironment_variables: None,
            mid_get_display_dpi: None,
            fid_separate_mouse_and_touch: None,
        }
    }
}

fn c_string(s:&str) -> CString{
    CString::new(s).unwrap()
}

unsafe fn get_activity_static_method_id(activity: &Activity, s1: &str, s2: &str) -> jmethodID{
    ((*activity.env.unwrap()).GetStaticMethodID.unwrap())(&mut activity.env.unwrap(), activity.activity_class.unwrap(), c_string(s1).as_ptr(),c_string(s2).as_ptr())
}

unsafe fn get_activity_static_field_id(activity: &Activity, s1: &str, s2: &str) -> jfieldID{
    ((*activity.env.unwrap()).GetStaticFieldID.unwrap())(&mut activity.env.unwrap(), activity.activity_class.unwrap(), c_string(s1).as_ptr(),c_string(s2).as_ptr())
}

/* Activity initialization -- called before SDL_main() to initialize JNI bindings */
#[cfg(target_os="android")]
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Java_cn_jy_lazydict_sdl2_SDLActivity_nativeSetupJNI(mut env: JNIEnv, cls: jclass){
    trace!("SDL nativeSetupJNI() jclass={:?} is_null={}", cls, cls.is_null());

    android_jni_setup_thread();
    trace!("test>>>>001");
    let mid_get_native_surface = ((*env).GetStaticMethodID.unwrap())(&mut env, cls, c_string("getNativeSurface").as_ptr(),c_string("()Landroid/view/Surface;").as_ptr());
    trace!("test>>>>002 mid_get_native_surface={:?}", mid_get_native_surface);


    ACTIVITY.with(|activity|{
        trace!("init Activity.");
        let mut activity = activity.borrow_mut();
        trace!("init>>>>>>>>001 is_null? {}", env.is_null());
        trace!("(*env).NewGlobalRef.is_none = {}", (*env).NewGlobalRef.is_none());
        let f = (*env).NewGlobalRef.unwrap();
        trace!("init>>>>>>>>0011");
        activity.activity_class = Some((f)(&mut env, cls));
        trace!("init>>>>>>>>0012");
        activity.env = Some(env);
        trace!("init>>>>>>>>002");

        activity.mid_get_native_surface = Some(get_activity_static_method_id(&activity,
                                    "getNativeSurface", "()Landroid/view/Surface;"));
        trace!("init>>>>>>>>003");
        activity.mid_set_activity_title = Some(get_activity_static_method_id(&activity, 
                                    "setActivityTitle","(Ljava/lang/String;)Z"));
        activity.mid_set_window_style = Some(get_activity_static_method_id(&activity, 
                                    "setWindowStyle","(Z)V"));
        activity.mid_set_orientation = Some(get_activity_static_method_id(&activity, 
                                    "setOrientation","(IIZLjava/lang/String;)V"));
        activity.mid_get_context = Some(get_activity_static_method_id(&activity, 
                                    "getContext","()Landroid/content/Context;"));
        activity.mid_is_android_tv = Some(get_activity_static_method_id(&activity, 
                                    "isAndroidTV","()Z"));
        activity.mid_input_get_input_device_ids = Some(get_activity_static_method_id(&activity, 
                                    "inputGetInputDeviceIds", "(I)[I"));
        activity.mid_send_message = Some(get_activity_static_method_id(&activity, 
                                    "sendMessage", "(II)Z"));
        activity.mid_show_text_input =  Some(get_activity_static_method_id(&activity, 
                                    "showTextInput", "(IIII)Z"));
        activity.mid_is_screen_keyboard_shown = Some(get_activity_static_method_id(&activity, 
                                    "isScreenKeyboardShown","()Z"));
        activity.mid_clipboard_set_text = Some(get_activity_static_method_id(&activity, 
                                    "clipboardSetText", "(Ljava/lang/String;)V"));
        activity.mid_clipboard_get_text = Some(get_activity_static_method_id(&activity, 
                                    "clipboardGetText", "()Ljava/lang/String;"));
        activity.mid_clipboard_has_text = Some(get_activity_static_method_id(&activity, 
                                    "clipboardHasText", "()Z"));
        activity.mid_open_apk_expansion_input_stream = Some(get_activity_static_method_id(&activity, 
                                    "openAPKExpansionInputStream", "(Ljava/lang/String;)Ljava/io/InputStream;"));

        activity.mid_get_manifest_vnvironment_variables = Some(get_activity_static_method_id(&activity, 
                                    "getManifestEnvironmentVariables", "()Z"));

        activity.mid_get_display_dpi = Some(get_activity_static_method_id(&activity,  "getDisplayDPI", "()Landroid/util/DisplayMetrics;"));

        if  activity.mid_get_native_surface.unwrap().is_null() ||
            activity.mid_set_activity_title.unwrap().is_null() ||
            activity.mid_set_window_style.unwrap().is_null() ||
            activity.mid_set_orientation.unwrap().is_null() || 
            activity.mid_get_context.unwrap().is_null()||
            activity.mid_is_android_tv.unwrap().is_null() ||
            activity.mid_input_get_input_device_ids.unwrap().is_null() ||
            activity.mid_send_message.unwrap().is_null() ||
            activity.mid_show_text_input.unwrap().is_null() ||
            activity.mid_is_screen_keyboard_shown.unwrap().is_null() ||
            activity.mid_clipboard_set_text.unwrap().is_null() || 
            activity.mid_clipboard_get_text.unwrap().is_null() ||
            activity.mid_clipboard_has_text.unwrap().is_null() ||
            activity.mid_open_apk_expansion_input_stream.unwrap().is_null() ||
            activity.mid_get_manifest_vnvironment_variables.unwrap().is_null()||
            activity.mid_get_display_dpi.unwrap().is_null(){
            error!("SDL Missing some Java callbacks, do you have the latest version of SDLActivity.java?");
        }

        activity.fid_separate_mouse_and_touch = Some(get_activity_static_field_id(&activity, 
                                    "mSeparateMouseAndTouch", "Z"));

        if activity.fid_separate_mouse_and_touch.unwrap().is_null() {
            error!("SDL Missing some Java static fields, do you have the latest version of SDLActivity.java?");
        }
    });

    check_jni_ready();
}

/* Library init */
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn JNI_OnLoad(mut vm: JavaVM, _reserved: c_void) -> jint{
    android_logger::init_once(Filter::default().with_min_level(Level::Trace));
    trace!("JNI_OnLoad called");
    let mut env:* mut c_void = 0 as *mut c_void;
    if ((*vm).GetEnv.unwrap())(&mut vm, &mut env, JNI_VERSION_1_4) != JNI_OK{
        error!("Failed to get the environment using GetEnv()");
        return -1;
    }
    trace!("JNI_OnLoad>>>001");
    /*
     * Create mThreadKey so we can keep track of the JNIEnv assigned to each thread
     * Refer to http://developer.android.com/guide/practices/design/jni.html for the rationale behind this
     */
    // if (pthread_key_create(&mThreadKey, Android_JNI_ThreadDestroyed) != 0) {
    //     __android_log_print(ANDROID_LOG_ERROR, "SDL", "Error initializing pthread key");
    // }
    android_jni_setup_thread();
    trace!("JNI_OnLoad>>>002");

    ACTIVITY.with(|activity|{
       activity.borrow_mut().java_vm = Some(vm); 
    });
    trace!("JNI_OnLoad>>>003");

    return JNI_VERSION_1_4;
}

unsafe fn android_jni_get_env() -> Option<JNIEnv>{
    let mut env:* mut c_void = 0 as *mut c_void;
    let mut status = -1;
    ACTIVITY.with(|activity|{
        let mut java_vm = activity.borrow().java_vm.unwrap();
        status = ((*java_vm).AttachCurrentThread.unwrap())(&mut java_vm, &mut env, 0 as *mut c_void);
    });
    
    if status < 0{
        error!("failed to attach current thread");
        return None;
    }

    /* From http://developer.android.com/guide/practices/jni.html
     * Threads attached through JNI must call DetachCurrentThread before they exit. If coding this directly is awkward,
     * in Android 2.0 (Eclair) and higher you can use pthread_key_create to define a destructor function that will be
     * called before the thread exits, and call DetachCurrentThread from there. (Use that key with pthread_setspecific
     * to store the JNIEnv in thread-local-storage; that way it'll be passed into your destructor as the argument.)
     * Note: The destructor is not called unless the stored value is != NULL
     * Note: You can call this function any number of times for the same thread, there's no harm in it
     *       (except for some lost CPU cycles)
     */
    //pthread_setspecific(mThreadKey, env);

    return Some(env as JNIEnv);
}

unsafe fn android_jni_setup_thread() -> i32{
    let env = android_jni_get_env();
    trace!("android_jni_get_env> env={:?}", env);
    1
}

fn check_jni_ready(){
    trace!("check_jni_ready");
    ACTIVITY.with(|activity|{
        let activity = activity.borrow_mut();
        if activity.activity_class.is_none() || activity.audio_manager_class.is_none() || activity.controller_manager_class.is_none(){
            trace!("check_jni_ready:未初始化完毕");
        }else{
            unsafe{ SDL_SetMainReady(); }
        }
    });
}

