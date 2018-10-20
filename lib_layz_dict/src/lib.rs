#[allow(non_snake_case)]

#[macro_use]
extern crate lazy_static;
#[macro_use] extern crate log;
extern crate android_logger;
extern crate zip;
extern crate image;
extern crate scoped_threadpool;
use log::Level;
extern crate jni;
use std::time::Instant;
use self::jni::JNIEnv;
use self::jni::objects::{JClass};
use jni::sys::{jint, jbyteArray};
use std::os::raw::{c_void};
extern crate sdl2;
extern crate rayon;
use sdl2::surface::Surface;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::pixels::PixelFormatEnum;
use std::thread;
extern crate libc;
use sdl2::rect::Rect;
extern crate bytes;
use image::{DynamicImage, Rgb, ImageBuffer};
mod utils;
mod jni_graphics;
mod native_window;
mod native_activity;
use native_window::*;
use native_activity::*;
use jni::sys::jboolean;
// use sdl2::rect::Point;
use std::cell::RefCell;
use std::sync::mpsc::{ Sender, channel};
// use std::sync::{Arc, Mutex};

const PIXEL_SIZE:i32 = 3; //RGB888

//const LEVEL:Level = Level::Info;
const LEVEL:Level = Level::Trace;

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
	window : Option<*mut ANativeWindow>, //Surface对应的NativeWindow
	preview_rgb_buffer: Vec<u8>, //yuv420转换成rgb888使用的buffer
	event_loop_sender: Option<Sender<i32>>, //
}
impl Application{
	pub fn new() -> Application{
		Application{ window: None, preview_rgb_buffer: vec![], event_loop_sender: None}
	}
}

// lazy_static!{
// 	ref PREIVEW_BUFFER = 
// }

thread_local!{
	pub static APP: RefCell<Application> = RefCell::new(Application::new());
	pub static BUFFER_RORATE: RefCell<Vec<u8>> = RefCell::new(vec![]);//图像旋转以后的buffer
}

//rgb888
fn lock_native_window<F>(window: *mut ANativeWindow, mut render: F) -> bool where F : FnMut((&ANativeWindow_Buffer, &mut [u8])){
	let mut buffer = ANativeWindow_Buffer{ width: 0, height: 0, stride: 0, format: 0, bits: 0 as *mut c_void, reserved: [0; 5] };
	let mut rect = ARect { left: 0, top: 0, right: 0, bottom: 0};
	let ret_code = unsafe{ ANativeWindow_lock(window, &mut buffer, &mut rect) };
	if ret_code !=0 {
		error!("ANativeWindow_lock 调用失败! {}", ret_code);
		return false;
	}
	let pixels = unsafe{ std::slice::from_raw_parts_mut(buffer.bits as *mut u8, (buffer.stride*buffer.height*PIXEL_SIZE) as usize) };
	trace!("lock_native_window {}x{} format={} stride={} pixel.len()={}", buffer.width, buffer.height, buffer.format, buffer.stride, pixels.len());
	render((&buffer, pixels));
	if unsafe{ ANativeWindow_unlockAndPost(window) } != 0{
		error!("ANativeWindow_unlockAndPost 调用失败!");
	}
	true
}

