#![allow(dead_code)]
use std::fs::File;
use std::io::Read;
use zip;
use std::io::{Result, Error, ErrorKind};
use rayon;

// 获取包名
pub fn get_package_name() -> String{
    match File::open("/proc/self/cmdline"){
        Ok(mut file) => {
            let mut contents = String::new();
            match file.read_to_string(&mut contents){
                Ok(_count) => String::from(contents.trim_matches(char::from(0))),
                Err(err) =>{
                    trace!("get_package_name>>{:?}", err);
                    String::new()
                }
            }
        }
        Err(err) => {
            trace!("get_package_name>>{:?}", err);
            String::new()
        }
    }
}

//获取apk路径
pub fn get_apk_file_path() -> String{
    let package = get_package_name();
    match File::open("/proc/self/maps"){
        Ok(mut file) => {
            let mut contents = String::new();
            match file.read_to_string(&mut contents){
                Ok(_count) =>{
                    let mut apk_file = "";
                    for line in contents.lines(){
                        if line.contains(".apk") && line.contains(&package){
                            for p in line.split(" "){
                                if p.ends_with("apk"){
                                    apk_file = p;
                                    break;
                                }
                            }
                        }
                    }
                    String::from(apk_file)
                }
                Err(err) =>{
                    error!("get_package_name>>{:?}", err);
                    String::new()
                }
            }
        }
        Err(err) => {
            error!("get_apk_file_path>>{:?}", err);
            String::new()
        }
    }
}

pub fn get_manifest_mf() -> String{
    match File::open(get_apk_file_path()){
        Ok(file) =>{
            trace!("开始解压文件..");
            let mut archive = zip::ZipArchive::new(file).unwrap();
            for i in 0..archive.len() {
                let mut file = archive.by_index(i).unwrap();
                let outpath = file.sanitized_name();
                if outpath.to_str().unwrap() == "META-INF/MANIFEST.MF"{
                    let mut manifest_mf = String::new();
                    file.read_to_string(&mut manifest_mf).unwrap();
                    return manifest_mf;
                }
            }
        }
        Err(err) => trace!("文件打开失败{:?}", err)
    }
    String::new()
}

pub fn load_file(filename: &str) -> Result<Vec<u8>>{
    let open = File::open(filename);
    if open.is_err(){
        Err(Error::new(ErrorKind::NotFound, format!("文件读取失败! {:?} {:?}", get_apk_file_path(), open.err())))
    }else{
        let mut file = open.unwrap();
        let mut content = vec![];
        let _ = file.read_to_end(&mut content);
        return Ok(content);
    }
}

pub fn load_assets(filename: &str) -> Result<Vec<u8>>{
    let open = File::open(get_apk_file_path());
    if open.is_err(){
        Err(Error::new(ErrorKind::NotFound, format!("apk读取失败! {:?} {:?}", get_apk_file_path(), open.err())))
    }else{
        let file = open.unwrap();
        let open = zip::ZipArchive::new(file);
        if open.is_err(){
            Err(Error::new(ErrorKind::NotFound, format!("apk解压失败! {:?} {:?}", get_apk_file_path(), open.err())))
        }else{
            let mut archive = open.unwrap();
            for i in 0..archive.len() {
                if let Ok(mut file) = archive.by_index(i){
                    let outpath = file.sanitized_name();
                    if let Some(path) = outpath.to_str(){
                        if path == &format!("assets/{}", filename){
                            let mut content = vec![];
                            let _ = file.read_to_end(&mut content);
                            return Ok(content);
                        }
                    }
                }
            }
            Err(Error::new(ErrorKind::NotFound, format!("文件未找到! {:?}", filename)))
        }
    }
}

pub fn copy_assets(filename:&str) -> Result<String>{
	use std::fs::File;
	use std::io::Write;
	let asset_file = format!("/data/data/{}/files/{}", get_package_name(), filename);
    let file = File::open(&asset_file);
    if file.is_ok(){
        return Ok(asset_file);
    }
    //文件不存在，创建
    match load_assets(filename){
        Ok(content) => {
            //从assets复制文件
            match File::create(&asset_file){
                Ok(mut file) => {
                    match file.write_all(&content){
                        Ok(_len) => return Ok(asset_file),
                        Err(err) => return Err(Error::new(ErrorKind::NotFound, format!("文件复制失败! {:?} {:?}", asset_file, err)))
                    }
                }
                Err(err) => return Err(Error::new(ErrorKind::NotFound, format!("文件复制失败! {:?} {:?}", asset_file, err)))
            }
        },
        Err(err) => return Err(Error::new(ErrorKind::NotFound, format!("文件复制失败! {:?} {:?}", asset_file, err)))
    }
}

use std::time::{Duration};
pub fn duration_to_milis(duration: &Duration) -> f64 {
    duration.as_secs() as f64 * 1000.0 + duration.subsec_nanos() as f64 / 1_000_000.0
}


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

	let multi_thread = true;

	if multi_thread{ //多线程 平均5ms (图片640x480)
		//每行一个线程执行
		// rayon::scope(|s| {
		// 	for (j, output_slice) in output.chunks_mut(width as usize*3).enumerate() {
		// 		s.spawn(move |_| {
		// 			let y = j as i32;
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
			//检查height的整除数 分成10个以内线程去执行
			let mut step = 1;
			for h in (2..11).rev().step_by(2){
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
    for (y, row) in src_buffer.chunks(src_stride).enumerate(){
        let j = y*3;
        for (x, pixel) in row.chunks(3).enumerate(){
            let p = (src_width-x-1)*new_stride+j;
            new_buffer[p] = pixel[0];
            new_buffer[p+1] = pixel[1];
            new_buffer[p+2] = pixel[2];
        }
    }
    (new_width, new_height)
}