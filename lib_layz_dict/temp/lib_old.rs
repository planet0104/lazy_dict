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
// use sdl2::rect::Point;
use std::cell::RefCell;
use std::sync::mpsc::{ Sender, channel};
// use std::sync::{Arc, Mutex};

const PIXEL_SIZE:i32 = 3; //RGB888

const LEVEL:Level = Level::Error;

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

extern "C" fn on_native_window_created(_activity: *mut ANativeActivity, window: *mut ANativeWindow){
	trace!("on_native_window_created");
	//设置格式
	if unsafe{ANativeWindow_setBuffersGeometry(window, ANativeWindow_getWidth(window), ANativeWindow_getHeight(window), WINDOW_FORMAT_RGB_888)} < 0{
		error!("ANativeWindow_setBuffersGeometry error!");
		return;
	}
	trace!("ANativeWindow_setBuffersGeometry成功.");
	//存储WINDOW
	APP.with(|app|{ app.borrow_mut().window = Some(window); });
}

extern "C" fn on_native_window_redraw_needed(_activity: *mut ANativeActivity, _window: *mut ANativeWindow){
	trace!("on_native_window_redraw_needed");
	let now = Instant::now();
	APP.with(|app|{
		draw_frame(&app.borrow(), |canvas|{
			//绘图
			canvas.set_draw_color(Color::RGB(0, 0, 255));//蓝底
			canvas.clear();
			canvas.set_draw_color(Color::RGB(255, 0, 0));//红色
			let _ = canvas.fill_rect(Some(Rect::new(100, 100, 200, 200)));
			canvas.set_draw_color(Color::RGB(0, 255, 0));//绿色
			let _ = canvas.fill_rect(Some(Rect::new(100, 300, 200, 200)));
			canvas.set_draw_color(Color::RGB(255, 255, 0));//黄色
			let _ = canvas.fill_rect(Some(Rect::new(100, 500, 200, 200)));
			canvas.present();
		});
	});
	trace!("redraw 耗时:{}ms", utils::duration_to_milis(&now.elapsed()));
}

extern "C" fn on_native_window_destroyed(_activity: *mut ANativeActivity, _window: *mut ANativeWindow){
	trace!("on_native_window_destroyed");
	APP.with(|app|{ app.borrow_mut().window = None; });
}

extern "C" fn on_destroy(_activity: *mut ANativeActivity){
	APP.with(|app|{ app.borrow_mut().activity = None; });
	trace!("on_destroy");
}

//rgb888
fn lock_native_window<F>(window: *mut ANativeWindow, render: F) where F : Fn((&ANativeWindow_Buffer, &mut [u8])){
	let mut buffer = ANativeWindow_Buffer{ width: 0, height: 0, stride: 0, format: 0, bits: 0 as *mut c_void, reserved: [0; 5] };
	let mut rect = ARect { left: 0, top: 0, right: 0, bottom: 0};
	let ret_code = unsafe{ ANativeWindow_lock(window, &mut buffer, &mut rect) };
	if ret_code !=0 {
		error!("ANativeWindow_lock 调用失败! {}", ret_code);
		return;
	}
	let pixels = unsafe{ std::slice::from_raw_parts_mut(buffer.bits as *mut u8, (buffer.stride*buffer.height*PIXEL_SIZE) as usize) };
	trace!("lock_native_window {}x{} format={} stride={} pixel.len()={}", buffer.width, buffer.height, buffer.format, buffer.stride, pixels.len());
	render((&buffer, pixels));
	if unsafe{ ANativeWindow_unlockAndPost(window) } != 0{
		error!("ANativeWindow_unlockAndPost 调用失败!");
	}
}

//rgb24
fn draw_frame<F>(app: &Application, render: F) where F : Fn(&mut Canvas<Surface>){
	lock_native_window(app.window.unwrap(), |(buffer, pixels)|{
		match Surface::from_data(pixels, buffer.width as u32, buffer.height as u32, (buffer.stride*PIXEL_SIZE) as u32, PixelFormatEnum::RGB24){
			Ok(surface) =>{
				match Canvas::from_surface(surface){
					Ok(mut canvas) =>{
						render(&mut canvas);
					}
					Err(err) => error!("Canvas创建失败 {:?}", err)
				}
			}
			Err(err) => error!("Surface创建失败! {:?}", err)
		}
	});
}

