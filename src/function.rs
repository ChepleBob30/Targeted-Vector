//! function.rs is the functional module of the Targeted Vector, including function declarations, struct definitions, and some auxiliary content.
use anyhow::Context;
use eframe::emath::Rect;
use eframe::epaint::textures::TextureOptions;
use eframe::epaint::Stroke;
use egui::{Color32, FontId, Frame, PointerButton, Pos2, Ui, Vec2};
use json::{object, JsonValue};
use rodio::{Decoder, OutputStream};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;
use walkdir::WalkDir;

/// 在 macOS 上搜索指定 .app 应用的绝对路径
#[allow(dead_code)]
pub fn find_app_bundle(
    app_name: &str,
    max_depth: usize,
    prioritize_finding: String,
) -> Option<PathBuf> {
    // 定义优先搜索的目录（按优先级排序）
    let mut search_paths = vec![];
    if prioritize_finding == *"".to_string() {
        search_paths.push(format!(
            "/Users/{}/Applications",
            whoami::username().unwrap_or_else(|_| "<unknown>".to_string())
        ));
        search_paths.push(format!(
            "/Users/{}/Downloads",
            whoami::username().unwrap_or_else(|_| "<unknown>".to_string())
        ));
        search_paths.push(format!(
            "/Users/{}/Documents",
            whoami::username().unwrap_or_else(|_| "<unknown>".to_string())
        ));
        search_paths.push(format!(
            "/Users/{}/Desktop",
            whoami::username().unwrap_or_else(|_| "<unknown>".to_string())
        ));
        search_paths.push("/Applications".to_string());
        search_paths.push(format!(
            "/Users/{}",
            whoami::username().unwrap_or_else(|_| "<unknown>".to_string())
        ));
    } else {
        search_paths.push(prioritize_finding);
    }
    for path in search_paths {
        for entry in WalkDir::new(path)
            .max_depth(max_depth) // 限制递归深度提升性能
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();
            println!("check{:?}", entry_path);
            if entry_path.is_dir()
                && entry_path.extension().is_some_and(|ext| ext == "app")
                && entry_path.file_stem().is_some_and(|name| name == app_name)
            {
                return Some(entry_path.canonicalize().unwrap());
            }
        }
    }

    None
}

pub fn value_to_bool(value: i32) -> bool {
    value > 0
}

// 创建格式化的JSON文件
#[allow(dead_code)]
pub fn create_pretty_json<P: AsRef<Path>>(path: P, data: JsonValue) -> anyhow::Result<()> {
    let parent_dir = path
        .as_ref()
        .parent()
        .ok_or_else(|| anyhow::anyhow!("无效的文件路径"))?;

    // 创建父目录（如果不存在）
    fs::create_dir_all(parent_dir)?;

    // 生成带缩进的JSON字符串（4空格缩进）
    let formatted = json::stringify_pretty(data, 4);

    // 写入文件（自动处理换行符）
    fs::write(path, formatted)?;
    Ok(())
}

// 复制并重新格式化JSON文件
#[allow(dead_code)]
pub fn copy_and_reformat_json<P: AsRef<Path>>(src: P, dest: P) -> anyhow::Result<()> {
    // 读取原始文件
    let content = fs::read_to_string(&src)?;

    // 解析JSON（自动验证格式）
    let parsed = json::parse(&content)?;

    // 使用格式化写入新文件
    create_pretty_json(dest, parsed)?;

    Ok(())
}

pub fn check_file_exists<P: AsRef<Path>>(path: P) -> bool {
    let path_ref = path.as_ref();
    if path_ref.exists() {
        true // 文件已存在时直接返回，不执行写入
    } else {
        // 文件不存在时，返回 false
        false
    }
}

// 通用 JSON 写入函数
#[allow(dead_code)]
pub fn write_to_json<P: AsRef<Path>>(path: P, data: JsonValue) -> anyhow::Result<()> {
    let parent_dir = path
        .as_ref()
        .parent()
        .ok_or_else(|| anyhow::anyhow!("无效的文件路径"))?;

    fs::create_dir_all(parent_dir)?;
    let formatted = json::stringify_pretty(data, 4);
    fs::write(path, formatted)?;
    Ok(())
}

// 通用 JSON 读取函数
pub fn read_from_json<P: AsRef<Path>>(path: P) -> anyhow::Result<JsonValue> {
    let content = fs::read_to_string(&path)
        .with_context(|| format!("无法读取文件: {}", path.as_ref().display()))?;
    json::parse(&content).with_context(|| format!("解析 JSON 失败: {}", path.as_ref().display()))
}

pub fn wav_player(wav_path: String) -> anyhow::Result<f64> {
    // 打开 WAV 文件
    let reader = hound::WavReader::open(&wav_path).context("无法打开 WAV 文件")?;

    // 获取 WAV 文件的规格
    let spec = reader.spec();
    let sample_rate = spec.sample_rate as f64;
    let total_samples = reader.len() as f64;

    // 计算时长（秒）
    let duration = total_samples / sample_rate;

    // 打开文件并创建解码器
    let file = BufReader::new(File::open(&wav_path).context("无法打开文件")?);
    let source = Decoder::new(file).context("无法解码音频文件")?;

    // 获取默认物理声音设备的输出流句柄
    let (_stream, stream_handle) = OutputStream::try_default().context("无法获取默认输出流")?;

    // 创建一个新的 Sink 来管理播放
    let sink = rodio::Sink::try_new(&stream_handle).context("无法创建 Sink")?;
    sink.append(source);

    sink.sleep_until_end(); // 等待音频播放结束
    Ok(duration)
}

fn load_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../Resources/assets/fonts/Text.ttf")).into(),
    );
    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "my_font".to_owned());
    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .push("my_font".to_owned());
    ctx.set_fonts(fonts);
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Config {
    pub launch_path: String,
    pub language: u8,
}

#[allow(dead_code)]
impl Config {
    fn to_json_value(&self) -> JsonValue {
        object! {
            launch_path: self.launch_path.clone(),
            language: self.language
        }
    }

