#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use jni::sys::{jobject, JNIEnv, JavaVM};
use std::os::raw::{c_void, c_char,c_int, c_float};
use native_window::{ARect, ANativeWindow};

pub type AInputQueue = c_void;
pub type ALooper = c_void;
pub type AInputEvent = c_void;
pub type ALooper_callbackFunc = extern fn(c_int, c_int, *mut c_void) -> c_int;
pub type AAssetManager = c_void;

#[repr(C)]
pub struct ANativeActivity {
     pub callbacks:             *mut ANativeActivityCallbacks,
     pub vm:                *mut JavaVM,
     pub env:               *mut JNIEnv,
     pub clazz:             jobject,
     pub internalDataPath:              *const c_char,
     pub externalDataPath:              *const c_char,
     pub sdkVersion:                i32,
     pub instance:              *mut c_void,
     pub assetManager:              *mut AAssetManager,
     pub obbPath:               *const c_char,
}

#[repr(C)]
pub struct ANativeActivityCallbacks {
     pub onStart:               extern fn(*mut ANativeActivity),
     pub onResume:              extern fn(*mut ANativeActivity),
     pub onSaveInstanceState:               extern fn(*mut ANativeActivity, *mut usize) -> *mut c_void,
     pub onPause:               extern fn(*mut ANativeActivity),
     pub onStop:                extern fn(*mut ANativeActivity),
     pub onDestroy:             extern fn(*mut ANativeActivity),
     pub onWindowFocusChanged:              extern fn(*mut ANativeActivity, c_int),
     pub onNativeWindowCreated:             extern fn(*mut ANativeActivity, *mut ANativeWindow),
     pub onNativeWindowResized:             extern fn(*mut ANativeActivity, *mut ANativeWindow),
     pub onNativeWindowRedrawNeeded:                extern fn(*mut ANativeActivity, *mut ANativeWindow),
     pub onNativeWindowDestroyed:               extern fn(*mut ANativeActivity, *mut ANativeWindow),
     pub onInputQueueCreated:               extern fn(*mut ANativeActivity, *mut AInputQueue),
     pub onInputQueueDestroyed:             extern fn(*mut ANativeActivity, *mut AInputQueue),
     pub onContentRectChanged:              extern fn(*mut ANativeActivity, *const ARect),
     pub onConfigurationChanged:                extern fn(*mut ANativeActivity),
     pub onLowMemory:               extern fn(*mut ANativeActivity),
}

