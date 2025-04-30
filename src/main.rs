//! Targeted Vector v0.8.0-alpha.1
//! Developer: Cheple_Bob
//! This is a rust shooter built on top of RustConstructor.
//! Special Thanks:
//! 试卷毁灭者: Give me some advice on how to make Targeted Vector.
//! Gavin: Help me improve some function.
use egui::IconData;
use function::App;
use std::sync::Arc;
// Only for macOS app generate.
// use function::Config;
// use function::find_app_bundle;
// use function::read_from_json;
// use function::write_to_json;

mod function;
mod pages;
fn main() {
    // Only for macOS app generate.
    // let mut config = Config { launch_path: "".to_string(), language: 0, login_user_name: "".to_string(), amount_languages: 0 };
    // let launch_path;
    // loop {
    //     match find_app_bundle("Targeted Vector", 100, config.launch_path.clone()) {
    //         // 搜索 app
    //         Some(path) => {
    //             launch_path = path.display().to_string().replace("/Targeted Vector.app", "");
    //             println!("找到应用路径: {}", path.display());
    //             std::env::set_current_dir(path.join("Contents/")).expect("改变路径失败！");
    //             break;
    //         },
    //         None => {
    //             panic!("RustConstructor Error[Launch failed]: Application not found!");
    //         },
    //     };
    // };
    // if let Ok(json_value) = read_from_json("Resources/config/Preferences.json") {
    //     if let Some(read_config) = Config::from_json_value(&json_value) {
    //         config = read_config;
    //     }
    // };
    // config.launch_path = launch_path;
    // let _ = write_to_json("Resources/config/Preferences.json", config.to_json_value());

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
        vsync: false,
        viewport: egui::ViewportBuilder::default()
            .with_icon(Arc::<IconData>::new(IconData {
                rgba: raw_data,
                width: w,
                height: h,
            }))
            .with_maximized(false)
            .with_min_inner_size([1280_f32, 720_f32]),
        ..Default::default()
    };
    let _ = eframe::run_native(
    "Targeted Vector",
    options,
    Box::new(|cc: &eframe::CreationContext| -> Result<Box<dyn eframe::App>, Box<dyn std::error::Error + Send + Sync>> {
        let app: App = App::new(cc);
        Ok(Box::new(app))
    }),
    );
}
