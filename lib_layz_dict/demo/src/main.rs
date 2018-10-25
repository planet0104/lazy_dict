extern crate image;
use std::time::{Duration, Instant};
mod retina;
use image::{ImageBuffer, Rgb};

/// 双峰直方图计算阈值
/// 
/// 参考文献: 梁华为.直接从双峰直方图确定二值化阈值 https://wenku.baidu.com/view/df7da2d5b14e852458fb57b5.html
///
/// # Params
/// - `pixels`: 图像数据
/// - `bpp`: 每个像素占用字节 
fn calc_threshold(pixels: &[u8], bpp: usize) -> u8{
    //统计灰度直方图数据
    let mut gray_count = vec![0; 256];
    let mut gray_sum = 0;
    let pixel_count = pixels.len()/bpp;//总像素数
    let mut gray_values = vec![0; pixel_count];
    //循环每个像素统计灰度总和和灰度平均值
    for (i, pixel) in pixels.chunks(bpp).enumerate(){
        let gray =  (77*(pixel[0] as usize) + 150*(pixel[1] as usize) + 29*(pixel[2] as usize)+ 128) >> 8;
        gray_count[gray] += 1;
        gray_sum += gray;
        gray_values[i] = gray as i32;
    }

    // (1) 计算图像灰度平均值、标准偏差(标准差)sigma
    let avg = (gray_sum as f32/pixel_count as f32) as i32;//计算灰度平均值
    let sigma = {//计算标准差
        //方差和
        let total:i32 = gray_values.iter().map(|v|{ (*v-avg)*(*v-avg) }).sum();
        (total as f32/pixel_count as f32).sqrt()//求出标准差
    };

    // (2) 以像素平均值为分界点，分别求出左、右部分的最大值的位置
    let mut left_max_pos = gray_count.get(0..avg as usize).unwrap().iter().enumerate().max_by(|(_, v1), (_, v2)|{ v1.cmp(v2) }).unwrap().0;
    let mut right_max_pos = gray_count.get(avg as usize..gray_count.len()).unwrap().iter().enumerate().max_by(|(_, v1), (_, v2)|{ v1.cmp(v2) }).unwrap().0+avg as usize;

    // (3) 若两峰位置相距较近(在标准偏差范围内)，说明该直方图的双峰中有一个峰很低，因此需要另寻低峰的位置，否则至第(7)步
    let dist = (right_max_pos as isize-left_max_pos as isize).abs(); //位置距离
    if dist<sigma as isize{//另寻低峰
        // (4) 求出像素灰度中值点位置
        gray_values.sort();//排序并取中间值
        let mid_value = gray_values[gray_values.len()/2];

        // (5) 如果midpos>avg表明小峰在大峰左边(较低灰度级)；否则，表明小峰在大峰右边(较高灰度级)，相应调整分界点位置
        // (6) 重新求出大、小峰的位置
        if mid_value>avg{//小峰在大峰左边
            //从原先左峰往左边移动sigma距离，寻找最高峰
            left_max_pos = gray_count.get(0..(left_max_pos-sigma as usize)).unwrap().iter().enumerate().max_by(|(_, v1), (_, v2)|{ v1.cmp(v2) }).unwrap().0;
        }else{//小峰在大峰右边
            //从原先右峰往右边移动sigma距离，寻找最高峰
            right_max_pos = gray_count.get((right_max_pos+sigma as usize)..gray_count.len()).unwrap().iter().enumerate().max_by(|(_, v1), (_, v2)|{ v1.cmp(v2) }).unwrap().0+(right_max_pos+sigma as usize);
        }
        (left_max_pos + (right_max_pos-left_max_pos)/2) as u8
    }else{
        // (7) 以两峰位置的中点做为阈值
        (left_max_pos + (right_max_pos-left_max_pos)/2) as u8
    }
}


fn main(){
    // let img = image::open("lena.jpg").unwrap().to_rgb();
    // let img = image::open("qq.jpg").unwrap().to_rgb();
    // let img = image::open("jt.png").unwrap().to_rgb();
    // let img = image::open("shou.png").unwrap().to_rgb();
    // let img = image::open("s.png").unwrap().to_rgb();
    // let img = image::open("aa.jpeg").unwrap().to_rgb();
    let img = image::open("bb.jpg").unwrap().to_rgb();
    let (width, height) = (img.width() as usize, img.height() as usize);
    let pixels = img.into_raw();

    let now = Instant::now();
    
    let t = calc_threshold(&pixels, 3);

    println!("耗时{}ms 阈值:{}", duration_to_milis(&now.elapsed()), t);

    //--------------------------------- 灰度直方图
    // let height = 100;
    // let width = 256;
    // let mut gray_bar = vec![255; width*height*3];
    // let max = *gray_count.iter().max().unwrap();
    // for (y, row) in gray_bar.chunks_mut(width*3).enumerate(){
    //     for (x, pixel) in row.chunks_mut(3).enumerate(){
    //         let h = ((gray_count[x] as f32/max as f32)*height as f32) as usize;
    //         if h>=(height-y){
    //             pixel[0] = 0;
    //             pixel[1] = 0;
    //             pixel[2] = 0;
    //         }
    //     }
    // }

    //println!("{:?} max={}", gray_count, max);

    //let newimg:ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(new_width as u32, new_height as u32, dst).unwrap();
    // let newimg:ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width as u32, height as u32, gray_bar).unwrap();
    // println!("width={}", newimg.width());
    // newimg.save("new.png").unwrap();
    return;

    
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