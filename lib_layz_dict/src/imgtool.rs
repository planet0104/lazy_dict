use rayon;

// This value is 2 ^ 18 - 1, and is used to clamp the RGB values before their
// ranges
// are normalized to eight bits.
const K_MAX_CHANNEL_VALUE:i32 = 262143;
///https://github.com/xvolica/Camera2-Yuv2Rgb/blob/master/jni/yuv2rgb.cpp
fn yuv_to_rgb(mut y:i32, mut u:i32, mut v:i32) -> (u8,u8,u8){
	use std::cmp::{min, max};

	y -= 16;
	u -= 128;
	v -= 128;
	if y<0{
		y=0;
	}

	let mut r = 1192 * y + 1634 * v;
	let mut g = 1192 * y - 833 * v - 400 * u;
	let mut b = 1192 * y + 2066 * u;

	r = min(K_MAX_CHANNEL_VALUE, max(0, r));
	g = min(K_MAX_CHANNEL_VALUE, max(0, g));
	b = min(K_MAX_CHANNEL_VALUE, max(0, b));

	r = (r>>10) & 0xff;
	g = (g>>10) & 0xff;
	b = (b>>10) & 0xff;

	//0xff000000 | (r as u32) << 24 | (g as u32) << 16 | (b as u32)<<8
	(r as u8, g as u8, b as u8)
}

pub fn yuv_420_to_rgb_888(y_data: &[u8], u_data: &[u8], v_data: &[u8], output:&mut[u8], width: i32, height:i32, y_row_stride: i32, uv_row_stride:i32, uv_pixel_stride:i32){

	let multi_thread = false;

	if multi_thread{ //多线程 平均5ms (图片640x480)
		//每行一个线程执行
		// rayon::scope(|s| {
		// 	for (j, output_slice) in output.chunks_mut(width as usize*3).enumerate() {
		// 		s.spawn(move |_| {
		// 			let y = j as i32;
        //https://crates.io/crates/image2
		// 			let iy = y_row_stride*y;
		// 			let uv_row_start = uv_row_stride*(y>>1);
		// 			let iu = uv_row_start;
		// 			let iv = uv_row_start;
		// 			let mut iout = 0;
		// 			for x in 0..width{
		// 				let uv_offset = (x>>1)*uv_pixel_stride;
		// 				let (r, g, b) = yuv_to_rgb(y_data[(iy+x) as usize] as i32, u_data[(iu+uv_offset) as usize] as i32, v_data[(iv+uv_offset) as usize] as i32);
		// 				output_slice[iout] = r; iout+=1;
		// 				output_slice[iout] = g; iout+=1;
		// 				output_slice[iout] = b; iout+=1;
		// 			}
		// 		});
		// 	}
		// });

		rayon::scope(|s| {
			let mut i = 0;
			//检查height的整除数 分成4个以内线程去执行
			let mut step = 1;
			for h in (2..5).rev().step_by(2){
				if height%h==0{
					step = (height/h) as usize;
					break;
				}
			}
			for (j, output_slice) in output.chunks_mut(width as usize*3*step).enumerate() {
				s.spawn(move |_| {
					let mut iout = 0;
					for ayi in 0..step{
						let y = (j+i+ayi) as i32;
						let iy = y_row_stride*y;
						let uv_row_start = uv_row_stride*(y>>1);
						let iu = uv_row_start;
						let iv = uv_row_start;
						for x in 0..width{
							let uv_offset = (x>>1)*uv_pixel_stride;
							let (r, g, b) = yuv_to_rgb(y_data[(iy+x) as usize] as i32, u_data[(iu+uv_offset) as usize] as i32, v_data[(iv+uv_offset) as usize] as i32);
							output_slice[iout] = r; iout+=1;
							output_slice[iout] = g; iout+=1;
							output_slice[iout] = b; iout+=1;
						}
					}
				});
				i += step-1;
			}
		});
	}else{
		//单线程 约11ms
		let mut iout = 0;
		for y in 0..height{
			let iy = y_row_stride*y;
			let uv_row_start = uv_row_stride*(y>>1);
			let iu = uv_row_start;
			let iv = uv_row_start;
			for x in 0..width{
				let uv_offset = (x>>1)*uv_pixel_stride;
				let (r, g, b) = yuv_to_rgb(y_data[(iy+x) as usize] as i32, u_data[(iu+uv_offset) as usize] as i32, v_data[(iv+uv_offset) as usize] as i32);
				output[iout] = r; iout+=1;
				output[iout] = g; iout+=1;
				output[iout] = b; iout+=1;
			}
		}
	}
}

