#![allow(dead_code)]
use std::fs::File;
use zip;
use std::io::{Write, Read, Result};
use std::sync::Mutex;

static PROC_SELF_CMD_LINE_SUBSTR_0:&str = "/proc/self";
static PROC_SELF_CMD_LINE_SUBSTR_1:&str = "/cmdline";

static PROC_SELF_MAPS_SUBSTR_0:&str = "/maps";

static APK_DOT_STR:&str = ".apk";
static APK_STR:&str = "base.apk";

static MANIFEST_SUBSTR_0:&str = "META-INF";
static MANIFEST_SUBSTR_1:&str = "/MANIFEST.MF";

//验证内容 1、包名 2、manifest.xml签名 3、classex.dex签名

pub static PKGNAME:&str = "cn.jy.lazydict";
pub static MANIFEST_XML_SHA1:&str = "RKHLR/UFUS2TtcegKRi0jv9+e+4=";
pub static CLASSES_DEX_SHA1:&str = "Uvpbv/a/AnqTb+ePdKsX2ebAtWo=";

lazy_static!{
    static ref RD_PKGNAME:String = get_package_name().unwrap();
    static ref RD_MANIFEST_XML_SHA1:Mutex<String> = Mutex::new(String::new());
    static ref RD_CLASSES_DEX_SHA1:Mutex<String> = Mutex::new(String::new());
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
    debug!("包名:{}", package);
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
        if path == outpath.to_str().unwrap(){
            let _ = file.read_to_string(&mut manifest_mf)?;
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