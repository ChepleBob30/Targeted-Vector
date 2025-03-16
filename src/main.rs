//! Targeted Vector v0.2.0-alpha.1
//! Developer: Cheple_Bob
//! This is a rust shooter built on top of RustConstructor.
//! Special Thanks:
//! 试卷毁灭者: Give me some advice on how to make Targeted Vector.
//! Gavin: Help me improve some function.
use egui::IconData;
use function::App;
use std::sync::Arc;

mod function;
mod pages;

fn main() {
    // let mut config = Config { launch_path: "".to_string() };
    // // 读取 JSON 文件
    // if let Some(read_config) = read_from_json("Resources/config/Preferences.json") {
    //     config = read_config
    // } else {
    //     eprintln!("阅读/解析 JSON 文件失败");
    // };
    // loop {
    //     match find_app_bundle("Targeted Vector", 100, config.launch_path.clone()) {
    //         // 搜索 app
    //         Some(path) => {
    //             config.launch_path = path.display().to_string().replace("/Targeted Vector.app", "");
    //             println!("找到应用路径: {}", path.display());
    //             std::env::set_current_dir(path.join("Contents/")).expect("改变路径失败！");
    //             break;
    //         },
    //         None => {
    //             eprintln!("未找到app！");
    //             config.launch_path = "".to_string();
    //         },
    //     };
    // };
    // 写入 JSON 文件
    // write_to_json("Resources/config/Preferences.json", &config);
    let img = image::load_from_memory_with_format(
        include_bytes!("../Resources/assets/images/icon.png"),
        image::ImageFormat::Png,
    )
        .unwrap();
    let rgba_data = img.into_rgba8();
    let (w, h) = (rgba_data.width(), rgba_data.height());
    let raw_data: Vec<u8> = rgba_data.into_raw();
    let options = eframe::NativeOptions {
        centered: true,
        viewport: egui::ViewportBuilder::default()
            .with_fullscreen(true)
            .with_active(true)
            .with_always_on_top()
            .with_icon(Arc::<IconData>::new(IconData {
                rgba: raw_data,
                width: w,
                height: h,
            }))
            .with_maximized(true)
            .with_min_inner_size([1280_f32, 720_f32]),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Targeted Vector",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    );
}