extern fn on_input_queue_created(_activity: *mut ANativeActivity, queue: *mut AInputQueue){
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

extern fn on_input_queue_destroyed(_activity: *mut ANativeActivity, _queue: *mut AInputQueue){
	APP.with(|app|{
		let mut app = app.borrow_mut();
		if app.event_loop_sender.is_some(){
			let _ = app.event_loop_sender.as_ref().unwrap().send(1);
			app.event_loop_sender = None;
		}
		trace!("事件循环线程结束");
	});
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn ANativeActivity_onCreate(activity: *mut ANativeActivity, _savedState: *mut c_void, _savedStateSize: *mut libc::size_t){
	//存储NativeActivity
	APP.with(|app|{ app.borrow_mut().activity = Some(activity); });

	//初始化logger
	android_logger::init_once(android_logger::Filter::default().with_min_level(LEVEL));
	trace!("ANativeActivity_onCreate");

	unsafe{
		(*(*activity).callbacks).onNativeWindowCreated = on_native_window_created;
		(*(*activity).callbacks).onNativeWindowRedrawNeeded = on_native_window_redraw_needed;
		(*(*activity).callbacks).onNativeWindowDestroyed = on_native_window_destroyed;
		(*(*activity).callbacks).onDestroy = on_destroy;
		(*(*activity).callbacks).onInputQueueCreated = on_input_queue_created;
		(*(*activity).callbacks).onInputQueueDestroyed = on_input_queue_destroyed;
	}
}

#[no_mangle]
pub extern fn JNI_OnLoad(_vm: *mut jni::sys::JavaVM, _reserved: *mut c_void) -> jint{
	android_logger::init_once(android_logger::Filter::default().with_min_level(LEVEL));
	trace!("JNI_OnLoad.");
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

fn yuv_420_to_rgb_8888(y_data: &mut[u8], u_data: &mut[u8], v_data:&mut[u8], output:&mut[u8], width: i32, height:i32, y_row_stride: i32, uv_row_stride:i32, uv_pixel_stride:i32){
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
}

#[no_mangle]
pub extern fn Java_cn_jy_lazydict_MainActivity_send(env: JNIEnv, class: JClass, y: jni::objects::JByteBuffer, u: jni::objects::JByteBuffer, v:jni::objects::JByteBuffer, raw_width:jint, raw_height:jint, y_row_stride: jint, uv_row_stride:jint, uv_pixel_stride:jint, sensor_orientation: jint, surface: jni::sys::jobject){
	trace!("send>>Java_cn_jy_lazydict_MainActivity_send width={}, height={} y_row_stride={} uv_row_stride={} uv_pixel_stride={}", raw_width, raw_height, y_row_stride, uv_row_stride, uv_pixel_stride);
	let mut now = Instant::now();
	let (raw_width, raw_height, y_row_stride, uv_row_stride, uv_pixel_stride) = (raw_width as i32, raw_height as i32, y_row_stride as i32, uv_row_stride as i32, uv_pixel_stride as i32);
	let y_src = env.get_direct_buffer_address(y).unwrap();
	let u_src = env.get_direct_buffer_address(u).unwrap();
	let v_src = env.get_direct_buffer_address(v).unwrap();

	//RGB24
	let mut rgb_data:Vec<u8> = vec![255; (raw_width*raw_height*PIXEL_SIZE) as usize];
	yuv_420_to_rgb_8888(y_src, u_src, v_src, &mut rgb_data, raw_width, raw_height, y_row_stride, uv_row_stride, uv_pixel_stride);
	trace!("转换rgb耗时:{}ms", utils::duration_to_milis(&now.elapsed())); now = Instant::now();

	//bitmap进行旋转
	let image_buffer:Option<ImageBuffer<Rgb<u8>, Vec<u8>>> = ImageBuffer::from_vec(raw_width as u32, raw_height as u32, rgb_data);
	if let Some(buffer) = image_buffer{
		let new_buffer = match sensor_orientation{
			90 => image::imageops::rotate90(&buffer),
			180 => image::imageops::rotate180(&buffer),
			270 => image::imageops::rotate270(&buffer),
			_ => buffer
		};

		let (width, height) = (new_buffer.width(), new_buffer.height());
		trace!("图片旋转成功，旋转角度:{} 图片大小{}x{} 耗时{}ms", sensor_orientation, width, height, utils::duration_to_milis(&now.elapsed()));
		//let dynamic_image = DynamicImage::ImageRgb8(new_buffer);
		let src_buffer = new_buffer.into_raw();

		let window = unsafe{ ANativeWindow_fromSurface(env.get_native_interface(), surface) };
		if window.is_null(){
			error!("ANativeWindow_fromSurface调用失败!");
			return;
		}
		
		lock_native_window(window, |(buffer, pixels)|{
			let now = Instant::now();
			//复制像素
			let line_size = (width as i32*PIXEL_SIZE) as usize;
			let mut line_id = 0;
			for i in (0..src_buffer.len()).step_by(line_size){//每一行
				let src_line = src_buffer.get(i..i+line_size).unwrap();
				let target_line = pixels.get_mut(line_id..line_id+line_size).unwrap();
				target_line.copy_from_slice(&src_line);
				line_id += (buffer.stride*PIXEL_SIZE) as usize;
			}
			trace!("像素复制耗时{}ms", utils::duration_to_milis(&now.elapsed()));
		});
	}else{
		error!("图片读取失败!");
	}

	/*
	APP.with(|app|{
		let app = app.borrow();
		draw_frame_buffer(&app, |(buffer, pixels)|{
			

			// match image::load_from_memory(&rgb_data){
			// 	Ok(image) => {
			// 		let rgb_image = image.to_rgb();
			// 		trace!("图片读取成功. image={:?}", rgb_image);
			// 	}
			// 	Err(err) => {
			// 		error!("图片读取失败:{:?}", err);	
			// 	}
			// }

			
			
			// let texture_creator = canvas.texture_creator();
			// let image_surface = Surface::from_data(rgba_data.as_mut_slice(), width as u32, height as u32, (width*PIXEL_SIZE) as u32, PixelFormatEnum::RGB24).unwrap();
			// let create_ret = texture_creator.create_texture_from_surface(image_surface);
			// match create_ret{
			// 	Ok(mut texture) => {
			// 		//match canvas.copy_ex(&mut texture, None, Some(Rect::new(100, 700, width as u32, height as u32)), sensor_orientation as f64, None, false ,false){
			// 		match canvas.copy(&mut texture, None, Some(Rect::new(100, 700, width as u32, height as u32))){
			// 			Ok(_) => {
			// 				// trace!("画图完成.");
			// 			}
			// 			Err(err) =>{
			// 				error!("画图 0007 {:?}", err);
			// 			}
			// 		}
			// 	}
			// 	Err(err) => {
			// 		error!("画图 0006 {:?}", err);
			// 	}
			// }
			// trace!("绘制耗时:{}ms", utils::duration_to_milis(&now.elapsed()));
		});
	});
	*/
	
	//priview_sdl(env, y, u ,v, width, height, y_row_stride, uv_row_stride, uv_pixel_stride, sensor_orientation);
}

fn priview_sdl(env: JNIEnv, y: jni::objects::JByteBuffer, u: jni::objects::JByteBuffer, v:jni::objects::JByteBuffer, width:jint, height:jint, y_row_stride: jint, uv_row_stride:jint, uv_pixel_stride:jint, sensor_orientation: jint){
	APP.with(|app|{
		let app = app.borrow();
		draw_frame(&app, |canvas|{
			let (width, height, y_row_stride, uv_row_stride, uv_pixel_stride) = (width as i32, height as i32, y_row_stride as i32, uv_row_stride as i32, uv_pixel_stride as i32);
			let y_src = env.get_direct_buffer_address(y).unwrap();
			let u_src = env.get_direct_buffer_address(u).unwrap();
			let v_src = env.get_direct_buffer_address(v).unwrap();

			//RGB24
			let mut now = Instant::now();
			let mut rgba_data:Vec<u8> = vec![255; (width*height*PIXEL_SIZE) as usize];
			yuv_420_to_rgb_8888(y_src, u_src, v_src, &mut rgba_data, width, height, y_row_stride, uv_row_stride, uv_pixel_stride);
			trace!("转换rgb耗时:{}ms", utils::duration_to_milis(&now.elapsed())); now = Instant::now();
			
			let texture_creator = canvas.texture_creator();
			let image_surface = Surface::from_data(rgba_data.as_mut_slice(), width as u32, height as u32, (width*PIXEL_SIZE) as u32, PixelFormatEnum::RGB24).unwrap();
			let create_ret = texture_creator.create_texture_from_surface(image_surface);
			match create_ret{
				Ok(mut texture) => {
					//match canvas.copy_ex(&mut texture, None, Some(Rect::new(100, 700, width as u32, height as u32)), sensor_orientation as f64, None, false ,false){
					match canvas.copy(&mut texture, None, Some(Rect::new(100, 700, width as u32, height as u32))){
						Ok(_) => {
							// trace!("画图完成.");
						}
						Err(err) =>{
							error!("画图 0007 {:?}", err);
						}
					}
				}
				Err(err) => {
					error!("画图 0006 {:?}", err);
				}
			}
			trace!("绘制耗时:{}ms", utils::duration_to_milis(&now.elapsed()));
		});
	});
}