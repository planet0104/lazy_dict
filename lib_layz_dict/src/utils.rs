#![allow(dead_code)]
use std::fs::File;
use zip;
use std::io::{Write, Read, Result};
use std::sync::Mutex;
use sha1::Sha1;

static PROC_SELF_CMD_LINE_SUBSTR_0:&str = "/proc/self";
static PROC_SELF_CMD_LINE_SUBSTR_1:&str = "/cmdline";

static PROC_SELF_MAPS_SUBSTR_0:&str = "/maps";

static APK_DOT_STR:&str = ".apk";
static APK_STR:&str = "base.apk";

static MANIFEST_SUBSTR_0:&str = "META-INF";
static MANIFEST_SUBSTR_1:&str = "/MANIFEST.MF";

//验证内容 1、包名 2、manifest.xml签名 3、classex.dex签名

pub static PKGNAME:&[u8] = &[14, 176, 195, 92, 33, 68, 50, 24, 72, 141, 198, 70, 241, 171, 7, 121];//cn.jy.lazydict

//测试
// pub static MANIFEST_XML_SHA1:&[u8] = &[142, 94, 123, 29, 215, 242, 74, 40, 227, 138, 68, 78, 225, 94, 116, 113];
// pub static CLASSES_DEX_SHA1:&[u8] = &[111, 66, 48, 231, 203, 4, 142, 162, 208, 246, 76, 193, 110, 241, 252, 210];
//生产
pub static MANIFEST_XML_SHA1:&[u8] = &[139, 132, 155, 237, 49, 118, 188, 11, 34, 47, 84, 61, 153, 186, 61, 118];
pub static CLASSES_DEX_SHA1:&[u8] = &[218, 82, 108, 32, 93, 181, 176, 12, 0, 17, 172, 88, 130, 192, 2, 79];

pub static XML_SHA1:&[u8] = &[132, 193, 211, 63, 85, 187, 184, 204, 251, 104, 177, 217, 94, 208, 110, 210, 150, 239, 203, 68, 80, 38, 51, 116, 16, 193, 198, 237, 88, 77, 13, 90];//两个xml

lazy_static!{
    //读取包名
    pub static ref RD_PKGNAME:Mutex<[u8; 16]> = Mutex::new([0; 16]);
    //读取manifest.xml签名
    pub static ref RD_MANIFEST_XML_SHA1:Mutex<[u8; 16]> = Mutex::new([0; 16]);
    //读取classex.dex签名
    pub static ref RD_CLASSES_DEX_SHA1:Mutex<[u8; 16]> = Mutex::new([0; 16]);
    //两个页面签名
    pub static ref RD_XML_SHA1:Mutex<[u8; 32]> = Mutex::new([0; 32]);
    pub static ref INIT_SUCCESS:Mutex<bool> = Mutex::new(false);
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

pub fn init() -> Result<String>{
    let pkg_name = get_package_name()?;
    //设置包名
    (*RD_PKGNAME.lock().unwrap()) = encode(&pkg_name);
    
    let manifest_mf = get_manifest_mf()?;

    //设置manifest.xml签名
    let mut read_xml = 0;
    let mut read_dex = 0;
    let mut read_view0 = 0;
    let mut read_view1 = 0;
    let mut view_data:[u8; 32] = [0; 32];
    for line in manifest_mf.lines(){
        if read_xml == 1{
            let mut iter = line.split_whitespace();
            let _ = iter.next();
            (*RD_MANIFEST_XML_SHA1.lock().unwrap()) = encode(iter.next().unwrap());
            read_xml = 2;
        }
        if read_dex == 1{
            let mut iter = line.split_whitespace();
            let _ = iter.next();
            (*RD_CLASSES_DEX_SHA1.lock().unwrap()) = encode(iter.next().unwrap());
            read_dex = 2;
        }
        if read_view0 == 1{
            let mut iter = line.split_whitespace();
            let _ = iter.next();
            let data = encode(iter.next().unwrap());
            for i in 0..16{
                view_data[i] = data[i];
            }
            read_view0 = 2;
        }
        if read_view1 == 1{
            let mut iter = line.split_whitespace();
            let _ = iter.next();
            let data = encode(iter.next().unwrap());
            for i in 0..16{
                view_data[16+i] = data[i];
            }
            read_view1 = 2;
        }
        if read_dex==2 && read_xml==2 && read_view0==2 && read_view1==2{
            (*RD_XML_SHA1.lock().unwrap()) = view_data;
            break;
        }
        if line.contains(&decode_base64("QW5kcm9pZE1hbmlmZXN0LnhtbA==")){
            read_xml = 1;
        }
        if line.contains(&decode_base64("Y2xhc3Nlcy5kZXg=")){
            read_dex = 1;
        }
        if line.contains("activity_camera.xml"){
            read_view0 = 1;
        }
        if line.contains("activity_splash.xml"){
            read_view1 = 1;
        }
    }
    (*INIT_SUCCESS.lock().unwrap()) = true;
    Ok(String::new())
}

fn decode_base64(s:&str) -> String{
	String::from_utf8(base64::decode(s).unwrap()).unwrap()
}

// 获取包名
pub fn get_package_name() -> Result<String>{
    let mut file = File::open(&format!("{}{}", PROC_SELF_CMD_LINE_SUBSTR_0, PROC_SELF_CMD_LINE_SUBSTR_1))?;
    let mut contents = String::new();
    let _count = file.read_to_string(&mut contents)?;
    Ok(String::from(contents.trim_matches(char::from(0))))
}

//获取apk路径
pub fn get_apk_file_path() -> Result<String>{
    let package = get_package_name()?;
    let mut file = File::open(&format!("{}{}", PROC_SELF_CMD_LINE_SUBSTR_0, PROC_SELF_MAPS_SUBSTR_0))?;
    let mut contents = String::new();
    let _count = file.read_to_string(&mut contents)?;
    let mut apk_file = "";
    for line in contents.lines(){
        if line.contains(APK_DOT_STR) && line.contains(&package){
            for p in line.split(" "){
                if p.ends_with(APK_STR){
                    apk_file = p;
                    break;
                }
            }
        }
    }
    Ok(String::from(apk_file))
}

pub fn get_manifest_mf() -> Result<String>{
    let file = File::open(get_apk_file_path()?)?;
    //trace!("开始解压文件..");
    let mut archive = zip::ZipArchive::new(file)?;
    let mut manifest_mf = String::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = file.sanitized_name();
        let path = &format!("{}{}", MANIFEST_SUBSTR_0, MANIFEST_SUBSTR_1);
        if let Some(p) = outpath.to_str(){
            if path == p{
                let _ = file.read_to_string(&mut manifest_mf)?;
            }
        }
    }
    Ok(manifest_mf)
}

