// let mut logo = None;
	// match utils::copy_assets("logo.png"){
	// 	Ok(logo_path) =>{
	// 		trace!("文件路径:{}", logo_path);
	// 		match utils::load_file(&logo_path){
	// 			Ok(data) => {
	// 				let decoder = png::Decoder::new(data.as_slice());
	// 				let (info, mut reader) = decoder.read_info().unwrap();
	// 				trace!("load_test {}x{}", info.width, info.height);
	// 			}
	// 			Err(err) => error!("load_test {:?}", err)
	// 		}
	// 		match Bitmap::load(&core, &logo_path){
	// 			Ok(bitmap) => logo = Some(bitmap),
	// 			Err(err) => error!("1.logo读取失败: {:?}", err)
	// 		}
	// 	},
	// 	Err(err) => error!("2.logo读取失败: {:?}", err)
	// }