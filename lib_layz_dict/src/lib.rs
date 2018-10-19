#[allow(non_snake_case)]

#[macro_use]
extern crate lazy_static;
#[macro_use] extern crate log;
extern crate android_logger;
extern crate zip;
extern crate image;
use log::Level;
extern crate jni;
use std::time::Instant;
use self::jni::JNIEnv;
use self::jni::objects::{JClass};
use jni::sys::{jint, jbyteArray};
use std::os::raw::{c_void};
extern crate sdl2;
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
	preview_rgb_buffer: &'static Vec<u8>, //yuv420转换成rgb888使用的buffer
	event_loop_sender: Option<Sender<i32>>, //
}
impl Application{
	pub fn new() -> Application{
		Application{ window: None, preview_rgb_buffer: &'static vec![], event_loop_sender: None}
	}
}

lazy_static!{
	ref PREIVEW_BUFFER = 
}

thread_local!{
	pub static APP: RefCell<Application> = RefCell::new(Application::new());
}

//rgb888
fn lock_native_window<F>(window: *mut ANativeWindow, render: F) -> bool where F : Fn((&ANativeWindow_Buffer, &mut [u8])){
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

// This value is 2 ^ 18 - 1, and is used to clamp the RGB values before their
// ranges
// are normalized to eight bits.
const K_MAX_CHANNEL_VALUE:i32 = 262143;
///https://github.com/xvolica/Camera2-Yuv2Rgb/blob/master/jni/yuv2rgb.cpp
fn yuv_to_rgb(mut y:i32, mut u:i32, mut v:i32) -> (u8,u8,u8){
	use std::cmp::{min, max};

	y -= 16;
	u -= 128;
	v -= 128;
	if y<0{
		y=0;
	}

	let mut r = 1192 * y + 1634 * v;
	let mut g = 1192 * y - 833 * v - 400 * u;
	let mut b = 1192 * y + 2066 * u;

	r = min(K_MAX_CHANNEL_VALUE, max(0, r));
	g = min(K_MAX_CHANNEL_VALUE, max(0, g));
	b = min(K_MAX_CHANNEL_VALUE, max(0, b));

	r = (r>>10) & 0xff;
	g = (g>>10) & 0xff;
	b = (b>>10) & 0xff;

	//0xff000000 | (r as u32) << 24 | (g as u32) << 16 | (b as u32)<<8
	(r as u8, g as u8, b as u8)
}

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;

fn yuv_420_to_rgb_888(y_data: Vec<u8>, u_data: Vec<u8>, v_data:Vec<u8>, output:&'static mut[u8], width: i32, height:i32, y_row_stride: i32, uv_row_stride:i32, uv_pixel_stride:i32){

	//两个线程去执行转换
	let y_data = Arc::new(y_data);
	let u_data = Arc::new(u_data);
	let v_data = Arc::new(v_data);

	let output = Arc::new(Mutex::new(output));
	let iout = Arc::new(Mutex::new(0));

	let (tx, rx) = mpsc::channel();
	let first_count = height/2;
	
	let (y_data_clone, u_data_clone, v_data_clone, output_clone, iout_clone, tx_clone) = (y_data.clone(), u_data.clone(), v_data.clone(), output.clone(), iout.clone(), tx.clone());
	//第一个线程 0~1/2行
	thread::spawn(move|| {
		for y in 0..first_count{
			let iy = y_row_stride*y;
			let uv_row_start = uv_row_stride*(y>>1);
			let iu = uv_row_start;
			let iv = uv_row_start;
			for x in 0..width{
				let uv_offset = (x>>1)*uv_pixel_stride;
				let (r, g, b) = yuv_to_rgb(y_data_clone[(iy+x) as usize] as i32, u_data_clone[(iu+uv_offset) as usize] as i32, v_data_clone[(iv+uv_offset) as usize] as i32);
				let mut output = output_clone.lock().unwrap();
				let mut iout =  iout_clone.lock().unwrap();
				output[*iout] = r; *iout+=1;
				output[*iout] = g; *iout+=1;
				output[*iout] = b; *iout+=1;
			}
		}
		tx.send(());
	});

	//第二个线程 剩下的行
	thread::spawn(move|| {
		for y in first_count..height{
			let iy = y_row_stride*y;
			let uv_row_start = uv_row_stride*(y>>1);
			let iu = uv_row_start;
			let iv = uv_row_start;
			for x in 0..width{
				let uv_offset = (x>>1)*uv_pixel_stride;
				let (r, g, b) = yuv_to_rgb(y_data_clone[(iy+x) as usize] as i32, u_data_clone[(iu+uv_offset) as usize] as i32, v_data_clone[(iv+uv_offset) as usize] as i32);
				let mut output = output_clone.lock().unwrap();
				let mut iout =  iout_clone.lock().unwrap();
				output[*iout] = r; *iout+=1;
				output[*iout] = g; *iout+=1;
				output[*iout] = b; *iout+=1;
			}
		}
		tx.send(());
	});

	for i in 0..2{
		rx.recv();
	}

	

	/*
	//    old

	let mut iout = 0;
	for y in 0..height{
		let iy = y_row_stride*y;
		let uv_row_start = uv_row_stride*(y>>1);
		let iu = uv_row_start;
		let iv = uv_row_start;
		for x in 0..width{
			let uv_offset = (x>>1)*uv_pixel_stride;
			let (r, g, b) = yuv_to_rgb(y_data[(iy+x) as usize] as i32, u_data[(iu+uv_offset) as usize] as i32, v_data[(iv+uv_offset) as usize] as i32);
			output[iout] = r; iout+=1;
			output[iout] = g; iout+=1;
			output[iout] = b; iout+=1;
		}
	}

	*/
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
pub extern fn Java_cn_jy_lazydict_MainActivity_renderPreview(env: JNIEnv, _class: JClass, y: jni::objects::JByteBuffer, u: jni::objects::JByteBuffer, v:jni::objects::JByteBuffer, raw_width:jint, raw_height:jint, y_row_stride: jint, uv_row_stride:jint, uv_pixel_stride:jint, sensor_orientation: jint) -> jboolean{
	trace!("send>>Java_cn_jy_lazydict_MainActivity_send width={}, height={} y_row_stride={} uv_row_stride={} uv_pixel_stride={}", raw_width, raw_height, y_row_stride, uv_row_stride, uv_pixel_stride);
	let mut success = false;

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
		let y_src = env.get_direct_buffer_address(y).unwrap();
		let u_src = env.get_direct_buffer_address(u).unwrap();
		let v_src = env.get_direct_buffer_address(v).unwrap();
		let buffer_size = (raw_width*raw_height*PIXEL_SIZE) as usize;
		//创建preview buffer
		if app.preview_rgb_buffer.len() != buffer_size{
			info!("创建preview buffer {}x{}", raw_width, raw_height);
			app.preview_rgb_buffer = &vec![255; buffer_size];
		}
		yuv_420_to_rgb_888(Vec::from(y_src), Vec::from(u_src), Vec::from(v_src), &mut app.preview_rgb_buffer, raw_width, raw_height, y_row_stride, uv_row_stride, uv_pixel_stride);
		trace!("转换rgb耗时:{}ms", utils::duration_to_milis(&now.elapsed())); now = Instant::now();

		//第二步 旋转图像(copy form preview_rgb_buffer)
		let image_buffer:Option<ImageBuffer<Rgb<u8>, Vec<u8>>> = ImageBuffer::from_vec(raw_width as u32, raw_height as u32, app.preview_rgb_buffer.clone());
		if image_buffer.is_none(){
			error!("ImageBuffer创建失败!");
			return;
		}
		//create new buffer
		let rotate_buffer = match sensor_orientation{
			90 => image::imageops::rotate90(&image_buffer.unwrap()),
			180 => image::imageops::rotate180(&image_buffer.unwrap()),
			270 => image::imageops::rotate270(&image_buffer.unwrap()),
			_ => image_buffer.unwrap()
		};
		let (width, height) = (rotate_buffer.width(), rotate_buffer.height());

		trace!("图片旋转成功，旋转角度:{} 图片大小{}x{} 耗时{}ms", sensor_orientation, width, height, utils::duration_to_milis(&now.elapsed()));

		//copy?
		let rotate_raw_buffer = rotate_buffer.into_raw();

		success = lock_native_window(app.window.unwrap(), |(buffer, pixels)|{
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
		});
	});

	success as jboolean
}