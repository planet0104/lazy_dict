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
extern crate graphics;
extern crate glutin;
extern crate shader_version;
use self::jni::objects::{JClass, JValue, JByteBuffer};
use jni::sys::{jint, jobject, jbyteArray};
use std::os::raw::{c_void};
extern crate sdl2;
use sdl2::surface::Surface;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::pixels::PixelFormatEnum;
mod utils;
extern crate gfx;
use gfx::traits::*;
use gfx::memory::Typed;
use gfx::format::{DepthStencil, Formatted, Srgba8};
use sdl2::rect::Rect;
extern crate gfx_graphics;
extern crate gfx_device_gl;
extern crate egl;
extern crate bytes;
mod jni_graphics;
mod native_window;
use jni_graphics::*;
use native_window::*;
use gfx::traits::*;
use gfx::memory::Typed;
use gfx::format::{DepthStencil, Formatted, Srgba8};
use std::path::Path;
use piston::window::{OpenGLWindow, Window, WindowSettings, Size};
use piston::input::{AfterRenderEvent, RenderEvent};
use piston::event_loop::{Events, EventSettings, EventLoop};
use gfx_graphics::{Gfx2d, GlyphCache, TextureSettings};

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


		{
			let context_builder = glutin::ContextBuilder::new()
				.with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGlEs, (3, 0)));
			//let glutin::ContextBuilder { pf_reqs, gl_attr } = context_builder;
        	//let gl_attr = gl_attr.map_sharing(|_ctxt| panic!("Context sharing is not allowed when using `new()`. Please instead use `new_shared()`."));
			//let gl_attr = gl_attr.clone().map_sharing(|c| &c.0.egl_context);

			let native_window = window;

			if native_window.is_null() {
				error!("Android's native window is null");
				return;
			}

			//创建 use glutin::api::egl::Context; (EglContext)
			
			// let context = try!(EglContext::new(egl, pf_reqs, &gl_attr, )
			// 			.and_then(|p| p.finish(native_window as *const _)));
			// calling `eglGetDisplay` or equivalent

			let display = egl::get_display(egl::EGL_DEFAULT_DISPLAY as *mut _);
			if display.is_none() {
            	error!("Could not create EGL display object");
				return;
			}
			let display = display.unwrap();
			trace!("egl::get_display执行成功");
			let egl_version = unsafe {
				use std::mem;
				let mut major: egl::EGLint = mem::uninitialized();
				let mut minor: egl::EGLint = mem::uninitialized();

				if !egl::initialize(display, &mut major, &mut minor){
					error!("eglInitialize failed");
					return;
				}
				(major, minor)
			};
			trace!("egl::initialize执行成功");
			let extensions = if egl_version >= (1, 2) {
				if let Some(p) = egl::query_string(display, egl::EGL_EXTENSIONS){
					let list = String::from_utf8(p.to_bytes().to_vec()).unwrap_or_else(|_| format!(""));
					list.split(' ').map(|e| e.to_string()).collect::<Vec<_>>()
				}else{
					vec![]
				}
			} else {
				vec![]
			};
			trace!("egl::query_string 执行成功");

			// binding the right API and choosing the version
			let (version, api) = unsafe {
				if egl_version >= (1, 4) {
					if egl::bind_api(egl::EGL_OPENGL_API){
						(None, glutin::Api::OpenGl)
					} else if egl::bind_api(egl::EGL_OPENGL_ES_API){
						(None, glutin::Api::OpenGlEs)
					} else {
						error!("不支持OpenGL!");
						return;
					}
				} else {
					(None, glutin::Api::OpenGlEs)
				}
			};
			trace!("egl::bind_api 执行成功");
			/*
			Ok(ContextPrototype {
				opengl: opengl,
				egl: egl,
				display: display,
				egl_version: egl_version,
				extensions: extensions,
				api: api,
				version: version,
				config_id: config_id,
				pixel_format: pixel_format,
			})
			 */
			//事件监听！！！！！！！！！！
			//	ctx.egl_context.on_surface_destroyed(); 调用 egl::CreateWindowSurface
			//	ctx.egl_context.on_surface_created(); 调用 egl::MakeCurrent

			// let (config_id, pixel_format) = unsafe {
			// 	choose_fbconfig(&egl, display, &egl_version, api, version, pf_reqs)?
			// };
			use gfx_graphics::{Gfx2d, GlyphCache, TextureSettings};
			use std::path::Path;

			let (mut device, mut factory) = gfx_device_gl::create(|s|
				egl::get_proc_address(s) as *const std::os::raw::c_void);
			let mut glyph_cache = GlyphCache::new(
					Path::new("assets/FiraSans-Regular.ttf"),
					factory.clone(),
					TextureSettings::new()
			).unwrap();

			let samples = 4;
			let aa = samples as gfx::texture::NumSamples;
			let dim = (640, 480, 1, aa.into());
			let color_format = <Srgba8 as Formatted>::get_format();
			let depth_format = <DepthStencil as Formatted>::get_format();
			let (output_color, output_stencil) = gfx_device_gl::create_main_targets_raw(dim, color_format.0, depth_format.0);
			
			let mut encoder = factory.create_command_buffer().into();
			let mut g2d = Gfx2d::new(shader_version::opengl::OpenGL::V3_2, &mut factory);

			g2d.draw(&mut encoder, &output_color, &output_stencil, args.viewport(), |c, g| {
                use graphics::*;

                clear([1.0; 4], g);
                text::Text::new_color([0.0, 0.5, 0.0, 1.0], 32).draw(
                    "Hello gfx_graphics!",
                    &mut glyph_cache,
                    &DrawState::default(),
                    c.transform.trans(10.0, 100.0),
                    g
                ).unwrap();
            });
			encoder.flush(&mut device);

			// let context = egl::get_current_context();
			// let display = egl::get_current_display();


		// 		let gl_attr = gl_attr.clone().map_sharing(|c| &c.0.egl_context);
		// 		let native_window = unsafe { android_glue::get_native_window() };
		// 		if native_window.is_null() {
		// 			return Err(OsError(format!("Android's native window is null")));
		// 		}
		// 		let egl = egl::ffi::egl::Egl;
		// let native_display = egl::NativeDisplay::Android;

		}


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