/*
//Method from Ketai project! Not mine! See below...
    void decodeYUV420SP(int[] rgb, byte[] yuv420sp, int width, int height) {
        final int frameSize = width * height;
        for (int j = 0, yp = 0; j < height; j++) {       int uvp = frameSize + (j >> 1) * width, u = 0, v = 0;
            for (int i = 0; i < width; i++, yp++) {
                int y = (0xff & ((int) yuv420sp[yp])) - 16;
                if (y < 0)
                    y = 0;
                if ((i & 1) == 0) {
                    v = (0xff & yuv420sp[uvp++]) - 128;
                    u = (0xff & yuv420sp[uvp++]) - 128;
                }

                int y1192 = 1192 * y;
                int r = (y1192 + 1634 * v);
                int g = (y1192 - 833 * v - 400 * u);
                int b = (y1192 + 2066 * u);

                if (r < 0)                  r = 0;               else if (r > 262143)
                    r = 262143;
                if (g < 0)                  g = 0;               else if (g > 262143)
                    g = 262143;
                if (b < 0)                  b = 0;               else if (b > 262143)
                    b = 262143;

                rgb[yp] = 0xff000000 | ((r << 6) & 0xff0000) | ((g >> 2) & 0xff00) | ((b >> 10) & 0xff);
            }
        }
    }
 */
/// android: YUV420SP 转 rgb
pub fn decode_yuv420sp(rgb:&mut [i32], data:&[u8], width:i32, height:i32){
    let frame_size = width * height;
    let mut yp = 0;
    for j in 0..height{
        let (mut uvp, mut u, mut v) = ((frame_size + (j >> 1) * width) as usize, 0, 0);
        for i in 0..width{
            let mut y = (0xff & data[yp] as i32) - 16;  
            if y < 0 { y = 0; }
            if i & 1 == 0{
                v = (0xff & data[uvp] as i32) - 128;
                uvp += 1;
                u = (0xff & data[uvp] as i32) - 128;  
                uvp += 1;
            }

            let y1192 = 1192 * y;  
            let mut r = y1192 + 1634 * v;
            let mut g = y1192 - 833 * v - 400 * u;
            let mut b = y1192 + 2066 * u;

            if r < 0 { r = 0; } else if r > 262143 { r = 262143; };
            if g < 0 { g = 0; } else if g > 262143 { g = 262143; }
            if b < 0 { b = 0;} else if b > 262143 { b = 262143; }

            rgb[yp] = 0xff000000u32 as i32 | ((r << 6) & 0xff0000) | ((g >> 2) & 0xff00) | ((b >> 10) & 0xff);
            yp += 1;
        }
    }
}

//顺时针旋转90度
pub fn rotate90_colors(src_buffer: &[i32], new_buffer:&mut [i32], src_width: i32, src_height:i32) -> (i32, i32){
    let (src_width, src_height) = (src_width as usize, src_height as usize);
    let (new_width, new_height) = (src_height, src_width);
    for (sy, row) in src_buffer.chunks(src_width).enumerate(){
        //tx = src_height-sy-1;
        //ty = sx;
        let tx = src_height-sy-1;
        for sx in 0..row.len(){
            new_buffer[sx*new_width+tx] = row[sx];
        }
    }
    (new_width as i32, new_height as i32)
}

//顺时针旋转180度
pub fn rotate180_colors(src_buffer:&[i32], new_buffer:&mut [i32], width: i32, height: i32) -> (i32, i32){
    let (width, height) = (width as usize, height as usize);
    let mut p = src_buffer.len()-1;
    for row in src_buffer.chunks(width){
        for pixel in 0..row.len(){
            new_buffer[p] = row[pixel];
            p -= 1;
        }
    }
    (width as i32, height as i32)
}

//顺时针旋转270度
pub fn rotate270_colors(src_buffer: &[i32], new_buffer:&mut [i32], src_width: i32, src_height:i32) -> (i32, i32){
    let (src_width, src_height) = (src_width as usize, src_height as usize);
    let (new_width, new_height) = (src_height, src_width);
    for (y, row) in src_buffer.chunks(src_width).enumerate(){//每一行
        let j = y;
        for x in 0..row.len(){//每一列
            let p = (src_width-x-1)*new_width+j;
            new_buffer[p] = row[x];
        }
    }
    (new_width as i32, new_height as i32)
}

