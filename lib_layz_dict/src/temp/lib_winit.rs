#[macro_use]
extern crate lazy_static;
#[macro_use] extern crate log;
extern crate android_logger;
extern crate zip;
extern crate png;
extern crate android_injected_glue;
extern crate android_glue;
use log::Level;
extern crate winit;
extern crate jni;
use self::jni::JNIEnv;
use self::jni::objects::{JClass, JValue, JByteBuffer};
use jni::sys::{jint, jobject};
#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate gfx_device_gl;
extern crate glutin;
extern crate lyon;

use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, Mutex};
mod utils;
use std::time::Instant;

use lyon::extra::rust_logo::build_logo_path;
use lyon::path::builder::*;
use lyon::math::*;
use lyon::tessellation::geometry_builder::{VertexConstructor, VertexBuffers, BuffersBuilder};
use lyon::tessellation::{FillTessellator, FillOptions};
use lyon::tessellation;
use lyon::path::default::Path;

use gfx::traits::{Device, FactoryExt};

use glutin::GlContext;

type ColorFormat = gfx::format::Rgba8;
type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex GpuFillVertex {
        position: [f32; 2] = "a_position",
    }

    pipeline fill_pipeline {
        vbo: gfx::VertexBuffer<GpuFillVertex> = (),
        out_color: gfx::RenderTarget<ColorFormat> = "out_color",
    }
}

// A very simple vertex constructor that only outputs the vertex position
struct VertexCtor;
impl VertexConstructor<tessellation::FillVertex, GpuFillVertex> for VertexCtor {
    fn new_vertex(&mut self, vertex: tessellation::FillVertex) -> GpuFillVertex {
        assert!(!vertex.position.x.is_nan());
        assert!(!vertex.position.y.is_nan());
        GpuFillVertex {
            // (ugly hack) tweak the vertext position so that the logo fits roughly
            // within the (-1.0, 1.0) range.
            position: (vertex.position * 0.0145 - vector(1.0, 1.0)).to_array(),
        }
    }
}

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
pub unsafe extern fn  ANativeActivity_onCreate(app: *mut (), ud: *mut (), udsize: usize) {
	use android_logger::Filter;
	android_logger::init_once(Filter::default().with_min_level(Level::Trace));
	trace!("winit>>ANativeActivity_onCreate!!!!");
	android_injected_glue::android_main2(app as *mut _, move |c, v| try_lyon(c, v));
	trace!("android_main2执行完毕.");
}

#[no_mangle]
pub fn try_lyon(_argc: isize, _char: *const *const u8){
	// Build a Path for the rust logo.
    let mut builder = SvgPathBuilder::new(Path::builder());
    build_logo_path(&mut builder);
    let path = builder.build();

    let mut tessellator = FillTessellator::new();

    let mut mesh: VertexBuffers<GpuFillVertex, u16> = VertexBuffers::new();

    tessellator.tessellate_path(
        path.path_iter(),
        &FillOptions::tolerance(0.01),
        &mut BuffersBuilder::new(&mut mesh, VertexCtor),
    ).unwrap();

	trace!(" -- fill: {} vertices {} indices", mesh.vertices.len(), mesh.indices.len());

	// Initialize glutin and gfx-rs (refer to gfx-rs examples for more details).
	trace!("try 001");
    //let mut events_loop = glutin::EventsLoop::new();
	trace!("try 002");
	// use glutin::dpi::LogicalSize;
    // let glutin_builder = glutin::WindowBuilder::new()
    //     .with_dimensions(LogicalSize::new(700.0, 700.0))
    //     .with_decorations(true)
    //     .with_title("Simple tessellation".to_string());

	let mut events_loop = winit::EventsLoop::new();

	let glutin_builder = winit::WindowBuilder::new();
	trace!("try 003");

	let context = glutin::ContextBuilder::new()
	.with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGlEs, (3, 0)));
	trace!("try 004");
	let (window, mut device, mut factory, mut main_fbo, mut main_depth) =
		gfx_window_glutin::init::<ColorFormat, DepthFormat>(glutin_builder, context, &events_loop);
	trace!("try 005");
}

#[no_mangle]
pub fn main(_argc: isize, _char: *const *const u8){
	trace!("winit>>main!!!!");

	use std::thread;

	let _handler = thread::spawn(|| {
		trace!("winit>>线程启动");
		let mut events_loop = winit::EventsLoop::new();

		let _window = winit::WindowBuilder::new()
			.with_title("A fantastic window!")
			.build(&events_loop)
			.unwrap();

		use winit::{ WindowEvent, KeyboardInput, Event};
		
		events_loop.run_forever(|event| {
			trace!("winit>>{:?}", event);
			match event {
				Event::WindowEvent{event, ..} => {
					match event{
						WindowEvent::CloseRequested => winit::ControlFlow::Break,
						// WindowEvent::Focused => {
							
						// 	winit::ControlFlow::Continue
						// }
						_ => winit::ControlFlow::Continue,
					}
				}
				_ => winit::ControlFlow::Continue,
			}
		});
	});
	trace!("线程启动完毕.");
}