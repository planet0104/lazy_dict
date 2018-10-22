extern crate libc;
#[macro_use]
extern crate lazy_static;
#[macro_use] extern crate log;
extern crate allegro;
extern crate allegro_sys;
extern crate allegro_font;
extern crate allegro_ttf;
extern crate android_logger;
extern crate zip;
extern crate png;
use log::Level;
extern crate bytes;

use allegro::*;
use allegro_font::*;
use allegro_ttf::*;

extern crate jni;
use jni::JNIEnv;
use jni::sys::{jbyteArray, jint};
use self::jni::objects::{JClass, JValue, JByteBuffer};

use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
mod utils;
use std::time::Instant;

lazy_static! {
	static ref IMAGE_SENDER:Arc<Mutex<Option<Sender<(Vec<u8>, i32, i32)>>>> = Arc::new(Mutex::new(None));
}

fn allegro_main(){
	error!("进入allegro_main..");
	let core = Core::init().unwrap();
	let font_addon = FontAddon::init(&core).unwrap();
	let ttf_addon = TtfAddon::init(&font_addon).unwrap();

	let display = Display::new(&core, 1600, 900).unwrap();
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

	(move || {
		 *IMAGE_SENDER.lock().unwrap() = Some(sender);
		 trace!("设置了IMAGE_SENDER Sender!!!");
	})();

	// let logo = utils::load_assets("rust.png").unwrap();
	// let decoder = png::Decoder::new(logo.as_slice());
	// let (info, mut reader) = decoder.read_info().unwrap();
	// trace!("logo.png {}x{}", info.width, info.height);
    // let mut img_data = vec![0; info.buffer_size()];
	// reader.next_frame(&mut img_data).unwrap();

	//创建内存位图
	core.set_new_bitmap_flags_flag(core.get_new_bitmap_flags().get() | 1);
	let mut capture = None;

	let img_data = {
		let logo = utils::load_assets("rust.png").unwrap();
		let decoder = png::Decoder::new(logo.as_slice());
		let (info, mut reader) = decoder.read_info().unwrap();
		trace!("logo.png {}x{}", info.width, info.height);
		let mut img_data = vec![0; info.buffer_size()];
		reader.next_frame(&mut img_data).unwrap();

		img_data
	};
	
	let mut redraw = true;
	timer.start();
	'exit: loop{
		trace!("loop001");
		//let image = receiver.try_recv();
		if redraw && queue.is_empty(){
			trace!("loop002");
			core.clear_to_color(Color::from_rgb_f(1.0, 1.0, 1.0));
			core.draw_text(font.as_ref().unwrap(), Color::from_rgb_f(0.0, 0.0, 1.0),
				(display.get_width() / 2) as f32, (display.get_height() / 2) as f32,
				FontAlign::Centre, "Hello Rust!");
			core.draw_text(font.as_ref().unwrap(), Color::from_rgb_f(0.0, 0.0, 0.0),
				(display.get_width() / 2) as f32, (display.get_height() / 2) as f32+128.0,
				FontAlign::Centre, "懒人字典");
			trace!("loop003");
			//更新图片
			let now = Instant::now();
			if let Ok((data, width, height)) = receiver.try_recv(){
				trace!("游戏循环 接收到图片!!!!!!!!");
				if capture.is_none(){
					capture = Some(Bitmap::new(&core, width, height).unwrap());
				}
				let bitmap = capture.as_ref().unwrap();
				//复制图像数据
				let a_bmp = bitmap.get_allegro_bitmap();
				let lock = unsafe{ allegro_sys::al_lock_bitmap(a_bmp, allegro_sys::ALLEGRO_PIXEL_FORMAT_RGB_888 as i32, allegro_sys::ALLEGRO_LOCK_WRITEONLY as i32) };
				let dat: *mut u8 = unsafe{ (*lock).data as *mut u8 };
				let slice:&mut [u8] = unsafe{ ::std::slice::from_raw_parts_mut(dat, data.len()) };
				//slice顺序 b,g,r
				//data的顺序 rgb
				for i in (0..data.len()).step_by(3){
					slice[i] = data[i+2];
					slice[i+1] = data[i+1];
					slice[i+2] = data[i];
				}
				unsafe{ allegro_sys::al_unlock_bitmap(a_bmp) };
			}
			trace!("try_recv耗时 {}ms", utils::duration_to_milis(&now.elapsed()));
			//显示图片
			if capture.is_some(){
				let cap = capture.as_ref().unwrap();
				let now = Instant::now();
				core.draw_bitmap_region(cap, 0.0, 0.0, 300.0, 300.0, 0.0, 0.0, BitmapDrawingFlags::zero());
				trace!("显示图片耗时 {}ms", utils::duration_to_milis(&now.elapsed()));
			}
			// //core.draw_scaled_bitmap(&logo, 0.0, 0.0, 10.0, 10.0, 0.0, 0.0, 100.0, 100.0, BitmapDrawingFlags::zero());

			// let now = Instant::now();
			// unsafe{
			// 	//复制图像数据
			// 	trace!("复制001");
			// 	let screen = allegro_sys::al_get_target_bitmap();
			// 	trace!("复制002");
			// 	let lock = allegro_sys::al_lock_bitmap_region(screen, 0, 0, 200, 200, allegro_sys::ALLEGRO_PIXEL_FORMAT_ANY as i32, allegro_sys::ALLEGRO_LOCK_WRITEONLY as i32);
			// 	if !lock.is_null() && !(*lock).data.is_null(){
			// 		//trace!("复制003 img_data.len()={} 672*371*3={}", img_data.len(), 200*200*3);
			// 		let format = (*lock).format;
			// 		let size = (*lock).pixel_size;
			// 		/*
			// 		fn rgb_24_2_565(r: u8, g: u8, b: u8) ->
			// 			{  
			// 				return (USHORT)(((unsigned(r) << 8) & 0xF800) |   
			// 						((unsigned(g) << 3) & 0x7E0)  |  
			// 						((unsigned(b) >> 3)));  
			// 			}  
			// 		*/
					
			// 		trace!("复制004 format={} size={}", format, size);
			// 		let dat: *mut u8 = (*lock).data as *mut u8;
			// 		trace!("复制005");
			// 		let slice:&mut [u8] = ::std::slice::from_raw_parts_mut(dat, 200*200*2);
			// 		trace!("复制006");
			// 		//slice顺序 bgr 565
			// 		//img_data的顺序 rgb
			// 		use bytes::{ByteOrder, LittleEndian};
			// 		for i in (0..slice.len()).step_by(2){
			// 			let mut buf = [0; 2];
			// 			LittleEndian::write_u16(&mut buf, rgb888_to_bgr565(255, 0, 0));
			// 			slice[i] = buf[0];
			// 			slice[i+1] = buf[1];
			// 			//slice[i+2] = 0;
			// 		}
			// 		// let mut slice_index = 0;
			// 		// for i in (0..img_data.len()).step_by(3){
			// 		// 	let b = rgb888_to_bgr565(img_data[i], img_data[i+1], img_data[i+2]);
			// 		// 	slice[slice_index] = b;
			// 		// 	slice_index += 1;
			// 		// 	//trace!("slice_index={} slice.len()={}", slice_index, slice.len());
			// 		// }
			// 		trace!("复制008");
			// 		allegro_sys::al_unlock_bitmap(screen);
			// 		trace!("复制009");
			// 	}
			// }
			//core.draw_bitmap(&test_logo, 0.0, 500.0, BitmapDrawingFlags::zero());
			//trace!("test_logo显示耗时 {}ms", utils::duration_to_milis(&now.elapsed()));
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

fn rgb888_to_bgr565(red: u8, green: u8, blue: u8) -> u16{
	let b = (blue as u16 >> 3) & 0x001F;
	let g = ((green as u16 >> 2) << 5) & 0x07E0;
	let r = ((red as u16 >> 3) << 11) & 0xF800;
	return b | g | r;
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

// #[no_mangle]
// pub unsafe extern fn Java_cn_jy_lazydict_MainActivity_send(env: JNIEnv, _: JClass, y: JByteBuffer, u: JByteBuffer, v:JByteBuffer, width:JValue, height:JValue){
// 	trace!("send>>Java_cn_jy_lazydict_MainActivity_send");
// 	let (width, height) = (width.i().unwrap(), height.i().unwrap());
// 	let y_src = env.get_direct_buffer_address(y).unwrap();
// 	let u_src = env.get_direct_buffer_address(u).unwrap();
// 	let v_src = env.get_direct_buffer_address(v).unwrap();
// 	let mut src = vec![];
// 	src.extend_from_slice(y_src);
// 	src.extend_from_slice(u_src);
// 	src.extend_from_slice(v_src);

// 	let num_of_pixel = width * height;
// 	let position_of_v = num_of_pixel;
// 	let position_of_u = num_of_pixel/4 + num_of_pixel;
// 	let mut rgb = vec![0; num_of_pixel as usize*3];
// 	for i in 0..height{
// 		let start_y = i*width;
// 		let step = (i/2)*(width/2);
// 		let start_v = position_of_v + step;
// 		let start_u = position_of_u + step;
// 		for j in 0..width{
// 			let y = start_y + j;
// 			let v = start_v + j/2;
// 			let u = start_u + j/2;
// 			let index = y*3;
// 			let tmp = yuv_to_rgb(src[u as usize], src[u as usize], src[v as usize]);
// 			rgb[index as usize] = tmp[0];
// 			rgb[index as usize+1] = tmp[1];
// 			rgb[index as usize+2] = tmp[2];
// 		}
// 	}
// 	let _ = IMAGE_SENDER.lock().unwrap().as_ref().unwrap().send((rgb, width, height));
// }

#[allow(non_snake_case)]
#[no_mangle]
pub extern fn Java_cn_jy_lazydict_MainActivity_test(env: JNIEnv, _: JClass, val:jint){
	trace!("Java_cn_jy_lazydict_MainActivity_test>>>>>>>val={}", val);
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern fn Java_cn_jy_lazydict_MainActivity_sendRgb(env: JNIEnv, _: JClass, src: jbyteArray, width:jint, height:jint){
	trace!("sendRgb>>>>>>> width={} height={}", width, height);
	let data = env.convert_byte_array(src).unwrap();
	trace!("sendRgb>>>>>>>convert_byte_array data.len()={}", data.len());
	let lock = IMAGE_SENDER.lock();
	if lock.is_err(){
		trace!("IMAGE_SENDER锁定失败 {:?}", lock.err());
	}else{
		let sender = lock.unwrap();
		if sender.is_none(){
			error!("IMAGE_SENDER sender为空！！");
		}else{
			let r = sender.as_ref().unwrap().send((data, width, height));
			trace!("IMAGE_SENDER send {:?}", r);
		}
	}
	trace!("sendRgb>>>>>>>003");
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