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

// const LEVEL:Level = Level::Error;
// const LEVEL:Level = Level::Trace;
const LEVEL:Level = Level::Debug;

//JNI加载完成
#[no_mangle]
pub extern fn JNI_OnLoad(_vm: *mut jni::sys::JavaVM, _reserved: *mut c_void) -> jint{
	android_logger::init_once(android_logger::Filter::default().with_min_level(LEVEL));
	info!("JNI_OnLoad.");
	jni::sys::JNI_VERSION_1_6
}

//根据坐标选择一个文字块
#[no_mangle]
pub extern fn Java_cn_jy_lazydict_Toolkit_getCharacterRect<'a>(env: JNIEnv, _activity: JClass, bitmap: JObject, x:jint, y:jint) -> jni::sys::jobject{
	let mje = |err|{ format!("getCharacterRect {:?}", err) };
	let mut select_rect = None;
	let result = (||->Result<(), String> {
		jni_graphics::lock_bitmap(&env, &bitmap, |info, pixels|{
			let stride = info.stride;
			let format = info.format;
			debug!("{}x{} stride={} format={}", info.width, info.height, stride, jni_graphics::get_format_name(format));
			//选择 160x160的一块图像进行二值化
			let rect = Rect::new(x as usize-80, y as usize-80, 160, 160);
			let mut sub = imgtool::get_rect(pixels, 4, info.stride, &rect)?;
			debug!("sub.len={}", sub.len());
			//二值化
			let bpp = 4;
			//计算阈值和像素灰度值
			let (threshold, gray_values) = imgtool::calc_threshold(&sub, bpp);
			//将原图像二值化
			imgtool::binary(&gray_values, &mut sub, bpp, threshold);
			//边缘检测
			let mut edges = vec![1; rect.width*rect.height]; //1为背景, 0为边缘
			imgtool::edge_detect_gray(&gray_values, &mut edges, rect.width, threshold);
			//根据edges分割
			for sub_rect in imgtool::split(0, 0, &mut edges, rect.width, rect.height){
				let (sleft, stop) = ((rect.left+sub_rect.left) as f32, (rect.top+sub_rect.top) as f32);
				let (sright, sbottom) = (sleft+sub_rect.width as f32, stop+sub_rect.height as f32);
				if sleft<x as f32 && sright>x as f32 && stop<y as f32 && sbottom>y as f32{
					select_rect = Some(jni_graphics::new_rectf(&env, sleft, stop, sright, sbottom).map_err(mje)?.into_inner());		
					break;
				}
			}
			if select_rect.is_some(){
				Ok(())
			}else{
				Err("没有匹配的区域".to_string())	
			}
		})?;
		Ok(())
	})();

	if result.is_err(){
		let err = result.err();
		error!("{:?}", &err);
		let _ = env.throw_new("java/lang/Exception", format!("{:?}", err));
		JObject::null().into_inner()
	}else{
		select_rect.unwrap()
	}
}

//YUV420SP转Bitmap
#[no_mangle]
pub extern fn Java_cn_jy_lazydict_Toolkit_decodeYUV420SP<'a>(env: JNIEnv, _activity: JClass, data: jni::sys::jbyteArray, width:jint, height:jint, camera_orientation: jint) -> jni::sys::jobject{
	let mje = |err|{ format!("转码失败 {:?}", err) };
	let mut bitmap = None;
	let result = (||->Result<(), String> {
		let data = env.convert_byte_array(data).map_err(mje)?;
		let mut colors = vec![0i32; (width*height) as usize];
		imgtool::decode_yuv420sp(&mut colors, &data, width, height);
		let mut rotate_buffer = vec![0i32; (width*height) as usize];

		//旋转
		let (rotate_buffer, new_width, new_height) = match camera_orientation{
			90 => {
				let (width, height) = imgtool::rotate90_colors(&colors, &mut rotate_buffer, width, height);
				(&rotate_buffer, width, height)
			}
			180 => {
				let (width, height) = imgtool::rotate180_colors(&colors, &mut rotate_buffer, width, height);
				(&rotate_buffer, width, height)
			}
			270 => {
				let (width, height) = imgtool::rotate270_colors(&colors, &mut rotate_buffer, width, height);
				(&rotate_buffer, width, height)
			}
			_ =>{
				//不用旋转，使用原buffer
				(&colors, width, height)
			}
		};

		let intarray = env.new_int_array(new_width*new_height).map_err(mje)?;
		env.set_int_array_region(intarray, 0, &rotate_buffer).map_err(mje)?;
		bitmap = Some(jni_graphics::create_java_bitmap_form_colors(&env, intarray, 0, new_width, new_width, new_height).map_err(mje)?.into_inner());
		Ok(())
	})();

	if result.is_err(){
		let err = result.err();
		error!("{:?}", &err);
		let _ = env.throw_new("java/lang/Exception", format!("{:?}", err));
		JObject::null().into_inner()
	}else{
		bitmap.unwrap()
	}
}