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
use jni::sys::{jint, jbyteArray};
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

//计算图片阈值，并返回像素灰度数组
#[no_mangle]
pub extern fn Java_cn_jy_lazydict_Toolkit_calcThreshold<'a>(env: JNIEnv, _activity: JClass, bitmap: JObject) -> jni::sys::jobject{
	let mje = |err|{ format!("calcThreshold {:?}", err) };
	let mut obj = None;
	let result = (||->Result<(), String> {
		jni_graphics::lock_bitmap(&env, &bitmap, |info, pixels|{
			debug!("info={:?} pixels.len()={}", info, pixels.len());
			//只支持argb888格式
			if info.format != jni_graphics::ANDROID_BITMAP_FORMAT_RGBA_8888{
				Err("图片格式只支持RGBA_8888!".to_string())
			}else{
				//计算整张图的阈值
				let (threshold, gray_values) = imgtool::calc_threshold(&pixels, info.width as usize, info.height as usize, info.stride as usize, 4);
				let mut gray_count = vec![0; 256];
				for chs in &gray_values{
					gray_count[*chs as usize] += 1;
				}
				debug!("计算阈值:{} gray_count={:?}", threshold, gray_count);
				let grays = env.byte_array_from_slice(&gray_values).map_err(mje)?;
				obj = Some(env.new_object("cn/jy/lazydict/ThresholdGray", "(IIII[B)V",
				&[
					JValue::from(threshold as i32),
					JValue::from(info.width as i32),
					JValue::from(info.height as i32),
					JValue::from(4),
					JValue::from(JObject::from(grays)),
				]).map_err(mje)?.into_inner());
				Ok(())
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
		obj.unwrap()
	}
}

//根据坐标选择一个文字块
#[no_mangle]
pub extern fn Java_cn_jy_lazydict_Toolkit_getCharacterRect<'a>(env: JNIEnv, _activity: JClass, obj:JObject, x:jint, y:jint) -> jni::sys::jobject{
	let mje = |err|{ format!("getCharacterRect {:?}", err) };
	let mut select_rect = None;
	let result = (||->Result<(), String> {
		//灰度图信息
		let all_gray_values = env.convert_byte_array(env.get_field(obj, "grays", "[B").map_err(mje)?.l().map_err(mje)?.into_inner()).map_err(mje)?;
		let threshold = env.get_field(obj, "threshold", "I").map_err(mje)?.i().map_err(mje)? as u8;
		let width = env.get_field(obj, "width", "I").map_err(mje)?.i().map_err(mje)?;
		// let height = env.get_field(obj, "height", "I").map_err(mje)?.i().map_err(mje)?;
		// let bpp = env.get_field(obj, "bpp", "I").map_err(mje)?.i().map_err(mje)?;
		debug!("({}x{}) threshold={} threshold.len()={}", x, y, threshold, all_gray_values.len());
		let mut gray_count = vec![0; 256];
		for chs in &all_gray_values{
			gray_count[*chs as usize] += 1;
		}
		debug!("getCharacterRect gray_count={:?}", gray_count);

		//选择 160x160的一块图像进行二值化
		let rect = Rect::new(x as usize-80, y as usize-80, 160, 160);
		debug!("截取区域{:?}", rect);
		// for chs in all_gray_values.chunks(width as usize){
		// 	debug!("{:?}", chs);
		// }
		let gray_values = imgtool::get_gray_rect(&all_gray_values, width, &rect)?;
		let mut gray_count = vec![0; 256];
		for chs in &gray_values{
			gray_count[*chs as usize] += 1;
		}
		debug!("截取以后的gray_count={:?}", gray_count);
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
pub extern fn Java_cn_jy_lazydict_Toolkit_decodeYUV420SP<'a>(env: JNIEnv, _activity: JClass, data: jbyteArray, width:jint, height:jint, camera_orientation: jint) -> jni::sys::jobject{
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