//
//       android/keycodes.h
//
pub const AKEYCODE_0: i32 = 7;
pub const AKEYCODE_1: i32 = 8;
pub const AKEYCODE_11: i32 = 227;
pub const AKEYCODE_12: i32 = 228;
pub const AKEYCODE_2: i32 = 9;
pub const AKEYCODE_3: i32 = 10;
pub const AKEYCODE_3D_MODE: i32 = 206;
pub const AKEYCODE_4: i32 = 11;
pub const AKEYCODE_5: i32 = 12;
pub const AKEYCODE_6: i32 = 13;
pub const AKEYCODE_7: i32 = 14;
pub const AKEYCODE_8: i32 = 15;
pub const AKEYCODE_9: i32 = 16;
pub const AKEYCODE_A: i32 = 29;
pub const AKEYCODE_ALT_LEFT: i32 = 57;
pub const AKEYCODE_ALT_RIGHT: i32 = 58;
pub const AKEYCODE_APOSTROPHE: i32 = 75;
pub const AKEYCODE_APP_SWITCH: i32 = 187;
pub const AKEYCODE_ASSIST: i32 = 219;
pub const AKEYCODE_AT: i32 = 77;
pub const AKEYCODE_AVR_INPUT: i32 = 182;
pub const AKEYCODE_AVR_POWER: i32 = 181;
pub const AKEYCODE_B: i32 = 30;
pub const AKEYCODE_BACK: i32 = 4;
pub const AKEYCODE_BACKSLASH: i32 = 73;
pub const AKEYCODE_BOOKMARK: i32 = 174;
pub const AKEYCODE_BREAK: i32 = 121;
pub const AKEYCODE_BRIGHTNESS_DOWN: i32 = 220;
pub const AKEYCODE_BRIGHTNESS_UP: i32 = 221;
pub const AKEYCODE_BUTTON_1: i32 = 188;
pub const AKEYCODE_BUTTON_10: i32 = 197;
pub const AKEYCODE_BUTTON_11: i32 = 198;
pub const AKEYCODE_BUTTON_12: i32 = 199;
pub const AKEYCODE_BUTTON_13: i32 = 200;
pub const AKEYCODE_BUTTON_14: i32 = 201;
pub const AKEYCODE_BUTTON_15: i32 = 202;
pub const AKEYCODE_BUTTON_16: i32 = 203;
pub const AKEYCODE_BUTTON_2: i32 = 189;
pub const AKEYCODE_BUTTON_3: i32 = 190;
pub const AKEYCODE_BUTTON_4: i32 = 191;
pub const AKEYCODE_BUTTON_5: i32 = 192;
pub const AKEYCODE_BUTTON_6: i32 = 193;
pub const AKEYCODE_BUTTON_7: i32 = 194;
pub const AKEYCODE_BUTTON_8: i32 = 195;
pub const AKEYCODE_BUTTON_9: i32 = 196;
pub const AKEYCODE_BUTTON_A: i32 = 96;
pub const AKEYCODE_BUTTON_B: i32 = 97;
pub const AKEYCODE_BUTTON_C: i32 = 98;
pub const AKEYCODE_BUTTON_L1: i32 = 102;
pub const AKEYCODE_BUTTON_L2: i32 = 104;
pub const AKEYCODE_BUTTON_MODE: i32 = 110;
pub const AKEYCODE_BUTTON_R1: i32 = 103;
pub const AKEYCODE_BUTTON_R2: i32 = 105;
pub const AKEYCODE_BUTTON_SELECT: i32 = 109;
pub const AKEYCODE_BUTTON_START: i32 = 108;
pub const AKEYCODE_BUTTON_THUMBL: i32 = 106;
pub const AKEYCODE_BUTTON_THUMBR: i32 = 107;
pub const AKEYCODE_BUTTON_X: i32 = 99;
pub const AKEYCODE_BUTTON_Y: i32 = 100;
pub const AKEYCODE_BUTTON_Z: i32 = 101;
pub const AKEYCODE_C: i32 = 31;
pub const AKEYCODE_CALCULATOR: i32 = 210;
pub const AKEYCODE_CALENDAR: i32 = 208;
pub const AKEYCODE_CALL: i32 = 5;
pub const AKEYCODE_CAMERA: i32 = 27;
pub const AKEYCODE_CAPS_LOCK: i32 = 115;
pub const AKEYCODE_CAPTIONS: i32 = 175;
pub const AKEYCODE_CHANNEL_DOWN: i32 = 167;
pub const AKEYCODE_CHANNEL_UP: i32 = 166;
pub const AKEYCODE_CLEAR: i32 = 28;
pub const AKEYCODE_COMMA: i32 = 55;
pub const AKEYCODE_CONTACTS: i32 = 207;
pub const AKEYCODE_CTRL_LEFT: i32 = 113;
pub const AKEYCODE_CTRL_RIGHT: i32 = 114;
pub const AKEYCODE_D: i32 = 32;
pub const AKEYCODE_DEL: i32 = 67;
pub const AKEYCODE_DPAD_CENTER: i32 = 23;
pub const AKEYCODE_DPAD_DOWN: i32 = 20;
pub const AKEYCODE_DPAD_LEFT: i32 = 21;
pub const AKEYCODE_DPAD_RIGHT: i32 = 22;
pub const AKEYCODE_DPAD_UP: i32 = 19;
pub const AKEYCODE_DVR: i32 = 173;
pub const AKEYCODE_E: i32 = 33;
pub const AKEYCODE_EISU: i32 = 212;
pub const AKEYCODE_ENDCALL: i32 = 6;
pub const AKEYCODE_ENTER: i32 = 66;
pub const AKEYCODE_ENVELOPE: i32 = 65;
pub const AKEYCODE_EQUALS: i32 = 70;
pub const AKEYCODE_ESCAPE: i32 = 111;
pub const AKEYCODE_EXPLORER: i32 = 64;
pub const AKEYCODE_F: i32 = 34;
pub const AKEYCODE_F1: i32 = 131;
pub const AKEYCODE_F10: i32 = 140;
pub const AKEYCODE_F11: i32 = 141;
pub const AKEYCODE_F12: i32 = 142;
pub const AKEYCODE_F2: i32 = 132;
pub const AKEYCODE_F3: i32 = 133;
pub const AKEYCODE_F4: i32 = 134;
pub const AKEYCODE_F5: i32 = 135;
pub const AKEYCODE_F6: i32 = 136;
pub const AKEYCODE_F7: i32 = 137;
pub const AKEYCODE_F8: i32 = 138;
pub const AKEYCODE_F9: i32 = 139;
pub const AKEYCODE_FOCUS: i32 = 80;
pub const AKEYCODE_FORWARD: i32 = 125;
pub const AKEYCODE_FORWARD_DEL: i32 = 112;
pub const AKEYCODE_FUNCTION: i32 = 119;
pub const AKEYCODE_G: i32 = 35;
pub const AKEYCODE_GRAVE: i32 = 68;
pub const AKEYCODE_GUIDE: i32 = 172;
pub const AKEYCODE_H: i32 = 36;
pub const AKEYCODE_HEADSETHOOK: i32 = 79;
pub const AKEYCODE_HELP: i32 = 259;
pub const AKEYCODE_HENKAN: i32 = 214;
pub const AKEYCODE_HOME: i32 = 3;
pub const AKEYCODE_I: i32 = 37;
pub const AKEYCODE_INFO: i32 = 165;
pub const AKEYCODE_INSERT: i32 = 124;
pub const AKEYCODE_J: i32 = 38;
pub const AKEYCODE_K: i32 = 39;
pub const AKEYCODE_KANA: i32 = 218;
pub const AKEYCODE_KATAKANA_HIRAGANA: i32 = 215;
pub const AKEYCODE_L: i32 = 40;
pub const AKEYCODE_LANGUAGE_SWITCH: i32 = 204;
pub const AKEYCODE_LAST_CHANNEL: i32 = 229;
pub const AKEYCODE_LEFT_BRACKET: i32 = 71;
pub const AKEYCODE_M: i32 = 41;
pub const AKEYCODE_MANNER_MODE: i32 = 205;
pub const AKEYCODE_MEDIA_AUDIO_TRACK: i32 = 222;
pub const AKEYCODE_MEDIA_CLOSE: i32 = 128;
pub const AKEYCODE_MEDIA_EJECT: i32 = 129;
pub const AKEYCODE_MEDIA_FAST_FORWARD: i32 = 90;
pub const AKEYCODE_MEDIA_NEXT: i32 = 87;
pub const AKEYCODE_MEDIA_PAUSE: i32 = 127;
pub const AKEYCODE_MEDIA_PLAY: i32 = 126;
pub const AKEYCODE_MEDIA_PLAY_PAUSE: i32 = 85;
pub const AKEYCODE_MEDIA_PREVIOUS: i32 = 88;
pub const AKEYCODE_MEDIA_RECORD: i32 = 130;
pub const AKEYCODE_MEDIA_REWIND: i32 = 89;
pub const AKEYCODE_MEDIA_STOP: i32 = 86;
pub const AKEYCODE_MEDIA_TOP_MENU: i32 = 226;
pub const AKEYCODE_MENU: i32 = 82;
pub const AKEYCODE_META_LEFT: i32 = 117;
pub const AKEYCODE_META_RIGHT: i32 = 118;
pub const AKEYCODE_MINUS: i32 = 69;
pub const AKEYCODE_MOVE_END: i32 = 123;
pub const AKEYCODE_MOVE_HOME: i32 = 122;
pub const AKEYCODE_MUHENKAN: i32 = 213;
pub const AKEYCODE_MUSIC: i32 = 209;
pub const AKEYCODE_MUTE: i32 = 91;
pub const AKEYCODE_N: i32 = 42;
pub const AKEYCODE_NOTIFICATION: i32 = 83;
pub const AKEYCODE_NUM: i32 = 78;
pub const AKEYCODE_NUMPAD_0: i32 = 144;
pub const AKEYCODE_NUMPAD_1: i32 = 145;
pub const AKEYCODE_NUMPAD_2: i32 = 146;
pub const AKEYCODE_NUMPAD_3: i32 = 147;
pub const AKEYCODE_NUMPAD_4: i32 = 148;
pub const AKEYCODE_NUMPAD_5: i32 = 149;
pub const AKEYCODE_NUMPAD_6: i32 = 150;
pub const AKEYCODE_NUMPAD_7: i32 = 151;
pub const AKEYCODE_NUMPAD_8: i32 = 152;
pub const AKEYCODE_NUMPAD_9: i32 = 153;
pub const AKEYCODE_NUMPAD_ADD: i32 = 157;
pub const AKEYCODE_NUMPAD_COMMA: i32 = 159;
pub const AKEYCODE_NUMPAD_DIVIDE: i32 = 154;
pub const AKEYCODE_NUMPAD_DOT: i32 = 158;
pub const AKEYCODE_NUMPAD_ENTER: i32 = 160;
pub const AKEYCODE_NUMPAD_EQUALS: i32 = 161;
pub const AKEYCODE_NUMPAD_LEFT_PAREN: i32 = 162;
pub const AKEYCODE_NUMPAD_MULTIPLY: i32 = 155;
pub const AKEYCODE_NUMPAD_RIGHT_PAREN: i32 = 163;
pub const AKEYCODE_NUMPAD_SUBTRACT: i32 = 156;
pub const AKEYCODE_NUM_LOCK: i32 = 143;
pub const AKEYCODE_O: i32 = 43;
pub const AKEYCODE_P: i32 = 44;
pub const AKEYCODE_PAGE_DOWN: i32 = 93;
pub const AKEYCODE_PAGE_UP: i32 = 92;
pub const AKEYCODE_PAIRING: i32 = 225;
pub const AKEYCODE_PERIOD: i32 = 56;
pub const AKEYCODE_PICTSYMBOLS: i32 = 94;
pub const AKEYCODE_PLUS: i32 = 81;
pub const AKEYCODE_POUND: i32 = 18;
pub const AKEYCODE_POWER: i32 = 26;
pub const AKEYCODE_PROG_BLUE: i32 = 186;
pub const AKEYCODE_PROG_GREEN: i32 = 184;
pub const AKEYCODE_PROG_RED: i32 = 183;
pub const AKEYCODE_PROG_YELLOW: i32 = 185;
pub const AKEYCODE_Q: i32 = 45;
pub const AKEYCODE_R: i32 = 46;
pub const AKEYCODE_RIGHT_BRACKET: i32 = 72;
pub const AKEYCODE_RO: i32 = 217;
pub const AKEYCODE_S: i32 = 47;
pub const AKEYCODE_SCROLL_LOCK: i32 = 116;
pub const AKEYCODE_SEARCH: i32 = 84;
pub const AKEYCODE_SEMICOLON: i32 = 74;
pub const AKEYCODE_SETTINGS: i32 = 176;
pub const AKEYCODE_SHIFT_LEFT: i32 = 59;
pub const AKEYCODE_SHIFT_RIGHT: i32 = 60;
pub const AKEYCODE_SLASH: i32 = 76;
pub const AKEYCODE_SLEEP: i32 = 223;
pub const AKEYCODE_SOFT_LEFT: i32 = 1;
pub const AKEYCODE_SOFT_RIGHT: i32 = 2;
pub const AKEYCODE_SPACE: i32 = 62;
pub const AKEYCODE_STAR: i32 = 17;
pub const AKEYCODE_STB_INPUT: i32 = 180;
pub const AKEYCODE_STB_POWER: i32 = 179;
pub const AKEYCODE_SWITCH_CHARSET: i32 = 95;
pub const AKEYCODE_SYM: i32 = 63;
pub const AKEYCODE_SYSRQ: i32 = 120;
pub const AKEYCODE_T: i32 = 48;
pub const AKEYCODE_TAB: i32 = 61;
pub const AKEYCODE_TV: i32 = 170;
pub const AKEYCODE_TV_ANTENNA_CABLE: i32 = 242;
pub const AKEYCODE_TV_AUDIO_DESCRIPTION: i32 = 252;
pub const AKEYCODE_TV_AUDIO_DESCRIPTION_MIX_DOWN: i32 = 254;
pub const AKEYCODE_TV_AUDIO_DESCRIPTION_MIX_UP: i32 = 253;
pub const AKEYCODE_TV_CONTENTS_MENU: i32 = 256;
pub const AKEYCODE_TV_DATA_SERVICE: i32 = 230;
pub const AKEYCODE_TV_INPUT: i32 = 178;
pub const AKEYCODE_TV_INPUT_COMPONENT_1: i32 = 249;
pub const AKEYCODE_TV_INPUT_COMPONENT_2: i32 = 250;
pub const AKEYCODE_TV_INPUT_COMPOSITE_1: i32 = 247;
pub const AKEYCODE_TV_INPUT_COMPOSITE_2: i32 = 248;
pub const AKEYCODE_TV_INPUT_HDMI_1: i32 = 243;
pub const AKEYCODE_TV_INPUT_HDMI_2: i32 = 244;
pub const AKEYCODE_TV_INPUT_HDMI_3: i32 = 245;
pub const AKEYCODE_TV_INPUT_HDMI_4: i32 = 246;
pub const AKEYCODE_TV_INPUT_VGA_1: i32 = 251;
pub const AKEYCODE_TV_MEDIA_CONTEXT_MENU: i32 = 257;
pub const AKEYCODE_TV_NETWORK: i32 = 241;
pub const AKEYCODE_TV_NUMBER_ENTRY: i32 = 234;
pub const AKEYCODE_TV_POWER: i32 = 177;
pub const AKEYCODE_TV_RADIO_SERVICE: i32 = 232;
pub const AKEYCODE_TV_SATELLITE: i32 = 237;
pub const AKEYCODE_TV_SATELLITE_BS: i32 = 238;
pub const AKEYCODE_TV_SATELLITE_CS: i32 = 239;
pub const AKEYCODE_TV_SATELLITE_SERVICE: i32 = 240;
pub const AKEYCODE_TV_TELETEXT: i32 = 233;
pub const AKEYCODE_TV_TERRESTRIAL_ANALOG: i32 = 235;
pub const AKEYCODE_TV_TERRESTRIAL_DIGITAL: i32 = 236;
pub const AKEYCODE_TV_TIMER_PROGRAMMING: i32 = 258;
pub const AKEYCODE_TV_ZOOM_MODE: i32 = 255;
pub const AKEYCODE_U: i32 = 49;
pub const AKEYCODE_UNKNOWN: i32 = 0;
pub const AKEYCODE_V: i32 = 50;
pub const AKEYCODE_VOICE_ASSIST: i32 = 231;
pub const AKEYCODE_VOLUME_DOWN: i32 = 25;
pub const AKEYCODE_VOLUME_MUTE: i32 = 164;
pub const AKEYCODE_VOLUME_UP: i32 = 24;
pub const AKEYCODE_W: i32 = 51;
pub const AKEYCODE_WAKEUP: i32 = 224;
pub const AKEYCODE_WINDOW: i32 = 171;
pub const AKEYCODE_X: i32 = 52;
pub const AKEYCODE_Y: i32 = 53;
pub const AKEYCODE_YEN: i32 = 216;
pub const AKEYCODE_Z: i32 = 54;
pub const AKEYCODE_ZENKAKU_HANKAKU: i32 = 211;
pub const AKEYCODE_ZOOM_IN: i32 = 168;
pub const AKEYCODE_ZOOM_OUT: i32 = 169;

