#[macro_use]
extern crate lazy_static;
#[macro_use] extern crate log;
extern crate android_logger;
extern crate zip;
// extern crate image;
use log::Level;
extern crate jni;
use std::time::Instant;
use self::jni::JNIEnv;
use self::jni::objects::{JClass, JValue, JByteBuffer};
use jni::sys::{jint, jobject, jbyteArray};
use std::os::raw::{c_void};
extern crate sdl2;
use sdl2::surface::Surface;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::pixels::PixelFormatEnum;
use std::thread;
mod utils;
extern crate libc;
use sdl2::rect::Rect;
extern crate bytes;
mod jni_graphics;
mod native_window;
mod native_activity;
use jni_graphics::*;
use native_window::*;
use native_activity::*;
use sdl2::rect::Point;
use std::cell::RefCell;
use std::sync::mpsc::{ Sender, Receiver, channel};
use std::sync::{Arc, Mutex};

pub struct InputQueue{
	queue: *mut AInputQueue
}
impl InputQueue{
	pub fn new(queue: *mut AInputQueue) -> InputQueue{
		InputQueue{queue}
	}
}
unsafe impl Send for InputQueue {}

pub struct Application{
	window : Option<*mut ANativeWindow>,
	activity: Option<*mut ANativeActivity>,
	event_loop_sender: Option<Sender<i32>>,
}
impl Application{
	pub fn new() -> Application{
		Application{ window: None, activity:None, event_loop_sender: None}
	}
}

thread_local!{
	pub static APP: RefCell<Application> = RefCell::new(Application::new());
}

extern "C" fn on_native_window_created(activity: *mut ANativeActivity, window: *mut ANativeWindow){
	trace!("on_native_window_created");
	//存储WINDOW
	APP.with(|app|{ app.borrow_mut().window = Some(window); });
}

extern "C" fn on_native_window_redraw_needed(activity: *mut ANativeActivity, window: *mut ANativeWindow){
	trace!("on_native_window_redraw_needed");
	let now = Instant::now();
	redraw(activity, window);
	trace!("redraw 耗时:{}ms", utils::duration_to_milis(&now.elapsed()));
}

extern "C" fn on_native_window_destroyed(activity: *mut ANativeActivity, window: *mut ANativeWindow){
	trace!("on_native_window_destroyed");
	APP.with(|app|{ app.borrow_mut().window = None; });
}

extern "C" fn on_destroy(activity: *mut ANativeActivity){
	APP.with(|app|{ app.borrow_mut().activity = None; });
	trace!("on_destroy");
}

fn is_activity_window_null(activity: *mut ANativeActivity, window: *mut ANativeWindow) ->bool{
	if activity.is_null(){
		error!("ANativeActivity is null.");
		true
	}else if window.is_null(){
		error!("ANativeWindow is null.");
		true
	}else{
		false
	}
}

fn redraw(activity: *mut ANativeActivity, window: *mut ANativeWindow){
	if is_activity_window_null(activity, window){ return;}
	//let now = Instant::now();
	let mut buffer = ANativeWindow_Buffer{ width: 0, height: 0, stride: 0, format: 0, bits: 0 as *mut c_void, reserved: [0; 5] };
	let mut rect = ARect { left: 0, top: 0, right: 0, bottom: 0};
	//trace!("000 耗时:{}ms", utils::duration_to_milis(&now.elapsed())); let now = Instant::now();
	let ret_code = unsafe{ ANativeWindow_lock(window, &mut buffer, &mut rect) };
	if ret_code !=0 {
		error!("ANativeWindow_lock 调用失败! {}", ret_code);
		return;
	}
	//trace!("001 耗时:{}ms", utils::duration_to_milis(&now.elapsed())); let now = Instant::now();

	let (pixel_size, pixel_format) = match buffer.format{
		WINDOW_FORMAT_RGBA_8888 => (4, PixelFormatEnum::RGBA8888),
		WINDOW_FORMAT_RGBX_8888 => (4, PixelFormatEnum::RGBX8888),
		WINDOW_FORMAT_RGB_565 => (2, PixelFormatEnum::RGB565),
		_ => return
	};
	let mut pixels = unsafe{ std::slice::from_raw_parts_mut(buffer.bits as *mut u8, (buffer.width*buffer.height*pixel_size) as usize) };
	//trace!("002 耗时:{}ms", utils::duration_to_milis(&now.elapsed())); let now = Instant::now();
	match Surface::from_data(&mut pixels, buffer.width as u32, buffer.height as u32, (buffer.stride * pixel_size) as u32, pixel_format){
		Ok(surface) =>{
			match Canvas::from_surface(surface){
				Ok(mut canvas) =>{
					//绘图
					canvas.set_draw_color(Color::RGB(0, 255, 0));
					canvas.clear();
					canvas.set_draw_color(Color::RGB(0, 0, 255));
					let _ = canvas.fill_rect(Some(Rect::new(0, 0, 200, 200)));
					canvas.present();
				}
				Err(err) => error!("Canvas创建失败 {:?}", err)
			}
		}
		Err(err) => error!("Surface创建失败! {:?}", err)
	}
	if unsafe{ ANativeWindow_unlockAndPost(window) } != 0{
		error!("ANativeWindow_unlockAndPost 调用失败!");
	}
	//trace!("{}x{} pixel_size={} pixel_format={:?} 耗时:{}ms", buffer.format, buffer.width, buffer.height, pixel_format, utils::duration_to_milis(&now.elapsed()));
}

