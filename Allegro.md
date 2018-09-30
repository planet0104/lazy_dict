# allegro

在Android中构建:

1、在 app/build.gradle中添加
    implementation 'org.liballeg:allegro5-release:5.2.4.1A'
2、修改Activity
    public class MainActivity extends AllegroActivity {
        static {
            System.loadLibrary("allegro");
            System.loadLibrary("allegro_primitives");
            System.loadLibrary("allegro_image");
            System.loadLibrary("allegro_font");
            System.loadLibrary("allegro_ttf");
            System.loadLibrary("allegro_audio");
            System.loadLibrary("allegro_acodec");
            System.loadLibrary("allegro_color");
        }
        public MainActivity() {
            super("my_allegro");
        }
    }

3、编写my_allegro代码
创建lib项目:
cargo new my_allegro --lib

---------------Cargo.toml--------------------------------------------

```toml

[package]
name = "my_allegro"
version = "0.1.0"
authors = ["planet"]

[dependencies]
allegro = {path="RustAllegro/allegro"}
allegro_font = {path="RustAllegro/allegro_font"}
log = "*"

[target.'cfg(target_os="android")'.dependencies]
jni = { version = "*", default-features = false }
android_logger = "0.5"

[lib]
crate-type = ["dylib"]

```

----------------------------------------------------------------------

--------------lib.rs--------------------------------------------------

```rust

#[macro_use] extern crate log;
#[macro_use] extern crate allegro;
extern crate allegro_font;
use log::Level;

#[cfg(target_os="android")]
extern crate android_logger;

use allegro::*;
use allegro_font::*;

fn allegro_main(){
    let core = Core::init().unwrap();
    let font_addon = FontAddon::init(&core).unwrap();

    let display = Display::new(&core, 800, 600).unwrap();
    let timer = Timer::new(&core, 1.0 / 60.0).unwrap();
    let font = Font::new_builtin(&font_addon).unwrap();

    let queue = EventQueue::new(&core).unwrap();
    queue.register_event_source(display.get_event_source());
    queue.register_event_source(timer.get_event_source());

    let mut redraw = true;
    timer.start();
    'exit: loop
    {
        if redraw && queue.is_empty()
        {
            core.clear_to_color(Color::from_rgb_f(0.0, 0.0, 0.0));
            core.draw_text(&font, Color::from_rgb_f(1.0, 1.0, 1.0),
                (display.get_width() / 2) as f32, (display.get_height() / 2) as f32,
                FontAlign::Centre, "Welcome to RustAllegro!");
            core.flip_display();
            redraw = false;
        }

        match queue.wait_for_event()
        {
            DisplayClose{..} => break 'exit,
            TimerTick{..} => redraw = true,
            _ => (),
        }
    }
}

#[no_mangle]
#[cfg(target_os="android")]
pub fn main(argc: i32, char:*mut u8){
    use android_logger::Filter;
    android_logger::init_once(
        Filter::default().with_min_level(Level::Trace));
    trace!("hello allegro!!");
    allegro_main();
}

```

4、配置链接库
在AndroidStudio中build apk，将apk中的libs文件夹解压，复制对应cpu型号的的so文件到rust的库目录，例如复制aarch64的库：
cp /mnt/c/lib/arm64-v8a/*.so ~/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/aarch64-linux-android/lib/

下载allegro5源码，将include/allegro5文件夹复制到/usr/include
sudo cp -r ~/allegro/include/allegro5/ /usr/include/

5、编译
因为RustAllegro编译会报错，我们将源码下载到项目文件夹中，下一步进行修改。
cargo build --target aarch64-linux-android
执行编译后，会发现RustAllegro编译报错: expected u8, found i8，将报错的函数的类型i8改为u8，就可以成功编译了

5、运行
将编译好的 libmy_allegro.so 复制到 AndroidStudio 项目的 app/src/main/jniLibs/ 对应的目录
例如jniLibs/arm64-v8a/libmy_allegro.so 对应 target/aarch64-linux-android/release/libmy_allegro.so