//
//       android/input.h
//
pub const AINPUT_EVENT_TYPE_KEY: i32 = 1;
pub const AINPUT_EVENT_TYPE_MOTION: i32 = 2;
pub const AINPUT_KEYBOARD_TYPE_ALPHABETIC: i32 = 2;
pub const AINPUT_KEYBOARD_TYPE_NONE: i32 = 0;
pub const AINPUT_KEYBOARD_TYPE_NON_ALPHABETIC: i32 = 1;
pub const AINPUT_MOTION_RANGE_ORIENTATION: i32 = 8;
pub const AINPUT_MOTION_RANGE_PRESSURE: i32 = 2;
pub const AINPUT_MOTION_RANGE_SIZE: i32 = 3;
pub const AINPUT_MOTION_RANGE_TOOL_MAJOR: i32 = 6;
pub const AINPUT_MOTION_RANGE_TOOL_MINOR: i32 = 7;
pub const AINPUT_MOTION_RANGE_TOUCH_MAJOR: i32 = 4;
pub const AINPUT_MOTION_RANGE_TOUCH_MINOR: i32 = 5;
pub const AINPUT_MOTION_RANGE_X: i32 = 0;
pub const AINPUT_MOTION_RANGE_Y: i32 = 1;
pub const AINPUT_SOURCE_ANY: i32 = -256;
pub const AINPUT_SOURCE_CLASS_BUTTON: i32 = 1;
pub const AINPUT_SOURCE_CLASS_JOYSTICK: i32 = 16;
pub const AINPUT_SOURCE_CLASS_MASK: i32 = 255;
pub const AINPUT_SOURCE_CLASS_NAVIGATION: i32 = 4;
pub const AINPUT_SOURCE_CLASS_NONE: i32 = 0;
pub const AINPUT_SOURCE_CLASS_POINTER: i32 = 2;
pub const AINPUT_SOURCE_CLASS_POSITION: i32 = 8;
pub const AINPUT_SOURCE_DPAD: i32 = 513;
pub const AINPUT_SOURCE_GAMEPAD: i32 = 1025;
pub const AINPUT_SOURCE_JOYSTICK: i32 = 16777232;
pub const AINPUT_SOURCE_KEYBOARD: i32 = 257;
pub const AINPUT_SOURCE_MOUSE: i32 = 8194;
pub const AINPUT_SOURCE_STYLUS: i32 = 16386;
pub const AINPUT_SOURCE_TOUCHPAD: i32 = 1048584;
pub const AINPUT_SOURCE_TOUCHSCREEN: i32 = 4098;
pub const AINPUT_SOURCE_TOUCH_NAVIGATION: i32 = 2097152;
pub const AINPUT_SOURCE_TRACKBALL: i32 = 65540;
pub const AINPUT_SOURCE_UNKNOWN: i32 = 0;
extern { pub fn AInputEvent_getDeviceId(event: *const AInputEvent) -> i32; }
extern { pub fn AInputEvent_getSource(event: *const AInputEvent) -> i32; }
pub const AKEY_EVENT_ACTION_DOWN: i32 = 0;
pub const AKEY_EVENT_ACTION_MULTIPLE: i32 = 2;
pub const AKEY_EVENT_ACTION_UP: i32 = 1;
pub const AKEY_EVENT_FLAG_CANCELED: i32 = 32;
pub const AKEY_EVENT_FLAG_CANCELED_LONG_PRESS: i32 = 256;
pub const AKEY_EVENT_FLAG_EDITOR_ACTION: i32 = 16;
pub const AKEY_EVENT_FLAG_FALLBACK: i32 = 1024;
pub const AKEY_EVENT_FLAG_FROM_SYSTEM: i32 = 8;
pub const AKEY_EVENT_FLAG_KEEP_TOUCH_MODE: i32 = 4;
pub const AKEY_EVENT_FLAG_LONG_PRESS: i32 = 128;
pub const AKEY_EVENT_FLAG_SOFT_KEYBOARD: i32 = 2;
pub const AKEY_EVENT_FLAG_TRACKING: i32 = 512;
pub const AKEY_EVENT_FLAG_VIRTUAL_HARD_KEY: i32 = 64;
pub const AKEY_EVENT_FLAG_WOKE_HERE: i32 = 1;
pub const AKEY_STATE_DOWN: i32 = 1;
pub const AKEY_STATE_UNKNOWN: i32 = -1;
pub const AKEY_STATE_UP: i32 = 0;
pub const AKEY_STATE_VIRTUAL: i32 = 2;
extern { pub fn AKeyEvent_getAction(key_event: *const AInputEvent) -> i32; }
extern { pub fn AKeyEvent_getDownTime(key_event: *const AInputEvent) -> i64; }
extern { pub fn AKeyEvent_getEventTime(key_event: *const AInputEvent) -> i64; }
extern { pub fn AKeyEvent_getFlags(key_event: *const AInputEvent) -> i32; }
extern { pub fn AKeyEvent_getMetaState(key_event: *const AInputEvent) -> i32; }
extern { pub fn AKeyEvent_getRepeatCount(key_event: *const AInputEvent) -> i32; }
extern { pub fn AKeyEvent_getScanCode(key_event: *const AInputEvent) -> i32; }
pub const AMETA_ALT_LEFT_ON: i32 = 16;
pub const AMETA_ALT_ON: i32 = 2;
pub const AMETA_ALT_RIGHT_ON: i32 = 32;
pub const AMETA_CAPS_LOCK_ON: i32 = 1048576;
pub const AMETA_CTRL_LEFT_ON: i32 = 8192;
pub const AMETA_CTRL_ON: i32 = 4096;
pub const AMETA_CTRL_RIGHT_ON: i32 = 16384;
pub const AMETA_FUNCTION_ON: i32 = 8;
pub const AMETA_META_LEFT_ON: i32 = 131072;
pub const AMETA_META_ON: i32 = 65536;
pub const AMETA_META_RIGHT_ON: i32 = 262144;
pub const AMETA_NONE: i32 = 0;
pub const AMETA_NUM_LOCK_ON: i32 = 2097152;
pub const AMETA_SCROLL_LOCK_ON: i32 = 4194304;
pub const AMETA_SHIFT_LEFT_ON: i32 = 64;
pub const AMETA_SHIFT_ON: i32 = 1;
pub const AMETA_SHIFT_RIGHT_ON: i32 = 128;
pub const AMETA_SYM_ON: i32 = 4;
pub const AMOTION_EVENT_ACTION_CANCEL: i32 = 3;
pub const AMOTION_EVENT_ACTION_DOWN: i32 = 0;
pub const AMOTION_EVENT_ACTION_HOVER_ENTER: i32 = 9;
pub const AMOTION_EVENT_ACTION_HOVER_EXIT: i32 = 10;
pub const AMOTION_EVENT_ACTION_HOVER_MOVE: i32 = 7;
pub const AMOTION_EVENT_ACTION_MASK: i32 = 255;
pub const AMOTION_EVENT_ACTION_MOVE: i32 = 2;
pub const AMOTION_EVENT_ACTION_OUTSIDE: i32 = 4;
pub const AMOTION_EVENT_ACTION_POINTER_DOWN: i32 = 5;
pub const AMOTION_EVENT_ACTION_POINTER_INDEX_MASK: i32 = 65280;
pub const AMOTION_EVENT_ACTION_POINTER_INDEX_SHIFT: i32 = 8;
pub const AMOTION_EVENT_ACTION_POINTER_UP: i32 = 6;
pub const AMOTION_EVENT_ACTION_SCROLL: i32 = 8;
pub const AMOTION_EVENT_ACTION_UP: i32 = 1;
pub const AMOTION_EVENT_AXIS_BRAKE: i32 = 23;
pub const AMOTION_EVENT_AXIS_DISTANCE: i32 = 24;
pub const AMOTION_EVENT_AXIS_GAS: i32 = 22;
pub const AMOTION_EVENT_AXIS_GENERIC_1: i32 = 32;
pub const AMOTION_EVENT_AXIS_GENERIC_10: i32 = 41;
pub const AMOTION_EVENT_AXIS_GENERIC_11: i32 = 42;
pub const AMOTION_EVENT_AXIS_GENERIC_12: i32 = 43;
pub const AMOTION_EVENT_AXIS_GENERIC_13: i32 = 44;
pub const AMOTION_EVENT_AXIS_GENERIC_14: i32 = 45;
pub const AMOTION_EVENT_AXIS_GENERIC_15: i32 = 46;
pub const AMOTION_EVENT_AXIS_GENERIC_16: i32 = 47;
pub const AMOTION_EVENT_AXIS_GENERIC_2: i32 = 33;
pub const AMOTION_EVENT_AXIS_GENERIC_3: i32 = 34;
pub const AMOTION_EVENT_AXIS_GENERIC_4: i32 = 35;
pub const AMOTION_EVENT_AXIS_GENERIC_5: i32 = 36;
pub const AMOTION_EVENT_AXIS_GENERIC_6: i32 = 37;
pub const AMOTION_EVENT_AXIS_GENERIC_7: i32 = 38;
pub const AMOTION_EVENT_AXIS_GENERIC_8: i32 = 39;
pub const AMOTION_EVENT_AXIS_GENERIC_9: i32 = 40;
pub const AMOTION_EVENT_AXIS_HAT_X: i32 = 15;
pub const AMOTION_EVENT_AXIS_HAT_Y: i32 = 16;
pub const AMOTION_EVENT_AXIS_HSCROLL: i32 = 10;
pub const AMOTION_EVENT_AXIS_LTRIGGER: i32 = 17;
pub const AMOTION_EVENT_AXIS_ORIENTATION: i32 = 8;
pub const AMOTION_EVENT_AXIS_PRESSURE: i32 = 2;
pub const AMOTION_EVENT_AXIS_RTRIGGER: i32 = 18;
pub const AMOTION_EVENT_AXIS_RUDDER: i32 = 20;
pub const AMOTION_EVENT_AXIS_RX: i32 = 12;
pub const AMOTION_EVENT_AXIS_RY: i32 = 13;
pub const AMOTION_EVENT_AXIS_RZ: i32 = 14;
pub const AMOTION_EVENT_AXIS_SIZE: i32 = 3;
pub const AMOTION_EVENT_AXIS_THROTTLE: i32 = 19;
pub const AMOTION_EVENT_AXIS_TILT: i32 = 25;
pub const AMOTION_EVENT_AXIS_TOOL_MAJOR: i32 = 6;
pub const AMOTION_EVENT_AXIS_TOOL_MINOR: i32 = 7;
pub const AMOTION_EVENT_AXIS_TOUCH_MAJOR: i32 = 4;
pub const AMOTION_EVENT_AXIS_TOUCH_MINOR: i32 = 5;
pub const AMOTION_EVENT_AXIS_VSCROLL: i32 = 9;
pub const AMOTION_EVENT_AXIS_WHEEL: i32 = 21;
pub const AMOTION_EVENT_AXIS_X: i32 = 0;
pub const AMOTION_EVENT_AXIS_Y: i32 = 1;
pub const AMOTION_EVENT_AXIS_Z: i32 = 11;
pub const AMOTION_EVENT_BUTTON_BACK: i32 = 8;
pub const AMOTION_EVENT_BUTTON_FORWARD: i32 = 16;
pub const AMOTION_EVENT_BUTTON_PRIMARY: i32 = 1;
pub const AMOTION_EVENT_BUTTON_SECONDARY: i32 = 2;
pub const AMOTION_EVENT_BUTTON_TERTIARY: i32 = 4;
pub const AMOTION_EVENT_EDGE_FLAG_BOTTOM: i32 = 2;
pub const AMOTION_EVENT_EDGE_FLAG_LEFT: i32 = 4;
pub const AMOTION_EVENT_EDGE_FLAG_NONE: i32 = 0;
pub const AMOTION_EVENT_EDGE_FLAG_RIGHT: i32 = 8;
pub const AMOTION_EVENT_EDGE_FLAG_TOP: i32 = 1;
pub const AMOTION_EVENT_FLAG_WINDOW_IS_OBSCURED: i32 = 1;
pub const AMOTION_EVENT_TOOL_TYPE_ERASER: i32 = 4;
pub const AMOTION_EVENT_TOOL_TYPE_FINGER: i32 = 1;
pub const AMOTION_EVENT_TOOL_TYPE_MOUSE: i32 = 3;
pub const AMOTION_EVENT_TOOL_TYPE_STYLUS: i32 = 2;
pub const AMOTION_EVENT_TOOL_TYPE_UNKNOWN: i32 = 0;

