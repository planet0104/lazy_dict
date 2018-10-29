#[allow(non_snake_case)]

#[macro_use]
extern crate lazy_static;
#[macro_use] extern crate log;
extern crate android_logger;
extern crate zip;
use log::Level;
extern crate jni;
use std::time::Instant;
use jni::{JNIEnv};
use self::jni::objects::{JObject, JString, JClass, JValue};
use jni::sys::{jint};
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
use std::thread;
//use std::sync::mpsc::{ Sender, channel};
// use std::sync::{Arc, Mutex};
extern crate jieba_rs;

use jieba_rs::Jieba;

const PIXEL_SIZE:i32 = 3; //RGB888

// const LEVEL:Level = Level::Error;
// const LEVEL:Level = Level::Trace;
const LEVEL:Level = Level::Debug;

pub struct Application<'a>{
	window : Option<*mut ANativeWindow>, //Surface对应的NativeWindow
	activity: Option<JClass<'a>>,
	start: bool,
	preview_rgb_buffer: Vec<u8>, //yuv420转换成rgb888使用的buffer
}

impl <'a> Application<'a>{
	pub fn new() -> Application<'a>{
		Application{ window: None, activity: None, start: false, preview_rgb_buffer: vec![]}
	}
}

thread_local!{
	pub static APP: RefCell<Application<'static>> = RefCell::new(Application::new());
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
pub extern fn Java_cn_jy_lazydict_MainActivity_setPreviewSurface(env: JNIEnv, class: JClass<'static>, surface: jni::sys::jobject) -> jboolean{
	let window = unsafe{ ANativeWindow_fromSurface(env.get_native_interface(), surface) };
	if window.is_null(){
		error!("ANativeWindow_fromSurface调用失败!");
		return false as jboolean;
	}

	//保存NativeWindow
	APP.with(|app|{
		let mut app = app.borrow_mut();
		app.start = false;	//默认启动时不开启识别
		app.window = Some(window);
		app.activity = Some(class);
	});
	true as jboolean
}

//文字识别完成
#[no_mangle]
pub extern fn Java_cn_jy_lazydict_MainActivity_onTextRecognized(_env: JNIEnv, _activity_class: JClass, _time: JValue, text: JString){
	trace!("onTextRecognized 文字识别完成: {:?}", JObject::from(text));
}

//开始识别
#[no_mangle]
pub extern fn Java_cn_jy_lazydict_MainActivity_startRecognize(_env: JNIEnv, _activity_class: JClass){
	debug!("开始文字识别.");
	APP.with(|app|{
		let mut app = app.borrow_mut();
		app.start = true;
	});
}

// fn recognize


///两个需要改动的
/// 1、识别文字时的图片处理开启单独线程 let env = attach_current_thread();
/// 2、识别的文字需要分割部分方块

fn render(app: &mut Application, result:&mut [i32;2], env: &JNIEnv, activity_class: JClass, y: jni::objects::JByteBuffer, u: jni::objects::JByteBuffer, v:jni::objects::JByteBuffer, raw_width:jint, raw_height:jint, y_row_stride: jint, uv_row_stride:jint, uv_pixel_stride:jint, sensor_orientation: jint) -> Result<(), String>{
	let mje = |err|{ format!("JNI调用失败 {:?}", err) };
	let rec = app.start;
	if app.window.is_none(){
		return Err(String::from("NativeWindow为空, 请先调用setPreviewSurface()."));
	}
	let window = app.window.unwrap();
	if window.is_null(){
		return Err(String::from("NativeWindow为空, 请先重新启动APP."));
	}
	

	//第一步,将YUV420转换为RGB888
	let mut now = Instant::now();
	let (raw_width, raw_height, y_row_stride, uv_row_stride, uv_pixel_stride) = (raw_width as i32, raw_height as i32, y_row_stride as i32, uv_row_stride as i32, uv_pixel_stride as i32);
	
	let (y_src, u_src, v_src) = (env.get_direct_buffer_address(y).map_err(mje)?,
								 env.get_direct_buffer_address(u).map_err(mje)?,
								 env.get_direct_buffer_address(v).map_err(mje)?);

	let buffer_size = (raw_width*raw_height*PIXEL_SIZE) as usize;
	//创建preview buffer
	if app.preview_rgb_buffer.len() != buffer_size{
		info!("创建preview buffer {}x{}", raw_width, raw_height);
		app.preview_rgb_buffer = vec![255; buffer_size];
	}

	imgtool::yuv_420_to_rgb_888(y_src, u_src, v_src, &mut app.preview_rgb_buffer, raw_width, raw_height, y_row_stride, uv_row_stride, uv_pixel_stride);
	trace!("转换rgb耗时:{}ms", utils::duration_to_milis(&now.elapsed())); now = Instant::now();

	utils::lock_native_window_rgb_888(window, |buffer, pixels|{
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

			//108ee9

			// let clip_width = (width as f32*0.4) as usize;
			// let clip_height = (width as f32*0.4) as usize;
			// let rect = Rect::new((width-clip_width)/2, (height-clip_height)/2, clip_width, clip_height);
			// imgtool::stroke_rect(&mut rotate_raw_buffer, width, &rect, &[16, 142, 233], 1, 3)?;

			//预览启动1.5秒以后再进行识别, 避免CPU使用过度卡住主线程动画
			let rec_result = (||->Result<(), String> {
				// if rec{
				// 	//取中间部分识别(因为要预分割，所以只支持水平、垂直模式的文字，混合模式不支持)
				// 	let clip_width = (width as f32*0.3) as usize;
				// 	let clip_height = (width as f32*0.2) as usize;
				// 	let rect = Rect::new((width-clip_width)/2, (height-clip_height)/2, clip_width, clip_height);
				// 	imgtool::stroke_rect(&mut rotate_raw_buffer, width, &rect, &[255, 255, 0], 1, 3)?;
				// 	//debug!("{:?}", rect);

				// 	let now = Instant::now();
					
				// 	let mut clip_buffer = imgtool::get_argb_rect_rgb(&rotate_raw_buffer, width, &rect)?;
					
				// 	let bpp = 4;
				// 	//计算阈值和像素灰度值
				// 	let (threshold, gray_values) = imgtool::calc_threshold(&clip_buffer, bpp);
				// 	//将原图像二值化
				// 	imgtool::binary(&gray_values, &mut clip_buffer, bpp, threshold);
				// 	//边缘检测
				// 	let mut edges = vec![1; clip_width*clip_height]; //1为背景, 0为边缘
				// 	imgtool::edge_detect_gray(&gray_values, &mut edges, clip_width, threshold);
				// 	//根据edges分割
				// 	let sub_rects = imgtool::split(0, 0, &mut edges, clip_width, clip_height);
				// 	if sub_rects.len()>0{
				// 		//创建图片数组和宽高数组
				// 		let size = sub_rects.len();

				// 		let bitmap_bytes_array = env.new_object_array(size as i32, "[B", JObject::null()).map_err(mje)?;
				// 		let width_array = env.new_int_array(size as i32).map_err(mje)?;
				// 		let height_array = env.new_int_array(size as i32).map_err(mje)?;

				// 		for pos in 0..sub_rects.len(){
				// 			let sub_rect = &sub_rects[pos];
				// 			//截取文字图块
				// 			let sub_clip_rect = Rect::new(
				// 				sub_rect.left,
				// 				sub_rect.top,
				// 				sub_rect.width,
				// 				sub_rect.height
				// 			);
				// 			//debug!("sub_clip_rect={:?}", sub_clip_rect);
				// 			let ch_clip_buffer = imgtool::get_rect(&clip_buffer, rect.width, &sub_clip_rect, bpp)?;
				// 			//转换jByteArray
				// 			let jbarray = env.byte_array_from_slice(&ch_clip_buffer).map_err(mje)?;
				// 			//添加图片数据
				// 			env.set_object_array_element(bitmap_bytes_array, pos as i32, JObject::from(jbarray)).map_err(mje)?;
				// 			env.set_int_array_region(width_array, pos as i32, &[sub_rect.width as i32]).map_err(mje)?;
				// 			env.set_int_array_region(height_array, pos as i32, &[sub_rect.height as i32]).map_err(mje)?;
				// 		}
				// 		//将图片数据发送到java
				// 		env.call_method(JObject::from(activity_class), "getText", "([[B[I[II)V",
				// 			&[	JValue::from(JObject::from(bitmap_bytes_array)),//bitmaps
				// 				JValue::from(JObject::from(width_array)),//width
				// 				JValue::from(JObject::from(height_array)),//height
				// 				JValue::from(4)
				// 			]).map_err(|err|{ format!("{:?}", err) })?;
				// 	}
				// 	debug!("耗时: {}ms", utils::duration_to_milis(&now.elapsed()));
				// }
				Ok(())
			})();
			if rec_result.is_err(){
				error!("文字识别失败: {:?}", rec_result.err());
			}
			
			let now = Instant::now();
			//复制像素
			let line_size = (width as i32*PIXEL_SIZE) as usize;
			let mut line_id = 0;
			for i in (0..rotate_raw_buffer.len()).step_by(line_size){//每一行
				if let Some(src_line) = rotate_raw_buffer.get(i..i+line_size){
					if let Some(target_line) = pixels.get_mut(line_id..line_id+line_size){
						target_line.copy_from_slice(&src_line);
						line_id += (buffer.stride*PIXEL_SIZE) as usize;
					}
				}
			}
			trace!("像素复制耗时{}ms", utils::duration_to_milis(&now.elapsed()));

			result[0] = width as i32;
			result[1] = height as i32;

			//Err("")
			Ok(())
		})
	})
}

