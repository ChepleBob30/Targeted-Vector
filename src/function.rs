//! function.rs is the functional module of the Targeted Vector, including function declarations, struct definitions, and some auxiliary content.
use anyhow::Context;
use eframe::emath::Rect;
use eframe::epaint::textures::TextureOptions;
use eframe::epaint::Stroke;
use egui::{Color32, FontId, PointerButton, Pos2, Ui, Vec2};
use json::{object, JsonValue};
use rodio::{Decoder, OutputStream};
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read};
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

#[allow(dead_code)]
pub fn write_to_json<P: AsRef<std::path::Path>>(path: P, config: &Config) {
    let json_value = config.to_json_value();
    let serialized = json_value.dump(); // 将 JSON 值序列化为字符串
    if let Err(e) = fs::write(path, serialized) {
        eprintln!("写入文件失败: {}", e);
    }
}

pub fn read_from_json<P: AsRef<std::path::Path>>(path: P) -> Option<Config> {
    match fs::read_to_string(path) {
        Ok(data) => match json::parse(&data) {
            Ok(parsed) => Config::from_json_value(&parsed),
            Err(e) => {
                eprintln!("解析JSON失败: {}", e);
                None
            }
        },
        Err(e) => {
            eprintln!("阅读文件失败: {}", e);
            None
        }
    }
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
#[derive(Debug)]
pub struct Config {
    pub launch_path: String,
}

#[allow(dead_code)]
impl Config {
    fn to_json_value(&self) -> JsonValue {
        object! {
            launch_path: self.launch_path.clone()
        }
    }

    fn from_json_value(value: &JsonValue) -> Option<Config> {
        Some(Config {
            launch_path: value["launch_path"].as_str()?.to_string(),
        })
    }
}

#[allow(dead_code)]
pub trait RustConstructorResource {}

pub struct PageData {
    pub forced_update: bool,
    pub change_page_updated: bool,
}

pub struct Timer {
    pub start_time: f32,
    pub total_time: f32,
    pub timer: Instant,
    pub now_time: f32,
    pub split_time: Vec<SplitTime>,
}

impl RustConstructorResource for ImageTexture {}
#[derive(Clone)]
pub struct ImageTexture {
    name: String,
    pub texture: Option<egui::TextureHandle>,
}

impl RustConstructorResource for CustomRect {}
#[derive(Clone)]
pub struct CustomRect {
    name: String,
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub rounding: f32,
    pub x_grid: [u32; 2],
    pub y_grid: [u32; 2],
    pub center_display: [bool; 4],
    pub color: [u8; 4],
    pub border_width: f32,
    pub border_color: [u8; 4],
}

impl RustConstructorResource for Image {}
#[derive(Clone)]
pub struct Image {
    name: String,
    pub image_texture: Option<egui::TextureHandle>,
    pub image_position: [f32; 2],
    pub image_size: [f32; 2],
    pub x_grid: [u32; 2],
    pub y_grid: [u32; 2],
    pub center_display: [bool; 4],
    pub alpha: u8,
    pub overlay_color: [u8; 4],
    pub use_overlay_color: bool,
}

impl RustConstructorResource for Text {}
#[derive(Clone)]
pub struct Text {
    name: String,
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
}

impl RustConstructorResource for ScrollBackground {}
#[derive(Clone)]
pub struct ScrollBackground {
    name: String,
    pub image_name: Vec<String>,
    pub horizontal_or_vertical: bool,
    pub left_and_top_or_right_and_bottom: bool,
    pub scroll_speed: u32,
    pub boundary: f32,
    pub resume_point: f32,
}

pub struct Variable {
    pub progress: i8,
}

impl RustConstructorResource for SplitTime {}
#[derive(Clone)]
pub struct SplitTime {
    name: String,
    pub time: [f32; 2],
}

impl RustConstructorResource for Switch {}
#[derive(Clone)]
#[allow(dead_code)]
pub struct Switch {
    name: String,
    pub switch_image_name: String,
    pub switch_texture_name: Vec<String>,
    pub enable_hover_click_image: [bool; 2],
    pub state: u32,
    pub use_overlay: bool,
    pub overlay_color: Vec<[u8; 4]>,
    pub click_method: PointerButton,
    pub last_time_clicked: bool
}

#[allow(dead_code)]
pub struct App {
    pub config: Config,
    pub page_id: i32,
    pub page: Page,
    pub page_status: [PageData; 4],
    pub resource_image: Vec<Image>,
    pub resource_text: Vec<Text>,
    pub resource_rect: Vec<CustomRect>,
    pub resource_scroll_background: Vec<ScrollBackground>,
    pub timer: Timer,
    pub last_window_size: [f32; 2],
    pub variables: Variable,
    pub resource_image_texture: Vec<ImageTexture>,
    pub resource_switch: Vec<Switch>,
}

pub enum Page {
    Launch,
    Home,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        load_fonts(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let mut config = Config {
            launch_path: "".to_string(),
        };
        // 写入 JSON 文件
        // write_to_json("Resources/config/Preferences.json", &config);
        // 读取 JSON 文件
        if let Some(read_config) = read_from_json("Resources/config/Preferences.json") {
            config = read_config
        } else {
            eprintln!("阅读/解析 JSON 文件失败");
        };
        Self {
            config,
            page_id: 0,
            page: Page::Launch,
            page_status: [
                PageData {
                    forced_update: true,
                    change_page_updated: false,
                },
                PageData {
                    forced_update: true,
                    change_page_updated: false,
                },
                PageData {
                    forced_update: true,
                    change_page_updated: false,
                },
                PageData {
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
            last_window_size: [0.0, 0.0],
            variables: Variable { progress: 0 },
            resource_image_texture: vec![],
            resource_switch: vec![],
        }
    }

    pub fn launch_page_preload(&mut self, ctx: &egui::Context) {
        for i in 0..self.page_status.len() {
            if i == 0 {
                self.add_image_texture(
                    "RC_Logo",
                    "Resources/assets/images/RC.png",
                    [false, false],
                    ctx,
                );
                self.add_image_texture(
                    "Binder_Logo",
                    "Resources/assets/images/Binder.png",
                    [false, false],
                    ctx,
                );
                self.add_image_texture(
                    "Mouse",
                    "Resources/assets/images/Mouse_white.png",
                    [false, false],
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
                    ["Organize", " 必达\n Binder"],
                    [0_f32, 0_f32, 40_f32, 1000_f32, 0.0],
                    [255, 255, 255, 0, 0, 0, 0],
                    [true, false, false, true],
                    false,
                    [1, 2, 1, 2],
                );
                self.add_text(
                    ["Mouse", " 请连接鼠标\n 以获得最佳游戏体验"],
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
                self.add_image_texture(
                    "Background",
                    "Resources/assets/images/wallpaper.jpg",
                    [false, false],
                    ctx,
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
                self.add_text(
                    ["Title", "靶向载体 v0.1.0"],
                    [0_f32, 0_f32, 70_f32, 1000_f32, 0.0],
                    [255, 255, 255, 255, 0, 0, 0],
                    [false, true, true, false],
                    true,
                    [1, 2, 1, 4],
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
            };
        }
    }

    pub fn new_page_update(&mut self, page_id: i32) {
        self.renew_timer();
        self.page_id = page_id;
        self.page_status[page_id as usize].change_page_updated = true;
        self.timer.split_time = vec![];
    }

    pub fn add_split_time(&mut self, name: &str) {
        self.timer.split_time.push(SplitTime {
            name: name.to_string(),
            time: [self.timer.now_time, self.timer.total_time],
        });
    }

    pub fn split_time(&mut self, name: &str) -> [f32; 2] {
        let id = self.track_resource("split_time", name);
        self.timer.split_time[id].time
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
        });
    }

    pub fn rect(&mut self, ui: &mut Ui, name: &str, ctx: &egui::Context) {
        let id = self.track_resource("rect", name);
        self.resource_rect[id].position[0] = match self.resource_rect[id].x_grid[1] {
            0 => self.resource_rect[id].position[0],
            _ => {
                (ctx.available_rect().width() as f64 / self.resource_rect[id].x_grid[1] as f64
                    * self.resource_rect[id].x_grid[0] as f64) as f32
            }
        };
        self.resource_rect[id].position[1] = match self.resource_rect[id].y_grid[1] {
            0 => self.resource_rect[id].position[1],
            _ => {
                (ctx.available_rect().height() as f64 / self.resource_rect[id].y_grid[1] as f64
                    * self.resource_rect[id].y_grid[0] as f64) as f32
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
        });
    }

    pub fn text(&mut self, ui: &mut Ui, name: &str, ctx: &egui::Context) {
        let id = self.track_resource("text", name);
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
            }
        };
        self.resource_text[id].position[1] = match self.resource_text[id].y_grid[1] {
            0 => self.resource_text[id].position[1],
            _ => {
                (ctx.available_rect().height() as f64 / self.resource_text[id].y_grid[1] as f64
                    * self.resource_text[id].y_grid[0] as f64) as f32
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

    pub fn track_resource(&mut self, resource_list_name: &str, resource_name: &str) -> usize {
        let mut id = 0;
        match resource_list_name.to_lowercase().as_str() {
            "image" => {
                for i in 0..self.resource_image.len() {
                    if self.resource_image[i].name == resource_name {
                        id = i;
                        break;
                    }
                }
            }
            "text" => {
                for i in 0..self.resource_text.len() {
                    if self.resource_text[i].name == resource_name {
                        id = i;
                        break;
                    }
                }
            }
            "rect" => {
                for i in 0..self.resource_rect.len() {
                    if self.resource_rect[i].name == resource_name {
                        id = i;
                        break;
                    }
                }
            }
            "scroll_background" => {
                for i in 0..self.resource_scroll_background.len() {
                    if self.resource_scroll_background[i].name == resource_name {
                        id = i;
                        break;
                    }
                }
            }
            "split_time" => {
                for i in 0..self.timer.split_time.len() {
                    if self.timer.split_time[i].name == resource_name {
                        id = i;
                        break;
                    }
                }
            }
            "image_texture" => {
                for i in 0..self.resource_image_texture.len() {
                    if self.resource_image_texture[i].name == resource_name {
                        id = i;
                        break;
                    }
                }
            }
            "switch" => {
                for i in 0..self.resource_switch.len() {
                    if self.resource_switch[i].name == resource_name {
                        id = i;
                        break;
                    }
                }
            }
            _ => panic!("\"{}\"资源列表不存在!", resource_list_name),
        };
        id
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
            image_id.push(self.track_resource("image", &i));
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
        let id = self.track_resource("scroll_background", name);
        let mut id2;
        for i in 0..self.resource_scroll_background[id].image_name.len() {
            self.image(
                ctx,
                &self.resource_scroll_background[id].image_name[i].clone(),
                ui,
            );
        }
        for i in 0..self.resource_scroll_background[id].image_name.len() {
            id2 = self.track_resource(
                "image",
                &self.resource_scroll_background[id].image_name[i].clone(),
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
        self.resource_image_texture.push(ImageTexture {
            name: name.to_string(),
            texture: image_texture,
        });
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
        let id = self.track_resource("image_texture", image_texture_name);
        self.resource_image.push(Image {
            name: name.to_string(),
            image_texture: self.resource_image_texture[id].texture.clone(),
            image_position: [position_size[0], position_size[1]],
            image_size: [position_size[2], position_size[3]],
            x_grid: [grid[0], grid[1]],
            y_grid: [grid[2], grid[3]],
            center_display: [center_display_and_use_overlay[0], center_display_and_use_overlay[1], center_display_and_use_overlay[2], center_display_and_use_overlay[3]],
            alpha: alpha_and_overlay_color[0],
            overlay_color: [alpha_and_overlay_color[1], alpha_and_overlay_color[2], alpha_and_overlay_color[3], alpha_and_overlay_color[4]],
            use_overlay_color: center_display_and_use_overlay[4],
        });
    }

    pub fn image(&mut self, ctx: &egui::Context, name: &str, ui: &mut Ui) {
        let id = self.track_resource("image", name);
        self.resource_image[id].image_position[0] = match self.resource_image[id].x_grid[1] {
            0 => self.resource_image[id].image_position[0],
            _ => {
                (ctx.available_rect().width() as f64 / self.resource_image[id].x_grid[1] as f64
                    * self.resource_image[id].x_grid[0] as f64) as f32
            }
        };
        self.resource_image[id].image_position[1] = match self.resource_image[id].y_grid[1] {
            0 => self.resource_image[id].image_position[1],
            _ => {
                (ctx.available_rect().height() as f64 / self.resource_image[id].y_grid[1] as f64
                    * self.resource_image[id].y_grid[0] as f64) as f32
            }
        };
        let pos_x;
        let pos_y;
        if self.resource_image[id].center_display[2] {
            pos_x = self.resource_image[id].image_position[0]
                - self.resource_image[id].image_size[0] / 2.0;
        } else if self.resource_image[id].center_display[0] {
            pos_x = self.resource_image[id].image_position[0];
        } else {
            pos_x =
                self.resource_image[id].image_position[0] - self.resource_image[id].image_size[0];
        };
        if self.resource_image[id].center_display[3] {
            pos_y = self.resource_image[id].image_position[1]
                - self.resource_image[id].image_size[1] / 2.0;
        } else if self.resource_image[id].center_display[1] {
            pos_y = self.resource_image[id].image_position[1];
        } else {
            pos_y =
                self.resource_image[id].image_position[1] - self.resource_image[id].image_size[1];
        };
        if let Some(texture) = &self.resource_image[id].image_texture {
            let rect = Rect::from_min_size(
                Pos2::new(pos_x, pos_y),
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
                    (self.resource_image[id].alpha as f32 *
                        self.resource_image[id].overlay_color[3] as f32 / 255.0) as u8
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
        click_method: PointerButton
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
                panic!("\"{}\"开关缺少/多出{}个资源！", name_and_switch_image_name[0], count * switch_amounts_state - overlay_color.len() as u32);
            };
            let id = self.track_resource("image", name_and_switch_image_name[1]);
            self.resource_image[id].use_overlay_color = true;
        } else if switch_texture_name.len() as u32 != count * switch_amounts_state {
                panic!("\"{}\"开关缺少/多出{}个资源！", name_and_switch_image_name[0], count * switch_amounts_state - switch_texture_name.len() as u32);
        };
        self.resource_switch.push(Switch {
            name: name_and_switch_image_name[0].to_string(),
            switch_texture_name,
            switch_image_name: name_and_switch_image_name[1].to_string(),
            enable_hover_click_image: [enable_hover_click_image_and_use_overlay[0], enable_hover_click_image_and_use_overlay[1]],
            state: 0,
            use_overlay: enable_hover_click_image_and_use_overlay[2],
            overlay_color,
            click_method,
            last_time_clicked: false,
        });
    }

    #[allow(dead_code)]
    pub fn switch(&mut self, name: &str, ui: &mut Ui, ctx: &egui::Context) {
        let id = self.track_resource("switch", name);
        let id2 = self.track_resource("image", &self.resource_switch[id].switch_image_name.clone());
        let id3;
        let rect =
            Rect::from_min_size(
                Pos2::new(self.resource_image[id2].image_position[0], self.resource_image[id2].image_position[1]),
                Vec2::new(self.resource_image[id2].image_size[0], self.resource_image[id2].image_size[1])
            );
        if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
            // 判断是否在矩形内
            if rect.contains(mouse_pos) {
                if ui.input(|i| i.pointer.button_down(self.resource_switch[id].click_method)) {
                    self.resource_switch[id].last_time_clicked = true;
                    if self.resource_switch[id].enable_hover_click_image[1] {
                        if self.resource_switch[id].use_overlay {
                            if self.resource_switch[id].enable_hover_click_image[0] {
                                self.resource_image[id2].overlay_color = self.resource_switch[id].overlay_color[(self.resource_switch[id].state + 2) as usize];
                            } else {
                                self.resource_image[id2].overlay_color = self.resource_switch[id].overlay_color[(self.resource_switch[id].state + 1) as usize];
                            };
                        } else {
                            if self.resource_switch[id].enable_hover_click_image[0] {
                                id3 = self.track_resource("image_texture", &self.resource_switch[id].switch_texture_name[(self.resource_switch[id].state + 2) as usize].clone());
                            } else {
                                id3 = self.track_resource("image_texture", &self.resource_switch[id].switch_texture_name[(self.resource_switch[id].state + 1) as usize].clone());
                            };
                            self.resource_image[id2].image_texture = self.resource_image_texture[id3].texture.clone();
                        };
                    } else if !self.resource_switch[id].enable_hover_click_image[0]{
                        if self.resource_switch[id].use_overlay {
                            self.resource_image[id2].overlay_color = self.resource_switch[id].overlay_color[self.resource_switch[id].state as usize];
                        } else {
                            id3 = self.track_resource("image_texture", &self.resource_switch[id].switch_texture_name[self.resource_switch[id].state as usize].clone());
                            self.resource_image[id2].image_texture = self.resource_image_texture[id3].texture.clone();
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
                            if self.resource_switch[id].state < (self.resource_switch[id].overlay_color.len() / count - 1) as u32 {
                                self.resource_switch[id].state += 1;
                            } else {
                                self.resource_switch[id].state = 0;
                            };
                        } else if self.resource_switch[id].state < (self.resource_switch[id].switch_texture_name.len() / count - 1) as u32 {
                                self.resource_switch[id].state += 1;
                            } else {
                                self.resource_switch[id].state = 0;
                            };
                        self.resource_switch[id].last_time_clicked = false;
                    };
                    if self.resource_switch[id].enable_hover_click_image[0] {
                        if self.resource_switch[id].use_overlay {
                            self.resource_image[id2].overlay_color = self.resource_switch[id].overlay_color[(self.resource_switch[id].state + 1) as usize];
                        } else {
                            id3 = self.track_resource("image_texture", &self.resource_switch[id].switch_texture_name[(self.resource_switch[id].state + 1) as usize].clone());
                            self.resource_image[id2].image_texture = self.resource_image_texture[id3].texture.clone();
                        };
                    } else if self.resource_switch[id].use_overlay {
                            self.resource_image[id2].overlay_color = self.resource_switch[id].overlay_color[self.resource_switch[id].state as usize];
                        } else {
                            id3 = self.track_resource("image_texture", &self.resource_switch[id].switch_texture_name[self.resource_switch[id].state as usize].clone());
                            self.resource_image[id2].image_texture = self.resource_image_texture[id3].texture.clone();
                        };
                };
            } else {
                self.resource_switch[id].last_time_clicked = false;
                if self.resource_switch[id].use_overlay {
                    self.resource_image[id2].overlay_color = self.resource_switch[id].overlay_color[self.resource_switch[id].state as usize];
                } else {
                    id3 = self.track_resource("image_texture", &self.resource_switch[id].switch_texture_name[self.resource_switch[id].state as usize].clone());
                    self.resource_image[id2].image_texture = self.resource_image_texture[id3].texture.clone();
                };
            };
        };
        self.image(ctx, &self.resource_switch[id].switch_image_name.clone(), ui);
    }
}
