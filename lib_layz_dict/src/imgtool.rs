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

//获取rgb图片区域
pub fn get_rect<'a>(buffer: &[u8], width: usize, rect:&Rect, pixel_size: usize) -> Result<Vec<u8>, &'a str>{
    let mut result = vec![];
    let stride = width*pixel_size;
    //先取匹配裁剪区的所有行
    let start = stride*(rect.top-1);
    let end = start + stride*rect.height;
    match buffer.get(start..end){
        Some(lines) => {
            for row in lines.chunks(stride){//每一行
                //复制每一行的裁剪区域
                match row.get((rect.left-1)*pixel_size..(rect.left-1)*pixel_size+rect.width*pixel_size){
                    Some(chunk) =>{
                        if chunk.len() == rect.width*pixel_size{
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

    if result.len() != rect.width*rect.height*pixel_size{
        return Err("get_rect失败 04!")
    }else{
        Ok(result)
    }
}