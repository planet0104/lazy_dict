#[macro_use]
extern crate lazy_static;
#[macro_use] extern crate log;
#[macro_use] extern crate allegro;
extern crate allegro_font;
extern crate allegro_ttf;
extern crate android_logger;
extern crate zip;
extern crate png;
extern crate glutin;
extern crate android_glue;
extern crate piston_window;
use log::Level;

use allegro::*;
use allegro_font::*;
use allegro_ttf::*;

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

fn allegro_main(){
	let core = Core::init().unwrap();
	let font_addon = FontAddon::init(&core).unwrap();
	let ttf_addon = TtfAddon::init(&font_addon).unwrap();

	let display = Display::new(&core, 800, 600).unwrap();
	let timer = Timer::new(&core, 1.0 / 60.0).unwrap();

	trace!("加载字体文件");
	let mut font = load_ttf_font(&ttf_addon, "FZKTJW.TTF", 128);
	if font.is_none(){
		font = Some(Font::new_builtin(&font_addon).unwrap());
	}
	trace!("加载完成。");

	let queue = EventQueue::new(&core).unwrap();
	queue.register_event_source(display.get_event_source());
	queue.register_event_source(timer.get_event_source());

	let (sender, receiver) = channel();

	*IMAGE_SENDER.lock().unwrap() = Some(sender);

	// let logo = utils::load_assets("logo.png").unwrap();
	// let decoder = png::Decoder::new(logo.as_slice());
	// let (info, mut reader) = decoder.read_info().unwrap();
    // let mut img_data = vec![0; info.buffer_size()];
	// reader.next_frame(&mut img_data).unwrap();

	let mut logo = None;
	match utils::copy_assets("logo.png"){
		Ok(logo_path) =>{
			trace!("文件路径:{}", logo_path);
			match utils::load_file(&logo_path){
				Ok(data) => {
					let decoder = png::Decoder::new(data.as_slice());
					let (info, mut reader) = decoder.read_info().unwrap();
					trace!("load_test {}x{}", info.width, info.height);
				}
				Err(err) => error!("load_test {:?}", err)
			}
			match Bitmap::load(&core, &logo_path){
				Ok(bitmap) => logo = Some(bitmap),
				Err(err) => error!("1.logo读取失败: {:?}", err)
			}
		},
		Err(err) => error!("2.logo读取失败: {:?}", err)
	}

	let mut redraw = true;
	timer.start();
	'exit: loop{
		//let image = receiver.try_recv();
		if redraw && queue.is_empty(){
			let now = Instant::now();
			core.clear_to_color(Color::from_rgb_f(1.0, 1.0, 1.0));
			core.draw_text(font.as_ref().unwrap(), Color::from_rgb_f(0.0, 0.0, 1.0),
				(display.get_width() / 2) as f32, (display.get_height() / 2) as f32,
				FontAlign::Centre, "Hello Rust!");
			core.draw_text(font.as_ref().unwrap(), Color::from_rgb_f(0.0, 0.0, 0.0),
				(display.get_width() / 2) as f32, (display.get_height() / 2) as f32+128.0,
				FontAlign::Centre, "懒人字典");
			if let Some(bitmap) = logo.as_ref(){
				core.draw_bitmap(bitmap, 200.0, 200.0, BitmapDrawingFlags::zero());
			}
			// for x in 0..info.width{
			// 	for y in 0..info.height{
			// 		let i = (y*info.width*4+x*4) as usize;
			// 		core.draw_pixel(x as f32, y as f32, Color::from_rgb(img_data[i], img_data[i+1], img_data[i+2]));
			// 		//core.put_pixel(x as i32, y as i32, Color::from_rgb(img_data[i], img_data[i+1], img_data[i+2]));
			// 	}
			// }
			//trace!("耗时{}ms", utils::duration_to_milis(&now.elapsed()));
			core.flip_display();
			redraw = false;
		}

		match queue.wait_for_event(){
			DisplayClose{..} => break 'exit,
			TimerTick{..} => redraw = true,
			_ => (),
		}
	}
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

use piston_window::*;
use android_glue::*;
// use glutin::api::android::ANativeActivity;

// #[repr(C)]
// pub struct android_app {
//     pub userData: *mut c_void,
//     pub onAppCmd: extern fn(*mut android_app, i32),
//     pub onInputEvent: extern fn(*mut android_app, *const AInputEvent) -> i32,
//     pub activity: *const ANativeActivity,
//     pub config: *const AConfiguration,
//     pub savedState: *mut c_void,
//     pub savedStateSize: usize,
//     pub looper: *mut ALooper,
//     pub inputQueue: *const AInputQueue,
//     pub window: *const ANativeWindow,
//     pub contentRect: ARect,
//     pub activityState: c_int,
//     pub destroyRequested: c_int,
// }

// #[no_mangle]
pub static mut ANDROID_APP: *mut glutin::Context = 0 as *mut glutin::Context;
// /// Returns the current Context.
// fn get_context() -> &'static Context {
//     let context = unsafe { (*ANDROID_APP).userData };
//     unsafe { std::mem::transmute(context) }
// }


https://github.com/jbg/conrod-android-skeleton

/// Adds a SyncEventHandler which will process sync events from the polling loop.
pub fn add_sync_event_handler(handler: Box<SyncEventHandler>) {
    let mut handlers = get_context().sync_event_handlers.lock().unwrap();
    handlers.push(handler);
}

#[no_mangle]
pub unsafe extern fn cargo_apk_injected_glue_add_sync_event_handler(handler: *mut ()) {
    let handler: Box<Box<SyncEventHandler>> = Box::from_raw(handler as *mut _);
    add_sync_event_handler(*handler);
}

#[no_mangle]
pub unsafe extern fn Java_cn_jy_lazydict_MainActivity_run(env: JNIEnv, _: JClass){
	let build =
        WindowSettings::new("Hello World!", [512; 2])
            .build();
	// let context = glutin::ContextBuilder::new()
    //     .with_vsync(true);
	// ANDROID_APP = context;
	if build.is_err(){
		error!("窗口创建失败! {:?}", build.err());
		return;
	}
	let mut window: PistonWindow = build.unwrap();
    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g| {
            clear([0.5, 0.5, 0.5, 1.0], g);
            rectangle([1.0, 0.0, 0.0, 1.0], // red
                      [0.0, 0.0, 100.0, 100.0], // rectangle
                      c.transform, g);
        });
    }
}

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
