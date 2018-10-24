#[derive(Debug)]
pub struct Point{
    pub x: i16,
    pub y: i16,
}

impl Point{
    pub fn new(x: i16, y:i16) -> Point{
        Point{x, y}
    }

    pub fn from_usize(x: usize, y: usize) -> Point{
        Point{x: x as i16, y: y as i16}
    }
}

impl Clone for Point{
    fn clone(&self) -> Point{
        Point{
            x: self.x,
            y: self.y
        }
    }
}

pub trait EdgePoints{
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn widthi(&self) -> i16;
    fn heighti(&self) -> i16;
    fn at(&mut self, x: i16, y: i16) -> &mut bool;
    fn atu(&mut self, x: usize, y: usize) -> &mut bool;
}

impl EdgePoints for Vec<Vec<bool>>{
    fn at(&mut self, x: i16, y: i16) -> &mut bool{
        &mut self[y as usize][x as usize]
    }
    fn atu(&mut self, x: usize, y: usize) -> &mut bool{
        &mut self[y][x]
    }
    fn height(&self) -> usize{
        self.len()
    }
    fn width(&self) -> usize{
        if self.len()>0{
            self[0].len()
        }else{
            0
        }
    }
    fn heighti(&self) -> i16{
        self.len() as i16
    }
    fn widthi(&self) -> i16{
        if self.len()>0{
            self[0].len() as i16
        }else{
            0
        }
    }
}

/*
    算法: 基于视网膜原理的边缘检测
    JiaYe 2018年1月

    视网膜水平细胞和双极细胞的功能如下:
    双极细胞 -- 亮光兴奋，弱光抑制。
    水平细胞 -- 亮光抑制，弱光兴奋，和双极细胞正好相反。

    算法：
    1.把每个像素点当作一个双极细胞，其右边和下边的像素点看作水平细胞，将像素点的亮度作为细胞输入。
    2.给定一个阈值，双极细胞和水平细胞根据阈值判断输入自身的是亮光还是弱光。
    3.计算将三个细胞的输出之和(双极细胞取两次)，如果没有抵消那么代表检测到一个边缘，否则没有检测到边缘。
    
    举例说明:
    
    B H B H B H
    H b h B H B
    B h B H B H
    H B H B H B

    上图中，字母代表图片的像素，B代表双极细胞, H代表水平细胞。
    小写b点代表当前像素点，那么当前像素点的输出等于4个细胞输出值之和除以4:
    pixel(1,1) = Sum(outB+outH+outB+outH)/4 (左下两个h点各取一次, b点取两次)))
    
    B和H的输出，根据亮度计算,如果像素亮度超过阈值，B输出255，H输出-255，没有超过阈值，二者都输出0。
*/

/// RGB888格式图像边缘检测
///
/// # Params
///
/// - `width`: 图像宽度.
/// - `height`: 图像宽度.
/// - `src`: 图像数据.
/// - `thresholds`: 阈值 0~255
/// 使用 result[y][x] 来获取点
// pub fn edge_detect(width:u32, height:u32, src:&Vec<u8>, thresholds:Vec<u8>) -> Vec<Vec<bool>>{

//     let mut edges = vec![vec![false; width as usize]; height as usize];

//     edge_detect_f(width, src, thresholds, &mut |i|{
//         let x = i/3%width as usize;
//         let y = i/3/width as usize;
//         *edges.atu(x, y) = true;
//     });

//     edges
// }

/// RGB24/RGB32图像边缘检测
///
/// # Params
///
/// - `width`: 图像宽度.
/// - `src`: 图像数据.
/// - `out`: 输出(背景为白色，前景为黑色)
/// - `thresholds`: 阈值 0~255
/// - `bpp`: 每个像素占用字节 
pub fn edge_detect_f(width:u32, bytepp: usize, src:&[u8], out:&mut [u8], thresholds:&[u8]){
    let size = src.len();
    for threshold in thresholds{
        let mut i = 0;
        while i<size{
            let (b1,b2,b3) = (i, i+1, i+2);
            let hrid = i+bytepp;
            let hbid = i+bytepp*width as usize;
            let b_out = calc_bipolar_cell(src[b1], src[b2], src[b3], *threshold as f32);
            
            if hrid<size && hbid < size{
                let hr_out = calc_horizontal_cell(src[hrid], src[hrid+1], src[hrid+2], *threshold as f32);
                let hb_out = calc_horizontal_cell(src[hbid], src[hbid+1], src[hbid+2], *threshold as f32);

                if b_out*2.0+hr_out+hb_out != 0.0{
                    
                }
            }
            i += bytepp;
        }
    }
}

/// 双极细胞 -- 亮光兴奋，弱光抑制
fn calc_bipolar_cell(r: u8, g:u8, b:u8, threshold: f32) -> f32{
    if 0.299*r as f32 + 0.587*g as f32 + 0.114*b as f32 >= threshold{
        1.0
    }else{
        -1.0
    }
}

