//纵向扩展像素
    let bpp = 3;
    let bpl = width*bpp;
    let mut extend = vec![255; width*height*bpp];

    for (y, row) in out.chunks(bpl).enumerate(){
        for (x, pixel) in row.chunks(bpp).enumerate(){
            if pixel[0] == 0{
                let i = y*bpl+x*bpp;
                //当前点
                extend[i] = 0;
                extend[i+1] = 0;
                extend[i+2] = 0;
                if y<height-1{
                    extend[i+bpl] = 0;
                    extend[i+bpl+1] = 0;
                    extend[i+bpl+2] = 0;
                    if bpp>3{
                        extend[i+bpl+3] = 255;//不透明
                    }
                }
                if y<height-2{
                    extend[i+bpl*2] = 0;
                    extend[i+bpl*2+1] = 0;
                    extend[i+bpl*2+2] = 0;
                    if bpp>3{
                        extend[i+bpl*2+3] = 255;//不透明
                    }
                }
                if y<height-3{
                    extend[i+bpl*3] = 0;
                    extend[i+bpl*3+1] = 0;
                    extend[i+bpl*3+2] = 0;
                    if bpp>3{
                        extend[i+bpl*3+3] = 255;//不透明
                    }
                }
            }
        }
    }




    //--------------------------------- 灰度直方图
    let height = 50;
    let mut gray_bar = vec![255; 256*height*3];
    for (y, row) in gray_bar.chunks_mut(256*3).enumerate(){
        for (x, pixel) in row.chunks_mut(3).enumerate(){
            let h = ((gray_count[x] as f32/max as f32)*height as f32) as usize;
            if h>=(height-y){
                pixel[0] = 0;
                pixel[1] = 0;
                pixel[2] = 0;
            }
        }
    }


    println!("{:?} max={}", gray_count, max);

    let newimg:ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(256, height as u32, gray_bar).unwrap();
    
    println!("width={}", newimg.width());
    newimg.save("new.png").unwrap();

    return;



    //--------------------------------- 灰度直方图
    // let height = 100;
    // let width = 256;
    // let mut gray_bar = vec![255; width*height*3];
    // let max = *gray_count.iter().max().unwrap();
    // for (y, row) in gray_bar.chunks_mut(width*3).enumerate(){
    //     for (x, pixel) in row.chunks_mut(3).enumerate(){
    //         let h = ((gray_count[x] as f32/max as f32)*height as f32) as usize;
    //         if h>=(height-y){
    //             pixel[0] = 0;
    //             pixel[1] = 0;
    //             pixel[2] = 0;
    //         }
    //     }
    // }

    //println!("{:?} max={}", gray_count, max);

    //let newimg:ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(new_width as u32, new_height as u32, dst).unwrap();
    // let newimg:ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width as u32, height as u32, gray_bar).unwrap();
    // println!("width={}", newimg.width());
    // newimg.save("new.png").unwrap();


            let mut total = 0;
        for info in infos.iter(){
            total += info.width*info.height;
        }
        let avg = if total>0{
            total/infos.len()
        }else{
            0
        };

        let info = SplitInfo::new(parent_left+left, parent_top+top, new_width, new_height);
        let area = info.width*info.height;
        // println!("面积:{}", info.width*info.height);
        if avg!=0 && avg/area>=2{
            false
        }else{
            infos.push(info);
            true
        }






            //过滤比例不对的方块
    let mut new_infos = vec![];
    for ifo in &infos{
        if ifo.height as f32 / ifo.width as f32>1.8
            || ifo.width as f32 / ifo.height as f32>1.8{
            continue;
        }else{
            new_infos.push(SplitInfo::new(ifo.left, ifo.top, ifo.width, ifo.height));
        }
    }


    
    //保存边缘图
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

    //println!("边缘检测{}ms", duration_to_milis(&now.elapsed())); let now = Instant::now();