// extern fn on_input_queue_created(_activity: *mut ANativeActivity, queue: *mut AInputQueue){
// 	APP.with(|app|{
// 		let mut app = app.borrow_mut();
// 		let (sender, receiver) = channel();
// 		app.event_loop_sender = Some(sender);
// 		//启动事件循环
// 		let input_queue = InputQueue::new(queue);
// 		thread::spawn(move || {
// 			trace!("事件循环线程已启动");
// 			loop{
// 				if let Ok(_some) = receiver.try_recv(){
// 					break;
// 				}
// 				if input_queue.queue.is_null(){
// 					error!("AInputQueue is null! 推出时间循环线程");
// 					break;
// 				}
// 				if unsafe{AInputQueue_hasEvents(input_queue.queue)}<0{
// 					//没有事件
// 					continue; 
// 				}
// 				let mut event: *mut AInputEvent = 0 as *mut c_void;
// 				unsafe{ AInputQueue_getEvent(input_queue.queue, &mut event); }
// 				if !event.is_null(){
// 					match unsafe{ AInputEvent_getType(event) }{
// 						AINPUT_EVENT_TYPE_MOTION =>{
// 							let cx = unsafe{ AMotionEvent_getX(event, 0) };
// 							let cy = unsafe{ AMotionEvent_getY(event, 0) };
// 							trace!("触摸事件 ({},{})", cx, cy);
// 							match unsafe{ AMotionEvent_getAction(event) } {
// 								AMOTION_EVENT_ACTION_DOWN => {
// 									trace!("手指按下 {},{}", cx, cy);
// 								}
// 								AMOTION_EVENT_ACTION_UP => {
// 									trace!("手指起开 {},{}", cx, cy);
// 								}
// 								_ => {}
// 							}
// 						}
// 						AINPUT_EVENT_TYPE_KEY => {
// 							trace!("键盘事件");
// 							match unsafe{ AKeyEvent_getAction(event) } {
// 								AKEY_EVENT_ACTION_DOWN => {
// 									trace!("键盘按下");
// 									match unsafe{ AKeyEvent_getKeyCode(event) } {
// 										AKEYCODE_BACK => {
// 											trace!("返回键按下");
// 										}
// 										_ => {}
// 									}
// 								}
// 								AKEY_EVENT_ACTION_UP => {
// 									trace!("返回键弹起");
// 								}
// 								_ => {}
// 							}
// 						}
// 						_ => {}
// 					}
// 					unsafe{AInputQueue_finishEvent(input_queue.queue, event, 0);}
// 				}
// 			}
// 			trace!("事件循环结束");
// 		});
// 	});
// }

// extern fn on_input_queue_destroyed(_activity: *mut ANativeActivity, _queue: *mut AInputQueue){
// 	APP.with(|app|{
// 		let mut app = app.borrow_mut();
// 		if app.event_loop_sender.is_some(){
// 			let _ = app.event_loop_sender.as_ref().unwrap().send(1);
// 			app.event_loop_sender = None;
// 		}
// 		trace!("事件循环线程结束");
// 	});
// }

#[no_mangle]
pub extern fn JNI_OnLoad(_vm: *mut jni::sys::JavaVM, _reserved: *mut c_void) -> jint{
	android_logger::init_once(android_logger::Filter::default().with_min_level(LEVEL));
	info!("JNI_OnLoad.");
	jni::sys::JNI_VERSION_1_6
}

#[no_mangle]
pub extern fn Java_cn_jy_lazydict_MainActivity_setPreviewSurface(env: JNIEnv, _class: JClass, surface: jni::sys::jobject) -> jboolean{
	let window = unsafe{ ANativeWindow_fromSurface(env.get_native_interface(), surface) };
	if window.is_null(){
		error!("ANativeWindow_fromSurface调用失败!");
		return false as jboolean;
	}

	//保存NativeWindow
	APP.with(|app|{ app.borrow_mut().window = Some(window); });

	true as jboolean
}

