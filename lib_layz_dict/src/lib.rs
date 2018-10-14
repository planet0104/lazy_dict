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
mod utils;
use sdl2::rect::Rect;
extern crate bytes;
mod jni_graphics;
mod native_window;
use jni_graphics::*;
use native_window::*;

#[allow(non_snake_case)]
#[no_mangle]
pub extern fn JNI_OnLoad(vm: *mut jni::sys::JavaVM, reserved: *mut c_void) -> jint{
	android_logger::init_once(android_logger::Filter::default().with_min_level(Level::Trace));
	trace!("JNI_OnLoad.");
	jni::sys::JNI_VERSION_1_6
}

//参考https://blog.csdn.net/u010593680/article/details/41410289
#[allow(non_snake_case)]
#[no_mangle]
pub extern fn Java_cn_jy_lazydict_MainActivity_drawSurface(env: JNIEnv, _: JClass, surface: jobject){
	trace!("drawSurface调用!");
	unsafe{
		let window = ANativeWindow_fromSurface(env.get_native_interface(), surface);

		trace!("NativeWindow is null={}", window.is_null());
		let mut buffer = ANativeWindow_Buffer{
			width: 0,
			height: 0,
			stride: 0,
			format: 0,
			bits: 0 as *mut c_void,
			reserved: [0; 5],
		};
		let mut rect = ARect {
			left: 0,
			top: 0,
			right: 0,
			bottom: 0,
		};
		let ret = ANativeWindow_lock(window, &mut buffer, &mut rect);
		if ret !=0{
			trace!("ANativeWindow_lock 调用失败! {}", ret);
			return;
		}
		trace!("buffer.format={} {}x{}", buffer.format, buffer.width, buffer.height); // WINDOW_FORMAT_RGB_565=4
		

		//不执行post不会绘图
		let ret = ANativeWindow_unlockAndPost(window);
		if ret != 0{
			trace!("ANativeWindow_unlockAndPost 调用失败!");
			return;
		}
		ANativeWindow_release(window);
		trace!("ANativeWindow_release.");
	}
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern fn Java_cn_jy_lazydict_SurfaceView_drawFrame(env: JNIEnv, _: JClass, bitmap: jobject){
	trace!("drawFrame!!!!!");
	let info = unsafe{
		let info_ptr: *mut AndroidBitmapInfo = Box::into_raw(Box::new(AndroidBitmapInfo{
			width: 0,
			height: 0,
			stride: 0,
			format: 0,
			flags: 0,
		}));
		let ret = AndroidBitmap_getInfo(env.get_native_interface(), bitmap, info_ptr);
		if ret<0{
			error!("AndroidBitmap_getInfo调用失败! {}", ret);
			return;
		}
		Box::from_raw(info_ptr)
	};
	trace!("图片 {}x{} format={}", info.width, info.height, get_format_name(info.format));
	let now = Instant::now();
	unsafe{
		let mut pixels = 0 as *mut c_void;
		let ret = AndroidBitmap_lockPixels(env.get_native_interface(), bitmap, &mut pixels);
		if ret<0{
			error!("AndroidBitmap_lockPixels! {}", ret);
			return;
		}
		let mut pixels = std::slice::from_raw_parts_mut(pixels as *mut u8, (info.width*info.height*2) as usize);
		//绘图
		let surface = Surface::from_data(&mut pixels, info.width, info.height, info.width*2, PixelFormatEnum::RGB565).unwrap();
		let mut canvas = Canvas::from_surface(surface).unwrap();
		canvas.set_draw_color(Color::RGB(255, 0, 0));
		canvas.clear();
		/*
		//图片 672x371=249312像素=747936字节
		let logo = utils::load_assets("rust.png").unwrap();
		let image = image::load_from_memory(&logo).unwrap().to_rgb();
		let (iw, ih) = (image.width(), image.height());
		//let mut buffer_rgb = image.into_raw();

		

		let mut buffer = vec![];//转换为RGB565
		for y in 0..371{
			for x in 0..672{
				let pixel = image.get_pixel(x, y);
				let rgb565 = rgb888_to_rgb565(pixel[0],pixel[1],pixel[2]);
				use bytes::{ByteOrder, LittleEndian};
				let mut buf = [0; 2];
				LittleEndian::write_u16(&mut buf, rgb565);
				buffer.push(buf[0]);
				buffer.push(buf[1]);
			}
		}

		let texture_creator = canvas.texture_creator();
		//trace!("buffer.len()={}", buffer.len());
		let mut texture = texture_creator.create_texture_from_surface(Surface::from_data(buffer.as_mut_slice(), iw, ih, iw*2, PixelFormatEnum::RGB565).unwrap()).unwrap();
		canvas.copy(&mut texture, None, Some(Rect::new(0, 0, iw, ih))).unwrap();

		*/
		canvas.present();

		let ret = AndroidBitmap_unlockPixels(env.get_native_interface(), bitmap);
		trace!("AndroidBitmap_unlockPixels! {} 耗时{}ms", ret, utils::duration_to_milis(&now.elapsed()));
	}
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

/*
// let result = canvas.with_texture_canvas(&mut texture, |texture_canvas| {
    //     texture_canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
    //     texture_canvas.clear();
    //     texture_canvas.set_draw_color(Color::RGBA(255, 0, 0, 255));
    //     texture_canvas.fill_rect(Rect::new(50, 50, 50, 50)).unwrap();
    // });
*/