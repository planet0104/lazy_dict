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
	if unsafe{ANativeWindow_setBuffersGeometry(window, ANativeWindow_getWidth(window), ANativeWindow_getHeight(window), WINDOW_FORMAT_RGBA_8888)} < 0{
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
			trace!("on_canvas_render!!");
			//绘图
			canvas.set_draw_color(Color::RGB(0, 255, 0));
			trace!("on_canvas_render0001");
			canvas.clear();
			trace!("on_canvas_render0002");
			canvas.set_draw_color(Color::RGB(0, 0, 255));
			trace!("on_canvas_render0003");
			let _ = canvas.fill_rect(Some(Rect::new(0, 0, 200, 200)));
			trace!("on_canvas_render0004");
			canvas.present();
			trace!("on_canvas_render0005");
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
	// trace!("{}x{} pixel_format={} stride={}", buffer.width, buffer.height, buffer.format, buffer.stride);
	let (pixel_size, pixel_format) = (4, PixelFormatEnum::RGBA8888);
	let mut pixels = unsafe{ std::slice::from_raw_parts_mut(buffer.bits as *mut u8, (buffer.width*buffer.height*pixel_size) as usize) };
	match Surface::from_data(&mut pixels, buffer.width as u32, buffer.height as u32, (buffer.stride*pixel_size) as u32, pixel_format){
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

// This value is 2 ^ 18 - 1, and is used to clamp the RGB values before their
// ranges
// are normalized to eight bits.
const kMaxChannelValue:i32 = 262143;
///https://github.com/xvolica/Camera2-Yuv2Rgb/blob/master/jni/yuv2rgb.cpp
fn yuv_to_rgb(mut y:i32, mut u:i32, mut v:i32) -> u32{
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

	r = min(kMaxChannelValue, max(0, r));
	g = min(kMaxChannelValue, max(0, g));
	b = min(kMaxChannelValue, max(0, b));

	r = (r>>10) & 0xff;
	g = (g>>10) & 0xff;
	b = (b>>10) & 0xff;

	0xff000000 | (r << 16) as u32 | (g << 8) as u32 | b as u32
}

fn yuv_420_to_rgba_8888(y_data: &mut[u8], u_data: &mut[u8], v_data:&mut[u8], output:&mut[u8], width: i32, height:i32, y_row_stride: i32, uv_row_stride:i32, uv_pixel_stride:i32){
	let mut iout = 0;
	for y in 0..height{
		let iy = y_row_stride*y;
		let uv_row_start = uv_row_stride*(y>>1);
		let iu = uv_row_start;
		let iv = uv_row_start;
		for x in 0..width{
			let uv_offset = (x>>1)*uv_pixel_stride;
			use bytes::{BytesMut, BufMut, LittleEndian, BigEndian};
			let color = yuv_to_rgb(y_data[(iy+x) as usize] as i32, u_data[(iu+uv_offset) as usize] as i32, v_data[(iv+uv_offset) as usize] as i32);
			let mut buf = [0; 4];
			LittleEndian::read_u32(&buf);
			output[iout] = 255; iout+=1;
			output[iout] = b; iout+=1;
			output[iout] = g; iout+=1;
			output[iout] = r; iout+=1;
		}
	}
}

#[no_mangle]
pub unsafe extern fn Java_cn_jy_lazydict_MainActivity_send(env: JNIEnv, _: JClass, y: jni::objects::JByteBuffer, u: jni::objects::JByteBuffer, v:jni::objects::JByteBuffer, width:jint, height:jint, y_row_stride: jint, uv_row_stride:jint, uv_pixel_stride:jint){
	trace!("send>>Java_cn_jy_lazydict_MainActivity_send width={}, height={} y_row_stride={} uv_row_stride={} uv_pixel_stride={}", width, height, y_row_stride, uv_row_stride, uv_pixel_stride);

	APP.with(|app|{
		let app = app.borrow();
		draw_frame(&app, |canvas|{
			let (width, height, y_row_stride, uv_row_stride, uv_pixel_stride) = (width as i32, height as i32, y_row_stride as i32, uv_row_stride as i32, uv_pixel_stride as i32);
			let y_src = env.get_direct_buffer_address(y).unwrap();
			let u_src = env.get_direct_buffer_address(u).unwrap();
			let v_src = env.get_direct_buffer_address(v).unwrap();
			let mut rgba_data = vec![255; (width*height*4) as usize];
			yuv_420_to_rgba_8888(y_src, u_src, v_src, &mut rgba_data, width, height, y_row_stride, uv_row_stride, uv_pixel_stride);

			// trace!("yuv_to_rgb OK. len={}", rgba_data.len());

			let texture_creator = canvas.texture_creator();
			// trace!("画图 0004");
			let image_surface = Surface::from_data(rgba_data.as_mut_slice(), width as u32, height as u32, width as u32*4, PixelFormatEnum::RGBA8888).unwrap();
			// trace!("画图 0005");
			let create_ret = texture_creator.create_texture_from_surface(image_surface);
			match create_ret{
				Ok(mut texture) => {
					match canvas.copy(&mut texture, None, Some(Rect::new(0, 0, width as u32, height as u32))){
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
		});
	});
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