//RGB顺时针旋转90度
pub fn rotate90(src_buffer: &[u8], new_buffer:&mut [u8], src_width: usize, src_height:usize) -> (usize, usize){
    let (new_width, new_height) = (src_height, src_width);
    for (y, row) in src_buffer.chunks(src_width*3).enumerate(){
        //tx = src_height-y-1;
        //ty = sx;
        let n = (src_height-y-1)*3;
        for (x, pixel) in row.chunks(3).enumerate(){
            let p = x*new_width*3+n;
            new_buffer[p] = pixel[0];
            new_buffer[p+1] = pixel[1];
            new_buffer[p+2] = pixel[2];
        }
    }
    (new_width, new_height)
}

//RGB顺时针旋转180度
pub fn rotate180(src_buffer:&[u8], new_buffer:&mut [u8], width: usize, height: usize) -> (usize, usize){
    let stride = width*3;
    let mut p = src_buffer.len()-1;
    for row in src_buffer.chunks(stride){
        for pixel in row.chunks(3){
            new_buffer[p-2] = pixel[0];
            new_buffer[p-1] = pixel[1];
            new_buffer[p] = pixel[2];
            p -= 3;
        }
    }
    (width, height)
}

//RGB顺时针旋转270度
pub fn rotate270(src_buffer: &[u8], new_buffer:&mut [u8], src_width: usize, src_height:usize) -> (usize, usize){
    let (new_width, new_height) = (src_height, src_width);
    let src_stride = src_width*3;
    let new_stride = new_width*3;
    for (y, row) in src_buffer.chunks(src_stride).enumerate(){//每一行
        let j = y*3;
        for (x, pixel) in row.chunks(3).enumerate(){//每一列
            let p = (src_width-x-1)*new_stride+j;
            new_buffer[p] = pixel[0];
            new_buffer[p+1] = pixel[1];
            new_buffer[p+2] = pixel[2];
        }
    }
    (new_width, new_height)
}

#[derive(Debug)]
pub struct Rect{
    pub left: usize,
    pub top: usize,
    pub width: usize,
    pub height: usize,
}
impl Rect{
    pub fn new(left:usize, top:usize, width:usize, height:usize) -> Rect{
        Rect{left, top, width, height}
    }
}

//绘制正方形
pub fn stroke_rect(buffer: &mut [u8], width: usize, rect:&Rect, color:&[u8], line_width:usize, pixel_size: usize) -> Result<(), String>{
    let w = line_width/2;
    //上边框
    let mut fill = Rect::new(rect.left-w, rect.top-w, rect.width+w*2, line_width);
    match fill_rect(buffer, width, &fill, color, pixel_size){
        Ok(()) => (),
        Err(err) => return Err(err)
    }
    //下边框
    fill.left = rect.left-w;
    fill.top = rect.top+rect.height-w;
    fill.width =  rect.width+w*2;
    fill.height = line_width;
    match fill_rect(buffer, width, &fill, color, pixel_size){
        Ok(()) => (),
        Err(err) => return Err(err)
    }
    //左边框
    fill.left = rect.left-w;
    fill.top = rect.top+w;
    fill.width =  line_width;
    fill.height = rect.height-w*2;
    match fill_rect(buffer, width, &fill, color, pixel_size){
        Ok(()) => (),
        Err(err) => return Err(err)
    }
    //右边框
    fill.left = rect.left+rect.width-w;
    fill.top = rect.top+w;
    fill.width =  line_width;
    fill.height = rect.height-w*2;
    match fill_rect(buffer, width, &fill, color, pixel_size){
        Ok(()) => (),
        Err(err) => return Err(err)
    }
    Ok(())
}