pub fn load_file(filename: &str) -> Result<Vec<u8>>{
    let mut file = File::open(filename)?;
    let mut content = vec![];
    file.read_to_end(&mut content)?;
    Ok(content)
}

pub fn load_assets(filename: &str) -> Result<Vec<u8>>{
    let file = File::open(get_apk_file_path()?)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut content = vec![];
    for i in 0..archive.len() {
        if let Ok(mut file) = archive.by_index(i){
            let outpath = file.sanitized_name();
            if let Some(path) = outpath.to_str(){
                if path == &format!("assets/{}", filename){
                    let _ = file.read_to_end(&mut content);
                }
            }
        }
    }
    Ok(content)
}

pub fn copy_assets(filename:&str) -> Result<String>{
	let asset_file = format!("/data/data/{}/files/{}", get_package_name()?, filename);
    match File::open(&asset_file){
        Ok(_file) => Ok(asset_file),
        _ =>{
            //文件不存在，创建
            let content = load_assets(filename)?;
            //从assets复制文件
            let mut file = File::create(&asset_file)?;
            file.write_all(&content)?;
            Ok(asset_file)
        }
    }
}

use std::time::{Duration};
pub fn duration_to_milis(duration: &Duration) -> f64 {
    duration.as_secs() as f64 * 1000.0 + duration.subsec_nanos() as f64 / 1_000_000.0
}

use native_window::{ARect, ANativeWindow_unlockAndPost, ANativeWindow_lock, ANativeWindow_Buffer, ANativeWindow };
use std::os::raw::{c_void};

//获取windows的缓冲区
pub fn lock_native_window_rgb_888<F>(window: *mut ANativeWindow, mut render: F) -> ::std::result::Result<(), String> where F : FnMut(&ANativeWindow_Buffer, &mut [u8])->::std::result::Result<(), String>{
	let mut buffer = ANativeWindow_Buffer{ width: 0, height: 0, stride: 0, format: 0, bits: 0 as *mut c_void, reserved: [0; 5] };
	let mut rect = ARect { left: 0, top: 0, right: 0, bottom: 0};
	let ret_code = unsafe{ ANativeWindow_lock(window, &mut buffer, &mut rect) };
	if ret_code !=0 {
		return Err(format!("ANativeWindow_lock 调用失败! {}", ret_code));
	}
	let pixels = unsafe{ ::std::slice::from_raw_parts_mut(buffer.bits as *mut u8, (buffer.stride*buffer.height*3) as usize) };
	//trace!("lock_native_window {}x{} format={} stride={} pixel.len()={}", buffer.width, buffer.height, buffer.format, buffer.stride, pixels.len());
	let ret = render(&buffer, pixels);
	unsafe{ ANativeWindow_unlockAndPost(window) };
    ret
}