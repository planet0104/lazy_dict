#![allow(dead_code)]
use std::fs::File;
use std::io::Read;
use zip;
use std::io::{Result, Error, ErrorKind};

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