
#参考 https://blog.csdn.net/u010593680/article/details/41410289

Surface转换成ANativeWindow:

`rust
let window = ANativeWindow_fromSurface(env.get_native_interface(), surface);

`

Bitmap转换成SDL2 Canvas:

`rust

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

`