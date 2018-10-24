
extern crate image;
use std::time::{Duration, Instant};
mod retina;
use image::{ImageBuffer, Rgb};
fn main(){
    let img = image::open("qq.jpg").unwrap().to_rgb();
    let (width, height) = (img.width() as usize, img.height() as usize);
    let pixels = img.into_raw();

    let now = Instant::now();
    
    //统计灰度等级数量(画成图则为灰度直方图)
    let mut gray_count = vec![0; 256];
    for pixel in pixels.chunks(3){
        let gray = (pixel[0] as usize* 19595 + pixel[1] as usize* 38469 +  pixel[2] as usize* 7472) >> 16;
        gray_count[gray] += 1;
    }
    let mut max = 0;
    for g in &gray_count{
        if *g>max{
            max = *g;
        }
    }
    
    let mut out = vec![255; width*height*3];
    retina::edge_detect(width, 3, &pixels, &mut out, 63);
    retina::edge_detect(width, 3, &pixels, &mut out, 97);

    println!("耗时{}ms", duration_to_milis(&now.elapsed()));

    //分割算法
    //https://blog.csdn.net/zhongyunde/article/details/7840717
    

    let newimg:ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width as u32, height as u32, out).unwrap();
    
    println!("width={}", newimg.width());
    newimg.save("new.png").unwrap();
    
}

pub fn duration_to_milis(duration: &Duration) -> f64 {
    duration.as_secs() as f64 * 1000.0 + duration.subsec_nanos() as f64 / 1_000_000.0
}