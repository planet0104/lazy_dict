
// extern fn on_input_queue_created(_activity: *mut ANativeActivity, queue: *mut AInputQueue){
// 	APP.with(|app|{
// 		let mut app = app.borrow_mut();
// 		let (sender, receiver) = channel();
// 		app.event_loop_sender = Some(sender);
// 		//启动事件循环
// 		let input_queue = InputQueue::new(queue);
// 		thread::spawn(move || {
// 			trace!("事件循环线程已启动");
// 			loop{
// 				if let Ok(_some) = receiver.try_recv(){
// 					break;
// 				}
// 				if input_queue.queue.is_null(){
// 					error!("AInputQueue is null! 推出时间循环线程");
// 					break;
// 				}
// 				if unsafe{AInputQueue_hasEvents(input_queue.queue)}<0{
// 					//没有事件
// 					continue; 
// 				}
// 				let mut event: *mut AInputEvent = 0 as *mut c_void;
// 				unsafe{ AInputQueue_getEvent(input_queue.queue, &mut event); }
// 				if !event.is_null(){
// 					match unsafe{ AInputEvent_getType(event) }{
// 						AINPUT_EVENT_TYPE_MOTION =>{
// 							let cx = unsafe{ AMotionEvent_getX(event, 0) };
// 							let cy = unsafe{ AMotionEvent_getY(event, 0) };
// 							trace!("触摸事件 ({},{})", cx, cy);
// 							match unsafe{ AMotionEvent_getAction(event) } {
// 								AMOTION_EVENT_ACTION_DOWN => {
// 									trace!("手指按下 {},{}", cx, cy);
// 								}
// 								AMOTION_EVENT_ACTION_UP => {
// 									trace!("手指起开 {},{}", cx, cy);
// 								}
// 								_ => {}
// 							}
// 						}
// 						AINPUT_EVENT_TYPE_KEY => {
// 							trace!("键盘事件");
// 							match unsafe{ AKeyEvent_getAction(event) } {
// 								AKEY_EVENT_ACTION_DOWN => {
// 									trace!("键盘按下");
// 									match unsafe{ AKeyEvent_getKeyCode(event) } {
// 										AKEYCODE_BACK => {
// 											trace!("返回键按下");
// 										}
// 										_ => {}
// 									}
// 								}
// 								AKEY_EVENT_ACTION_UP => {
// 									trace!("返回键弹起");
// 								}
// 								_ => {}
// 							}
// 						}
// 						_ => {}
// 					}
// 					unsafe{AInputQueue_finishEvent(input_queue.queue, event, 0);}
// 				}
// 			}
// 			trace!("事件循环结束");
// 		});
// 	});
// }

// extern fn on_input_queue_destroyed(_activity: *mut ANativeActivity, _queue: *mut AInputQueue){
// 	APP.with(|app|{
// 		let mut app = app.borrow_mut();
// 		if app.event_loop_sender.is_some(){
// 			let _ = app.event_loop_sender.as_ref().unwrap().send(1);
// 			app.event_loop_sender = None;
// 		}
// 		trace!("事件循环线程结束");
// 	});
// }