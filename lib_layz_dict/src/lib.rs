#[allow(non_snake_case)]

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
	//存储WINDOW
	APP.with(|app|{ app.borrow_mut().window = Some(window); });
}

extern "C" fn on_native_window_redraw_needed(_activity: *mut ANativeActivity, _window: *mut ANativeWindow){
	trace!("on_native_window_redraw_needed");
	let now = Instant::now();
	// APP.with(|app|{
	// 	draw_frame(&app.borrow(), |canvas|{
	// 		trace!("on_canvas_render!!");
	// 		//绘图
	// 		canvas.set_draw_color(Color::RGB(0, 255, 0));
	// 		trace!("on_canvas_render0001");
	// 		canvas.clear();
	// 		trace!("on_canvas_render0002");
	// 		canvas.set_draw_color(Color::RGB(0, 0, 255));
	// 		trace!("on_canvas_render0003");
	// 		let _ = canvas.fill_rect(Some(Rect::new(0, 0, 200, 200)));
	// 		trace!("on_canvas_render0004");
	// 		canvas.present();
	// 		trace!("on_canvas_render0005");
	// 	});
	// });
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

fn draw_frame<F>(app: &Application, render: F) where F : Fn(&mut Canvas<Surface>){
	if app.activity.is_none() || app.activity.unwrap().is_null(){
		error!("draw_frame>ANativeActivity is null.");
		return;
	}
	if app.window.is_none() || app.window.unwrap().is_null(){
		error!("draw_frame>ANativeWindow is null.");
		return;
	}
	//let now = Instant::now();
	let mut buffer = ANativeWindow_Buffer{ width: 0, height: 0, stride: 0, format: 0, bits: 0 as *mut c_void, reserved: [0; 5] };
	let mut rect = ARect { left: 0, top: 0, right: 0, bottom: 0};
	//trace!("000 耗时:{}ms", utils::duration_to_milis(&now.elapsed())); let now = Instant::now();
	let ret_code = unsafe{ ANativeWindow_lock(app.window.unwrap(), &mut buffer, &mut rect) };
	if ret_code !=0 {
		error!("ANativeWindow_lock 调用失败! {}", ret_code);
		return;
	}

	let (pixel_size, pixel_format) = match buffer.format{
		WINDOW_FORMAT_RGBA_8888 => (4, PixelFormatEnum::RGBA8888),
		WINDOW_FORMAT_RGBX_8888 => (4, PixelFormatEnum::RGBX8888),
		WINDOW_FORMAT_RGB_888 => (4, PixelFormatEnum::RGB888),
		WINDOW_FORMAT_RGB_565 => (2, PixelFormatEnum::RGB565),
		_ => return
	};
	let mut pixels = unsafe{ std::slice::from_raw_parts_mut(buffer.bits as *mut u8, (buffer.width*buffer.height*pixel_size) as usize) };
	match Surface::from_data(&mut pixels, buffer.width as u32, buffer.height as u32, (buffer.stride * pixel_size) as u32, pixel_format){
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
	if unsafe{ ANativeWindow_unlockAndPost(app.window.unwrap()) } != 0{
		error!("ANativeWindow_unlockAndPost 调用失败!");
	}
	//trace!("{}x{} pixel_size={} pixel_format={:?} 耗时:{}ms", buffer.format, buffer.width, buffer.height, pixel_format, utils::duration_to_milis(&now.elapsed()));
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
	android_logger::init_once(android_logger::Filter::default().with_min_level(Level::Trace));
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
	android_logger::init_once(android_logger::Filter::default().with_min_level(Level::Trace));
	trace!("JNI_OnLoad.");
	jni::sys::JNI_VERSION_1_6
}

#[allow(dead_code)]
fn rgb888_to_rgb565(red: u8, green: u8, blue: u8) -> u16{
	let b = (blue as u16 >> 3) & 0x001F;
	let g = ((green as u16 >> 2) << 5) & 0x07E0;
	let r = ((red as u16 >> 3) << 11) & 0xF800;
	return r | g | b;
}

fn yuv_to_rgb(y:u8, u:u8,  v:u8) -> [u8;3]{
	let mut r = (y&0xff) as f64 + 1.4075 * ((v&0xff)-128) as f64;
	let mut g = (y&0xff) as f64 - 0.3455 * ((u&0xff)-128) as f64 - 0.7169*((v&0xff)-128) as f64;
	let mut b = (y&0xff) as f64 + 1.779 * ((u&0xff)-128) as f64;
	
	if r<0.0 { r=0.0; }
	if r>255.0 { r=255.0; }
	if g<0.0 { g=0.0; }
	if g>255.0 { g=255.0; }
	if b<0.0 { b=0.0; }
	if b>255.0 { b=255.0; }
	[r as u8, g as u8, b as u8]
}

// #[allow(non_snake_case)]
// #[no_mangle]
// pub extern fn Java_cn_jy_lazydict_MainActivity_setPreviewType(_env: JNIEnv, _: JClass, preivew_type: jint) -> jni::sys::jboolean{
// 	trace!("Java_cn_jy_lazydict_MainActivity_setPreviewType {}", preivew_type);
// 	let mut success = false;
// 	APP.with(|app|{
// 		if let Some(window) = app.borrow().window{
// 			if unsafe{ANativeWindow_setBuffersGeometry(window, ANativeWindow_getWidth(window), ANativeWindow_getHeight(window), preivew_type)} == 0{
// 				success = true;
// 			}
// 		}
// 	});
// 	success as jni::sys::jboolean
// }

#[no_mangle]
pub unsafe extern fn Java_cn_jy_lazydict_MainActivity_send(env: JNIEnv, _: JClass, y: jni::objects::JByteBuffer, u: jni::objects::JByteBuffer, v:jni::objects::JByteBuffer, width:jint, height:jint){
	trace!("send>>Java_cn_jy_lazydict_MainActivity_send");
	// if width!=0{return; }
	// APP.with(|app|{
	// 	let app = app.borrow();
	// 	draw_frame(&app, |canvas|{
	// 		let (width, height) = (width as i32, height as i32);
	// 		let y_src = env.get_direct_buffer_address(y).unwrap();
	// 		let u_src = env.get_direct_buffer_address(u).unwrap();
	// 		let v_src = env.get_direct_buffer_address(v).unwrap();
	// 		let mut src = vec![];
	// 		src.extend_from_slice(y_src);
	// 		src.extend_from_slice(u_src);
	// 		src.extend_from_slice(v_src);
	// 		let num_of_pixel = width * height;
	// 		let position_of_v = num_of_pixel;
	// 		let position_of_u = num_of_pixel/4 + num_of_pixel;
	// 		let mut rgb = vec![0; num_of_pixel as usize*3];
	// 		for i in 0..height{
	// 			let start_y = i*width;
	// 			let step = (i/2)*(width/2);
	// 			let start_v = position_of_v + step;
	// 			let start_u = position_of_u + step;
	// 			for j in 0..width{
	// 				let y = start_y + j;
	// 				let v = start_v + j/2;
	// 				let u = start_u + j/2;
	// 				let index = y*3;
	// 				let tmp = yuv_to_rgb(src[u as usize], src[u as usize], src[v as usize]);
	// 				rgb[index as usize] = tmp[0];
	// 				rgb[index as usize+1] = tmp[1];
	// 				rgb[index as usize+2] = tmp[2];
	// 			}
	// 		}

	// 		let texture_creator = canvas.texture_creator();
	// 		trace!("画图 0004");
	// 		let image_surface = Surface::from_data(rgb.as_mut_slice(), width as u32, height as u32, width as u32*3, PixelFormatEnum::RGB888).unwrap();
	// 		trace!("画图 0005");
	// 		let mut texture = texture_creator.create_texture_from_surface(image_surface).unwrap();
	// 		trace!("画图 0006");
	// 		canvas.copy(&mut texture, None, Some(Rect::new(0, 0, width as u32, height as u32))).unwrap();
	// 		trace!("画图 0007");
	// 	});
	// });
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern fn Java_cn_jy_lazydict_MainActivity_sendRgb(env: JNIEnv, _: JClass, preview_bytes: jbyteArray, preview_width: jint, preview_height: jint, preview_row_stride: jint){
	// APP.with(|app|{
	// 	let app = app.borrow();
	// 	draw_frame(&app, |canvas|{
	// 		let mut preivew_buffer = env.convert_byte_array(preview_bytes).unwrap();
	// 		trace!("画图 0001");
	// 		//绘图
	// 		canvas.set_draw_color(Color::RGB(0, 255, 0));
	// 		trace!("画图 0002");
	// 		canvas.clear();
	// 		trace!("画图 0003");
	// 		{
	// 			let texture_creator = canvas.texture_creator();
	// 			trace!("画图 0004");
	// 			let image_surface = Surface::from_data(preivew_buffer.as_mut_slice(), preview_width as u32, preview_height as u32, preview_row_stride as u32, PixelFormatEnum::RGB565).unwrap();
	// 			trace!("画图 0005");
	// 			let mut texture = texture_creator.create_texture_from_surface(image_surface).unwrap();
	// 			trace!("画图 0006");
	// 			canvas.copy(&mut texture, None, Some(Rect::new(0, 0, preview_width as u32, preview_height as u32))).unwrap();
	// 			trace!("画图 0007");
	// 		}
	// 		canvas.present();
	// 	});
	// });
}