//预览图片
#[no_mangle]
pub extern fn Java_cn_jy_lazydict_MainActivity_renderPreview<'a>(env: JNIEnv, activity_class: JClass, y: jni::objects::JByteBuffer, u: jni::objects::JByteBuffer, v:jni::objects::JByteBuffer, raw_width:jint, raw_height:jint, y_row_stride: jint, uv_row_stride:jint, uv_pixel_stride:jint, sensor_orientation: jint) -> jni::sys::jintArray{
	//trace!("send>>Java_cn_jy_lazydict_MainActivity_send width={}, height={} y_row_stride={} uv_row_stride={} uv_pixel_stride={}", raw_width, raw_height, y_row_stride, uv_row_stride, uv_pixel_stride);
	let mut result = [-1, -1];
	APP.with(|app|{
		let mut app = app.borrow_mut();
		match render(&mut *app, &mut result, &env, activity_class, y, u, v, raw_width, raw_height, y_row_stride, uv_row_stride, uv_pixel_stride, sensor_orientation){
			Ok(_) => (),
			Err(err) => error!("渲染失败 {:?}", err)
		}
	});

	if let Ok(arr) = env.new_int_array(2){
		let _ = env.set_int_array_region(arr, 0, &[result[0] as jint, result[1] as jint]);
		arr
	}else{
		0 as jni::sys::jintArray
	}
}