#[macro_use] extern crate log;
#[macro_use] extern crate allegro;
extern crate allegro_font;
use log::Level;

#[cfg(target_os="android")]
extern crate android_logger;

use allegro::*;
use allegro_font::*;

fn allegro_main(){
	let core = Core::init().unwrap();
	let font_addon = FontAddon::init(&core).unwrap();

	let display = Display::new(&core, 800, 600).unwrap();
	let timer = Timer::new(&core, 1.0 / 60.0).unwrap();
	let font = Font::new_builtin(&font_addon).unwrap();

	let queue = EventQueue::new(&core).unwrap();
	queue.register_event_source(display.get_event_source());
	queue.register_event_source(timer.get_event_source());

	let mut redraw = true;
	timer.start();
	'exit: loop
	{
		if redraw && queue.is_empty()
		{
			core.clear_to_color(Color::from_rgb_f(0.0, 0.0, 0.0));
			core.draw_text(&font, Color::from_rgb_f(1.0, 1.0, 1.0),
				(display.get_width() / 2) as f32, (display.get_height() / 2) as f32,
				FontAlign::Centre, "Welcome to RustAllegro!");
			core.flip_display();
			redraw = false;
		}

		match queue.wait_for_event()
		{
			DisplayClose{..} => break 'exit,
			TimerTick{..} => redraw = true,
			_ => (),
		}
	}
}

#[no_mangle]
#[cfg(target_os="android")]
pub fn main(argc: i32, char:*mut u8){
	use android_logger::Filter;
	android_logger::init_once(
        Filter::default().with_min_level(Level::Trace));
	trace!("hello allegro!");
	allegro_main();
}