//填充正方形
pub fn fill_rect(buffer: &mut [u8], width: usize, rect:&Rect, color:&[u8], pixel_size: usize) -> Result<(), String>{
    let stride = width*pixel_size;
    //先取匹配裁剪区的所有行
    let start = stride*(rect.top-1);
    let end = start + stride*rect.height;
    match buffer.get_mut(start..end){
        Some(lines) => {
            for row in lines.chunks_mut(stride){//每一行
                //复制每一行的裁剪区域
                match row.get_mut((rect.left-1)*pixel_size..(rect.left-1)*pixel_size+rect.width*pixel_size){
                    Some(chunk) =>{
                        if chunk.len() == rect.width*pixel_size{
                            for pixel in chunk.chunks_mut(pixel_size){
                                for k in 0..pixel_size{
                                    pixel[k] = color[k]
                                }
                            }
                        }else{
                            return Err(String::from("fill_rect失败 01!"));
                        }
                    }
                    None => return Err(String::from("fill_rect失败 02!"))
                }
            }
        },
        None => return Err(String::from("fill_rect失败 03!"))
    }
    Ok(())
}

//获取rgb图片区域
pub fn get_argb_rect_rgb<'a>(buffer: &[u8], width: usize, rect:&Rect) -> Result<Vec<u8>, &'a str>{
    let mut result = vec![];
    let stride = width*3;
    //先取匹配裁剪区的所有行
    let start = stride*(rect.top-1);
    let end = start + stride*rect.height;
    match buffer.get(start..end){
        Some(lines) => {
            for row in lines.chunks(stride){//每一行
                //复制每一行的裁剪区域
                match row.get((rect.left-1)*3..(rect.left-1)*3+rect.width*3){
                    Some(chunk) =>{
                        if chunk.len() == rect.width*3{
                            //result.extend_from_slice(chunk);
                            for pixel in chunk.chunks(3){
                                result.push(pixel[0]);
                                result.push(pixel[1]);
                                result.push(pixel[2]);
                                result.push(255);
                            }
                        }else{
                            return Err("get_argb_rect_rgb失败 01!");
                        }
                    }
                    None => return Err("get_argb_rect_rgb失败 02!")
                }
            }
        },
        None => return Err("get_argb_rect_rgb失败 03!")
    }

    if result.len() != rect.width*rect.height*4{
        return Err("get_argb_rect_rgb失败 04!")
    }else{
        Ok(result)
    }
}

/// 获取灰度图区域
pub fn get_gray_rect<'a>(buffer: &[u8], width:i32, rect:&Rect) -> Result<Vec<u8>, &'a str>{
    let width = width as usize;
    let mut result = vec![];
    //先取匹配裁剪区的所有行
    let start = width*(rect.top-1);
    let end = start + width*rect.height;
    match buffer.get(start..end){
        Some(lines) => {
            for row in lines.chunks(width){//每一行
                //复制每一行的裁剪区域
                match row.get(rect.left-1..rect.left-1+rect.width){
                    Some(chunk) =>{
                        if chunk.len() == rect.width{
                            //debug!("chunk={:?}", chunk);
                            result.extend_from_slice(chunk);
                        }else{
                            return Err("get_rect失败 01!");
                        }
                    }
                    None => return Err("get_rect失败 02!")
                }
            }
        },
        None => return Err("get_rect失败 03!")
    }

    if result.len() != rect.width*rect.height{
        return Err("get_rect失败 04!")
    }else{
        Ok(result)
    }
}

/// 获取图片区域
pub fn get_rect<'a>(buffer: &[u8], bpp:usize, stride:u32, rect:&Rect) -> Result<Vec<u8>, &'a str>{
    let stride = stride as usize;
    let mut result = vec![];
    //先取匹配裁剪区的所有行
    let start = stride*(rect.top-1);
    let end = start + stride*rect.height;
    match buffer.get(start..end){
        Some(lines) => {
            for row in lines.chunks(stride){//每一行
                //复制每一行的裁剪区域
                match row.get((rect.left-1)*bpp..(rect.left-1)*bpp+rect.width*bpp){
                    Some(chunk) =>{
                        if chunk.len() == rect.width*bpp{
                            result.extend_from_slice(chunk);
                        }else{
                            return Err("get_rect失败 01!");
                        }
                    }
                    None => return Err("get_rect失败 02!")
                }
            }
        },
        None => return Err("get_rect失败 03!")
    }

    if result.len() != rect.width*rect.height*bpp{
        return Err("get_rect失败 04!")
    }else{
        Ok(result)
    }
}

