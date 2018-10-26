extern crate image;
use std::time::{Duration, Instant};
mod imgtools;
use image::{GenericImage, Rgb, ImageBuffer};
extern crate imageproc;
extern crate resize;
use resize::Pixel::RGB24;
use resize::Type::*;

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

fn split(infos: &mut Vec<SplitInfo>, edges:&[u16], width: usize, height: usize){
    //println!("split>>> {},{},{},{}", left, top, width, height);

    //首先裁剪上下左右的黑白像素, 将 edges替换
    //去除上黑边
    let (mut left, mut top, mut right, mut bottom) = (0, 0, width, height);
    for row in edges.chunks(width){
        let sum:u16 = row.iter().sum();//全为1的是底线
        if sum == width as u16{
            top += 1;
        }else{
            //不是底线停止
            break;
        }
    }
    //去除下黑边
    for row in edges.chunks(width).rev(){
        let sum:u16 = row.iter().sum();
        if sum == width as u16{
            bottom -= 1;
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

    检查黑边去除是否正确！！！

    println!("split_info={:?}", split_info);
    let mut new_edges = vec![];//获取对应坐标区域的像素数据
    //宽度xTOP+LEFT 至 宽度x(TOP+HEIGHT-1)+LEFT
    let sub = edges.get(width*split_info.top+split_info.left..width*(split_info.top+split_info.height)+split_info.left);
    if sub.is_none(){
        return;
    }
    for slice in sub.unwrap().chunks(width){
        //检查每个slice和{ left: 31, top: 11, width: 106, height: 41 }区域是否对应
        //println!("{:?}", slice.get(0..split_info.width));
        new_edges.extend_from_slice(slice.get(0..split_info.width).unwrap());
    }
    //重新计算相关参数
    let (width, height) = (split_info.width, split_info.height); //当前检测图像的宽高
    let total_pixels = new_edges.len();
    let edges = new_edges;
    println!("total_pixels={}", width*height);
    //1为背景 0为边缘
    //------------------ 寻找横向分割线 ------------
    let mut ysplits:Vec<usize> = vec![]; //存储所有横向分割线的y坐标
    let mut ycount = 0;
    for (y, row) in edges.chunks(width).enumerate(){
        let sum:u16 = row.iter().sum();
        if sum == width as u16{//整行都是白色像素的为分割线
            //println!("横分割线: y={} width={}", y, width);
            if ycount == 0{
                ysplits.push(y);
                ycount += 1;
            }else{
                if ysplits[ycount-1]+1 == y{//不要连续的分割线
                    ysplits[ycount-1] = y;
                }else{
                    ysplits.push(y);
                    ycount += 1;
                }
            }
        }
    }
    //寻找纵向分割线
    let mut xsplits:Vec<usize> = vec![];
    let mut xcount = 0;
    for x in 0..width{
        let sum:u16 = edges.get(x..total_pixels).unwrap().chunks(width).map(|slice| slice[0]).sum();
        //println!("sum={} x={}", sum, x);
        if sum == height as u16{
            //println!("竖分割线: x={}", x);
            if xcount == 0{
                xsplits.push(x);
                xcount += 1;
            }else{
                if xsplits[xcount-1]+1 == x{//不要连续的分割线
                    xsplits[xcount-1] = x;
                }else{
                    xsplits.push(x);
                    xcount += 1;
                }
            }
        }
    }
    
    println!("ysplits={:?}", ysplits);
    println!("xsplits={:?}", xsplits);

    //无法分割说明找到分割快
    // if ysplits.len() == 0 && xsplits.len() == 0{
    //     split_info.left += parent_left;
    //     split_info.top += parent_top;
    //     infos.push(split_info);
    // }else{
    //     //println!("ysplits.len()={}, xsplits.len()={}", ysplits.len(), xsplits.len());
    //     //根据分割线交叉点，分割每一块
        
    //     let mut last_y = 0;
    //     ysplits.push(height-1);
    //     xsplits.push(width-1);
    //     for y in ysplits{
    //         let mut last_x = 0;
    //         for x in &xsplits{
    //             let (left, top, block_width, block_height) = (last_x, last_y, x-last_x, y-last_y);
    //            //println!("left:{}, top:{}, width:{}, height:{}", left, top, block_width, block_height);
    //             if block_width==1 || block_height==1{
    //                 continue;
    //             }
    //             println!("left:{}, top:{}, width:{}, height:{}", left, top, block_width, block_height);
    //             //let split_info = SplitInfo::new(left, top, block_height, block_height);
                
    //             //let rect = imageproc::rect::Rect::at(left as i32, top as i32).of_size(block_width as u32, block_height as u32);
    //            // imageproc::drawing::dhollow_rect_mut(image, rect, Rgb([255u8, 0u8, 0u8]));
    //             //image.save("output.png").unwrap();

    //             //get sub edges and split
    //             //println!("{:?}", edges);
                
    //             let mut sub_edges = vec![];//获取对应坐标区域的像素数据
    //             for slice in edges.get(width*last_y+last_x..width*y-(width-x)).unwrap().chunks(width){
    //                 sub_edges.extend_from_slice(slice.get(0..block_width).unwrap());
    //             }
    //             //计算像素数，如果像素数为空(全是黑色像素)，忽略
    //             //println!("{:?}", sub_edges);
    //             let pixel_count:u16 = sub_edges.iter().sum();
    //             // println!("像素数:{}  白色素数:{} block_width={}, block_height={}", sub_edges.len(), sub_edges.len()-pixel_count as usize, block_width, block_height);
    //             if !(pixel_count==sub_edges.len() as u16){
    //                 split(infos, &sub_edges, parent_left+left, parent_top+top, block_width, block_height);
    //             }else{
    //                 println!("全黑像素，忽略!");
    //             }
                
    //             last_x = *x;
    //         }
    //         last_y = y;
    //     }

    // }
    // (ysplits, xsplits)
}

fn main(){
    let img = image::open("img10.jpg").unwrap().to_rgb();
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

    println!("边缘检测{}ms", duration_to_milis(&now.elapsed())); let now = Instant::now();

    let mut output:ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width as u32, height as u32, pixels).unwrap();

    //根据edges分割
    let mut rects = vec![];
    split(&mut rects, &edges, width, height);

    for rect in rects{
        imageproc::drawing::draw_hollow_rect_mut(&mut output, imageproc::rect::Rect::at(rect.left as i32, rect.top as i32).of_size(rect.width as u32, rect.height as u32), Rgb([255u8, 0u8, 0u8]));
    }
    

    //let hlines = edges.chunks(width).enumerate(); 
    //let vlines = edges.chunks(); //纵向分割线
    

    /*
    图片类型
    1、检查图图片是否有横向、纵向贯穿线，如果有，那么按照贯穿先分割，直到不能再分割为止
    2、如果没有贯穿线，按照一整个文字识别

    */

    //切割黑色像素
    

    //绘制横向分割线
    println!("图片大小: {}x{}", width, height);
    // for y in ysplits{
    //     imageproc::drawing::dline_segment_mut(&mut output, (0.0, y as f32), (width as f32, y as f32), Rgb([255u8, 0u8, 0u8]));
    // }
    // for x in xsplits{
    //     imageproc::drawing::dline_segment_mut(&mut output, (x as f32, 0.0), (x as f32, height as f32), Rgb([255u8, 0u8, 0u8]));
    // }

    //切割结果处理: 根据切合图片的比例，正方形按照单个识别，横向长方形按照单行识别，纵向长方形不识别。

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