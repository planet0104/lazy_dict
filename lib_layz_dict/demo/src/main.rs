// extern crate image;
// use std::time::{Duration, Instant};
// mod imgtools;
// use image::{Rgb, Rgba, ImageBuffer};
// extern crate imageproc;
extern crate base64;
extern crate sha1;
extern crate flate2;
extern crate md5;
use sha1::Sha1;

fn main(){
    // let img = image::open("img3.jpg").unwrap().to_rgba();
    // let (width, height) = (img.width() as usize, img.height() as usize);
    // let mut pixels = img.into_raw();

    // let now = Instant::now();

    // let bpp = 4;
    // //计算阈值和像素灰度值
    // let (threshold, gray_values) = imgtools::calc_threshold(&pixels, bpp);
    // //将原图像二值化
    // imgtools::binary(&gray_values, &mut pixels, bpp, 72);
    // //边缘检测
    // let mut edges = vec![1; width*height]; //1为背景, 0为边缘
    // imgtools::edge_detect_gray(&gray_values, &mut edges, width, threshold);

    // //保存边缘图
    // let mut bimg:ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width as u32, height as u32);
    // let mut i = 0;
    // for y in 0..height{
    //     for x in 0..width{
    //         if edges[i] == 0{
    //             bimg.put_pixel(x as u32, y as u32, Rgb([255u8, 0u8, 0u8]));
    //         }
    //         i += 1;
    //     }
    // }
    // let _ = bimg.save("edge.png");

    // // (170,102)
    

    // //根据edges分割
    // let rects = imgtools::split(0, 0, &mut edges, width, height);

    // println!("分割耗时 {}ms", duration_to_milis(&now.elapsed()));
    // println!("图片大小: {}x{}", width, height);
    // let mut output:ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(width as u32, height as u32, pixels).unwrap();
    // for rect in rects{
    //     imageproc::drawing::draw_hollow_rect_mut(&mut output, imageproc::rect::Rect::at(rect.left as i32, rect.top as i32).of_size(rect.width as u32, rect.height as u32), Rgba([255u8, 0u8, 0u8, 120u8]));
    // }
    // output.save("output.png").unwrap();

    println!("{:?}", encode("cn.jy.lazydict"));

    //生产
    println!("{:?}", encode("geeLNvX1mz4R50B3U8aqC94mARk="));//manifest.xml
    println!("{:?}", encode("+N17v9bZyVNKwgm8ulfdLwhtUfs="));//classex.dex

    //测试
    // println!("{:?}", encode("FPqob2khsJo4dFrzO5vsh08LRG4="));
    // println!("{:?}", encode("mbFnaSVudjxyHEAUkjlJGO9b2tY="));

    println!("{:?}", encode("3lvi1WeiZGQK9h4KC8lL1uY/rlA="));//camera.xml
    println!("{:?}", encode("/OiNSRnvMAJFnEj9jOeDH0nYqac="));//splash.xml
}

fn encode(s:&str) -> [u8; 16]{
    let base64 = base64::encode(s);
    let mut hasher = Sha1::new();
    hasher.update( base64.as_bytes() );
    let mut sha1 = hasher.digest().bytes();
    sha1.rotate_left(13);
    [
        sha1[2],sha1[3],sha1[4],sha1[5],
        sha1[6],sha1[7],sha1[8],sha1[9],
        sha1[10],sha1[11],sha1[12],sha1[13],
        sha1[14],sha1[15],sha1[16],sha1[17],
    ]
}

// pub fn duration_to_milis(duration: &Duration) -> f64 {
//     duration.as_secs() as f64 * 1000.0 + duration.subsec_nanos() as f64 / 1_000_000.0
// }