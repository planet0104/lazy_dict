extern crate image;
use std::time::{Duration, Instant};
mod imgtools;
use image::{Rgb, ImageBuffer};
extern crate imageproc;

#[derive(Debug)]
struct SplitInfo{
    left: usize,
    top: usize,
    width: usize,
    height: usize
}
impl SplitInfo{
    pub fn new(left:usize, top:usize, width: usize, height: usize) -> SplitInfo{
        SplitInfo{
            left, top, width, height
        }
    }
}

/// parent_left 父级
fn split(infos: &mut Vec<SplitInfo>, parent_left:usize, parent_top:usize, edges:&[u16], width: usize, height: usize){
    //println!("split>>> {},{},{},{}", left, top, width, height);

    //首先裁剪上下左右的黑白像素, 将 edges替换
    //去除上黑边
    let (mut left, mut top, mut right, mut bottom) = (0, 0, width, height);
    for (y, row) in edges.chunks(width).enumerate(){
        let sum:u16 = row.iter().sum();//全为1的是底线
        if sum == width as u16{
            top = y+1;
        }else{
            //不是底线停止
            break;
        }
    }
    //去除下黑边
    for (y, row) in edges.chunks(width).enumerate().rev(){
        let sum:u16 = row.iter().sum();
        if sum == width as u16{
            bottom = y;
        }else{
            break;
        }
    }
    //去除左黑边
    let tp = edges.len();//总长度
    for x in 0..width{
        let sum:u16 = edges.get(x..tp).unwrap().chunks(width).map(|slice| slice[0]).sum();
        if sum == height as u16{
            left += 1;
        }else{
            break;
        }
    }
    //去除右黑边
    for x in (0..width).rev(){
        let sum:u16 = edges.get(x..tp).unwrap().chunks(width).map(|slice| slice[0]).sum();
        if sum == height as u16{
            right -= 1;
        }else{
            break;
        }
    }

    //println!("{},{},{},{}", left, top, right, bottom);
    if right==0 || bottom==0{//无效检测
        return;
    }

    //获取裁剪区域的像素数据
    let mut new_edges = vec![];
    //宽度xTOP+LEFT ~ 宽度x(BOTTOM-1)+RIGHT
    for slice in edges.get(width*top+left..width*(bottom-1)+right).unwrap().chunks(width){
        //取宽度对应的数据
        //println!("{:?}", slice.get(0..(right-left)));
        new_edges.extend_from_slice(slice.get(0..(right-left)).unwrap());
    }

    let (new_width, new_height) = (right-left, bottom-top);

    
    //------------------ 水平分割线 ------------
    let mut horizontal_array:Vec<usize> = vec![];//存储y坐标
    let mut count = 0;
    for (y, row) in new_edges.chunks(new_width).enumerate(){
        let sum:u16 = row.iter().sum();
        if sum == new_width as u16{//整行都是白色像素的为分割线
            if count == 0{
                horizontal_array.push(y);
                count += 1;
            }else{
                if horizontal_array[count-1]+1 == y{//不要连续的分割线
                    horizontal_array[count-1] = y;
                }else{
                    horizontal_array.push(y);
                    count += 1;
                }
            }
        }
    }

    //寻找纵向分割线
    let mut vertical_array:Vec<usize> = vec![];//存储x坐标
    count = 0;
    let total_pixels = new_width*new_height;
    for x in 0..width{
        //垂直线扫描
        if let Some(slice) = new_edges.get(x..total_pixels){//过滤无效
            let sum:u16 = slice.chunks(new_width).map(|slice| slice[0]).sum();
            if sum == new_height as u16{
                //println!("竖分割线: x={}", x);
                if count == 0{
                    vertical_array.push(x);
                    count += 1;
                }else{
                    if vertical_array[count-1]+1 == x{//不要连续的分割线
                        vertical_array[count-1] = x;
                    }else{
                        vertical_array.push(x);
                        count += 1;
                    }
                }
            }
        }
    }

    //如果纵向和横向分割线为0，说明是单独的文字块了，返回
    if horizontal_array.len() == 0 && vertical_array.len() == 0{
        let info = SplitInfo::new(parent_left+left, parent_top+top, new_width, new_height);
        // println!("方块 {:?}", info);
        infos.push(info);
    }else{
    //否则需要拆分
        //补0和结尾
        // println!("补之前: {:?}", horizontal_array);
        // println!("补之前: {:?}", vertical_array);
        horizontal_array.insert(0, 0);
        horizontal_array.push(new_height);
        vertical_array.insert(0, 0);
        vertical_array.push(new_width);

        // println!("{:?}", horizontal_array);
        // println!("{:?}", vertical_array);

        这里拆分之前，要检测是否过度拆分， “杰、们、会”等字不应该再次拆分
        至于标点逗号，能否根据平均宽高来过滤？
        至于半个字，也要过滤掉

        for y in 1..horizontal_array.len(){
            for x in 1..vertical_array.len(){
                let (split_left, split_top, split_right, split_bottom) = (vertical_array[x-1], horizontal_array[y-1], vertical_array[x], horizontal_array[y]);
                let split_width = split_right-split_left;
                let split_height = split_bottom-split_top;
                // println!("{},{},{},{}", split_left, split_top, split_width, split_height);
                //获取字块像素
                let mut split_edges = vec![];
                //宽度xTOP+LEFT ~ 宽度x(BOTTOM-1)+RIGHT
                for slice in new_edges.get(new_width*split_top+split_left..new_width*(split_bottom-1)+split_right).unwrap().chunks(new_width){
                    //取宽度对应的数据
                    //println!("{:?}", slice.get(0..split_width));
                    split_edges.extend_from_slice(slice.get(0..split_width).unwrap());
                }
                //递归继续分割
                split(infos, parent_left+left+split_left, parent_top+top+split_top, &split_edges, split_width, split_height);
            }
        }
    }
}