/// 双峰直方图计算阈值
/// 
/// 参考文献: 梁华为.直接从双峰直方图确定二值化阈值 https://wenku.baidu.com/view/df7da2d5b14e852458fb57b5.html
///
/// # Params
/// - `pixels`: 图像数据
/// - `width`: 图片宽度
/// - `height`: 图片高度
/// - `stride`: 行跨度
/// - `bpp`: 每个像素占用字节(3或4 即 RGB/RGBA)
/// 
/// # Return
/// - (u8, Vec<u8>) (阈值, 每个像素对应灰度)
pub fn calc_threshold(pixels: &[u8], width:usize, height:usize, stride:usize, bpp: usize) -> (u8, Vec<u8>){
    //统计灰度直方图数据
    let mut gray_count = vec![0; 256];
    let mut gray_sum = 0;
    let pixel_count = width*height;//总像素数
    let mut gray_values = vec![0u8; pixel_count];
    //循环每个像素统计灰度总和和灰度平均值
    let mut i = 0;
    for row in pixels.chunks(stride*bpp){
        for pixel in row.get(0..width*bpp).unwrap().chunks(bpp){
            let gray =  (77*(pixel[0] as usize) + 150*(pixel[1] as usize) + 29*(pixel[2] as usize)+ 128) >> 8;
            gray_count[gray] += 1;
            gray_sum += gray;
            gray_values[i] = gray as u8;    
            i += 1;
        }
    }
    debug!("gray_count={:?}", gray_count);

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

#[derive(Debug)]
pub struct SplitInfo{
    pub left: usize,
    pub top: usize,
    pub width: usize,
    pub height: usize
}
impl SplitInfo{
    pub fn new(left:usize, top:usize, width: usize, height: usize) -> SplitInfo{
        SplitInfo{
            left, top, width, height
        }
    }
}

/// 分割图片过滤
/// Params
/// parent_left 父图像块在整张图片中的的left
/// parent_top 父图像块在整张图片中的的top
/// edges 检测到的边缘
/// width edges图片宽度
/// height edges图片高度
/// 返回:
/// 切块坐标信息数组
pub fn split(parent_left:usize, parent_top:usize, edges:&mut [u16], width: usize, height: usize) -> Vec<SplitInfo>{
    let infos = split_filter(parent_left, parent_top, edges, width, height);
    //过滤比例不对的方块(和正方形差距太远的)
    let mut new_infos = vec![];
    let mut total_area = 0;
    for ifo in &infos{
        if ifo.height as f32 / ifo.width as f32>1.6
            || ifo.width as f32 / ifo.height as f32>1.6{
            continue;
        }else{
            total_area += ifo.width*ifo.height;
            new_infos.push(SplitInfo::new(ifo.left, ifo.top, ifo.width, ifo.height));
        }
    }
    //面积和平均面积差距太大的过滤掉
    let infos = new_infos;
    let avg_area = total_area as f32/infos.len() as f32;
    let mut new_infos = vec![];
    for ifo in &infos{
        if ifo.height * ifo.width < (avg_area/1.5) as usize{
            continue;
        }else{
            new_infos.push(SplitInfo::new(ifo.left, ifo.top, ifo.width, ifo.height));
        }
    }
    new_infos
}

/// 分割图片
/// 
pub fn split_filter(parent_left:usize, parent_top:usize, edges:&mut [u16], width: usize, height: usize) -> Vec<SplitInfo>{
    let mut infos = vec![];

    //打通纵向连通率比较低的竖线
    // for x in 0..width{
    //     //垂直线扫描
    //     if let Some(slice) = edges.get_mut(x..width*height){//过滤无效
    //         let sum:u16 = slice.chunks(width).map(|slice| slice[0]).sum();
    //         if sum == (height as f32*0.95) as u16{
    //             slice.chunks_mut(width).for_each(|slice| slice[0] = 1 );
    //         }
    //     }
    // }

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
        return infos;
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
        //检查比例, 和像素数
        infos.push(SplitInfo::new(parent_left+left, parent_top+top, new_width, new_height));
    }else{
    //否则需要拆分
    // let r = random::<f64>();
    // println!("{}=>需要拆分:{:?}", r, SplitInfo::new(parent_left+left, parent_top+top, new_width, new_height));
        //补0和结尾
        horizontal_array.insert(0, 0);
        horizontal_array.push(new_height);
        vertical_array.insert(0, 0);
        vertical_array.push(new_width);

        //检查所有分割块
        let mut sub_infos = vec![];
        for y in 1..horizontal_array.len(){
            for x in 1..vertical_array.len(){
                let (split_left, split_top, split_right, split_bottom) = (vertical_array[x-1], horizontal_array[y-1], vertical_array[x], horizontal_array[y]);
                let split_width = split_right-split_left;
                let split_height = split_bottom-split_top;
                //获取字块像素
                let mut split_edges = vec![];
                //宽度xTOP+LEFT ~ 宽度x(BOTTOM-1)+RIGHT
                for slice in new_edges.get(new_width*split_top+split_left..new_width*(split_bottom-1)+split_right).unwrap().chunks(new_width){
                    //取宽度对应的数据
                    split_edges.extend_from_slice(slice.get(0..split_width).unwrap());
                }
                let sss = split_filter(parent_left+left+split_left, parent_top+top+split_top, &mut split_edges, split_width, split_height);
                //递归继续分割
                //println!("{:?}, len={}",sss, sss.len());
                if sss.len()>0{
                    sub_infos.push(sss);
                }
            }
        }

        //过滤不符合的分割模式
        //println!("{}=>拆分结果len={:?}", r, sub_infos.len());

        if
            //！、会、们: (一分二，有一个不合格)
            (sub_infos.len() == 2 && sub_infos[0].len() == 1 && sub_infos[1].len() ==1)//
            ||
            //杰: 1分1,4
            (sub_infos.len() == 2 && sub_infos[0].len() == 1 && sub_infos[1].len() ==4) ||
            //三: 1分3
            (sub_infos.len() == 3 && sub_infos[0].len() == 1 && sub_infos[1].len() ==1&& sub_infos[2].len() ==1)
            {
            let mut error = false;
            for s1 in &sub_infos{
                for ifo in s1{
                    if ifo.height as f32 / ifo.width as f32>1.6
                        || ifo.width as f32 / ifo.height as f32>1.6{
                        error = true;
                        break;
                    }
                }
                if error{
                    break;
                }
            }
            if error{
                infos.push(SplitInfo::new(parent_left+left, parent_top+top, new_width, new_height));
            }else{
                for ss in sub_infos{
                    infos.extend(ss);
                }    
            }
        }else{
            for ss in sub_infos{
                infos.extend(ss);
            }
        }
    }

    //最终过滤
    //如果有两个连续的比较窄的竖条，他们是同一个字，将两个竖条合成一个方块
    let l = infos.len();
    let mut i = 0;
    let mut new_infos = vec![];
    while i<l{
        let a = &infos[i];
        if i<l-1{
            let b = &infos[i+1];
            if a.height as f32 / a.width as f32>1.8
                && b.height as f32 / b.width as f32>1.8{
                new_infos.push(SplitInfo::new(a.left, a.top, b.left-a.left+b.width, b.top-a.top+b.height));
                i += 2;
                continue;
            }
        }
        new_infos.push(SplitInfo::new(a.left, a.top, a.width, a.height));
        i += 1;
    }
    new_infos
}


