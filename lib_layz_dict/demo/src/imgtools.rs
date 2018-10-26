/// 双峰直方图计算阈值
/// 
/// 参考文献: 梁华为.直接从双峰直方图确定二值化阈值 https://wenku.baidu.com/view/df7da2d5b14e852458fb57b5.html
///
/// # Params
/// - `pixels`: 图像数据
/// - `bpp`: 每个像素占用字节 
/// 
/// # Return
/// - (u8, Vec<u8>) (阈值, 每个像素对应灰度)
pub fn calc_threshold(pixels: &[u8], bpp: usize) -> (u8, Vec<u8>){
    //统计灰度直方图数据
    let mut gray_count = vec![0; 256];
    let mut gray_sum = 0;
    let pixel_count = pixels.len()/bpp;//总像素数
    let mut gray_values = vec![0u8; pixel_count];
    //循环每个像素统计灰度总和和灰度平均值
    for (i, pixel) in pixels.chunks(bpp).enumerate(){
        let gray =  (77*(pixel[0] as usize) + 150*(pixel[1] as usize) + 29*(pixel[2] as usize)+ 128) >> 8;
        gray_count[gray] += 1;
        gray_sum += gray;
        gray_values[i] = gray as u8;
    }

    // (1) 计算图像灰度平均值、标准偏差(标准差)sigma
    let avg = (gray_sum as f32/pixel_count as f32) as i32;//计算灰度平均值
    let sigma = {//计算标准差
        //方差和
        let total:i32 = gray_values.iter().map(|v|{ (*v as i32-avg)*(*v as i32-avg) }).sum();
        (total as f32/pixel_count as f32).sqrt()//求出标准差
    };

    // (2) 以像素平均值为分界点，分别求出左、右部分的最大值的位置
    let mut left_max_pos = gray_count.get(0..avg as usize).unwrap().iter().enumerate().max_by(|(_, v1), (_, v2)|{ v1.cmp(v2) }).unwrap().0;
    let mut right_max_pos = gray_count.get(avg as usize..gray_count.len()).unwrap().iter().enumerate().max_by(|(_, v1), (_, v2)|{ v1.cmp(v2) }).unwrap().0+avg as usize;

    // (3) 若两峰位置相距较近(在标准偏差范围内)，说明该直方图的双峰中有一个峰很低，因此需要另寻低峰的位置，否则至第(7)步
    let dist = (right_max_pos as isize-left_max_pos as isize).abs(); //位置距离
    if dist<sigma as isize{//另寻低峰
        // (4) 求出像素灰度中值点位置
        let mut sort_gray_values = gray_values.clone();
        sort_gray_values.sort();//排序并取中间值
        let mid_value = sort_gray_values[sort_gray_values.len()/2];

        // (5) 如果midpos>avg表明小峰在大峰左边(较低灰度级)；否则，表明小峰在大峰右边(较高灰度级)，相应调整分界点位置
        // (6) 重新求出大、小峰的位置
        if mid_value as i32>avg{//小峰在大峰左边
            //从原先左峰往左边移动sigma距离，寻找最高峰
            left_max_pos = gray_count.get(0..(left_max_pos-sigma as usize)).unwrap().iter().enumerate().max_by(|(_, v1), (_, v2)|{ v1.cmp(v2) }).unwrap().0;
        }else{//小峰在大峰右边
            //从原先右峰往右边移动sigma距离，寻找最高峰
            right_max_pos = gray_count.get((right_max_pos+sigma as usize)..gray_count.len()).unwrap().iter().enumerate().max_by(|(_, v1), (_, v2)|{ v1.cmp(v2) }).unwrap().0+(right_max_pos+sigma as usize);
        }
        ((left_max_pos + (right_max_pos-left_max_pos)/2) as u8, gray_values)
    }else{
        // (7) 以两峰位置的中点做为阈值
        ((left_max_pos + (right_max_pos-left_max_pos)/2) as u8, gray_values)
    }
}

/// 灰度图像边缘检测，填充为白底黑前景的RGB/RGBA图
///
/// # Params
///
/// - `gray_values`: 每个像素对应的灰度值
/// - `output`: 设置像素对应为黑值[u16]
/// - `bpl`: 每行占用字节
/// - `bpp`: 每个像素占用字节
/// - `thresholds`: 阈值
pub fn edge_detect_gray(gray_values:&[u8], output:&mut [u16], width: usize, threshold:u8){
    // 检测格式:
    //
    //     B H
    //     H
    //
    // B 当前像素 Bipolar cell 双极细胞
    // H 周围像素 Horizontal cell 水平细胞    
    let pixels_count = gray_values.len();
    for i in 0..pixels_count{
        let hrid = i+1;//右边一个像素[水平细胞]的位置(向后偏移一个像素)
        let hbid = i+width;//下边一个像素[水平细胞]位置(向后偏移一行)

        //当前像素[双极细胞]输出     -- 亮光兴奋，弱光抑制
        let b_out = if gray_values[i] >= threshold{ 1 }else{ -1 };
        
        //计算周围像素[水平细胞]输出  -- 亮光抑制，弱光兴奋
        if hrid<pixels_count && hbid < pixels_count{
            let hr_out = if gray_values[hrid] >= threshold{ -1 }else{ 1 };
            let hb_out = if gray_values[hbid] >= threshold{ -1 }else{ 1 };
            if b_out+b_out+hr_out+hb_out != 0{
                output[i] = 0;
            }
        }
    }
}

/// 图像二值化
///
/// # Params
///
/// - `gray_values`: 每个像素对应的灰度值
/// - `out`: 输出RGB/RGBA(背景为白色，前景为黑色)
/// - `bpp`: 每个像素占用字节
/// - `thresholds`: 阈值
pub fn binary(gray_values:&[u8], out:&mut [u8], bpp: usize, threshold:u8){
    for (i, pixel) in out.chunks_mut(bpp).enumerate(){
        if gray_values[i] >= threshold{
            pixel[0] = 0;
            pixel[1] = 0;
            pixel[2] = 0;
        }else{
            pixel[0] = 255;
            pixel[1] = 255;
            pixel[2] = 255;
        };
    }
}