#[no_mangle]
pub extern fn Java_cn_jy_lazydict_MainActivity_renderPreview(env: JNIEnv, _class: JClass, y: jni::objects::JByteBuffer, u: jni::objects::JByteBuffer, v:jni::objects::JByteBuffer, raw_width:jint, raw_height:jint, y_row_stride: jint, uv_row_stride:jint, uv_pixel_stride:jint, sensor_orientation: jint) -> jni::sys::jintArray{
	//trace!("send>>Java_cn_jy_lazydict_MainActivity_send width={}, height={} y_row_stride={} uv_row_stride={} uv_pixel_stride={}", raw_width, raw_height, y_row_stride, uv_row_stride, uv_pixel_stride);
	let mut result = [-1, -1];

	APP.with(|app|{
		let mut app = app.borrow_mut();

		if app.window.is_none(){
			error!("NativeWindow为空, 请先调用setPreviewSurface().");
			return;
		}
		if app.window.unwrap().is_null(){
			error!("NativeWindow为空, 请先重新启动APP.");
			return;
		}

		//第一步,将YUV420转换为RGB888
		let mut now = Instant::now();
		let (raw_width, raw_height, y_row_stride, uv_row_stride, uv_pixel_stride) = (raw_width as i32, raw_height as i32, y_row_stride as i32, uv_row_stride as i32, uv_pixel_stride as i32);
		let (y_src, u_src, v_src) = (env.get_direct_buffer_address(y), env.get_direct_buffer_address(u), env.get_direct_buffer_address(v));
		if y_src.is_err() || u_src.is_err() || v_src.is_err(){
			error!("请检查yuv数据是为空!");
			return;
		}
		let (y_src, u_src, v_src) = (y_src.unwrap(), u_src.unwrap(), v_src.unwrap());
		let buffer_size = (raw_width*raw_height*PIXEL_SIZE) as usize;
		//创建preview buffer
		if app.preview_rgb_buffer.len() != buffer_size{
			info!("创建preview buffer {}x{}", raw_width, raw_height);
			app.preview_rgb_buffer = vec![255; buffer_size];
		}
		utils::yuv_420_to_rgb_888(y_src, u_src, v_src, &mut app.preview_rgb_buffer, raw_width, raw_height, y_row_stride, uv_row_stride, uv_pixel_stride);
		trace!("转换rgb耗时:{}ms", utils::duration_to_milis(&now.elapsed())); now = Instant::now();

		lock_native_window(app.window.unwrap(), |(buffer, pixels)|{
			//第二步 旋转图像(copy form preview_rgb_buffer)
			BUFFER_RORATE.with(|rotate_buffer|{
				let mut rotate_buffer = rotate_buffer.borrow_mut();
				if rotate_buffer.len()!=app.preview_rgb_buffer.len(){
					info!("创建rotate buffer {}x{}", raw_width, raw_height);
					*rotate_buffer = vec![255; app.preview_rgb_buffer.len()];
				}
				let (rotate_raw_buffer, width, height) = match sensor_orientation{
					90 => {
						let (width, height) = utils::rotate90(&mut app.preview_rgb_buffer, &mut rotate_buffer, raw_width as usize, raw_height as usize);
						(&(*rotate_buffer), width, height)
					}
					180 => {
						let (width, height) = utils::rotate180(&mut app.preview_rgb_buffer, &mut rotate_buffer, raw_width as usize, raw_height as usize);
						(&(*rotate_buffer), width, height)
					}
					270 => {
						let (width, height) = utils::rotate270(&mut app.preview_rgb_buffer, &mut rotate_buffer, raw_width as usize, raw_height as usize);
						(&(*rotate_buffer), width, height)
					}
					_ =>{
						//不用旋转，使用原buffer
						(&app.preview_rgb_buffer, raw_width as usize, raw_height as usize)
					}
				};

				trace!("图片旋转成功，旋转角度:{} 图片大小{}x{} 耗时{}ms", sensor_orientation, width, height, utils::duration_to_milis(&now.elapsed()));

				let now = Instant::now();
				//复制像素
				let line_size = (width as i32*PIXEL_SIZE) as usize;
				let mut line_id = 0;
				for i in (0..rotate_raw_buffer.len()).step_by(line_size){//每一行
					let src_line = rotate_raw_buffer.get(i..i+line_size).unwrap();
					let target_line = pixels.get_mut(line_id..line_id+line_size).unwrap();
					target_line.copy_from_slice(&src_line);
					line_id += (buffer.stride*PIXEL_SIZE) as usize;
				}
				trace!("像素复制耗时{}ms", utils::duration_to_milis(&now.elapsed()));

				result[0] = width as i32;
				result[1] = height as i32;
			});
		});
	});

	if let Ok(arr) = env.new_int_array(2){
		let _ = env.set_int_array_region(arr, 0, &[result[0] as jint, result[1] as jint]);
		arr
	}else{
		0 as jni::sys::jintArray
	}
}