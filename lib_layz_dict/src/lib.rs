#[allow(non_snake_case)]

#[macro_use]
extern crate lazy_static;
#[macro_use] extern crate log;
extern crate android_logger;
extern crate zip;
use log::Level;
extern crate jni;
use std::time::Instant;
use self::jni::JNIEnv;
use self::jni::objects::{JClass};
use jni::sys::{jint, jbyteArray};
use std::os::raw::{c_void};
extern crate rayon;
extern crate libc;
extern crate bytes;
mod utils;
mod jni_graphics;
mod native_window;
mod native_activity;
mod imgtool;
use native_window::*;
use jni::sys::jboolean;
use std::cell::RefCell;
use imgtool::Rect;
//use std::thread;
//use std::sync::mpsc::{ Sender, channel};
// use std::sync::{Arc, Mutex};
extern crate jieba_rs;

use jieba_rs::Jieba;

const PIXEL_SIZE:i32 = 3; //RGB888

//const LEVEL:Level = Level::Info;
const LEVEL:Level = Level::Trace;

pub struct Application{
	window : Option<*mut ANativeWindow>, //Surface对应的NativeWindow
	preview_rgb_buffer: Vec<u8>, //yuv420转换成rgb888使用的buffer
}

impl Application{
	pub fn new() -> Application{
		Application{ window: None, preview_rgb_buffer: vec![]}
	}
}

thread_local!{
	pub static APP: RefCell<Application> = RefCell::new(Application::new());
	pub static BUFFER_RORATE: RefCell<Vec<u8>> = RefCell::new(vec![]);//图像旋转以后的buffer
}

//JNI加载完成
#[no_mangle]
pub extern fn JNI_OnLoad(_vm: *mut jni::sys::JavaVM, _reserved: *mut c_void) -> jint{
	android_logger::init_once(android_logger::Filter::default().with_min_level(LEVEL));
	info!("JNI_OnLoad.");
	jni::sys::JNI_VERSION_1_6
}

///初始化SurfaceView
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

//预览图片
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
		imgtool::yuv_420_to_rgb_888(y_src, u_src, v_src, &mut app.preview_rgb_buffer, raw_width, raw_height, y_row_stride, uv_row_stride, uv_pixel_stride);
		trace!("转换rgb耗时:{}ms", utils::duration_to_milis(&now.elapsed())); now = Instant::now();

		utils::lock_native_window_rgb_888(app.window.unwrap(), |(buffer, pixels)|{
			//第二步 旋转图像(copy form preview_rgb_buffer)
			BUFFER_RORATE.with(|rotate_buffer|{
				let mut rotate_buffer = rotate_buffer.borrow_mut();
				if rotate_buffer.len()!=app.preview_rgb_buffer.len(){
					info!("创建rotate buffer {}x{}", raw_width, raw_height);
					*rotate_buffer = vec![255; app.preview_rgb_buffer.len()];
				}
				let (mut rotate_raw_buffer, width, height) = match sensor_orientation{
					90 => {
						let (width, height) = imgtool::rotate90(&mut app.preview_rgb_buffer, &mut rotate_buffer, raw_width as usize, raw_height as usize);
						(&mut (*rotate_buffer), width, height)
					}
					180 => {
						let (width, height) = imgtool::rotate180(&mut app.preview_rgb_buffer, &mut rotate_buffer, raw_width as usize, raw_height as usize);
						(&mut (*rotate_buffer), width, height)
					}
					270 => {
						let (width, height) = imgtool::rotate270(&mut app.preview_rgb_buffer, &mut rotate_buffer, raw_width as usize, raw_height as usize);
						(&mut (*rotate_buffer), width, height)
					}
					_ =>{
						//不用旋转，使用原buffer
						(&mut app.preview_rgb_buffer, raw_width as usize, raw_height as usize)
					}
				};

				trace!("图片旋转成功，旋转角度:{} 图片大小{}x{} 耗时{}ms", sensor_orientation, width, height, utils::duration_to_milis(&now.elapsed()));

				let now = Instant::now(); 
				//取中间约为图片宽度40%的像素区域进行识别 480x720, Rect { left: 384, top: 624, width: 192, height: 192 }
				let clip_size = (width as f32*0.4) as usize;
				let rect = Rect::new((width-clip_size)/2, (height-clip_size)/2, clip_size, clip_size);

				let _ = imgtool::fill_rect(&mut rotate_raw_buffer, width, &Rect::new(rect.left, rect.top, 10, 10), [255, 0, 0]);
				let _ = imgtool::fill_rect(&mut rotate_raw_buffer, width, &Rect::new(rect.left+rect.width, rect.top, 10, 10), [255, 0, 0]);
				let _ = imgtool::fill_rect(&mut rotate_raw_buffer, width, &Rect::new(rect.left+rect.width, rect.top+rect.height, 10, 10), [255, 0, 0]);
				let _ = imgtool::fill_rect(&mut rotate_raw_buffer, width, &Rect::new(rect.left, rect.top+rect.height, 10, 10), [255, 0, 0]);
				let stroke_result = imgtool::stroke_rect(&mut rotate_raw_buffer, width, &rect, [255, 255, 0], 1);
				let _ = imgtool::stroke_rect(&mut rotate_raw_buffer, width, &Rect::new(10, 10, 200, 200), [0, 0, 255], 3);
				let _ = imgtool::stroke_rect(&mut rotate_raw_buffer, width, &Rect::new(100, 100, 60, 60), [0, 255, 255], 10);

				let clip_buffer = imgtool::get_rect(&rotate_raw_buffer, width, &rect);
				match clip_buffer{
					Ok(clip_buffer) =>{
						//trace!("截取结果: rect={:?} clip_buffer.len()={} 耗时: {}ms", rect, clip_buffer.len(), utils::duration_to_milis(&now.elapsed()));
					},
					Err(err) =>{
						trace!("截取结果: {}", err);
					}
				}
				trace!("截取 画点 {:?} 耗时: {}ms {:?}", rect, utils::duration_to_milis(&now.elapsed()), stroke_result);

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