    fn from_json_value(value: &JsonValue) -> Option<Config> {
        Some(Config {
            launch_path: value["launch_path"].as_str()?.to_string(),
            language: value["language"].as_u8()?,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GameText {
    pub game_text: HashMap<String, Vec<String>>,
}

impl GameText {
    pub fn from_json_value(value: &JsonValue) -> Option<GameText> {
        // 检查 game_text 字段是否为对象
        if !value["game_text"].is_object() {
            return None;
        }

        // 遍历对象键值对
        let mut parsed = HashMap::new();
        for (key, val) in value["game_text"].entries() {
            if let JsonValue::Array(arr) = val {
                let str_vec: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
                parsed.insert(key.to_string(), str_vec);
            }
        }

        Some(GameText { game_text: parsed })
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct User {
    pub version: u8,
    pub name: String,
    pub password: String,
    pub language: u8,
    pub wallpaper: String,
}

#[allow(dead_code)]
impl User {
    pub fn from_json_value(value: &JsonValue) -> Option<User> {
        Some(User {
            version: value["version"].as_u8()?,
            name: value["name"].as_str()?.to_string(),
            password: value["password"].as_str()?.to_string(),
            language: value["language"].as_u8()?,
            wallpaper: value["wallpaper"].as_str()?.to_string(),
        })
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum Value {
    Bool(bool),
    Int(i32),
    UInt(u32),
    Float(f32),
    Vec(Vec<Value>),
    String(String),
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Value::Int(i)
    }
}

impl From<u32> for Value {
    fn from(u: u32) -> Self {
        Value::UInt(u)
    }
}

impl From<f32> for Value {
    fn from(f: f32) -> Self {
        Value::Float(f)
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(v: Vec<T>) -> Self {
        Value::Vec(v.into_iter().map(|x| x.into()).collect())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

/// The method to get the index of the resource in the vector.
/// # Arguments
/// * `resource_list` - target vector.
/// * `resource_name` - target resource name.
/// * `error_log` - if method didn't find your resource, it will print the error message.
/// # Returns
/// your resource's index.
/// # Panics
/// if resource doesn't exist, it will panic.
/// # Examples
/// ```
/// track_resource(self.resource_image, "title_image", "Image_Vector");
/// ```
pub fn track_resource<T: RustConstructorResource>(
    resource_list: Vec<T>,
    resource_name: &str,
    error_log: &str,
) -> usize {
    let mut id: i32 = -1;
    for (i, _a) in resource_list.iter().enumerate() {
        if resource_list[i].name() == resource_name {
            id = i as i32;
            break;
        };
    }
    if id == -1 {
        panic!(
            "RustConstructor Error[Track lost]: \"{}\" does not have a resource \"{}\"!",
            error_log, resource_name
        );
    }
    id as usize
}

pub trait RustConstructorResource {
    fn name(&self) -> &str;
}

impl RustConstructorResource for PageData {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone)]
pub struct PageData {
    pub name: String,
    pub forced_update: bool,
    pub change_page_updated: bool,
}

#[derive(Clone)]
pub struct Timer {
    pub start_time: f32,
    pub total_time: f32,
    pub timer: Instant,
    pub now_time: f32,
    pub split_time: Vec<SplitTime>,
}

impl RustConstructorResource for ImageTexture {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone)]
pub struct ImageTexture {
    pub name: String,
    pub texture: Option<egui::TextureHandle>,
}

impl RustConstructorResource for CustomRect {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone)]
pub struct CustomRect {
    pub name: String,
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub rounding: f32,
    pub x_grid: [u32; 2],
    pub y_grid: [u32; 2],
    pub center_display: [bool; 4],
    pub color: [u8; 4],
    pub border_width: f32,
    pub border_color: [u8; 4],
    pub origin_position: [f32; 2],
}

impl RustConstructorResource for Image {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone)]
pub struct Image {
    pub name: String,
    pub image_texture: Option<egui::TextureHandle>,
    pub image_position: [f32; 2],
    pub image_size: [f32; 2],
    pub x_grid: [u32; 2],
    pub y_grid: [u32; 2],
    pub center_display: [bool; 4],
    pub alpha: u8,
    pub overlay_color: [u8; 4],
    pub use_overlay_color: bool,
    pub origin_position: [f32; 2],
}

impl RustConstructorResource for Text {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone)]
pub struct Text {
    pub name: String,
    pub text_content: String,
    pub font_size: f32,
    pub rgba: [u8; 4],
    pub position: [f32; 2],
    pub center_display: [bool; 4],
    pub wrap_width: f32,
    pub write_background: bool,
    pub background_rgb: [u8; 3],
    pub rounding: f32,
    pub x_grid: [u32; 2],
    pub y_grid: [u32; 2],
    pub origin_position: [f32; 2],
}

impl RustConstructorResource for ScrollBackground {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone)]
pub struct ScrollBackground {
    pub name: String,
    pub image_name: Vec<String>,
    pub horizontal_or_vertical: bool,
    pub left_and_top_or_right_and_bottom: bool,
    pub scroll_speed: u32,
    pub boundary: f32,
    pub resume_point: f32,
}

impl RustConstructorResource for Variable {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct Variable {
    pub name: String,
    pub value: Value,
}

impl RustConstructorResource for SplitTime {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone)]
pub struct SplitTime {
    pub name: String,
    pub time: [f32; 2],
}

impl RustConstructorResource for Switch {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct Switch {
    pub name: String,
    pub switch_image_name: String,
    pub switch_texture_name: Vec<String>,
    pub enable_hover_click_image: [bool; 2],
    pub state: u32,
    pub use_overlay: bool,
    pub overlay_color: Vec<[u8; 4]>,
    pub click_method: Vec<PointerButton>,
    pub last_time_clicked: bool,
    pub last_time_clicked_index: usize,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct App {
    pub config: Config,
    pub game_text: GameText,
    pub login_user_name: String,
    pub frame: Frame,
    pub page: String,
    pub resource_page: Vec<PageData>,
    pub resource_image: Vec<Image>,
    pub resource_text: Vec<Text>,
    pub resource_rect: Vec<CustomRect>,
    pub resource_scroll_background: Vec<ScrollBackground>,
    pub timer: Timer,
    pub variables: Vec<Variable>,
    pub resource_image_texture: Vec<ImageTexture>,
    pub resource_switch: Vec<Switch>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        load_fonts(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let mut config = Config {
            launch_path: "".to_string(),
            language: 0,
        };
        let mut game_text = GameText {
            game_text: HashMap::new(),
        };
        // 写入 JSON 文件
        // let json_value = config.to_json_value();
        // write_to_json("Resources/config/Preferences.json", json_value)
        //     .expect("写入 JSON 文件失败");
        // 读取 JSON 文件
        if let Ok(json_value) = read_from_json("Resources/config/Preferences.json") {
            if let Some(read_config) = Config::from_json_value(&json_value) {
                config = read_config;
            }
        }
        if let Ok(json_value) = read_from_json("Resources/config/GameText.json") {
            if let Some(read_game_text) = GameText::from_json_value(&json_value) {
                game_text = read_game_text;
            }
        }
        Self {
            config,
            game_text,
            login_user_name: "".to_string(),
            frame: Frame {
                ..Default::default()
            },
            page: "Launch".to_string(),
            resource_page: vec![
                PageData {
                    name: "Launch".to_string(),
                    forced_update: true,
                    change_page_updated: false,
                },
                PageData {
                    name: "Login".to_string(),
                    forced_update: true,
                    change_page_updated: false,
                },
                PageData {
                    name: "Home_Page".to_string(),
                    forced_update: true,
                    change_page_updated: false,
                },
                PageData {
                    name: "Home_Setting".to_string(),
                    forced_update: true,
                    change_page_updated: false,
                },
            ],
            resource_image: vec![],
            resource_text: vec![],
            resource_rect: vec![],
            resource_scroll_background: vec![],
            timer: Timer {
                start_time: 0.0,
                total_time: 0.0,
                timer: Instant::now(),
                now_time: 0.0,
                split_time: vec![],
            },
            variables: vec![],
            resource_image_texture: vec![],
            resource_switch: vec![],
        }
    }

    pub fn switch_page(&mut self, page: &str) {
        self.page = self.resource_page[track_resource(self.resource_page.clone(), page, "page")]
            .name
            .to_string();
    }

    pub fn launch_page_preload(&mut self, ctx: &egui::Context) {
        let game_text = self.game_text.game_text.clone();
        for i in 0..self.resource_page.len() {
            if i == 0 {
                self.add_image_texture(
                    "RC_Logo",
                    "Resources/assets/images/rc.png",
                    [false, false],
                    true,
                    ctx,
                );
                self.add_image_texture(
                    "Binder_Logo",
                    "Resources/assets/images/binder.png",
                    [false, false],
                    true,
                    ctx,
                );
                self.add_image_texture(
                    "Mouse",
                    "Resources/assets/images/mouse_white.png",
                    [false, false],
                    true,
                    ctx,
                );
                self.add_image(
                    "RC_Logo",
                    [0_f32, 0_f32, 130_f32, 130_f32],
                    [1, 2, 1, 2],
                    [false, false, false, true, false],
                    [0, 0, 0, 0, 0],
                    "RC_Logo",
                );
                self.add_image(
                    "Binder_Logo",
                    [0_f32, 0_f32, 150_f32, 150_f32],
                    [1, 2, 1, 2],
                    [false, false, false, true, false],
                    [0, 0, 0, 0, 0],
                    "Binder_Logo",
                );
                self.add_image(
                    "Mouse",
                    [0_f32, 0_f32, 150_f32, 150_f32],
                    [1, 2, 1, 2],
                    [false, false, false, true, false],
                    [0, 0, 0, 0, 0],
                    "Mouse",
                );
                self.add_text(
                    ["Powered", " Powered by\n Rust Constructor"],
                    [0_f32, 0_f32, 40_f32, 1000_f32, 0.0],
                    [255, 255, 255, 0, 0, 0, 0],
                    [true, false, false, true],
                    false,
                    [1, 2, 1, 2],
                );
                self.add_text(
                    [
                        "Organize",
                        &*game_text["organize"][self.config.language as usize].clone(),
                    ],
                    [0_f32, 0_f32, 40_f32, 1000_f32, 0.0],
                    [255, 255, 255, 0, 0, 0, 0],
                    [true, false, false, true],
                    false,
                    [1, 2, 1, 2],
                );
                self.add_text(
                    [
                        "Mouse",
                        &*game_text["connect_mouse"][self.config.language as usize].clone(),
                    ],
                    [0_f32, 0_f32, 40_f32, 1000_f32, 0.0],
                    [255, 255, 255, 0, 0, 0, 0],
                    [true, false, false, true],
                    false,
                    [1, 2, 1, 2],
                );
                self.add_rect(
                    "Background",
                    [
                        0_f32,
                        0_f32,
                        ctx.available_rect().width(),
                        ctx.available_rect().height(),
                        0_f32,
                    ],
                    [1, 2, 1, 2],
                    [false, false, true, true],
                    [0, 0, 0, 255, 255, 255, 255, 255],
                    0.0,
                );
                let _ = std::thread::spawn(|| {
                    let _ = wav_player("Resources/assets/sounds/Launch.wav".to_string());
                });
            } else if i == 1 {
                if self.config.language == 0 {
                    self.add_image_texture(
                        "Title",
                        "Resources/assets/images/zh_title.png",
                        [false, false],
                        true,
                        ctx,
                    );
                    self.add_image(
                        "Title",
                        [0_f32, 0_f32, 510_f32, 150_f32],
                        [1, 2, 1, 4],
                        [true, true, true, true, false],
                        [255, 0, 0, 0, 0],
                        "Title",
                    );
                } else {
                    self.add_image_texture(
                        "Title",
                        "Resources/assets/images/en_title.png",
                        [false, false],
                        true,
                        ctx,
                    );
                    self.add_image(
                        "Title",
                        [0_f32, 0_f32, 900_f32, 130_f32],
                        [1, 2, 1, 4],
                        [true, true, true, true, false],
                        [255, 0, 0, 0, 0],
                        "Title",
                    );
                };
                self.add_image_texture(
                    "Background",
                    "Resources/assets/images/wallpaper.jpg",
                    [false, false],
                    true,
                    ctx,
                );
                self.add_image_texture(
                    "Power",
                    "Resources/assets/images/power.png",
                    [false, false],
                    true,
                    ctx,
                );
                self.add_image_texture(
                    "Register",
                    "Resources/assets/images/register.png",
                    [false, false],
                    true,
                    ctx,
                );
                self.add_image_texture(
                    "Login",
                    "Resources/assets/images/login.png",
                    [false, false],
                    true,
                    ctx,
                );
                self.add_image_texture(
                    "Gun_Logo",
                    "Resources/assets/images/logo_gun.png",
                    [false, false],
                    true,
                    ctx,
                );
                self.add_image_texture(
                    "Reg_Complete",
                    "Resources/assets/images/check.png",
                    [false, false],
                    true,
                    ctx,
                );
                self.add_image(
                    "Reg_Complete",
                    [0_f32, 0_f32, 100_f32, 100_f32],
                    [1, 2, 1, 2],
                    [true, true, true, false, false],
                    [255, 0, 0, 0, 0],
                    "Reg_Complete",
                );
                self.add_image(
                    "Gun_Logo",
                    [0_f32, 0_f32, 100_f32, 100_f32],
                    [1, 2, 1, 2],
                    [true, true, true, false, false],
                    [255, 0, 0, 0, 0],
                    "Gun_Logo",
                );
                self.add_image(
                    "Power",
                    [-75_f32, 25_f32, 50_f32, 50_f32],
                    [1, 2, 7, 8],
                    [true, true, true, true, false],
                    [255, 0, 0, 0, 0],
                    "Power",
                );
                self.add_image(
                    "Register",
                    [75_f32, 25_f32, 50_f32, 50_f32],
                    [1, 2, 7, 8],
                    [true, true, true, true, false],
                    [255, 0, 0, 0, 0],
                    "Register",
                );
                self.add_image(
                    "Login",
                    [0_f32, 25_f32, 50_f32, 50_f32],
                    [1, 2, 7, 8],
                    [true, true, true, true, false],
                    [255, 0, 0, 0, 0],
                    "Login",
                );
                self.add_image(
                    "Background",
                    [
                        0_f32,
                        0_f32,
                        ctx.available_rect().width(),
                        ctx.available_rect().height(),
                    ],
                    [1, 0, 1, 0],
                    [true, true, false, false, false],
                    [255, 0, 0, 0, 0],
                    "Background",
                );
                self.add_image(
                    "Wallpaper1",
                    [0_f32, 0_f32, 0_f32, 0_f32],
                    [1, 0, 1, 0],
                    [true, false, false, false, false],
                    [255, 0, 0, 0, 0],
                    "Background",
                );
                self.add_image(
                    "Wallpaper2",
                    [0_f32, 0_f32, 0_f32, 0_f32],
                    [1, 0, 1, 0],
                    [true, false, false, false, false],
                    [255, 0, 0, 0, 0],
                    "Background",
                );
                self.add_scroll_background(
                    "ScrollWallpaper",
                    vec!["Wallpaper1".to_string(), "Wallpaper2".to_string()],
                    true,
                    true,
                    3,
                    [
                        ctx.available_rect().width(),
                        ctx.available_rect().height(),
                        0_f32,
                        0_f32,
                        -ctx.available_rect().width(),
                    ],
                );
                self.add_switch(
                    ["Power", "Power"],
                    vec![],
                    [false, true, true],
                    1,
                    vec![[255, 255, 255, 255], [200, 200, 200, 255]],
                    vec![
                        PointerButton::Primary,
                        PointerButton::Secondary,
                        PointerButton::Middle,
                        PointerButton::Extra1,
                        PointerButton::Extra2,
                    ],
                );
                self.add_switch(
                    ["Register", "Register"],
                    vec![],
                    [false, true, true],
                    1,
                    vec![[255, 255, 255, 255], [200, 200, 200, 255]],
                    vec![
                        PointerButton::Primary,
                        PointerButton::Secondary,
                        PointerButton::Middle,
                        PointerButton::Extra1,
                        PointerButton::Extra2,
                    ],
                );
                self.add_switch(
                    ["Login", "Login"],
                    vec![],
                    [false, true, true],
                    1,
                    vec![[255, 255, 255, 255], [200, 200, 200, 255]],
                    vec![
                        PointerButton::Primary,
                        PointerButton::Secondary,
                        PointerButton::Middle,
                        PointerButton::Extra1,
                        PointerButton::Extra2,
                    ],
                );
            } else if i == 2 {
                self.add_image_texture(
                    "Home",
                    "Resources/assets/images/home.png",
                    [false, false],
                    true,
                    ctx,
                );
                self.add_image_texture(
                    "Settings",
                    "Resources/assets/images/settings.png",
                    [false, false],
                    true,
                    ctx,
                );
                self.add_image(
                    "Home_Home",
                    [0_f32, -20_f32, 50_f32, 50_f32],
                    [1, 3, 1, 1],
                    [true, false, true, false, false],
                    [255, 0, 0, 0, 0],
                    "Home",
                );
                self.add_image(
                    "Home_Settings",
                    [0_f32, -20_f32, 50_f32, 50_f32],
                    [2, 3, 1, 1],
                    [true, false, true, false, false],
                    [255, 0, 0, 0, 0],
                    "Settings",
                );
                self.add_switch(
                    ["Home_Home", "Home_Home"],
                    vec![],
                    [true, true, true],
                    1,
                    vec![
                        [255, 255, 255, 255],
                        [180, 180, 180, 255],
                        [150, 150, 150, 255],
                    ],
                    vec![PointerButton::Primary],
                );
                self.add_switch(
                    ["Home_Settings", "Home_Settings"],
                    vec![],
                    [true, true, true],
                    1,
                    vec![
                        [255, 255, 255, 255],
                        [180, 180, 180, 255],
                        [150, 150, 150, 255],
                    ],
                    vec![PointerButton::Primary],
                );
                self.add_rect(
                    "Dock_Background",
                    [
                        0_f32,
                        -10_f32,
                        ctx.available_rect().width() - 100_f32,
                        70_f32,
                        20_f32,
                    ],
                    [1, 2, 1, 1],
                    [true, false, true, false],
                    [150, 150, 150, 160, 255, 255, 255, 255],
                    0.0,
                );
            };
        }
    }

    pub fn check_updated(&mut self, name: &str) -> bool {
        if self.resource_page[track_resource(self.resource_page.clone(), name, "page")]
            .change_page_updated
        {
            true
        } else {
            self.new_page_update(name);
            false
        }
    }

    pub fn new_page_update(&mut self, name: &str) {
        self.renew_timer();
        let page = self.resource_page.clone();
        self.resource_page[track_resource(page, name, "page")].change_page_updated = true;
        self.timer.split_time = vec![];
    }

    pub fn dock(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let id = track_resource(self.resource_rect.clone(), "Dock_Background", "rect");
        if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
            let rect = egui::Rect::from_min_size(
                egui::Pos2::new(0_f32, ctx.available_rect().height() - 80_f32),
                egui::Vec2::new(ctx.available_rect().width(), 80_f32),
            );
            self.modify_var("dock_active_status", rect.contains(mouse_pos));
            let image = self.resource_image.clone();
            if self.var_b("dock_active_status") {
                for _ in 0..5 {
                    if self.resource_rect[id].origin_position[1] > -10_f32 {
                        for i in 0..self.resource_switch.len() {
                            if self.resource_switch[i].name.contains("Home_") {
                                self.resource_image[track_resource(
                                    image.clone(),
                                    &self.resource_switch[i].switch_image_name,
                                    "image",
                                )]
                                .origin_position[1] -= 1_f32;
                            };
                        }
                        self.resource_rect[id].origin_position[1] -= 1_f32;
                    } else {
                        break;
                    };
                }
            } else if !self.var_b("dock_active_status") {
                for _ in 0..5 {
                    if self.resource_rect[id].origin_position[1] < 80_f32 {
                        for i in 0..self.resource_switch.len() {
                            if self.resource_switch[i].name.contains("Home_") {
                                self.resource_image[track_resource(
                                    image.clone(),
                                    &self.resource_switch[i].switch_image_name,
                                    "image",
                                )]
                                .origin_position[1] += 1_f32;
                            };
                        }
                        self.resource_rect[id].origin_position[1] += 1_f32;
                    } else {
                        break;
                    };
                }
            };
            self.rect(ui, "Dock_Background", ctx);
            if self.switch("Home_Home", ui, ctx, true)[0] != 5 {
                self.new_page_update("Home_Page");
                self.switch_page("Home_Page");
            };
            if self.switch("Home_Settings", ui, ctx, true)[0] != 5 {
                self.new_page_update("Home_Setting");
                self.switch_page("Home_Setting");
            };
        };
        self.resource_rect[id].size[0] = ctx.available_rect().width() - 100_f32;
    }

    pub fn add_split_time(&mut self, name: &str) {
        self.timer.split_time.push(SplitTime {
            name: name.to_string(),
            time: [self.timer.now_time, self.timer.total_time],
        });
    }

    pub fn split_time(&mut self, name: &str) -> [f32; 2] {
        self.timer.split_time[track_resource(self.timer.split_time.clone(), name, "split_time")]
            .time
    }

    pub fn renew_timer(&mut self) {
        self.timer.start_time = self.timer.total_time;
        self.timer.split_time = vec![];
    }

    pub fn update_timer(&mut self) {
        let elapsed = self.timer.timer.elapsed();
        let seconds = elapsed.as_secs();
        let milliseconds = elapsed.subsec_millis();
        self.timer.total_time = seconds as f32 + milliseconds as f32 / 1000.0;
        self.timer.now_time = self.timer.total_time - self.timer.start_time
    }

    pub fn add_rect(
        &mut self,
        name: &str,
        position_size_and_rounding: [f32; 5],
        grid: [u32; 4],
        center_display: [bool; 4],
        color: [u8; 8],
        border_width: f32,
    ) {
        self.resource_rect.push(CustomRect {
            name: name.to_string(),
            position: [position_size_and_rounding[0], position_size_and_rounding[1]],
            size: [position_size_and_rounding[2], position_size_and_rounding[3]],
            rounding: position_size_and_rounding[4],
            x_grid: [grid[0], grid[1]],
            y_grid: [grid[2], grid[3]],
            center_display,
            color: [color[0], color[1], color[2], color[3]],
            border_width,
            border_color: [color[4], color[5], color[6], color[7]],
            origin_position: [position_size_and_rounding[0], position_size_and_rounding[1]],
        });
    }

    pub fn rect(&mut self, ui: &mut Ui, name: &str, ctx: &egui::Context) {
        let id = track_resource(self.resource_rect.clone(), name, "rect");
        self.resource_rect[id].position[0] = match self.resource_rect[id].x_grid[1] {
            0 => self.resource_rect[id].position[0],
            _ => {
                (ctx.available_rect().width() as f64 / self.resource_rect[id].x_grid[1] as f64
                    * self.resource_rect[id].x_grid[0] as f64) as f32
                    + self.resource_rect[id].origin_position[0]
            }
        };
        self.resource_rect[id].position[1] = match self.resource_rect[id].y_grid[1] {
            0 => self.resource_rect[id].position[1],
            _ => {
                (ctx.available_rect().height() as f64 / self.resource_rect[id].y_grid[1] as f64
                    * self.resource_rect[id].y_grid[0] as f64) as f32
                    + self.resource_rect[id].origin_position[1]
            }
        };
        let pos_x;
        let pos_y;
        if self.resource_rect[id].center_display[2] {
            pos_x = self.resource_rect[id].position[0] - self.resource_rect[id].size[0] / 2.0;
        } else if self.resource_rect[id].center_display[0] {
            pos_x = self.resource_rect[id].position[0];
        } else {
            pos_x = self.resource_rect[id].position[0] - self.resource_rect[id].size[0];
        };
        if self.resource_rect[id].center_display[3] {
            pos_y = self.resource_rect[id].position[1] - self.resource_rect[id].size[1] / 2.0;
        } else if self.resource_rect[id].center_display[1] {
            pos_y = self.resource_rect[id].position[1];
        } else {
            pos_y = self.resource_rect[id].position[1] - self.resource_rect[id].size[1];
        };
        ui.painter().rect(
            Rect::from_min_max(
                Pos2::new(pos_x, pos_y),
                Pos2::new(
                    pos_x + self.resource_rect[id].size[0],
                    pos_y + self.resource_rect[id].size[1],
                ),
            ),
            self.resource_rect[id].rounding,
            Color32::from_rgba_premultiplied(
                self.resource_rect[id].color[0],
                self.resource_rect[id].color[1],
                self.resource_rect[id].color[2],
                self.resource_rect[id].color[3],
            ),
            Stroke {
                width: self.resource_rect[id].border_width,
                color: Color32::from_rgba_premultiplied(
                    self.resource_rect[id].border_color[0],
                    self.resource_rect[id].border_color[1],
                    self.resource_rect[id].border_color[2],
                    self.resource_rect[id].border_color[3],
                ),
            },
        );
    }

    pub fn add_text(
        &mut self,
        name_and_content: [&str; 2],
        position_font_size_wrap_width_rounding: [f32; 5],
        color: [u8; 7],
        center_display: [bool; 4],
        write_background: bool,
        grid: [u32; 4],
    ) {
        self.resource_text.push(Text {
            name: name_and_content[0].to_string(),
            text_content: name_and_content[1].to_string(),
            font_size: position_font_size_wrap_width_rounding[2],
            rgba: [color[0], color[1], color[2], color[3]],
            position: [
                position_font_size_wrap_width_rounding[0],
                position_font_size_wrap_width_rounding[1],
            ],
            center_display,
            wrap_width: position_font_size_wrap_width_rounding[3],
            write_background,
            background_rgb: [color[4], color[5], color[6]],
            rounding: position_font_size_wrap_width_rounding[4],
            x_grid: [grid[0], grid[1]],
            y_grid: [grid[2], grid[3]],
            origin_position: [
                position_font_size_wrap_width_rounding[0],
                position_font_size_wrap_width_rounding[1],
            ],
        });
    }

    pub fn text(&mut self, ui: &mut Ui, name: &str, ctx: &egui::Context) {
        let id = track_resource(self.resource_text.clone(), name, "text");
        // 计算文本大小
        let galley = ui.fonts(|f| {
            f.layout(
                self.resource_text[id].text_content.to_string(),
                FontId::proportional(self.resource_text[id].font_size),
                Color32::from_rgba_unmultiplied(
                    self.resource_text[id].rgba[0],
                    self.resource_text[id].rgba[1],
                    self.resource_text[id].rgba[2],
                    self.resource_text[id].rgba[3],
                ),
                self.resource_text[id].wrap_width,
            )
        });
        let text_size = galley.size();
        self.resource_text[id].position[0] = match self.resource_text[id].x_grid[1] {
            0 => self.resource_text[id].position[0],
            _ => {
                (ctx.available_rect().width() as f64 / self.resource_text[id].x_grid[1] as f64
                    * self.resource_text[id].x_grid[0] as f64) as f32
                    + self.resource_text[id].origin_position[0]
            }
        };
        self.resource_text[id].position[1] = match self.resource_text[id].y_grid[1] {
            0 => self.resource_text[id].position[1],
            _ => {
                (ctx.available_rect().height() as f64 / self.resource_text[id].y_grid[1] as f64
                    * self.resource_text[id].y_grid[0] as f64) as f32
                    + self.resource_text[id].origin_position[1]
            }
        };
        let pos_x;
        let pos_y;
        if self.resource_text[id].center_display[2] {
            pos_x = self.resource_text[id].position[0] - text_size.x / 2.0;
        } else if self.resource_text[id].center_display[0] {
            pos_x = self.resource_text[id].position[0];
        } else {
            pos_x = self.resource_text[id].position[0] - text_size.x;
        };
        if self.resource_text[id].center_display[3] {
            pos_y = self.resource_text[id].position[1] - text_size.y / 2.0;
        } else if self.resource_text[id].center_display[1] {
            pos_y = self.resource_text[id].position[1];
        } else {
            pos_y = self.resource_text[id].position[1] - text_size.y;
        };
        // 使用绝对定位放置文本
        let position = Pos2::new(pos_x, pos_y);
        if self.resource_text[id].write_background {
            let rect = Rect::from_min_size(position, text_size);
            // 绘制背景颜色
            ui.painter().rect_filled(
                rect,
                self.resource_text[id].rounding,
                Color32::from_rgb(
                    self.resource_text[id].background_rgb[0],
                    self.resource_text[id].background_rgb[1],
                    self.resource_text[id].background_rgb[2],
                ),
            ); // 背景色
        };
        // 绘制文本
        ui.painter().galley(
            position,
            galley,
            Color32::from_rgba_unmultiplied(
                self.resource_text[id].rgba[0],
                self.resource_text[id].rgba[1],
                self.resource_text[id].rgba[2],
                self.resource_text[id].rgba[3], // 应用透明度
            ),
        );
    }

    fn read_image_to_vec(&mut self, path: &str) -> Vec<u8> {
        let mut file = File::open(path).expect("打开图片文件失败");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("读取图片文件失败");
        buffer
    }

    pub fn add_var<T: Into<Value>>(&mut self, name: &str, value: T) {
        self.variables.push(Variable {
            name: name.to_string(),
            value: value.into(),
        });
    }

    #[allow(dead_code)]
    pub fn modify_var<T: Into<Value>>(&mut self, name: &str, value: T) {
        let id = self.variables.clone();
        self.variables[track_resource(id, name, "variables")].value = value.into();
    }

    #[allow(dead_code)]
    pub fn var(&mut self, name: &str) -> Value {
        self.variables[track_resource(self.variables.clone(), name, "variables")]
            .clone()
            .value
    }

    pub fn var_i(&mut self, name: &str) -> i32 {
        match &self.variables[track_resource(self.variables.clone(), name, "variables")].value {
            // 直接访问 value 字段
            Value::Int(i) => *i,
            _ => panic!("RustConstructor Error[Variable load failed]: The variable \"{}\" is not of type i32", name),
        }
    }

    #[allow(dead_code)]
    pub fn var_u(&mut self, name: &str) -> u32 {
        match &self.variables[track_resource(self.variables.clone(), name, "variables")].value {
            Value::UInt(u) => *u,
            _ => panic!("RustConstructor Error[Variable load failed]: The variable \"{}\" is not of type u32", name),
        }
    }

    #[allow(dead_code)]
    pub fn var_f(&mut self, name: &str) -> f32 {
        match &self.variables[track_resource(self.variables.clone(), name, "variables")].value {
            Value::Float(f) => *f,
            _ => panic!("RustConstructor Error[Variable load failed]: The variable \"{}\" is not of type f32", name),
        }
    }

    pub fn var_b(&mut self, name: &str) -> bool {
        match &self.variables[track_resource(self.variables.clone(), name, "variables")].value {
            Value::Bool(b) => *b,
            _ => panic!("RustConstructor Error[Variable load failed]: The variable \"{}\" is not of type bool", name),
        }
    }

    pub fn var_v(&mut self, name: &str) -> Vec<Value> {
        match &self.variables[track_resource(self.variables.clone(), name, "variables")].value {
            Value::Vec(v) => v.clone(),
            _ => panic!("RustConstructor Error[Variable load failed]: The variable \"{}\" is not of type Vec", name),
        }
    }

    pub fn var_s(&mut self, name: &str) -> String {
        match &self.variables[track_resource(self.variables.clone(), name, "variables")].value {
            Value::String(s) => s.clone(),
            _ => panic!("RustConstructor Error[Variable load failed]: The variable \"{}\" is not of type String", name),
        }
    }

    #[allow(dead_code)]
    pub fn var_decode_b(&mut self, target: Value) -> bool {
        match target {
            Value::Bool(b) => {
                // 处理布尔值
                b
            }
            _ => {
                panic!("RustConstructor Error[Variable decode failed]: The variable should be of type bool");
            }
        }
    }

    #[allow(dead_code)]
    pub fn var_decode_i(&mut self, target: Value) -> i32 {
        match target {
            Value::Int(i) => {
                // 处理i32整型
                i
            }
            _ => {
                panic!("RustConstructor Error[Variable decode failed]: The variable should be of type i32");
            }
        }
    }

    #[allow(dead_code)]
    pub fn var_decode_u(&mut self, target: Value) -> u32 {
        match target {
            Value::UInt(u) => {
                // 处理u32无符号整型
                u
            }
            _ => {
                panic!("RustConstructor Error[Variable decode failed]: The variable should be of type u32");
            }
        }
    }

    pub fn var_decode_f(&mut self, target: Value) -> f32 {
        match target {
            Value::Float(f) => {
                // 处理浮点数
                f
            }
            _ => {
                panic!("RustConstructor Error[Variable decode failed]: The variable should be of type f32");
            }
        }
    }

    #[allow(dead_code)]
    pub fn var_decode_s(&mut self, target: Value) -> String {
        match target {
            Value::String(s) => {
                // 处理字符串
                s
            }
            _ => {
                panic!("RustConstructor Error[Variable decode failed]: The variable should be of type String");
            }
        }
    }

    pub fn add_scroll_background(
        &mut self,
        name: &str,
        image_name: Vec<String>,
        horizontal_or_vertical: bool,
        left_and_top_or_right_and_bottom: bool,
        scroll_speed: u32,
        size_position_boundary: [f32; 5],
    ) {
        let mut image_id = vec![];
        for i in image_name.clone().into_iter() {
            image_id.push(track_resource(self.resource_image.clone(), &i, "image"));
            continue;
        }
        for (count, _i) in image_id.clone().into_iter().enumerate() {
            self.resource_image[image_id[count]].x_grid = [0, 0];
            self.resource_image[image_id[count]].y_grid = [0, 0];
            self.resource_image[image_id[count]].center_display = [true, true, false, false];
            self.resource_image[image_id[count]].image_size =
                [size_position_boundary[0], size_position_boundary[1]];
            let mut temp_position;
            if horizontal_or_vertical {
                temp_position = size_position_boundary[2];
            } else {
                temp_position = size_position_boundary[3];
            };
            if horizontal_or_vertical {
                for _j in 0..count {
                    if left_and_top_or_right_and_bottom {
                        temp_position += size_position_boundary[0];
                    } else {
                        temp_position -= size_position_boundary[0];
                    };
                }
                self.resource_image[image_id[count]].image_position =
                    [temp_position, size_position_boundary[3]];
            } else {
                for _j in 0..count {
                    if left_and_top_or_right_and_bottom {
                        temp_position += size_position_boundary[1];
                    } else {
                        temp_position -= size_position_boundary[1];
                    };
                }
                self.resource_image[image_id[count]].image_position =
                    [size_position_boundary[2], temp_position];
            };
        }
        let resume_point = if horizontal_or_vertical {
            self.resource_image[image_id[image_id.len() - 1]].image_position[0]
        } else {
            self.resource_image[image_id[image_id.len() - 1]].image_position[1]
        };
        self.resource_scroll_background.push(ScrollBackground {
            name: name.to_string(),
            image_name,
            horizontal_or_vertical,
            left_and_top_or_right_and_bottom,
            scroll_speed,
            boundary: size_position_boundary[4],
            resume_point,
        });
    }

    pub fn scroll_background(&mut self, ui: &mut Ui, name: &str, ctx: &egui::Context) {
        let id = track_resource(
            self.resource_scroll_background.clone(),
            name,
            "scroll_background",
        );
        let mut id2;
        for i in 0..self.resource_scroll_background[id].image_name.len() {
            self.image(
                ui,
                &self.resource_scroll_background[id].image_name[i].clone(),
                ctx,
            );
        }
        for i in 0..self.resource_scroll_background[id].image_name.len() {
            id2 = track_resource(
                self.resource_image.clone(),
                &self.resource_scroll_background[id].image_name[i].clone(),
                "image",
            );
            if self.resource_scroll_background[id].horizontal_or_vertical {
                if self.resource_scroll_background[id].left_and_top_or_right_and_bottom {
                    for _j in 0..self.resource_scroll_background[id].scroll_speed {
                        self.resource_image[id2].image_position[0] -= 1_f32;
                        self.scroll_background_check_boundary(id, id2);
                    }
                } else {
                    for _j in 0..self.resource_scroll_background[id].scroll_speed {
                        self.resource_image[id2].image_position[0] += 1_f32;
                        self.scroll_background_check_boundary(id, id2);
                    }
                };
            } else if self.resource_scroll_background[id].left_and_top_or_right_and_bottom {
                for _j in 0..self.resource_scroll_background[id].scroll_speed {
                    self.resource_image[id2].image_position[1] -= 1_f32;
                    self.scroll_background_check_boundary(id, id2);
                }
            } else {
                for _j in 0..self.resource_scroll_background[id].scroll_speed {
                    self.resource_image[id2].image_position[1] += 1_f32;
                    self.scroll_background_check_boundary(id, id2);
                }
            };
        }
    }

    fn scroll_background_check_boundary(&mut self, id: usize, id2: usize) {
        if self.resource_scroll_background[id].horizontal_or_vertical {
            if self.resource_scroll_background[id].left_and_top_or_right_and_bottom {
                if self.resource_image[id2].image_position[0]
                    <= self.resource_scroll_background[id].boundary
                {
                    self.resource_image[id2].image_position[0] =
                        self.resource_scroll_background[id].resume_point;
                };
            } else if self.resource_image[id2].image_position[0]
                >= self.resource_scroll_background[id].boundary
            {
                self.resource_image[id2].image_position[0] =
                    self.resource_scroll_background[id].resume_point;
            };
        } else if self.resource_scroll_background[id].left_and_top_or_right_and_bottom {
            if self.resource_image[id2].image_position[1]
                <= self.resource_scroll_background[id].boundary
            {
                self.resource_image[id2].image_position[1] =
                    self.resource_scroll_background[id].resume_point;
            };
        } else if self.resource_image[id2].image_position[1]
            >= self.resource_scroll_background[id].boundary
        {
            self.resource_image[id2].image_position[1] =
                self.resource_scroll_background[id].resume_point;
        };
    }

    pub fn add_image_texture(
        &mut self,
        name: &str,
        path: &str,
        flip: [bool; 2],
        create_new_resource: bool,
        ctx: &egui::Context,
    ) {
        let img_bytes = self.read_image_to_vec(path);
        let img = image::load_from_memory(&img_bytes).unwrap();
        let rgba_data = match flip {
            [true, true] => img.fliph().flipv().into_rgba8(),
            [true, false] => img.fliph().into_rgba8(),
            [false, true] => img.flipv().into_rgba8(),
            _ => img.into_rgba8(),
        };
        let (w, h) = (rgba_data.width(), rgba_data.height());
        let raw_data: Vec<u8> = rgba_data.into_raw();

        let color_image =
            egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &raw_data);
        let image_texture = Some(ctx.load_texture("", color_image, TextureOptions::LINEAR));
        if create_new_resource {
            self.resource_image_texture.push(ImageTexture {
                name: name.to_string(),
                texture: image_texture,
            });
        } else {
            let id = self.resource_image_texture.clone();
            self.resource_image_texture[track_resource(id, name, "image_texture")].texture =
                image_texture;
        };
    }

    pub fn add_image(
        &mut self,
        name: &str,
        position_size: [f32; 4],
        grid: [u32; 4],
        center_display_and_use_overlay: [bool; 5],
        alpha_and_overlay_color: [u8; 5],
        image_texture_name: &str,
    ) {
        self.resource_image.push(Image {
            name: name.to_string(),
            image_texture: self.resource_image_texture[track_resource(
                self.resource_image_texture.clone(),
                image_texture_name,
                "image_texture",
            )]
            .texture
            .clone(),
            image_position: [position_size[0], position_size[1]],
            image_size: [position_size[2], position_size[3]],
            x_grid: [grid[0], grid[1]],
            y_grid: [grid[2], grid[3]],
            center_display: [
                center_display_and_use_overlay[0],
                center_display_and_use_overlay[1],
                center_display_and_use_overlay[2],
                center_display_and_use_overlay[3],
            ],
            alpha: alpha_and_overlay_color[0],
            overlay_color: [
                alpha_and_overlay_color[1],
                alpha_and_overlay_color[2],
                alpha_and_overlay_color[3],
                alpha_and_overlay_color[4],
            ],
            use_overlay_color: center_display_and_use_overlay[4],
            origin_position: [position_size[0], position_size[1]],
        });
    }

    pub fn image(&mut self, ui: &Ui, name: &str, ctx: &egui::Context) {
        let id = track_resource(self.resource_image.clone(), name, "image");
        self.resource_image[id].image_position[0] = match self.resource_image[id].x_grid[1] {
            0 => self.resource_image[id].image_position[0],
            _ => {
                (ctx.available_rect().width() as f64 / self.resource_image[id].x_grid[1] as f64
                    * self.resource_image[id].x_grid[0] as f64) as f32
                    + self.resource_image[id].origin_position[0]
            }
        };
        self.resource_image[id].image_position[1] = match self.resource_image[id].y_grid[1] {
            0 => self.resource_image[id].image_position[1],
            _ => {
                (ctx.available_rect().height() as f64 / self.resource_image[id].y_grid[1] as f64
                    * self.resource_image[id].y_grid[0] as f64) as f32
                    + self.resource_image[id].origin_position[1]
            }
        };
        if self.resource_image[id].center_display[2] {
            self.resource_image[id].image_position[0] -=
                self.resource_image[id].image_size[0] / 2.0;
        } else if !self.resource_image[id].center_display[0] {
            self.resource_image[id].image_position[0] -= self.resource_image[id].image_size[0];
        };
        if self.resource_image[id].center_display[3] {
            self.resource_image[id].image_position[1] -=
                self.resource_image[id].image_size[1] / 2.0;
        } else if !self.resource_image[id].center_display[1] {
            self.resource_image[id].image_position[1] -= self.resource_image[id].image_size[1];
        };
        if let Some(texture) = &self.resource_image[id].image_texture {
            let rect = Rect::from_min_size(
                Pos2::new(
                    self.resource_image[id].image_position[0],
                    self.resource_image[id].image_position[1],
                ),
                Vec2::new(
                    self.resource_image[id].image_size[0],
                    self.resource_image[id].image_size[1],
                ),
            );
            let color = if self.resource_image[id].use_overlay_color {
                // 创建颜色覆盖
                Color32::from_rgba_unmultiplied(
                    self.resource_image[id].overlay_color[0],
                    self.resource_image[id].overlay_color[1],
                    self.resource_image[id].overlay_color[2],
                    // 将图片透明度与覆盖颜色透明度相乘
                    (self.resource_image[id].alpha as f32
                        * self.resource_image[id].overlay_color[3] as f32
                        / 255.0) as u8,
                )
            } else {
                Color32::from_white_alpha(self.resource_image[id].alpha)
            };

            ui.painter().image(
                texture.into(),
                rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                color,
            );
        };
    }

    #[allow(dead_code)]
    pub fn add_switch(
        &mut self,
        name_and_switch_image_name: [&str; 2],
        switch_texture_name: Vec<String>,
        enable_hover_click_image_and_use_overlay: [bool; 3],
        switch_amounts_state: u32,
        overlay_color: Vec<[u8; 4]>,
        click_method: Vec<PointerButton>,
    ) {
        let mut count = 1;
        if enable_hover_click_image_and_use_overlay[0] {
            count += 1;
        };
        if enable_hover_click_image_and_use_overlay[1] {
            count += 1;
        };
        if enable_hover_click_image_and_use_overlay[2] {
            if overlay_color.len() as u32 != count * switch_amounts_state {
                panic!(
                    "RustConstructor Error[Switch load failed]: \"{}\" switch is missing/extra {} resources!",
                    name_and_switch_image_name[0],
                    count * switch_amounts_state - overlay_color.len() as u32
                );
            };
            let id = self.resource_image.clone();
            self.resource_image[track_resource(id, name_and_switch_image_name[1], "image")]
                .use_overlay_color = true;
        } else if switch_texture_name.len() as u32 != count * switch_amounts_state {
            panic!(
                "RustConstructor Error[Switch load failed]: \"{}\" switch is missing/extra {} resources!",
                name_and_switch_image_name[0],
                count * switch_amounts_state - switch_texture_name.len() as u32
            );
        };
        self.resource_switch.push(Switch {
            name: name_and_switch_image_name[0].to_string(),
            switch_texture_name,
            switch_image_name: name_and_switch_image_name[1].to_string(),
            enable_hover_click_image: [
                enable_hover_click_image_and_use_overlay[0],
                enable_hover_click_image_and_use_overlay[1],
            ],
            state: 0,
            use_overlay: enable_hover_click_image_and_use_overlay[2],
            overlay_color,
            click_method,
            last_time_clicked: false,
            last_time_clicked_index: 0,
        });
    }

    #[allow(dead_code)]
    pub fn switch(
        &mut self,
        name: &str,
        ui: &mut Ui,
        ctx: &egui::Context,
        enable: bool,
    ) -> [usize; 2] {
        let mut activated = [5, 0];
        let id = track_resource(self.resource_switch.clone(), name, "switch");
        let id2 = track_resource(
            self.resource_image.clone(),
            &self.resource_switch[id].switch_image_name.clone(),
            "image",
        );
        let id3;
        let rect = Rect::from_min_size(
            Pos2::new(
                self.resource_image[id2].image_position[0],
                self.resource_image[id2].image_position[1],
            ),
            Vec2::new(
                self.resource_image[id2].image_size[0],
                self.resource_image[id2].image_size[1],
            ),
        );
        if enable {
            if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                // 判断是否在矩形内
                if rect.contains(mouse_pos) {
                    let mut clicked = vec![];
                    let mut active = false;
                    for u in 0..self.resource_switch[id].click_method.len() as u32 {
                        clicked.push(ui.input(|i| {
                            i.pointer
                                .button_down(self.resource_switch[id].click_method[u as usize])
                        }));
                        if clicked[u as usize] {
                            active = true;
                            self.resource_switch[id].last_time_clicked_index = u as usize;
                            break;
                        };
                    }
                    if active {
                        self.resource_switch[id].last_time_clicked = true;
                        if self.resource_switch[id].enable_hover_click_image[1] {
                            if self.resource_switch[id].use_overlay {
                                if self.resource_switch[id].enable_hover_click_image[0] {
                                    self.resource_image[id2].overlay_color =
                                        self.resource_switch[id].overlay_color
                                            [(self.resource_switch[id].state + 2) as usize];
                                } else {
                                    self.resource_image[id2].overlay_color =
                                        self.resource_switch[id].overlay_color
                                            [(self.resource_switch[id].state + 1) as usize];
                                };
                            } else {
                                if self.resource_switch[id].enable_hover_click_image[0] {
                                    id3 = track_resource(
                                        self.resource_image_texture.clone(),
                                        &self.resource_switch[id].switch_texture_name
                                            [(self.resource_switch[id].state + 2) as usize]
                                            .clone(),
                                        "image_texture",
                                    );
                                } else {
                                    id3 = track_resource(
                                        self.resource_image_texture.clone(),
                                        &self.resource_switch[id].switch_texture_name
                                            [(self.resource_switch[id].state + 1) as usize]
                                            .clone(),
                                        "image_texture",
                                    );
                                };
                                self.resource_image[id2].image_texture =
                                    self.resource_image_texture[id3].texture.clone();
                            };
                        } else if !self.resource_switch[id].enable_hover_click_image[0] {
                            if self.resource_switch[id].use_overlay {
                                self.resource_image[id2].overlay_color = self.resource_switch[id]
                                    .overlay_color
                                    [self.resource_switch[id].state as usize];
                            } else {
                                id3 = track_resource(
                                    self.resource_image_texture.clone(),
                                    &self.resource_switch[id].switch_texture_name
                                        [self.resource_switch[id].state as usize]
                                        .clone(),
                                    "image_texture",
                                );
                                self.resource_image[id2].image_texture =
                                    self.resource_image_texture[id3].texture.clone();
                            };
                        };
                    } else {
                        if self.resource_switch[id].last_time_clicked {
                            let mut count = 1;
                            if self.resource_switch[id].enable_hover_click_image[0] {
                                count += 1;
                            };
                            if self.resource_switch[id].enable_hover_click_image[1] {
                                count += 1;
                            };
                            if self.resource_switch[id].use_overlay {
                                if self.resource_switch[id].state
                                    < (self.resource_switch[id].overlay_color.len() / count - 1)
                                        as u32
                                {
                                    self.resource_switch[id].state += 1;
                                } else {
                                    self.resource_switch[id].state = 0;
                                };
                            } else if self.resource_switch[id].state
                                < (self.resource_switch[id].switch_texture_name.len() / count - 1)
                                    as u32
                            {
                                self.resource_switch[id].state += 1;
                            } else {
                                self.resource_switch[id].state = 0;
                            };
                            activated[0] = self.resource_switch[id].last_time_clicked_index;
                            self.resource_switch[id].last_time_clicked = false;
                        };
                        if self.resource_switch[id].enable_hover_click_image[0] {
                            if self.resource_switch[id].use_overlay {
                                self.resource_image[id2].overlay_color = self.resource_switch[id]
                                    .overlay_color
                                    [(self.resource_switch[id].state + 1) as usize];
                            } else {
                                id3 = track_resource(
                                    self.resource_image_texture.clone(),
                                    &self.resource_switch[id].switch_texture_name
                                        [(self.resource_switch[id].state + 1) as usize]
                                        .clone(),
                                    "image_texture",
                                );
                                self.resource_image[id2].image_texture =
                                    self.resource_image_texture[id3].texture.clone();
                            };
                        } else if self.resource_switch[id].use_overlay {
                            self.resource_image[id2].overlay_color = self.resource_switch[id]
                                .overlay_color
                                [self.resource_switch[id].state as usize];
                        } else {
                            id3 = track_resource(
                                self.resource_image_texture.clone(),
                                &self.resource_switch[id].switch_texture_name
                                    [self.resource_switch[id].state as usize]
                                    .clone(),
                                "image_texture",
                            );
                            self.resource_image[id2].image_texture =
                                self.resource_image_texture[id3].texture.clone();
                        };
                    };
                } else {
                    self.resource_switch[id].last_time_clicked = false;
                    if self.resource_switch[id].use_overlay {
                        self.resource_image[id2].overlay_color = self.resource_switch[id]
                            .overlay_color[self.resource_switch[id].state as usize];
                    } else {
                        id3 = track_resource(
                            self.resource_image_texture.clone(),
                            &self.resource_switch[id].switch_texture_name
                                [self.resource_switch[id].state as usize]
                                .clone(),
                            "image_texture",
                        );
                        self.resource_image[id2].image_texture =
                            self.resource_image_texture[id3].texture.clone();
                    };
                };
            };
        } else {
            self.resource_switch[id].last_time_clicked = false;
            if self.resource_switch[id].use_overlay {
                self.resource_image[id2].overlay_color =
                    self.resource_switch[id].overlay_color[self.resource_switch[id].state as usize];
            } else {
                id3 = track_resource(
                    self.resource_image_texture.clone(),
                    &self.resource_switch[id].switch_texture_name
                        [self.resource_switch[id].state as usize]
                        .clone(),
                    "image_texture",
                );
                self.resource_image[id2].image_texture =
                    self.resource_image_texture[id3].texture.clone();
            };
        };
        self.image(ui, &self.resource_switch[id].switch_image_name.clone(), ctx);
        activated[1] = self.resource_switch[id].state as usize;
        activated
    }
}