#[link(name = "android")]
extern {
    pub fn AInputQueue_attachLooper(queue: *mut AInputQueue, looper: *mut ALooper, ident: c_int, callback: ALooper_callbackFunc, data: *mut c_void);
    pub fn AInputQueue_detachLooper(queue: *mut AInputQueue);
    pub fn AInputQueue_preDispatchEvent(queue: *mut AInputQueue, event: *mut AInputEvent) -> i32;
    pub fn AInputQueue_getEvent(queue: *mut AInputQueue, outEvent: *mut *mut AInputEvent) -> i32;
    pub fn AInputQueue_finishEvent(queue: *mut AInputQueue, event: *mut AInputEvent, handled: c_int);

    pub fn AKeyEvent_getKeyCode(key_event: *const AInputEvent) -> i32;
    pub fn AInputEvent_getType(event: *const AInputEvent) -> i32;
    pub fn AInputQueue_hasEvents(queue: *mut AInputQueue) -> i32;

    pub fn AMotionEvent_getAxisValue(motion_event: *const AInputEvent, axis: i32, pointer_index: usize) -> c_float;  
    pub fn AMotionEvent_getButtonState(motion_event: *const AInputEvent) -> i32;  
    pub fn AMotionEvent_getDownTime(motion_event: *const AInputEvent) -> i64;  
    pub fn AMotionEvent_getEdgeFlags(motion_event: *const AInputEvent) -> i32;  
    pub fn AMotionEvent_getEventTime(motion_event: *const AInputEvent) -> i64;  
    pub fn AMotionEvent_getFlags(motion_event: *const AInputEvent) -> i32;  
    pub fn AMotionEvent_getHistoricalAxisValue(motion_event: *const AInputEvent, axis: i32, pointer_index: usize, history_index: usize) -> c_float;  
    pub fn AMotionEvent_getHistoricalEventTime(motion_event: *const AInputEvent, history_index: usize) -> i64;  
    pub fn AMotionEvent_getHistoricalOrientation(motion_event: *const AInputEvent, pointer_index: usize, history_index: usize) -> c_float;  
    pub fn AMotionEvent_getHistoricalPressure(motion_event: *const AInputEvent, pointer_index: usize, history_index: usize) -> c_float;  
    pub fn AMotionEvent_getHistoricalRawX(motion_event: *const AInputEvent, pointer_index: usize, history_index: usize) -> c_float;  
    pub fn AMotionEvent_getHistoricalRawY(motion_event: *const AInputEvent, pointer_index: usize, history_index: usize) -> c_float;  
    pub fn AMotionEvent_getHistoricalSize(motion_event: *const AInputEvent, pointer_index: usize, history_index: usize) -> c_float;  
    pub fn AMotionEvent_getHistoricalToolMajor(motion_event: *const AInputEvent, pointer_index: usize, history_index: usize) -> c_float;  
    pub fn AMotionEvent_getHistoricalToolMinor(motion_event: *const AInputEvent, pointer_index: usize, history_index: usize) -> c_float;  
    pub fn AMotionEvent_getHistoricalTouchMajor(motion_event: *const AInputEvent, pointer_index: usize, history_index: usize) -> c_float;  
    pub fn AMotionEvent_getHistoricalTouchMinor(motion_event: *const AInputEvent, pointer_index: usize, history_index: usize) -> c_float;  
    pub fn AMotionEvent_getHistoricalX(motion_event: *const AInputEvent, pointer_index: usize, history_index: usize) -> c_float;  
    pub fn AMotionEvent_getHistoricalY(motion_event: *const AInputEvent, pointer_index: usize, history_index: usize) -> c_float;  
    pub fn AMotionEvent_getHistorySize(motion_event: *const AInputEvent) -> usize;  
    pub fn AMotionEvent_getMetaState(motion_event: *const AInputEvent) -> i32;  
    pub fn AMotionEvent_getOrientation(motion_event: *const AInputEvent, pointer_index: usize) -> c_float;  
    pub fn AMotionEvent_getPointerCount(motion_event: *const AInputEvent) -> usize;  
    pub fn AMotionEvent_getPointerId(motion_event: *const AInputEvent, pointer_index: usize) -> i32;  
    pub fn AMotionEvent_getPressure(motion_event: *const AInputEvent, pointer_index: usize) -> c_float;  
    pub fn AMotionEvent_getRawX(motion_event: *const AInputEvent, pointer_index: usize) -> c_float;  
    pub fn AMotionEvent_getRawY(motion_event: *const AInputEvent, pointer_index: usize) -> c_float;  
    pub fn AMotionEvent_getSize(motion_event: *const AInputEvent, pointer_index: usize) -> c_float;  
    pub fn AMotionEvent_getToolMajor(motion_event: *const AInputEvent, pointer_index: usize) -> c_float;  
    pub fn AMotionEvent_getToolMinor(motion_event: *const AInputEvent, pointer_index: usize) -> c_float;  
    pub fn AMotionEvent_getToolType(motion_event: *const AInputEvent, pointer_index: usize) -> i32;  
    pub fn AMotionEvent_getTouchMajor(motion_event: *const AInputEvent, pointer_index: usize) -> c_float;  
    pub fn AMotionEvent_getTouchMinor(motion_event: *const AInputEvent, pointer_index: usize) -> c_float;  
    pub fn AMotionEvent_getXOffset(motion_event: *const AInputEvent) -> c_float;  
    pub fn AMotionEvent_getXPrecision(motion_event: *const AInputEvent) -> c_float;  
    pub fn AMotionEvent_getAction(motion_event: *const AInputEvent) -> i32;
    pub fn AMotionEvent_getX(motion_event: *const AInputEvent, pointer_index: usize) -> c_float;
    pub fn AMotionEvent_getY(motion_event: *const AInputEvent, pointer_index: usize) -> c_float;
    pub fn AMotionEvent_getYOffset(motion_event: *const AInputEvent) -> c_float;
    pub fn AMotionEvent_getYPrecision(motion_event: *const AInputEvent) -> c_float;
}