///水平细胞 -- 亮光抑制，弱光兴奋
fn calc_horizontal_cell(r: u8, g:u8, b:u8, threshold: f32) -> f32{
    if 0.299*r as f32 + 0.587*g as f32 + 0.114*b as f32 >= threshold{
        -1.0
    }else{
        1.0
    }
}

//简单计算颜色距离
//最大距离: 255*255*3=195075 二进制: 00000000_00000010_11111010_00000011
// fn color_diff(r1: u8, g1:u8, b1: u8, r2: u8, g2:u8, b2: u8) -> u32{
//     ((r2 as i32-r1 as i32)*(r2 as i32-r1 as i32) + (g2 as i32-g1 as i32)*(g2 as i32-g1 as i32) + (b2 as i32-b1 as i32)*(b2 as i32-b1 as i32)) as u32
// }

// 8邻域
const NEIGHBORS:[Point; 8] = [ Point{ x:0, y:1 }, Point{ x:1, y:1}, Point{x:1, y:0}, Point{x:1, y:-1}, 
                             Point{x:0, y:-1}, Point{x:-1, y:-1}, Point{x:-1, y:0}, Point{x:-1, y:1} ];

//八邻域边缘跟踪
pub fn edge_track(mut edges:Vec<Vec<bool>>)->Vec<Vec<Point>>{
	let mut result = vec![];

    let mut curr_d:i32 = 0;
 
	// 边缘跟踪
	for x in 1..edges.width()-1{
        for y in  1..edges.height()-1{
			// 起始点及当前点
			let mut b_pt = Point::from_usize(x, y);
			let mut c_pt = Point::from_usize(x, y);
 
			// 如果当前点为前景点
			if *edges.atu(x, y){
                let mut edge_t = vec![];
				//let mut tra_flag = false;
				// 存入
				edge_t.push(c_pt.clone());
				*edges.at(c_pt.x, c_pt.y) = false;    // 用过的点直接给设置为0
 
				// 进行跟踪
				loop{
					// 循环八次
                    let mut counts = 0;
					while counts<8{
						// 防止索引出界
						if curr_d >= 8{
							curr_d -= 8;
						}
						if curr_d < 0{
							curr_d += 8;
						}
 
						// 当前点坐标
						// 跟踪的过程，应该是个连续的过程，需要不停的更新搜索的root点
						c_pt = Point::new(b_pt.x + NEIGHBORS[curr_d as usize].x, b_pt.y + NEIGHBORS[curr_d as usize].y);
 
						// 边界判断
						if (c_pt.x > 0) && (c_pt.x < edges.widthi() - 1) &&
							(c_pt.y > 0) && (c_pt.y < edges.heighti() - 1)
						{
							// 如果存在边缘
							if *edges.at(c_pt.x, c_pt.y){
								curr_d -= 2;   // 更新当前方向
								edge_t.push(c_pt.clone());
								*edges.at(c_pt.x, c_pt.y) = false;
 
								// 更新b_pt:跟踪的root点
								b_pt.x = c_pt.x;
								b_pt.y = c_pt.y;

								break;   // 跳出for循环
							}
						}
						curr_d += 1;
                        counts += 1;
					}   // end for
					// 跟踪的终止条件：如果8邻域都不存在边缘
					if 8 == counts{
						// 清零
						curr_d = 0;
						//tra_flag = true;
						result.push(edge_t);
						break;
					}
				}  // end if
			}  // end while
		}
    }

    result
}

/// 边缘矢量化 减少点数
/// # Params
/// - `contours`: 跟踪到的边缘
/// - `segment`: 线段长度.
/// 返回矢量化以后的点
pub fn contours_vectorize(contours: &Vec<Vec<Point>>, min_len:usize, segment:f32) -> Vec<Vec<Point>>{
    let mut vectors = vec![];

    for contour in contours{
        if contour.len()<=min_len{
            vectors.push(contour.clone());
            continue;
        }
        let mut points = vec![contour[0].clone()];
        for i in 1..contour.len(){
            let pi = points.len()-1;
            let dist = (points[pi].x - contour[i].x)*(points[pi].x - contour[i].x)+(points[pi].y - contour[i].y)*(points[pi].y - contour[i].y);
            if (dist as f32).sqrt() >= segment{
                points.push(contour[i].clone());
            }
        }

        //只有一个点补充一个点
        if points.len() == 1{
            points.push(contour[0].clone());
        }else{
            //points最后一个点如果不等于轮廓的最后一个点, 补充最后一个点
            let lenp = points.len()-1;
            let lenc = contour.len()-1;
            if points[lenp].x != contour[lenc].x || points[lenp].y != contour[lenc].y{
                points.push(contour[lenc].clone());
            }
        }

        vectors.push(points);
    }

    vectors
}