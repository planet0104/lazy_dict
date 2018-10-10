extern crate sdl2;
#[macro_use] extern crate log;
extern crate android_logger;
extern crate jni_sys;
extern crate libc;
extern crate thread_id;

use log::Level;
use android_logger::Filter;

mod sdl_android;
pub use sdl_android::*;

#[no_mangle]
pub fn main(_argc: i32, _char:*mut u8){
	android_logger::init_once(Filter::default().with_min_level(Level::Trace));
    trace!("main>>>>>>>>>>>>>>>>>>>>>");
}