/*
//RGB565 转 Bitmap
#[no_mangle]
pub extern fn Java_cn_jy_lazydict_RustApp_convertRGB565<'a>(env: JNIEnv, _activity: JClass, data: jni::sys::jbyteArray, width:jint, height:jint) -> jni::sys::jintArray{

	let data = env.convert_byte_array(data).unwrap();
	debug!("width={}, height={}, data.len()={}", width, height, data.len());

	let data: &[u16] = unsafe {
		std::slice::from_raw_parts(
			data.as_ptr() as *const u16,
			data.len()/2,
		)
	};
	
	let mut colors = vec![0i32; (width*height) as usize];

	for i in 0..data.len(){
		/*
			1,
            UInt32 r = (rgb565 & 0xF800) >> 11;
            UInt32 g = (rgb565 & 0x07E0) >> 5;  
            UInt32 b = rgb565 & 0x001F;

            2,
            r = r << 3 | r >> 2;
            g = g << 2 | g >> 4;
            b = b << 3 | b >> 2;
		 */
		let r = (data[i] as u32 & 0xF800) >> 11;
        let g = (data[i] as u32 & 0x07E0) >> 5;
        let b = data[i] as u32 & 0x001F;
		let r = r<<3|r>>2;
		let g = g<<2|g>>4;
		let b = b<<3|b>>2;
		colors[i] = (255<<24|r<<16|g<<8|b) as i32;
	}

	let intarray = env.new_int_array(width*height).unwrap();
	env.set_int_array_region(intarray, 0, &colors).unwrap();

	intarray
}
 */