extern crate libc;
#[macro_use]
extern crate lazy_static;
#[macro_use] extern crate log;
#[macro_use] extern crate allegro;
extern crate allegro_sys;
extern crate allegro_font;
extern crate allegro_ttf;
extern crate android_logger;
extern crate zip;
extern crate png;
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
	error!("进入allegro_main..");
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

	let logo = utils::load_assets("rust.png").unwrap();
	let decoder = png::Decoder::new(logo.as_slice());
	let (info, mut reader) = decoder.read_info().unwrap();
	trace!("logo.png {}x{}", info.width, info.height);
    let mut img_data = vec![0; info.buffer_size()];
	reader.next_frame(&mut img_data).unwrap();

	//let mut now = Instant::now();

	//创建内存位图
	core.set_new_bitmap_flags_flag(core.get_new_bitmap_flags().get() | 1);
	let logo = Bitmap::new(&core, info.width as i32, info.height as i32).unwrap();
	unsafe{
		//复制图像数据
		let p = logo.get_allegro_bitmap();
		let lock = allegro_sys::al_lock_bitmap(p, allegro_sys::ALLEGRO_PIXEL_FORMAT_RGB_888 as i32, allegro_sys::ALLEGRO_LOCK_READWRITE as i32);
		let dat: *mut u8 = (*lock).data as *mut u8;
		let slice:&mut [u8] = ::std::slice::from_raw_parts_mut(dat, img_data.len());
		//slice顺序 b,g,r
		//img_data的顺序 rgb
		for i in (0..img_data.len()).step_by(3){
			slice[i] = img_data[i+2];
			slice[i+1] = img_data[i+1];
			slice[i+2] = img_data[i];
		}
		allegro_sys::al_unlock_bitmap(p);
	}
	
	let mut redraw = true;
	timer.start();
	'exit: loop{
		//let image = receiver.try_recv();
		if redraw && queue.is_empty(){
			core.clear_to_color(Color::from_rgb_f(1.0, 1.0, 1.0));
			core.draw_text(font.as_ref().unwrap(), Color::from_rgb_f(0.0, 0.0, 1.0),
				(display.get_width() / 2) as f32, (display.get_height() / 2) as f32,
				FontAlign::Centre, "Hello Rust!");
			core.draw_text(font.as_ref().unwrap(), Color::from_rgb_f(0.0, 0.0, 0.0),
				(display.get_width() / 2) as f32, (display.get_height() / 2) as f32+128.0,
				FontAlign::Centre, "懒人字典");
			//core.draw_scaled_bitmap(&logo, 0.0, 0.0, 10.0, 10.0, 0.0, 0.0, 100.0, 100.0, BitmapDrawingFlags::zero());
			core.draw_bitmap(&logo, 0.0, 0.0, BitmapDrawingFlags::zero());
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