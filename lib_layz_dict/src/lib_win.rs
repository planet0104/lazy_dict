#[macro_use]
extern crate lazy_static;
#[macro_use] extern crate log;
// #[macro_use] extern crate allegro;
// extern crate allegro_font;
// extern crate allegro_ttf;
// extern crate input;
// extern crate window;
// extern crate gl;
// extern crate shader_version;
extern crate android_logger;
// extern crate glium;
extern crate zip;
extern crate png;
extern crate android_injected_glue;
extern crate android_glue;
// extern crate glutin;
// extern crate android_glue;
// extern crate piston_window;
use log::Level;
// extern crate winit_window;
extern crate winit;
// mod glutin_window;

// use piston_window::*;
// use glutin_window::GlutinWindow;
// use piston_window::WindowSettings;

// use allegro::*;
// use allegro_font::*;
// use allegro_ttf::*;

extern crate jni;
extern crate jni_sys;
use self::jni::JNIEnv;
use self::jni::objects::{JClass, JValue, JByteBuffer};
use jni_sys::{jint, jobject};

use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, Mutex};
mod utils;
use std::time::Instant;

lazy_static! {
	static ref IMAGE_SENDER:Arc<Mutex<Option<Sender<(Vec<u8>, usize, usize)>>>> = Arc::new(Mutex::new(None));
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


#[no_mangle]
pub unsafe extern fn Java_cn_jy_lazydict_MainActivity_send(env: JNIEnv, _: JClass, y: JByteBuffer, u: JByteBuffer, v:JByteBuffer, width:jint, height:jint){
	trace!("send>>Java_cn_jy_lazydict_MainActivity_send");
	let (width, height) = (width as i32, height as i32);
	let y_src = env.get_direct_buffer_address(y).unwrap();
	let u_src = env.get_direct_buffer_address(u).unwrap();
	let v_src = env.get_direct_buffer_address(v).unwrap();
	let mut src = vec![];
	src.extend_from_slice(y_src);
	src.extend_from_slice(u_src);
	src.extend_from_slice(v_src);

	let num_of_pixel = width * height;
	let position_of_v = num_of_pixel;
	let position_of_u = num_of_pixel/4 + num_of_pixel;
	let mut rgb = vec![0; num_of_pixel as usize*3];
	for i in 0..height{
		let start_y = i*width;
		let step = (i/2)*(width/2);
		let start_v = position_of_v + step;
		let start_u = position_of_u + step;
		for j in 0..width{
			let y = start_y + j;
			let v = start_v + j/2;
			let u = start_u + j/2;
			let index = y*3;
			let tmp = yuv_to_rgb(src[u as usize], src[u as usize], src[v as usize]);
			rgb[index as usize] = tmp[0];
			rgb[index as usize+1] = tmp[1];
			rgb[index as usize+2] = tmp[2];
		}
	}
	let _ = IMAGE_SENDER.lock().unwrap().as_ref().unwrap().send((rgb, width as usize, height as usize));
}

#[no_mangle]
pub unsafe extern fn Java_cn_jy_lazydict_MainActivity_sendRgb(env: JNIEnv, _: JClass, src: JByteBuffer, width:jint, height:jint){
	let ptr_src = env.get_direct_buffer_address(src).unwrap();
	let mut src = vec![];
	src.extend_from_slice(ptr_src);
	let _ = IMAGE_SENDER.lock().unwrap().as_ref().unwrap().send((src, width as usize, height as usize));
}

// use piston_window::*;
// use android_glue::*;

#[no_mangle]
#[inline(never)]
#[allow(non_snake_case)]
pub extern "C" fn android_main(app: *mut ()) {
	use android_logger::Filter;
	android_logger::init_once(Filter::default().with_min_level(Level::Trace));
    //cargo_apk_injected_glue::android_main2(app as *mut _, move |c, v| unsafe { main(c, v) });
	trace!("winit>>android_main!!!!");
}

//use android_injected_glue::ffi::ANativeActivity;

#[no_mangle]
pub unsafe extern fn  ANativeActivity_onCreate(app: *mut (), ud: *mut (), udsize: usize) {
	use android_logger::Filter;
	android_logger::init_once(Filter::default().with_min_level(Level::Trace));
	trace!("winit>>ANativeActivity_onCreate!!!!");
	android_injected_glue::android_main2(app as *mut _, move |c, v| unsafe { main(c, v) });
}

#[no_mangle]
pub fn main(_argc: isize, _char: *const *const u8){
	//use android_logger::Filter;
	//android_logger::init_once(Filter::default().with_min_level(Level::Trace));
	//allegro_main();
	trace!("winit>>main!!!!");
	//--------------------------------------------------------------------
	//--------------------------------------------------------------------
	// extern crate piston_window;
	// use piston_window::*;

	// let mut window: PistonWindow = WindowSettings::new("Hello Piston!", (640, 480))
	// 	.exit_on_esc(true)
	// 	.build()
	// 	.unwrap_or_else(|e| { panic!("Failed to build PistonWindow: {}", e) });
	// use glutin::GlContext;
	// let mut events_loop = glutin::EventsLoop::new();
    // let window = glutin::WindowBuilder::new();
    // let context = glutin::ContextBuilder::new().with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGlEs, (3, 0)));
	// trace!("context创建完成");
	// let result = glutin::GlWindow::new(window, context, &events_loop);
	//trace!("gl_window创建结果： {:?}", result);

    //let _ = unsafe { gl_window.make_current() };

    //println!("Pixel format of the window's GL context: {:?}", gl_window.get_pixel_format());

	//let gl = support::load(&gl_window.context());

	//=============================================================

	trace!("window>>创建结束");
	
	// while let Some(e) = window.next() {
	// 	window.draw_2d(&e, |_c, g| {
	// 		clear([0.5, 1.0, 0.5, 1.0], g);
	// 	});
	// }

	//--------------------------------------------------------------------
	//--------------------------------------------------------------------
	let mut events_loop = winit::EventsLoop::new();
	trace!("winit>>step 1");

    let _window = winit::WindowBuilder::new()
        .with_title("A fantastic window!")
        .build(&events_loop)
        .unwrap();
	trace!("winit>>step 2");

    events_loop.run_forever(|event| {
        trace!("winit>>{:?}", event);
        match event {
            winit::Event::WindowEvent {
                event: winit::WindowEvent::CloseRequested,
                ..
            } => winit::ControlFlow::Break,
            _ => winit::ControlFlow::Continue,
        }
    });

}

#[no_mangle]
pub unsafe extern fn Java_cn_jy_lazydict_MainActivity_winit(env: JNIEnv, _: JClass){
	use android_logger::Filter;
	android_logger::init_once(Filter::default().with_min_level(Level::Trace));
	trace!("winit>>进入!");
}

/*

fn load_ttf_font(ttf:&TtfAddon, filename:&str, size:i32) -> Option<Font>{
	match utils::copy_assets(filename){
		Ok(ttf_path) => Some(ttf.load_ttf_font(&ttf_path, size, TtfFlags::zero()).unwrap()),
		Err(err) =>{
			error!("字体文件加载失败: {:?}", err);
			None
		}
	}
}

#[no_mangle]
pub fn main(_argc: i32, _char:*mut u8){
	use android_logger::Filter;
	android_logger::init_once(Filter::default().with_min_level(Level::Trace));
	allegro_main();
}

*/