fn main(){
    let img = image::open("img08.jpg").unwrap().to_rgb();
    let (width, height) = (img.width() as usize, img.height() as usize);
    let mut pixels = img.into_raw();

    let now = Instant::now();

    let bpp = 3;

    //计算阈值和像素灰度值
    let (threshold, gray_values) = imgtools::calc_threshold(&pixels, bpp);
    println!("计算阈值{}ms 阈值:{}", duration_to_milis(&now.elapsed()), threshold); let now = Instant::now();
    //将原图像二值化
    imgtools::binary(&gray_values, &mut pixels, bpp, threshold);
    println!("二值化{}ms", duration_to_milis(&now.elapsed())); let now = Instant::now();

    //边缘检测
    let mut edges = vec![1; width*height]; //1为背景, 0为边缘
    imgtools::edge_detect_gray(&gray_values, &mut edges, width, threshold);

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

    println!("边缘检测{}ms", duration_to_milis(&now.elapsed())); let now = Instant::now();

    let mut output:ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width as u32, height as u32, pixels).unwrap();

    //根据edges分割
    let mut rects = vec![];
    split(&mut rects, 0, 0, &edges, width, height);

    for rect in rects{
        imageproc::drawing::draw_hollow_rect_mut(&mut output, imageproc::rect::Rect::at(rect.left as i32, rect.top as i32).of_size(rect.width as u32, rect.height as u32), Rgb([255u8, 0u8, 0u8]));
    }

    println!("图片大小: {}x{}", width, height);
    println!("分割耗时 {}ms", duration_to_milis(&now.elapsed()));

    //分割算法
    //https://blog.csdn.net/zhongyunde/article/details/7840717
    //http://www.danvk.org/2015/01/07/finding-blocks-of-text-in-an-image-using-python-opencv-and-numpy.html
    //http://www.danvk.org/2015/01/09/extracting-text-from-an-image-using-ocropus.html
    
    
    output.save("output.png").unwrap();
}

pub fn duration_to_milis(duration: &Duration) -> f64 {
    duration.as_secs() as f64 * 1000.0 + duration.subsec_nanos() as f64 / 1_000_000.0
}