extern fn on_input_queue_created(activity: *mut ANativeActivity, queue: *mut AInputQueue){
	APP.with(|app|{
		let mut app = app.borrow_mut();
		let (sender, receiver) = channel();
		app.event_loop_sender = Some(sender);
		//启动事件循环
		let input_queue = InputQueue::new(queue);
		thread::spawn(move || {
			trace!("事件循环线程已启动");
			loop{
				if let Ok(_some) = receiver.try_recv(){
					break;
				}
				if input_queue.queue.is_null(){
					error!("AInputQueue is null! 推出时间循环线程");
					break;
				}
				if unsafe{AInputQueue_hasEvents(input_queue.queue)}<0{
					//没有事件
					continue; 
				}
				let mut event: *mut AInputEvent = 0 as *mut c_void;
				unsafe{ AInputQueue_getEvent(input_queue.queue, &mut event); }
				if !event.is_null(){
					match unsafe{ AInputEvent_getType(event) }{
						AINPUT_EVENT_TYPE_MOTION =>{
							let cx = unsafe{ AMotionEvent_getX(event, 0) };
							let cy = unsafe{ AMotionEvent_getY(event, 0) };
							trace!("触摸事件 ({},{})", cx, cy);
							match unsafe{ AMotionEvent_getAction(event) } {
								AMOTION_EVENT_ACTION_DOWN => {
									trace!("手指按下 {},{}", cx, cy);
								}
								AMOTION_EVENT_ACTION_UP => {
									trace!("手指起开 {},{}", cx, cy);
								}
								_ => {}
							}
						}
						AINPUT_EVENT_TYPE_KEY => {
							trace!("键盘事件");
							match unsafe{ AKeyEvent_getAction(event) } {
								AKEY_EVENT_ACTION_DOWN => {
									trace!("键盘按下");
									match unsafe{ AKeyEvent_getKeyCode(event) } {
										AKEYCODE_BACK => {
											trace!("返回键按下");
										}
										_ => {}
									}
								}
								AKEY_EVENT_ACTION_UP => {
									trace!("返回键弹起");
								}
								_ => {}
							}
						}
						_ => {}
					}
					unsafe{AInputQueue_finishEvent(input_queue.queue, event, 0);}
				}
			}
			trace!("事件循环结束");
		});
	});
}

extern fn on_input_queue_destroyed(activity: *mut ANativeActivity, queue: *mut AInputQueue){
	APP.with(|app|{
		let mut app = app.borrow_mut();
		if app.event_loop_sender.is_some(){
			let _ = app.event_loop_sender.as_ref().unwrap().send(1);
			app.event_loop_sender = None;
		}
		trace!("事件循环线程结束");
	});
}

#[no_mangle]
pub extern "C" fn ANativeActivity_onCreate(activity: *mut ANativeActivity, savedState: *mut c_void, savedStateSize: *mut libc::size_t){
	//存储NativeActivity
	APP.with(|app|{ app.borrow_mut().activity = Some(activity); });

	//初始化logger
	android_logger::init_once(android_logger::Filter::default().with_min_level(Level::Trace));
	trace!("ANativeActivity_onCreate");

	//NativeWindow创建
	unsafe{
		(*(*activity).callbacks).onNativeWindowCreated = on_native_window_created;
		(*(*activity).callbacks).onNativeWindowRedrawNeeded = on_native_window_redraw_needed;
		(*(*activity).callbacks).onNativeWindowDestroyed = on_native_window_destroyed;
		(*(*activity).callbacks).onDestroy = on_destroy;
		(*(*activity).callbacks).onInputQueueCreated = on_input_queue_created;
		(*(*activity).callbacks).onInputQueueDestroyed = on_input_queue_destroyed;
	}
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern fn JNI_OnLoad(vm: *mut jni::sys::JavaVM, reserved: *mut c_void) -> jint{
	android_logger::init_once(android_logger::Filter::default().with_min_level(Level::Trace));
	trace!("JNI_OnLoad.");
	jni::sys::JNI_VERSION_1_6
}

fn rgb888_to_rgb565(red: u8, green: u8, blue: u8) -> u16{
	let b = (blue as u16 >> 3) & 0x001F;
	let g = ((green as u16 >> 2) << 5) & 0x07E0;
	let r = ((red as u16 >> 3) << 11) & 0xF800;
	return r | g | b;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern fn Java_cn_jy_lazydict_MainActivity_sendRgb(env: JNIEnv, _: JClass, src: jbyteArray, width:jint, height:jint){
	// use android_logger::Filter;
	// android_logger::init_once(Filter::default().with_min_level(Level::Trace));
	// trace!("sendRgb>>>>>>> width={} height={}", width, height);
	// let data = env.convert_byte_array(src).unwrap();
	// trace!("sendRgb>>>>>>>convert_byte_array data.len()={}", data.len());
	// trace!("surface ok!");
}