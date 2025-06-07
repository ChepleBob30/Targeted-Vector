//! function.rs is the functional module of the Targeted Vector, including function declarations, struct definitions, and some auxiliary content.
use anyhow::Context;
use eframe::emath::Rect;
use eframe::epaint::textures::TextureOptions;
use eframe::epaint::Stroke;
use egui::{Color32, FontId, Frame, PointerButton, Pos2, Ui, Vec2};
use json::JsonValue;
use kira::manager::backend::cpal;
use kira::manager::AudioManager;
use kira::sound::static_sound::StaticSoundData;
use std::collections::hash_map;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
use std::time::Instant;
use std::vec::Vec;
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

pub fn kira_play_wav(path: &str) -> anyhow::Result<f64> {
    let mut manager: kira::manager::AudioManager<cpal::CpalBackend> =
        AudioManager::new(kira::manager::AudioManagerSettings::default())?;
    let sound_data = StaticSoundData::from_file(path, Default::default())?;
    let duration = sound_data.duration().as_secs_f64();

    manager.play(sound_data)?;
    std::thread::sleep(std::time::Duration::from_secs_f64(duration));
    Ok(duration)
}

pub fn general_click_feedback() {
    std::thread::spawn(|| {
        kira_play_wav("Resources/assets/sounds/Click.wav").unwrap();
    });
}

pub fn count_files_recursive(dir: &Path, target: &str) -> std::io::Result<usize> {
    let mut count = 0;
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                count += count_files_recursive(&path, target)?;
            } else if path.file_name().unwrap().to_string_lossy().contains(target) {
                count += 1;
            }
        }
    }
    Ok(count)
}

pub fn list_files_recursive(path: &Path, prefix: &str) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut matches = Vec::new();

    for entry in std::fs::read_dir(path)? {
        // 遍历目录
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // 递归处理子目录
            matches.extend(list_files_recursive(&path, prefix)?);
        } else if let Some(file_name) = path.file_name() {
            if file_name.to_string_lossy().contains(prefix) {
                matches.push(path);
            }
        }
    }

    Ok(matches)
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
    pub login_user_name: String,
    pub amount_languages: u8,
    pub rc_strict_mode: bool,
}

impl Config {
    pub fn from_json_value(value: &JsonValue) -> Option<Config> {
        Some(Config {
            launch_path: value["launch_path"].as_str()?.to_string(),
            language: value["language"].as_u8()?,
            login_user_name: value["login_user_name"].as_str()?.to_string(),
            amount_languages: value["amount_languages"].as_u8()?,
            rc_strict_mode: value["rc_strict_mode"].as_bool()?,
        })
    }
    pub fn to_json_value(&self) -> JsonValue {
        json::object! {
            launch_path: self.launch_path.clone(),
            language: self.language,
            login_user_name: self.login_user_name.clone(),
            amount_languages: self.amount_languages,
            rc_strict_mode: self.rc_strict_mode,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OperationGlobal {
    pub target_point: u32,
    pub storage_bullet: u32,
    pub cost: u32,
    pub cost_recover_speed: f32,
    pub instrument_ceiling: u32,
    pub target_line: Vec<[f32; 2]>,
}

impl OperationGlobal {
    #[allow(dead_code)]
    pub fn from_json_value(value: &JsonValue) -> Option<OperationGlobal> {
        Some(OperationGlobal {
            target_point: value["target_point"].as_u32()?,
            storage_bullet: value["storage_bullet"].as_u32()?,
            cost: value["cost"].as_u32()?,
            cost_recover_speed: value["cost_recover_speed"].as_f32()?,
            instrument_ceiling: value["instrument_ceiling"].as_u32()?,
            target_line: value["target_line"]
                .members()
                .filter_map(|arr| Some([arr[0].as_f32()?, arr[1].as_f32()?]))
                .collect(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct OperationTargetEnemy {
    pub enemy_recognition_name: String,
    pub enemy_position: [f32; 2],
    pub enemy_size: [f32; 2],
    pub enemy_path: Vec<String>,
    pub enemy_approach_time: f32,
}

impl OperationTargetEnemy {
    pub fn from_json_value(value: &JsonValue) -> Option<OperationTargetEnemy> {
        Some(OperationTargetEnemy {
            enemy_recognition_name: value["enemy_recognition_name"].as_str()?.to_string(),
            enemy_position: [
                value["enemy_position"][0].as_f32()?,
                value["enemy_position"][1].as_f32()?,
            ],
            enemy_size: [
                value["enemy_size"][0].as_f32()?,
                value["enemy_size"][1].as_f32()?,
            ],
            enemy_path: value["enemy_path"]
                .members()
                .map(|s| s.to_string())
                .collect(),
            enemy_approach_time: value["enemy_approach_time"].as_f32()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub global: OperationGlobal,
    pub target_enemy: Vec<OperationTargetEnemy>,
}

impl Operation {
    pub fn from_json_value(value: &JsonValue) -> Option<Operation> {
        Some(Operation {
            global: OperationGlobal::from_json_value(&value["global"])?,
            target_enemy: value["target_enemy"]
                .members()
                .map(OperationTargetEnemy::from_json_value)
                .collect::<Option<Vec<_>>>()?,
        })
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MovePath {
    pub move_status: [bool; 4],
    pub move_time: f32,
}

impl MovePath {
    #[allow(dead_code)]
    pub fn from_json_value(value: &JsonValue) -> Option<MovePath> {
        Some(MovePath {
            move_status: [
                value["move_status"][0].as_bool()?,
                value["move_status"][1].as_bool()?,
                value["move_status"][2].as_bool()?,
                value["move_status"][3].as_bool()?,
            ],
            move_time: value["move_time"].as_f32()?,
        })
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Enemy {
    pub enemy_id: String,
    pub enemy_hp: f32,
    pub enemy_def: f32,
    pub enemy_speed: f32,
    pub enemy_invincible_time: f32,
    pub enemy_image_count: u32,
    pub enemy_tag: Vec<String>,
    pub enemy_image: String,
    pub enemy_image_type: String,
    pub enemy_minus_target_point: u32,
    pub enemy_position: [f32; 2],
    pub enemy_move_path: Vec<MovePath>,
    pub enemy_detected: bool,
    pub enemy_activated: bool,
    pub enemy_activated_time: f32,
    pub enemy_offset: [f32; 2],
    pub enemy_size: [f32; 2],
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct JsonReadEnemy {
    pub enemy_recognition_name: String,
    pub enemy_name: Vec<String>,
    pub enemy_hp: f32,
    pub enemy_def: f32,
    pub enemy_speed: f32,
    pub enemy_invincible_time: f32,
    pub enemy_image_count: u32,
    pub enemy_tag: Vec<String>,
    pub enemy_image: String,
    pub enemy_image_type: String,
    pub enemy_minus_target_point: u32,
}

impl JsonReadEnemy {
    #[allow(dead_code)]
    pub fn from_json_value(value: &JsonValue) -> Option<JsonReadEnemy> {
        Some(JsonReadEnemy {
            enemy_recognition_name: value["enemy_recognition_name"].as_str()?.to_string(),
            enemy_hp: value["enemy_hp"].as_f32()?,
            enemy_def: value["enemy_def"].as_f32()?,
            enemy_speed: value["enemy_speed"].as_f32()?,
            enemy_invincible_time: value["enemy_invincible_time"].as_f32()?,
            enemy_image_count: value["enemy_image_count"].as_u32()?,
            enemy_tag: value["enemy_tag"]
                .members()
                .map(|s| s.to_string())
                .collect(),
            enemy_image: value["enemy_image"].as_str()?.to_string(),
            enemy_image_type: value["enemy_image_type"].as_str()?.to_string(),
            enemy_minus_target_point: value["enemy_minus_target_point"].as_u32()?,
            enemy_name: value["enemy_name"]
                .members()
                .map(|s| s.to_string())
                .collect(),
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
pub struct Level {
    pub level_name: String,
    pub level_name_expand: Vec<String>,
    pub level_description: Vec<String>,
    pub level_type: String,
    pub level_position: [f32; 2],
    pub level_initial_status: i8,
}

#[derive(Debug, Clone)]
pub struct Map {
    pub map_name: Vec<String>,
    pub map_author: String,
    pub map_image: String,
    pub map_width: f32,
    pub map_scroll_offset: f32,
    pub map_operation_background: String,
    pub map_operation_background_expand: String,
    pub map_description: Vec<String>,
    pub map_intro: String,
    pub map_content: Vec<Level>,
    pub map_connecting_line: Vec<[String; 2]>,
}

#[allow(dead_code)]
impl Map {
    pub fn from_json_value(value: &JsonValue) -> Option<Self> {
        Some(Self {
            map_name: value["map_name"]
                .members()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            map_author: value["map_author"].as_str()?.to_string(),
            map_image: value["map_image"].as_str()?.to_string(),
            map_width: value["map_width"].as_f32()?,
            map_scroll_offset: value["map_scroll_offset"].as_f32()?,
            map_operation_background: value["map_operation_background"].as_str()?.to_string(),
            map_operation_background_expand: value["map_operation_background_expand"]
                .as_str()?
                .to_string(),
            map_description: value["map_description"]
                .members()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            map_intro: value["map_intro"].as_str()?.to_string(),
            map_content: value["map_content"]
                .members()
                .filter_map(|v| {
                    Some(Level {
                        level_name: v["level_name"].as_str()?.to_string(),
                        level_name_expand: v["level_name_expand"]
                            .members()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect(),
                        level_description: v["level_description"]
                            .members()
                            .filter_map(|d| d.as_str().map(String::from))
                            .collect(),
                        level_type: v["level_type"].as_str()?.to_string(),
                        level_position: [
                            v["level_position"][0].as_f32()?,
                            v["level_position"][1].as_f32()?,
                        ],
                        level_initial_status: v["level_initial_status"].as_i8()?,
                    })
                })
                .collect(),
            map_connecting_line: value["map_connecting_line"]
                .members()
                .filter_map(|v| {
                    let vec: Vec<String> = v
                        .members() // 先收集到 Vec
                        .filter_map(|s| s.as_str().map(String::from))
                        .collect();
                    let pair: [String; 2] = vec.try_into().ok()?; // 再转数组
                    Some(pair)
                })
                .collect(),
        })
    }
    pub fn to_json_value(&self) -> JsonValue {
        json::object! {
            map_name: self.map_name.clone(),
            map_author: self.map_author.clone(),
            map_image: self.map_image.clone(),
            map_width: self.map_width,
            map_scroll_offset: self.map_scroll_offset,
            map_operation_background: self.map_operation_background.clone(),
            map_operation_background_expand: self.map_operation_background_expand.clone(),
            map_description: self.map_description.clone(),
            map_intro: self.map_intro.clone(),
            map_content: self.map_content.iter().map(|l| {
                json::object! {
                    level_name: l.level_name.clone(),
                    level_name_expand: l.level_name_expand.clone(),
                    level_description: l.level_description.clone(),
                    level_type: l.level_type.clone(),
                    level_position: [l.level_position[0], l.level_position[1]],
                    level_initial_status: l.level_initial_status // 新增缺失字段
                }
            }).collect::<Vec<_>>(),
            map_connecting_line: self.map_connecting_line.iter().map(|pair| { // 新增连接线字段
                json::array![pair[0].clone(), pair[1].clone()]
            }).collect::<Vec<_>>()
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Gun {
    pub gun_recognition_name: String,
    pub gun_name: Vec<String>,
    pub gun_size: [f32; 2],
    pub gun_image: String,
    pub gun_shoot_sound: String,
    pub gun_shoot_speed: f32,
    pub gun_reload_time: f32,
    pub gun_basic_damage: f32,
    pub gun_catridge_clip: u32,
    pub gun_recoil: f32,
    pub gun_temperature_degree: u32,
    pub gun_tag: Vec<String>,
    pub gun_initial_level: i32,
    pub gun_no_bullet_shoot_sound: String,
    pub gun_reload_sound: String,
    pub gun_reload_bullet_sound: String,
    pub gun_reload_interval: f32,
}

#[allow(dead_code)]
impl Gun {
    pub fn from_json_value(value: &JsonValue) -> Option<Gun> {
        Some(Gun {
            gun_recognition_name: value["gun_recognition_name"].as_str()?.to_string(),
            gun_name: value["gun_name"]
                .members()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            gun_size: value["gun_size"]
                .members()
                .filter_map(|v: &JsonValue| v.as_f32())
                .collect::<Vec<f32>>()
                .try_into()
                .ok()
                .unwrap_or([0.0, 0.0]),
            gun_image: value["gun_image"].as_str()?.to_string(),
            gun_shoot_sound: value["gun_shoot_sound"].as_str()?.to_string(),
            gun_shoot_speed: value["gun_shoot_speed"].as_f32()?,
            gun_reload_time: value["gun_reload_time"].as_f32()?,
            gun_basic_damage: value["gun_basic_damage"].as_f32()?,
            gun_catridge_clip: value["gun_catridge_clip"].as_u32()?,
            gun_recoil: value["gun_recoil"].as_f32()?,
            gun_temperature_degree: value["gun_temperature_degree"].as_u32()?,
            gun_tag: value["gun_tag"]
                .members()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            gun_initial_level: value["gun_initial_level"].as_i32()?,
            gun_no_bullet_shoot_sound: value["gun_no_bullet_shoot_sound"].as_str()?.to_string(),
            gun_reload_bullet_sound: value["gun_reload_bullet_sound"].as_str()?.to_string(),
            gun_reload_sound: value["gun_reload_sound"].as_str()?.to_string(),
            gun_reload_interval: value["gun_reload_interval"].as_f32()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct UserLevelStatus {
    pub level_name: String,
    pub level_map: String,
    pub level_status: i8,
}

#[derive(Debug, Clone)]
pub struct UserGunStatus {
    pub gun_recognition_name: String,
    pub gun_level: i32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct User {
    pub version: u8,
    pub name: String,
    pub password: String,
    pub language: u8,
    pub wallpaper: String,
    pub current_map: String,
    pub level_status: Vec<UserLevelStatus>,
    pub gun_status: Vec<UserGunStatus>,
    pub settings: HashMap<String, String>,
    pub current_level: String,
}

#[allow(dead_code)]
impl User {
    pub fn from_json_value(value: &JsonValue) -> Option<User> {
        let mut parsed = HashMap::new();
        for (key, val) in value["game_text"].entries() {
            if let JsonValue::String(string) = val {
                let str: String = string.clone();
                parsed.insert(key.to_string(), str);
            };
        }
        Some(User {
            version: value["version"].as_u8()?,
            name: value["name"].as_str()?.to_string(),
            password: value["password"].as_str()?.to_string(),
            language: value["language"].as_u8()?,
            wallpaper: value["wallpaper"].as_str()?.to_string(),
            current_map: value["current_map"].as_str()?.to_string(),
            level_status: value["level_status"]
                .members()
                .filter_map(|v| {
                    Some(UserLevelStatus {
                        level_name: v["level_name"].as_str()?.to_string(),
                        level_map: v["level_map"].as_str()?.to_string(),
                        level_status: v["level_status"].as_i8()?,
                    })
                })
                .collect(),
            gun_status: value["gun_status"]
                .members()
                .filter_map(|v| {
                    Some(UserGunStatus {
                        gun_recognition_name: v["gun_recognition_name"].as_str()?.to_string(),
                        gun_level: v["gun_level"].as_i32()?,
                    })
                })
                .collect(),
            settings: parsed,
            current_level: value["current_level"].as_str()?.to_string(),
        })
    }

    pub fn to_json_value(&self) -> JsonValue {
        json::object! {
            version: self.version,
            name: self.name.clone(),
            password: self.password.clone(),
            language: self.language,
            wallpaper: self.wallpaper.clone(),
            current_map: self.current_map.clone(),
            level_status: self.level_status.iter().map(|l| json::object! {
                level_name: l.level_name.clone(),
                level_map: l.level_map.clone(),
                level_status: l.level_status,
            }).collect::<Vec<_>>(),
            gun_status: self.gun_status.iter().map(|l| json::object! {
                gun_recognition_name: l.gun_recognition_name.clone(),
                gun_level: l.gun_level,
            }).collect::<Vec<_>>(),
            settings: self.settings.iter().fold(json::object! {}, |mut obj, (k, v)| {
                obj.insert(k, v.clone()).expect("插入设置项失败");
                obj
            }),
            current_level: self.current_level.clone(),
        }
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

#[derive(Clone, Debug)]
pub struct ReportState {
    pub current_page: String,
    pub current_total_runtime: f32,
    pub current_page_runtime: f32,
}

#[derive(Clone, Debug)]
pub struct Problem {
    pub severity_level: SeverityLevel,
    pub problem: String,
    pub annotation: String,
    pub report_state: ReportState,
}

#[derive(Clone, Debug)]
pub enum SeverityLevel {
    MildWarning,
    SevereWarning,
    Error,
}

pub fn check_resource_exist<T: RustConstructorResource>(
    resource_list: Vec<T>,
    resource_name: &str,
) -> bool {
    let mut found_resource: bool = false;
    for (i, _a) in resource_list.iter().enumerate() {
        if resource_list[i].name() == resource_name {
            found_resource = true;
            break;
        };
    }
    found_resource
}

pub trait RustConstructorResource {
    fn name(&self) -> &str;

    fn expose_type(&self) -> &str;

    fn reg_render_resource(&self, render_list: &mut Vec<RenderResource>);
}

impl RustConstructorResource for PageData {
    fn name(&self) -> &str {
        &self.name
    }

    fn expose_type(&self) -> &str {
        &self.discern_type
    }

    fn reg_render_resource(&self, render_list: &mut Vec<RenderResource>) {
        render_list.push(RenderResource {
            discern_type: self.expose_type().to_string(),
            name: self.name.to_string(),
        });
    }
}

#[derive(Clone, Debug)]
pub struct PageData {
    pub discern_type: String,
    pub name: String,
    pub forced_update: bool,
    pub change_page_updated: bool,
}

#[derive(Clone, Debug)]
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

    fn expose_type(&self) -> &str {
        &self.discern_type
    }

    fn reg_render_resource(&self, render_list: &mut Vec<RenderResource>) {
        render_list.push(RenderResource {
            discern_type: self.expose_type().to_string(),
            name: self.name.to_string(),
        });
    }
}

#[derive(Clone)]
pub struct ImageTexture {
    pub discern_type: String,
    pub name: String,
    pub texture: Option<egui::TextureHandle>,
    pub cite_path: String,
}

impl RustConstructorResource for CustomRect {
    fn name(&self) -> &str {
        &self.name
    }

    fn expose_type(&self) -> &str {
        &self.discern_type
    }

    fn reg_render_resource(&self, render_list: &mut Vec<RenderResource>) {
        render_list.push(RenderResource {
            discern_type: self.expose_type().to_string(),
            name: self.name.to_string(),
        });
    }
}

#[derive(Clone, Debug)]
pub struct CustomRect {
    pub discern_type: String,
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

    fn expose_type(&self) -> &str {
        &self.discern_type
    }

    fn reg_render_resource(&self, render_list: &mut Vec<RenderResource>) {
        render_list.push(RenderResource {
            discern_type: self.expose_type().to_string(),
            name: self.name.to_string(),
        });
    }
}

#[derive(Clone)]
pub struct Image {
    pub discern_type: String,
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
    pub origin_cite_texture: String,
}

impl RustConstructorResource for Text {
    fn name(&self) -> &str {
        &self.name
    }

    fn expose_type(&self) -> &str {
        &self.discern_type
    }

    fn reg_render_resource(&self, render_list: &mut Vec<RenderResource>) {
        render_list.push(RenderResource {
            discern_type: self.expose_type().to_string(),
            name: self.name.to_string(),
        });
    }
}

#[derive(Clone, Debug)]
pub struct Text {
    pub discern_type: String,
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

    fn expose_type(&self) -> &str {
        &self.discern_type
    }

    fn reg_render_resource(&self, render_list: &mut Vec<RenderResource>) {
        render_list.push(RenderResource {
            discern_type: self.expose_type().to_string(),
            name: self.name.to_string(),
        });
    }
}

#[derive(Clone, Debug)]
pub struct ScrollBackground {
    pub discern_type: String,
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

    fn expose_type(&self) -> &str {
        &self.discern_type
    }

    fn reg_render_resource(&self, render_list: &mut Vec<RenderResource>) {
        render_list.push(RenderResource {
            discern_type: self.expose_type().to_string(),
            name: self.name.to_string(),
        });
    }
}

#[derive(Clone, Debug)]
pub struct Variable {
    pub discern_type: String,
    pub name: String,
    pub value: Value,
}

impl RustConstructorResource for SplitTime {
    fn name(&self) -> &str {
        &self.name
    }

    fn expose_type(&self) -> &str {
        &self.discern_type
    }

    fn reg_render_resource(&self, render_list: &mut Vec<RenderResource>) {
        render_list.push(RenderResource {
            discern_type: self.expose_type().to_string(),
            name: self.name.to_string(),
        });
    }
}

#[derive(Clone, Debug)]
pub struct SplitTime {
    pub discern_type: String,
    pub name: String,
    pub time: [f32; 2],
}

impl RustConstructorResource for Switch {
    fn name(&self) -> &str {
        &self.name
    }

    fn expose_type(&self) -> &str {
        &self.discern_type
    }

    fn reg_render_resource(&self, render_list: &mut Vec<RenderResource>) {
        render_list.push(RenderResource {
            discern_type: self.expose_type().to_string(),
            name: self.name.to_string(),
        });
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Switch {
    pub discern_type: String,
    pub name: String,
    pub appearance: Vec<SwitchData>,
    pub switch_image_name: String,
    pub enable_hover_click_image: [bool; 2],
    pub state: u32,
    pub click_method: Vec<SwitchClickAction>,
    pub last_time_clicked: bool,
    pub last_time_clicked_index: usize,
    pub animation_count: u32,
}

#[derive(Clone, Debug)]
pub struct RenderResource {
    pub discern_type: String,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct SwitchData {
    pub texture: String,
    pub color: [u8; 4],
}

#[derive(Clone, Debug)]
pub struct SwitchClickAction {
    pub click_method: PointerButton,
    pub action: bool,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct App {
    pub config: Config,
    pub game_text: GameText,
    pub render_resource_list: Vec<RenderResource>,
    pub problem_list: Vec<Problem>,
    pub storage_gun_content: Vec<Gun>,
    pub login_user_config: User,
    pub frame: Frame,
    pub vertrefresh: f32,
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
    pub frame_times: Vec<f32>,
    pub last_frame_time: Option<f64>,
    pub enemy_list: Vec<Enemy>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        load_fonts(&cc.egui_ctx);
        let mut config = Config {
            launch_path: "".to_string(),
            language: 0,
            login_user_name: "".to_string(),
            amount_languages: 0,
            rc_strict_mode: false,
        };
        let mut game_text = GameText {
            game_text: HashMap::new(),
        };
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
            render_resource_list: Vec::new(),
            problem_list: Vec::new(),
            storage_gun_content: Vec::new(),
            login_user_config: User {
                version: 0,
                name: "".to_string(),
                password: "".to_string(),
                language: 0,
                wallpaper: "".to_string(),
                current_map: "".to_string(),
                level_status: vec![],
                gun_status: vec![],
                settings: hash_map::HashMap::new(),
                current_level: "".to_string(),
            },
            frame: Frame {
                ..Default::default()
            },
            vertrefresh: 0.01,
            page: "Launch".to_string(),
            resource_page: vec![
                PageData {
                    discern_type: "PageData".to_string(),
                    name: "Launch".to_string(),
                    forced_update: true,
                    change_page_updated: false,
                },
                PageData {
                    discern_type: "PageData".to_string(),
                    name: "Login".to_string(),
                    forced_update: true,
                    change_page_updated: false,
                },
                PageData {
                    discern_type: "PageData".to_string(),
                    name: "Home_Page".to_string(),
                    forced_update: true,
                    change_page_updated: false,
                },
                PageData {
                    discern_type: "PageData".to_string(),
                    name: "Home_Setting".to_string(),
                    forced_update: true,
                    change_page_updated: false,
                },
                PageData {
                    discern_type: "PageData".to_string(),
                    name: "Home_Select_Map".to_string(),
                    forced_update: true,
                    change_page_updated: false,
                },
                PageData {
                    discern_type: "PageData".to_string(),
                    name: "Select_Level".to_string(),
                    forced_update: true,
                    change_page_updated: false,
                },
                PageData {
                    discern_type: "PageData".to_string(),
                    name: "Operation".to_string(),
                    forced_update: true,
                    change_page_updated: false,
                },
                PageData {
                    discern_type: "PageData".to_string(),
                    name: "Error".to_string(),
                    forced_update: true,
                    change_page_updated: false,
                },
            ],
            resource_image: Vec::new(),
            resource_text: Vec::new(),
            resource_rect: Vec::new(),
            resource_scroll_background: Vec::new(),
            timer: Timer {
                start_time: 0.0,
                total_time: 0.0,
                timer: Instant::now(),
                now_time: 0.0,
                split_time: Vec::new(),
            },
            variables: Vec::new(),
            resource_image_texture: Vec::new(),
            resource_switch: Vec::new(),
            frame_times: Vec::new(),
            last_frame_time: None,
            enemy_list: Vec::new(),
        }
    }

    pub fn switch_page(&mut self, page: &str) {
        self.page = page.to_string();
    }

    pub fn launch_page_preload(&mut self, ctx: &egui::Context) {
        let game_text = self.game_text.game_text.clone();
        self.add_image_texture(
            "Error",
            "Resources/assets/images/error.png",
            [false, false],
            true,
            ctx,
        );
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
            "Error",
            [0_f32, 0_f32, 130_f32, 130_f32],
            [1, 2, 1, 2],
            [true, true, true, true, false],
            [255, 0, 0, 0, 0],
            "Error",
        );
        self.add_image(
            "RC_Logo",
            [-25_f32, 0_f32, 130_f32, 130_f32],
            [1, 2, 1, 2],
            [false, false, false, true, false],
            [0, 0, 0, 0, 0],
            "RC_Logo",
        );
        self.add_image(
            "Binder_Logo",
            [-25_f32, 0_f32, 150_f32, 150_f32],
            [1, 2, 1, 2],
            [false, false, false, true, false],
            [0, 0, 0, 0, 0],
            "Binder_Logo",
        );
        self.add_image(
            "Mouse",
            [-25_f32, 0_f32, 150_f32, 150_f32],
            [1, 2, 1, 2],
            [false, false, false, true, false],
            [0, 0, 0, 0, 0],
            "Mouse",
        );
        self.add_text(
            [
                "Powered",
                &*game_text["powered"][self.config.language as usize].clone(),
            ],
            [25_f32, 0_f32, 40_f32, 1000_f32, 0.0],
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
            [25_f32, 0_f32, 40_f32, 1000_f32, 0.0],
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
            [25_f32, 0_f32, 40_f32, 1000_f32, 0.0],
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
        std::thread::spawn(|| {
            kira_play_wav("Resources/assets/sounds/Launch.wav").unwrap();
        });
        for i in 0..self.config.amount_languages {
            self.add_image_texture(
                &format!("{}_Title", i),
                &format!("Resources/assets/images/{}_title.png", i),
                [false, false],
                true,
                ctx,
            );
            self.add_image(
                &format!("{}_Title", i),
                [0_f32, 0_f32, 510_f32, 150_f32],
                [1, 2, 1, 4],
                [true, true, true, true, false],
                [255, 0, 0, 0, 0],
                &format!("{}_Title", i),
            );
        }
        let id = self.track_resource(self.resource_image.clone(), "1_Title");
        self.resource_image[id].image_size = [900_f32, 130_f32];
        self.add_image_texture(
            "Background",
            "Resources/assets/images/wallpaper.png",
            [false, false],
            true,
            ctx,
        );
        self.add_image_texture(
            "Background2",
            "Resources/assets/images/wallpaper.png",
            [true, false],
            true,
            ctx,
        );
        self.add_image_texture(
            "Shutdown",
            "Resources/assets/images/shutdown.png",
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
            "Shutdown",
            [-75_f32, 25_f32, 50_f32, 50_f32],
            [1, 2, 7, 8],
            [true, true, true, true, false],
            [255, 0, 0, 0, 0],
            "Shutdown",
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
            "Background2",
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
        self.add_text(
            ["Date", ""],
            [0_f32, 20_f32, 30_f32, 1000_f32, 0.0],
            [255, 255, 255, 255, 0, 0, 0],
            [true, false, true, false],
            false,
            [1, 2, 1, 6],
        );
        self.add_text(
            ["Time", ""],
            [0_f32, 0_f32, 100_f32, 1000_f32, 0.0],
            [255, 255, 255, 255, 0, 0, 0],
            [true, true, true, false],
            false,
            [1, 2, 1, 6],
        );
        self.add_switch(
            ["Shutdown", "Shutdown"],
            vec![
                SwitchData {
                    texture: "Shutdown".to_string(),
                    color: [255, 255, 255, 255],
                },
                SwitchData {
                    texture: "Shutdown".to_string(),
                    color: [180, 180, 180, 255],
                },
                SwitchData {
                    texture: "Shutdown".to_string(),
                    color: [150, 150, 150, 255],
                },
            ],
            [true, true, true],
            1,
            vec![SwitchClickAction {
                click_method: PointerButton::Primary,
                action: true,
            }],
        );
        self.add_switch(
            ["Register", "Register"],
            vec![
                SwitchData {
                    texture: "Register".to_string(),
                    color: [255, 255, 255, 255],
                },
                SwitchData {
                    texture: "Register".to_string(),
                    color: [180, 180, 180, 255],
                },
                SwitchData {
                    texture: "Register".to_string(),
                    color: [150, 150, 150, 255],
                },
            ],
            [true, true, true],
            1,
            vec![SwitchClickAction {
                click_method: PointerButton::Primary,
                action: true,
            }],
        );
        self.add_switch(
            ["Login", "Login"],
            vec![
                SwitchData {
                    texture: "Login".to_string(),
                    color: [255, 255, 255, 255],
                },
                SwitchData {
                    texture: "Login".to_string(),
                    color: [180, 180, 180, 255],
                },
                SwitchData {
                    texture: "Login".to_string(),
                    color: [150, 150, 150, 255],
                },
            ],
            [true, true, true],
            1,
            vec![SwitchClickAction {
                click_method: PointerButton::Primary,
                action: true,
            }],
        );
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
        self.add_image_texture(
            "Power",
            "Resources/assets/images/power.png",
            [false, false],
            true,
            ctx,
        );
        self.add_image_texture(
            "Logout",
            "Resources/assets/images/logout.png",
            [false, false],
            true,
            ctx,
        );
        self.add_image_texture(
            "Journey",
            "Resources/assets/images/journey.png",
            [false, false],
            true,
            ctx,
        );
        self.add_image(
            "Home_Home",
            [0_f32, -20_f32, 50_f32, 50_f32],
            [2, 5, 1, 1],
            [true, false, true, false, false],
            [255, 0, 0, 0, 0],
            "Home",
        );
        self.add_image(
            "Home_Settings",
            [0_f32, -20_f32, 50_f32, 50_f32],
            [4, 5, 1, 1],
            [true, false, true, false, false],
            [255, 0, 0, 0, 0],
            "Settings",
        );
        self.add_image(
            "Home_Journey",
            [0_f32, -20_f32, 50_f32, 50_f32],
            [3, 5, 1, 1],
            [true, false, true, false, false],
            [255, 0, 0, 0, 0],
            "Power",
        );
        self.add_image(
            "Home_Power",
            [0_f32, -20_f32, 50_f32, 50_f32],
            [1, 5, 1, 1],
            [true, false, true, false, false],
            [255, 0, 0, 0, 0],
            "Power",
        );
        self.add_switch(
            ["Home_Home", "Home_Home"],
            vec![
                SwitchData {
                    texture: "Home".to_string(),
                    color: [255, 255, 255, 255],
                },
                SwitchData {
                    texture: "Home".to_string(),
                    color: [180, 180, 180, 255],
                },
                SwitchData {
                    texture: "Home".to_string(),
                    color: [150, 150, 150, 255],
                },
            ],
            [true, true, true],
            1,
            vec![SwitchClickAction {
                click_method: PointerButton::Primary,
                action: true,
            }],
        );
        self.add_switch(
            ["Home_Journey", "Home_Journey"],
            vec![
                SwitchData {
                    texture: "Journey".to_string(),
                    color: [255, 255, 255, 255],
                },
                SwitchData {
                    texture: "Journey".to_string(),
                    color: [180, 180, 180, 255],
                },
                SwitchData {
                    texture: "Journey".to_string(),
                    color: [150, 150, 150, 255],
                },
            ],
            [true, true, true],
            1,
            vec![SwitchClickAction {
                click_method: PointerButton::Primary,
                action: true,
            }],
        );
        self.add_switch(
            ["Home_Settings", "Home_Settings"],
            vec![
                SwitchData {
                    texture: "Settings".to_string(),
                    color: [255, 255, 255, 255],
                },
                SwitchData {
                    texture: "Settings".to_string(),
                    color: [180, 180, 180, 255],
                },
                SwitchData {
                    texture: "Settings".to_string(),
                    color: [150, 150, 150, 255],
                },
            ],
            [true, true, true],
            1,
            vec![SwitchClickAction {
                click_method: PointerButton::Primary,
                action: true,
            }],
        );
        self.add_switch(
            ["Home_Power", "Home_Power"],
            vec![
                SwitchData {
                    texture: "Power".to_string(),
                    color: [255, 255, 255, 255],
                },
                SwitchData {
                    texture: "Power".to_string(),
                    color: [180, 180, 180, 255],
                },
                SwitchData {
                    texture: "Power".to_string(),
                    color: [150, 150, 150, 255],
                },
                SwitchData {
                    texture: "Logout".to_string(),
                    color: [255, 255, 255, 255],
                },
                SwitchData {
                    texture: "Logout".to_string(),
                    color: [180, 180, 180, 255],
                },
                SwitchData {
                    texture: "Logout".to_string(),
                    color: [150, 150, 150, 255],
                },
            ],
            [true, true, true],
            2,
            vec![
                SwitchClickAction {
                    click_method: PointerButton::Primary,
                    action: false,
                },
                SwitchClickAction {
                    click_method: PointerButton::Secondary,
                    action: true,
                },
            ],
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
            [100, 100, 100, 125, 255, 255, 255, 255],
            0.0,
        );
        self.add_image_texture(
            "Forward",
            "Resources/assets/images/go.png",
            [false, false],
            true,
            ctx,
        );
        self.add_image_texture(
            "Backward",
            "Resources/assets/images/go.png",
            [true, false],
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
            "Start_Operation",
            "Resources/assets/images/start_operation.png",
            [false, false],
            true,
            ctx,
        );
        self.add_image(
            "Start_Operation",
            [-200_f32, 0_f32, 50_f32, 50_f32],
            [1, 1, 3, 4],
            [true, true, true, false, false],
            [255, 0, 0, 0, 0],
            "Start_Operation",
        );
        self.add_image(
            "Forward",
            [150_f32, -100_f32, 50_f32, 50_f32],
            [1, 2, 1, 1],
            [true, false, false, false, false],
            [255, 0, 0, 0, 0],
            "Forward",
        );
        self.add_image(
            "Backward",
            [-150_f32, -100_f32, 50_f32, 50_f32],
            [1, 2, 1, 1],
            [false, false, false, false, false],
            [255, 0, 0, 0, 0],
            "Backward",
        );
        self.add_switch(
            ["Start_Operation", "Start_Operation"],
            vec![
                SwitchData {
                    texture: "Start_Operation".to_string(),
                    color: [255, 255, 255, 255],
                },
                SwitchData {
                    texture: "Start_Operation".to_string(),
                    color: [180, 180, 180, 255],
                },
                SwitchData {
                    texture: "Start_Operation".to_string(),
                    color: [150, 150, 150, 255],
                },
            ],
            [true, true, true],
            1,
            vec![SwitchClickAction {
                click_method: PointerButton::Primary,
                action: false,
            }],
        );
        self.add_switch(
            ["Forward", "Forward"],
            vec![
                SwitchData {
                    texture: "Forward".to_string(),
                    color: [255, 255, 255, 255],
                },
                SwitchData {
                    texture: "Forward".to_string(),
                    color: [180, 180, 180, 255],
                },
                SwitchData {
                    texture: "Forward".to_string(),
                    color: [150, 150, 150, 255],
                },
            ],
            [true, true, true],
            1,
            vec![SwitchClickAction {
                click_method: PointerButton::Primary,
                action: false,
            }],
        );
        self.add_switch(
            ["Backward", "Backward"],
            vec![
                SwitchData {
                    texture: "Backward".to_string(),
                    color: [255, 255, 255, 255],
                },
                SwitchData {
                    texture: "Backward".to_string(),
                    color: [180, 180, 180, 255],
                },
                SwitchData {
                    texture: "Backward".to_string(),
                    color: [150, 150, 150, 255],
                },
            ],
            [true, true, true],
            1,
            vec![SwitchClickAction {
                click_method: PointerButton::Primary,
                action: false,
            }],
        );
        self.add_rect(
            "Error_Pages_Background",
            [
                0_f32,
                0_f32,
                ctx.available_rect().width(),
                ctx.available_rect().height(),
                0_f32,
            ],
            [1, 2, 1, 2],
            [false, false, true, true],
            [31, 103, 179, 255, 255, 255, 255, 255],
            0.0,
        );
        self.add_text(
            ["Error_Pages_Sorry", ":("],
            [0_f32, 0_f32, 100_f32, 1000_f32, 0.0],
            [255, 255, 255, 255, 0, 0, 0],
            [true, true, false, false],
            false,
            [1, 5, 1, 6],
        );
        self.add_text(
            ["Error_Pages_Reason", ""],
            [0_f32, 0_f32, 40_f32, 1000_f32, 0.0],
            [255, 255, 255, 255, 0, 0, 0],
            [true, true, false, false],
            false,
            [1, 5, 2, 6],
        );
        self.add_text(
            ["Error_Pages_Solution", ""],
            [0_f32, 0_f32, 20_f32, 1000_f32, 0.0],
            [255, 255, 255, 255, 0, 0, 0],
            [true, true, false, false],
            false,
            [1, 5, 3, 6],
        );
        self.add_rect(
            "Cut_To_Background",
            [
                0_f32,
                0_f32,
                ctx.available_rect().width(),
                ctx.available_rect().height(),
                0_f32,
            ],
            [1, 2, 1, 2],
            [false, false, true, true],
            [0, 0, 0, 0, 255, 255, 255, 255],
            0.0,
        );
        self.add_image_texture(
            "Scroll_Forward",
            "Resources/assets/images/scroll_remind.png",
            [true, false],
            true,
            ctx,
        );
        self.add_image(
            "Scroll_Forward",
            [0_f32, 0_f32, 50_f32, ctx.available_rect().height()],
            [1, 1, 1, 2],
            [false, true, false, true, false],
            [100, 0, 0, 0, 0],
            "Scroll_Forward",
        );
        self.add_image_texture(
            "Back",
            "Resources/assets/images/back.png",
            [true, false],
            true,
            ctx,
        );
        self.add_image(
            "Back",
            [60_f32, 10_f32, 50_f32, 50_f32],
            [0, 1, 0, 1],
            [true, true, false, false, false],
            [255, 0, 0, 0, 0],
            "Back",
        );
        self.add_switch(
            ["Back", "Back"],
            vec![
                SwitchData {
                    texture: "Back".to_string(),
                    color: [255, 255, 255, 255],
                },
                SwitchData {
                    texture: "Back".to_string(),
                    color: [180, 180, 180, 255],
                },
                SwitchData {
                    texture: "Back".to_string(),
                    color: [150, 150, 150, 255],
                },
            ],
            [true, true, true],
            1,
            vec![SwitchClickAction {
                click_method: PointerButton::Primary,
                action: false,
            }],
        );
        self.add_image_texture(
            "Scroll_Backward",
            "Resources/assets/images/scroll_remind.png",
            [false, false],
            true,
            ctx,
        );
        self.add_image(
            "Scroll_Backward",
            [0_f32, 0_f32, 50_f32, ctx.available_rect().height()],
            [0, 1, 1, 2],
            [true, true, false, true, false],
            [100, 0, 0, 0, 0],
            "Scroll_Backward",
        );
        self.add_rect(
            "Level_Information_Background",
            [0_f32, 0_f32, 400_f32, ctx.available_rect().height(), 0_f32],
            [1, 1, 1, 2],
            [true, false, false, true],
            [0, 0, 0, 240, 255, 255, 255, 255],
            0.0,
        );
        self.add_text(
            ["Level_Title", ""],
            [-200_f32, 30_f32, 60_f32, 300_f32, 0.0],
            [255, 255, 255, 255, 0, 0, 0],
            [true, true, true, false],
            false,
            [1, 1, 0, 0],
        );
        self.add_text(
            ["Level_Description", ""],
            [-200_f32, 0_f32, 20_f32, 300_f32, 0.0],
            [255, 255, 255, 255, 0, 0, 0],
            [true, true, true, false],
            false,
            [1, 1, 1, 4],
        );
        self.add_rect(
            "Operation_Status_Bar",
            [0_f32, 0_f32, 1080_f32, 70_f32, 20_f32],
            [1, 2, 0, 0],
            [true, true, true, false],
            [100, 100, 100, 125, 240, 255, 255, 255],
            0.0,
        );
        self.add_image_texture(
            "Target_Point",
            "Resources/assets/images/target_point.png",
            [false, false],
            true,
            ctx,
        );
        self.add_image(
            "Target_Point",
            [0_f32, 0_f32, 50_f32, 50_f32],
            [0, 0, 0, 0],
            [true, true, true, false, false],
            [200, 0, 0, 0, 0],
            "Target_Point",
        );
        self.add_image_texture(
            "Target_Enemy",
            "Resources/assets/images/target_enemy.png",
            [false, false],
            true,
            ctx,
        );
        self.add_image(
            "Target_Enemy",
            [0_f32, 0_f32, 50_f32, 50_f32],
            [0, 0, 0, 0],
            [true, true, true, false, false],
            [200, 0, 0, 0, 0],
            "Target_Enemy",
        );
        self.add_image_texture(
            "Bullet",
            "Resources/assets/images/bullet.png",
            [false, false],
            true,
            ctx,
        );
        self.add_image(
            "Bullet",
            [0_f32, 0_f32, 50_f32, 50_f32],
            [0, 0, 0, 0],
            [true, true, true, false, false],
            [200, 0, 0, 0, 0],
            "Bullet",
        );
        self.add_image_texture(
            "Cost",
            "Resources/assets/images/cost.png",
            [false, false],
            true,
            ctx,
        );
        self.add_image(
            "Cost",
            [0_f32, 0_f32, 50_f32, 50_f32],
            [0, 0, 0, 0],
            [true, true, true, false, false],
            [200, 0, 0, 0, 0],
            "Cost",
        );
        self.add_image_texture(
            "Bullets",
            "Resources/assets/images/bullets.png",
            [false, false],
            true,
            ctx,
        );
        self.add_image(
            "Bullets",
            [0_f32, 10_f32, 20_f32, 20_f32],
            [0, 0, 0, 0],
            [true, true, false, false, false],
            [255, 0, 0, 0, 0],
            "Bullets",
        );
        self.add_image_texture(
            "Bullets_Reload",
            "Resources/assets/images/bullets_reload.png",
            [false, false],
            true,
            ctx,
        );
        self.add_image(
            "Bullets_Reload",
            [0_f32, 10_f32, 20_f32, 20_f32],
            [0, 0, 0, 0],
            [true, true, false, false, false],
            [255, 0, 0, 0, 0],
            "Bullets_Reload",
        );
        self.add_text(
            ["Surplus_Bullets", ""],
            [0_f32, 0_f32, 20_f32, 300_f32, 0.0],
            [255, 255, 255, 255, 0, 0, 0],
            [false, true, false, false],
            false,
            [0, 0, 0, 0],
        );
        self.add_rect(
            "Pause_Background",
            [0_f32, 0_f32, 1280_f32, 720_f32, 0_f32],
            [1, 2, 1, 2],
            [true, true, true, true],
            [0, 0, 0, 125, 255, 255, 255, 255],
            0.0,
        );
        self.add_text(
            ["Pause_Text", ""],
            [0_f32, 0_f32, 40_f32, 500_f32, 0.0],
            [255, 255, 255, 255, 0, 0, 0],
            [true, true, true, true],
            false,
            [1, 2, 1, 4],
        );
        self.add_text(
            ["Target_Point_Text", ""],
            [0_f32, 0_f32, 40_f32, 200_f32, 0.0],
            [255, 255, 255, 255, 0, 0, 0],
            [true, true, false, false],
            false,
            [0, 0, 0, 0],
        );
        self.add_text(
            ["Target_Enemy_Text", ""],
            [0_f32, 0_f32, 40_f32, 200_f32, 0.0],
            [255, 255, 255, 255, 0, 0, 0],
            [true, true, false, false],
            false,
            [0, 0, 0, 0],
        );
        self.add_text(
            ["Bullet_Text", ""],
            [0_f32, 0_f32, 40_f32, 200_f32, 0.0],
            [255, 255, 255, 255, 0, 0, 0],
            [true, true, false, false],
            false,
            [0, 0, 0, 0],
        );
        self.add_text(
            ["Cost_Text", ""],
            [0_f32, 0_f32, 40_f32, 200_f32, 0.0],
            [255, 255, 255, 255, 0, 0, 0],
            [true, true, false, false],
            false,
            [0, 0, 0, 0],
        );
    }

    pub fn fade(
        &mut self,
        fade_in_or_out: bool,
        ctx: &egui::Context,
        ui: &mut Ui,
        split_time_name: &str,
        resource_name: &str,
    ) -> u8 {
        let cut_to_rect_id = self.track_resource(self.resource_rect.clone(), resource_name);
        self.resource_rect[cut_to_rect_id].size =
            [ctx.available_rect().width(), ctx.available_rect().height()];
        if self.timer.now_time - self.split_time(split_time_name)[0] >= self.vertrefresh {
            self.add_split_time(split_time_name, true);
            if fade_in_or_out {
                if self.resource_rect[cut_to_rect_id].color[3] < 255 {
                    for _ in 0..20 {
                        self.resource_rect[cut_to_rect_id].color[3] =
                            self.resource_rect[cut_to_rect_id].color[3].saturating_add(1);
                    }
                };
            } else if self.resource_rect[cut_to_rect_id].color[3] > 0 {
                for _ in 0..20 {
                    if self.resource_rect[cut_to_rect_id].color[3] > 0 {
                        self.resource_rect[cut_to_rect_id].color[3] -= 1;
                    };
                }
            };
        };
        self.rect(ui, resource_name, ctx);
        self.resource_rect[cut_to_rect_id].color[3]
    }

    #[allow(dead_code)]
    pub fn add_enemy(
        &mut self,
        enemy_hp_def_speed_invincible_time_position_activated_time_and_size: [f32; 9],
        enemy_image_count_minus_target_point: [u32; 2],
        enemy_tag_and_move_path: [Vec<String>; 2],
        enemy_name_image_and_type: [String; 3],
        enemy_detected_and_activated: [bool; 2],
        ctx: &egui::Context,
    ) {
        let mut move_path = Vec::new();
        for i in 0..enemy_tag_and_move_path[1].len() {
            if let Ok(json_value) = read_from_json(
                format!(
                    "Resources/config/path_{}.json",
                    enemy_tag_and_move_path[1][i]
                )
                .to_lowercase(),
            ) {
                if let Some(read_path) = MovePath::from_json_value(&json_value) {
                    move_path.push(read_path);
                };
            };
        }
        self.enemy_list.push(Enemy {
            enemy_id: enemy_name_image_and_type[0].clone(),
            enemy_hp: enemy_hp_def_speed_invincible_time_position_activated_time_and_size[0],
            enemy_def: enemy_hp_def_speed_invincible_time_position_activated_time_and_size[1],
            enemy_speed: enemy_hp_def_speed_invincible_time_position_activated_time_and_size[2],
            enemy_invincible_time:
                enemy_hp_def_speed_invincible_time_position_activated_time_and_size[3],
            enemy_image_count: enemy_image_count_minus_target_point[0],
            enemy_tag: enemy_tag_and_move_path[0].clone(),
            enemy_image: enemy_name_image_and_type[1].clone(),
            enemy_image_type: enemy_name_image_and_type[2].clone(),
            enemy_minus_target_point: enemy_image_count_minus_target_point[1],
            enemy_position: [
                enemy_hp_def_speed_invincible_time_position_activated_time_and_size[4],
                enemy_hp_def_speed_invincible_time_position_activated_time_and_size[5],
            ],
            enemy_move_path: move_path,
            enemy_detected: enemy_detected_and_activated[0],
            enemy_activated: enemy_detected_and_activated[1],
            enemy_activated_time:
                enemy_hp_def_speed_invincible_time_position_activated_time_and_size[6],
            enemy_offset: [0_f32, 0_f32],
            enemy_size: [
                enemy_hp_def_speed_invincible_time_position_activated_time_and_size[7],
                enemy_hp_def_speed_invincible_time_position_activated_time_and_size[8],
            ],
        });
        for i in 0..enemy_image_count_minus_target_point[0] {
            if !check_resource_exist(
                self.resource_image_texture.clone(),
                &format!("{}_{}", enemy_name_image_and_type[0], i),
            ) {
                self.add_image_texture(
                    &format!("{}_{}", enemy_name_image_and_type[0], i),
                    &format!(
                        "{}_{}{}",
                        enemy_name_image_and_type[1], i, enemy_name_image_and_type[2]
                    ),
                    [false, false],
                    true,
                    ctx,
                );
            };
        }
        self.add_image(
            &enemy_name_image_and_type[0],
            [
                enemy_hp_def_speed_invincible_time_position_activated_time_and_size[4],
                enemy_hp_def_speed_invincible_time_position_activated_time_and_size[5],
                enemy_hp_def_speed_invincible_time_position_activated_time_and_size[7],
                enemy_hp_def_speed_invincible_time_position_activated_time_and_size[8],
            ],
            [0, 0, 0, 0],
            [false, false, true, false, true],
            [255, 255, 255, 255, 255],
            &format!("{}_0", enemy_name_image_and_type[0]),
        );
    }

    pub fn enemy_refresh(&mut self, ctx: &egui::Context, ui: &Ui) {
        if let Ok(json_value) = read_from_json(self.login_user_config.current_level.clone()) {
            if let Some(read_operation) = Operation::from_json_value(&json_value) {
                for i in 0..read_operation.target_enemy.len() {
                    let mut found_enemy = -1;
                    for u in 0..self.enemy_list.len() {
                        if self.enemy_list[u].enemy_id == format!("json_{}", i) {
                            found_enemy = u as i32;
                            break;
                        };
                    }
                    if found_enemy == -1 {
                        if let Ok(json_value) = read_from_json(
                            format!(
                                "Resources/config/enemy_{}.json",
                                read_operation.target_enemy[i].enemy_recognition_name
                            )
                            .to_lowercase(),
                        ) {
                            if let Some(read_enemy) = JsonReadEnemy::from_json_value(&json_value) {
                                self.add_enemy(
                                    [
                                        read_enemy.enemy_hp,
                                        read_enemy.enemy_def,
                                        read_enemy.enemy_speed,
                                        read_enemy.enemy_invincible_time,
                                        read_operation.target_enemy[i].enemy_position[0],
                                        read_operation.target_enemy[i].enemy_position[1],
                                        read_operation.target_enemy[i].enemy_approach_time,
                                        read_operation.target_enemy[i].enemy_size[0],
                                        read_operation.target_enemy[i].enemy_size[1],
                                    ],
                                    [
                                        read_enemy.enemy_image_count,
                                        read_enemy.enemy_minus_target_point,
                                    ],
                                    [
                                        read_enemy.enemy_tag,
                                        read_operation.target_enemy[i].enemy_path.clone(),
                                    ],
                                    [
                                        format!("json_{}", i),
                                        read_enemy.enemy_image.clone(),
                                        read_enemy.enemy_image_type.clone(),
                                    ],
                                    [true, false],
                                    ctx,
                                );
                            };
                        };
                    };
                }
                for u in 0..self.enemy_list.len() {
                    let id = self.track_resource(
                        self.resource_image.clone(),
                        &self.enemy_list[u].enemy_id.clone(),
                    );
                    self.resource_image[id].origin_position = [
                        (ctx.available_rect().width() - 1280_f32) / 2_f32
                            + self.enemy_list[u].enemy_position[0]
                            + self.enemy_list[u].enemy_offset[0],
                        (ctx.available_rect().height() - 720_f32) / 2_f32
                            + self.enemy_list[u].enemy_position[1]
                            + self.enemy_list[u].enemy_offset[1],
                    ];
                    self.image(ui, &self.enemy_list[u].enemy_id.clone(), ctx);
                }
            };
        };
    }

    pub fn problem_report(
        &mut self,
        problem: &str,
        severity_level: SeverityLevel,
        annotation: &str,
    ) {
        std::thread::spawn(|| {
            kira_play_wav("Resources/assets/sounds/Error.wav").unwrap();
        });
        self.problem_list.push(Problem {
            severity_level,
            problem: problem.to_string(),
            annotation: annotation.to_string(),
            report_state: ReportState {
                current_page: self.page.clone(),
                current_total_runtime: self.timer.total_time,
                current_page_runtime: self.timer.now_time,
            },
        });
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
    pub fn track_resource<T: RustConstructorResource + std::clone::Clone>(
        &mut self,
        resource_list: Vec<T>,
        resource_name: &str,
    ) -> usize {
        let list = resource_list.clone();
        if check_resource_exist(list, resource_name) {
            let mut id: i32 = -1;
            for (i, _a) in resource_list.iter().enumerate() {
                if resource_list[i].name() == resource_name {
                    id = i as i32;
                    break;
                };
            }
            id as usize
        } else {
            if self.config.rc_strict_mode {
                panic!(
                    "{}{}",
                    self.game_text.game_text["error_track_resource_not_found"]
                        [self.config.language as usize]
                        .clone(),
                    resource_name
                )
            } else {
                self.problem_report(
                    &format!(
                        "{}{}",
                        self.game_text.game_text["error_track_resource_not_found"]
                            [self.config.language as usize]
                            .clone(),
                        resource_name
                    ),
                    SeverityLevel::SevereWarning,
                    &self.game_text.game_text["error_track_resource_not_found_annotation"]
                        [self.config.language as usize]
                        .clone(),
                );
            };
            0
        }
    }

    pub fn check_updated(&mut self, name: &str) -> bool {
        let id = self.track_resource(self.resource_page.clone(), name);
        if self.resource_page[id].change_page_updated {
            true
        } else {
            self.new_page_update(name);
            false
        }
    }

    pub fn new_page_update(&mut self, name: &str) {
        self.timer.start_time = self.timer.total_time;
        let page = self.resource_page.clone();
        self.update_timer();
        let id = self.track_resource(page, name);
        self.resource_page[id].change_page_updated = true;
    }

    pub fn wallpaper(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let image = self.resource_image.clone();
        let id = self.track_resource(image, "Home_Wallpaper");
        self.resource_image[id].image_size =
            [ctx.available_rect().width(), ctx.available_rect().height()];
        self.image(ui, "Home_Wallpaper", ctx);
    }

    pub fn dock(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let id = self.track_resource(self.resource_rect.clone(), "Dock_Background");
        if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
            let rect = egui::Rect::from_min_size(
                egui::Pos2::new(0_f32, ctx.available_rect().height() - 80_f32),
                egui::Vec2::new(ctx.available_rect().width(), 80_f32),
            );
            self.modify_var("dock_active_status", rect.contains(mouse_pos));
            let image = self.resource_image.clone();
            if self.timer.now_time - self.split_time("dock_animation")[0] >= self.vertrefresh {
                self.add_split_time("dock_animation", true);
                if self.var_b("dock_active_status") {
                    for _ in 0..5 {
                        if self.resource_rect[id].origin_position[1] > -10_f32 {
                            for i in 0..self.resource_switch.len() {
                                if self.resource_switch[i].name.contains("Home_") {
                                    let id = self.track_resource(
                                        image.clone(),
                                        &self.resource_switch[i].switch_image_name.clone(),
                                    );
                                    self.resource_image[id].origin_position[1] -= 1_f32;
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
                                    let id = self.track_resource(
                                        image.clone(),
                                        &self.resource_switch[i].switch_image_name.clone(),
                                    );
                                    self.resource_image[id].origin_position[1] += 1_f32;
                                };
                            }
                            self.resource_rect[id].origin_position[1] += 1_f32;
                        } else {
                            break;
                        };
                    }
                };
            };
            self.rect(ui, "Dock_Background", ctx);
            if self.switch("Home_Home", ui, ctx, true, true)[0] == 0 {
                self.timer.start_time = self.timer.total_time;
                self.update_timer();
                self.add_split_time("dock_animation", true);
                self.add_split_time("title_animation", true);
                self.switch_page("Home_Page");
            };
            if self.switch("Home_Settings", ui, ctx, true, true)[0] == 0 {
                self.timer.start_time = self.timer.total_time;
                self.update_timer();
                self.add_split_time("dock_animation", true);
                self.switch_page("Home_Setting");
            };
            let id2 = self.track_resource(self.resource_switch.clone(), "Home_Power");
            if self.switch("Home_Power", ui, ctx, true, true)[0] == 0 {
                write_to_json(
                    format!("Resources/config/user_{}.json", self.config.login_user_name),
                    self.login_user_config.to_json_value(),
                )
                .unwrap();
                if self.resource_switch[id2].state == 0 {
                    write_to_json(
                        "Resources/config/Preferences.json",
                        self.config.to_json_value(),
                    )
                    .unwrap();
                    exit(0);
                } else {
                    self.config.login_user_name = "".to_string();
                    self.timer.start_time = self.timer.total_time;
                    self.update_timer();
                    self.add_split_time("ScrollWallpaper", true);
                    self.switch_page("Login");
                }
            };
            if self.switch("Home_Journey", ui, ctx, true, true)[0] == 0 {
                self.timer.start_time = self.timer.total_time;
                self.update_timer();
                self.add_split_time("dock_animation", true);
                if check_resource_exist(self.timer.split_time.clone(), "map_select_animation") {
                    self.add_split_time("map_select_animation", true);
                };
                self.switch_page("Home_Select_Map");
            };
        };
        self.resource_rect[id].size[0] = ctx.available_rect().width() - 100_f32;
    }

    pub fn update_frame_stats(&mut self, ctx: &egui::Context) {
        let current_time = ctx.input(|i| i.time);
        if let Some(last) = self.last_frame_time {
            let delta = (current_time - last) as f32;
            self.frame_times.push(delta);
            const MAX_SAMPLES: usize = 120;
            if self.frame_times.len() > MAX_SAMPLES {
                let remove_count = self.frame_times.len() - MAX_SAMPLES;
                self.frame_times.drain(0..remove_count);
            }
        }
        self.last_frame_time = Some(current_time);
    }

    pub fn current_fps(&self) -> f32 {
        if self.frame_times.is_empty() {
            0.0
        } else {
            1.0 / (self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32)
        }
    }

    pub fn add_split_time(&mut self, name: &str, reset: bool) {
        if reset {
            for i in 0..self.timer.split_time.len() {
                if self.timer.split_time[i].name == name {
                    self.timer.split_time.remove(i);
                    break;
                }
            }
        };
        self.timer.split_time.push(SplitTime {
            discern_type: "SplitTime".to_string(),
            name: name.to_string(),
            time: [self.timer.now_time, self.timer.total_time],
        });
    }

    pub fn split_time(&mut self, name: &str) -> [f32; 2] {
        let id = self.track_resource(self.timer.split_time.clone(), name);
        self.timer.split_time[id].time
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
            discern_type: "CustomRect".to_string(),
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
        let id = self.track_resource(self.resource_rect.clone(), name);
        self.resource_rect[id].reg_render_resource(&mut self.render_resource_list);
        self.resource_rect[id].position[0] = match self.resource_rect[id].x_grid[1] {
            0 => self.resource_rect[id].origin_position[0],
            _ => {
                (ctx.available_rect().width() as f64 / self.resource_rect[id].x_grid[1] as f64
                    * self.resource_rect[id].x_grid[0] as f64) as f32
                    + self.resource_rect[id].origin_position[0]
            }
        };
        self.resource_rect[id].position[1] = match self.resource_rect[id].y_grid[1] {
            0 => self.resource_rect[id].origin_position[1],
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
            discern_type: "Text".to_string(),
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
        let id = self.track_resource(self.resource_text.clone(), name);
        self.resource_text[id].reg_render_resource(&mut self.render_resource_list);
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
            0 => self.resource_text[id].origin_position[0],
            _ => {
                (ctx.available_rect().width() as f64 / self.resource_text[id].x_grid[1] as f64
                    * self.resource_text[id].x_grid[0] as f64) as f32
                    + self.resource_text[id].origin_position[0]
            }
        };
        self.resource_text[id].position[1] = match self.resource_text[id].y_grid[1] {
            0 => self.resource_text[id].origin_position[1],
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

    pub fn get_text_size(&mut self, resource_name: &str, ui: &mut Ui) -> [f32; 2] {
        if check_resource_exist(self.resource_text.clone(), resource_name) {
            let id = self.track_resource(self.resource_text.clone(), resource_name);
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
            [galley.size().x, galley.size().y]
        } else {
            [0_f32, 0_f32]
        }
    }

    fn read_image_to_vec(&mut self, path: &str) -> Vec<u8> {
        let mut file =
            File::open(path).unwrap_or(File::open("Resources/assets/images/error.png").unwrap());
        if !check_file_exists(path) {
            if self.config.rc_strict_mode {
                panic!(
                    "{}: {}",
                    self.game_text.game_text["error_image_open_failed"]
                        [self.config.language as usize],
                    path
                );
            } else {
                self.problem_report(
                    &format!(
                        "{}: {}",
                        self.game_text.game_text["error_image_open_failed"]
                            [self.config.language as usize],
                        path
                    ),
                    SeverityLevel::SevereWarning,
                    &self.game_text.game_text["error_image_open_failed_annotation"]
                        [self.config.language as usize]
                        .clone(),
                );
            };
        };
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        buffer
    }

    pub fn add_var<T: Into<Value>>(&mut self, name: &str, value: T) {
        self.variables.push(Variable {
            discern_type: "Variable".to_string(),
            name: name.to_string(),
            value: value.into(),
        });
    }

    #[allow(dead_code)]
    pub fn modify_var<T: Into<Value>>(&mut self, name: &str, value: T) {
        let id = self.track_resource(self.variables.clone(), name);
        self.variables[id].value = value.into();
    }

    #[allow(dead_code)]
    pub fn var(&mut self, name: &str) -> Value {
        let id = self.track_resource(self.variables.clone(), name);
        self.variables[id].clone().value
    }

    pub fn var_i(&mut self, name: &str) -> i32 {
        if check_resource_exist(self.variables.clone(), name) {
            let id = self.track_resource(self.variables.clone(), name);
            match &self.variables[id].value {
                // 直接访问 value 字段
                Value::Int(i) => *i,
                _ => {
                    if self.config.rc_strict_mode {
                        panic!(
                            "\"{}\" {}",
                            name,
                            self.game_text.game_text["error_variable_not_i32_type"]
                                [self.config.language as usize]
                                .clone()
                        );
                    } else {
                        self.problem_report(
                            &format!(
                                "\"{}\" {}",
                                name,
                                self.game_text.game_text["error_variable_not_i32_type"]
                                    [self.config.language as usize]
                                    .clone()
                            ),
                            SeverityLevel::SevereWarning,
                            &self.game_text.game_text["error_variable_wrong_type_annotation"]
                                [self.config.language as usize]
                                .clone(),
                        );
                        0
                    }
                }
            }
        } else if self.config.rc_strict_mode {
            panic!(
                "\"{}\" {}",
                name,
                self.game_text.game_text["error_variable_not_i32_type"]
                    [self.config.language as usize]
                    .clone()
            );
        } else {
            self.problem_report(
                &format!(
                    "\"{}\" {}",
                    name,
                    self.game_text.game_text["error_variable_not_i32_type"]
                        [self.config.language as usize]
                        .clone()
                ),
                SeverityLevel::SevereWarning,
                &self.game_text.game_text["error_variable_wrong_type_annotation"]
                    [self.config.language as usize]
                    .clone(),
            );
            0
        }
    }

    #[allow(dead_code)]
    pub fn var_u(&mut self, name: &str) -> u32 {
        if check_resource_exist(self.variables.clone(), name) {
            let id = self.track_resource(self.variables.clone(), name);
            match &self.variables[id].value {
                // 直接访问 value 字段
                Value::UInt(u) => *u,
                _ => {
                    if self.config.rc_strict_mode {
                        panic!(
                            "\"{}\" {}",
                            name,
                            self.game_text.game_text["error_variable_not_u32_type"]
                                [self.config.language as usize]
                                .clone()
                        );
                    } else {
                        self.problem_report(
                            &format!(
                                "\"{}\" {}",
                                name,
                                self.game_text.game_text["error_variable_not_u32_type"]
                                    [self.config.language as usize]
                                    .clone()
                            ),
                            SeverityLevel::SevereWarning,
                            &self.game_text.game_text["error_variable_wrong_type_annotation"]
                                [self.config.language as usize]
                                .clone(),
                        );
                        0
                    }
                }
            }
        } else if self.config.rc_strict_mode {
            panic!(
                "\"{}\" {}",
                name,
                self.game_text.game_text["error_variable_not_u32_type"]
                    [self.config.language as usize]
                    .clone()
            );
        } else {
            self.problem_report(
                &format!(
                    "\"{}\" {}",
                    name,
                    self.game_text.game_text["error_variable_not_u32_type"]
                        [self.config.language as usize]
                        .clone()
                ),
                SeverityLevel::SevereWarning,
                &self.game_text.game_text["error_variable_wrong_type_annotation"]
                    [self.config.language as usize]
                    .clone(),
            );
            0
        }
    }

    #[allow(dead_code)]
    pub fn var_f(&mut self, name: &str) -> f32 {
        if check_resource_exist(self.variables.clone(), name) {
            let id = self.track_resource(self.variables.clone(), name);
            match &self.variables[id].value {
                // 直接访问 value 字段
                Value::Float(f) => *f,
                _ => {
                    if self.config.rc_strict_mode {
                        panic!(
                            "\"{}\" {}",
                            name,
                            self.game_text.game_text["error_variable_not_f32_type"]
                                [self.config.language as usize]
                                .clone()
                        );
                    } else {
                        self.problem_report(
                            &format!(
                                "\"{}\" {}",
                                name,
                                self.game_text.game_text["error_variable_not_f32_type"]
                                    [self.config.language as usize]
                                    .clone()
                            ),
                            SeverityLevel::SevereWarning,
                            &self.game_text.game_text["error_variable_wrong_type_annotation"]
                                [self.config.language as usize]
                                .clone(),
                        );
                        0_f32
                    }
                }
            }
        } else if self.config.rc_strict_mode {
            panic!(
                "\"{}\" {}",
                name,
                self.game_text.game_text["error_variable_not_f32_type"]
                    [self.config.language as usize]
                    .clone()
            );
        } else {
            self.problem_report(
                &format!(
                    "\"{}\" {}",
                    name,
                    self.game_text.game_text["error_variable_not_f32_type"]
                        [self.config.language as usize]
                        .clone()
                ),
                SeverityLevel::SevereWarning,
                &self.game_text.game_text["error_variable_wrong_type_annotation"]
                    [self.config.language as usize]
                    .clone(),
            );
            0_f32
        }
    }

    pub fn var_b(&mut self, name: &str) -> bool {
        if check_resource_exist(self.variables.clone(), name) {
            let id = self.track_resource(self.variables.clone(), name);
            match &self.variables[id].value {
                // 直接访问 value 字段
                Value::Bool(b) => *b,
                _ => {
                    if self.config.rc_strict_mode {
                        panic!(
                            "\"{}\" {}",
                            name,
                            self.game_text.game_text["error_variable_not_bool_type"]
                                [self.config.language as usize]
                                .clone()
                        );
                    } else {
                        self.problem_report(
                            &format!(
                                "\"{}\" {}",
                                name,
                                self.game_text.game_text["error_variable_not_bool_type"]
                                    [self.config.language as usize]
                                    .clone()
                            ),
                            SeverityLevel::SevereWarning,
                            &self.game_text.game_text["error_variable_wrong_type_annotation"]
                                [self.config.language as usize]
                                .clone(),
                        );
                        false
                    }
                }
            }
        } else if self.config.rc_strict_mode {
            panic!(
                "\"{}\" {}",
                name,
                self.game_text.game_text["error_variable_not_bool_type"]
                    [self.config.language as usize]
                    .clone()
            );
        } else {
            self.problem_report(
                &format!(
                    "\"{}\" {}",
                    name,
                    self.game_text.game_text["error_variable_not_bool_type"]
                        [self.config.language as usize]
                        .clone()
                ),
                SeverityLevel::SevereWarning,
                &self.game_text.game_text["error_variable_wrong_type_annotation"]
                    [self.config.language as usize]
                    .clone(),
            );
            false
        }
    }

    pub fn var_v(&mut self, name: &str) -> Vec<Value> {
        if check_resource_exist(self.variables.clone(), name) {
            let id = self.track_resource(self.variables.clone(), name);
            match &self.variables[id].value {
                // 直接访问 value 字段
                Value::Vec(v) => v.clone(),
                _ => {
                    if self.config.rc_strict_mode {
                        panic!(
                            "\"{}\" {}",
                            name,
                            self.game_text.game_text["error_variable_not_vec_type"]
                                [self.config.language as usize]
                                .clone()
                        );
                    } else {
                        self.problem_report(
                            &format!(
                                "\"{}\" {}",
                                name,
                                self.game_text.game_text["error_variable_not_vec_type"]
                                    [self.config.language as usize]
                                    .clone()
                            ),
                            SeverityLevel::SevereWarning,
                            &self.game_text.game_text["error_variable_wrong_type_annotation"]
                                [self.config.language as usize]
                                .clone(),
                        );
                        Vec::new()
                    }
                }
            }
        } else if self.config.rc_strict_mode {
            panic!(
                "\"{}\" {}",
                name,
                self.game_text.game_text["error_variable_not_vec_type"]
                    [self.config.language as usize]
                    .clone()
            );
        } else {
            self.problem_report(
                &format!(
                    "\"{}\" {}",
                    name,
                    self.game_text.game_text["error_variable_not_vec_type"]
                        [self.config.language as usize]
                        .clone()
                ),
                SeverityLevel::SevereWarning,
                &self.game_text.game_text["error_variable_wrong_type_annotation"]
                    [self.config.language as usize]
                    .clone(),
            );
            Vec::new()
        }
    }

    pub fn var_s(&mut self, name: &str) -> String {
        if check_resource_exist(self.variables.clone(), name) {
            let id = self.track_resource(self.variables.clone(), name);
            match &self.variables[id].value {
                // 直接访问 value 字段
                Value::String(s) => s.clone(),
                _ => {
                    if self.config.rc_strict_mode {
                        panic!(
                            "\"{}\" {}",
                            name,
                            self.game_text.game_text["error_variable_not_string_type"]
                                [self.config.language as usize]
                                .clone()
                        );
                    } else {
                        self.problem_report(
                            &format!(
                                "\"{}\" {}",
                                name,
                                self.game_text.game_text["error_variable_not_string_type"]
                                    [self.config.language as usize]
                                    .clone()
                            ),
                            SeverityLevel::SevereWarning,
                            &self.game_text.game_text["error_variable_wrong_type_annotation"]
                                [self.config.language as usize]
                                .clone(),
                        );
                        String::new()
                    }
                }
            }
        } else if self.config.rc_strict_mode {
            panic!(
                "\"{}\" {}",
                name,
                self.game_text.game_text["error_variable_not_string_type"]
                    [self.config.language as usize]
                    .clone()
            );
        } else {
            self.problem_report(
                &format!(
                    "\"{}\" {}",
                    name,
                    self.game_text.game_text["error_variable_not_string_type"]
                        [self.config.language as usize]
                        .clone()
                ),
                SeverityLevel::SevereWarning,
                &self.game_text.game_text["error_variable_wrong_type_annotation"]
                    [self.config.language as usize]
                    .clone(),
            );
            String::new()
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
                if self.config.rc_strict_mode {
                    panic!(
                        "\"{:?}\" {}",
                        target,
                        self.game_text.game_text["error_variable_not_bool_type"]
                            [self.config.language as usize]
                            .clone()
                    );
                } else {
                    self.problem_report(
                        &format!(
                            "\"{:?}\" {}",
                            target,
                            self.game_text.game_text["error_variable_not_bool_type"]
                                [self.config.language as usize]
                                .clone()
                        ),
                        SeverityLevel::SevereWarning,
                        &self.game_text.game_text["error_variable_wrong_type_annotation"]
                            [self.config.language as usize]
                            .clone(),
                    );
                    false
                }
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
                if self.config.rc_strict_mode {
                    panic!(
                        "\"{:?}\" {}",
                        target,
                        self.game_text.game_text["error_variable_not_i32_type"]
                            [self.config.language as usize]
                            .clone()
                    );
                } else {
                    self.problem_report(
                        &format!(
                            "\"{:?}\" {}",
                            target,
                            self.game_text.game_text["error_variable_not_i32_type"]
                                [self.config.language as usize]
                                .clone()
                        ),
                        SeverityLevel::SevereWarning,
                        &self.game_text.game_text["error_variable_wrong_type_annotation"]
                            [self.config.language as usize]
                            .clone(),
                    );
                    0
                }
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
                if self.config.rc_strict_mode {
                    panic!(
                        "\"{:?}\" {}",
                        target,
                        self.game_text.game_text["error_variable_not_u32_type"]
                            [self.config.language as usize]
                            .clone()
                    );
                } else {
                    self.problem_report(
                        &format!(
                            "\"{:?}\" {}",
                            target,
                            self.game_text.game_text["error_variable_not_u32_type"]
                                [self.config.language as usize]
                                .clone()
                        ),
                        SeverityLevel::SevereWarning,
                        &self.game_text.game_text["error_variable_wrong_type_annotation"]
                            [self.config.language as usize]
                            .clone(),
                    );
                    0
                }
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
                if self.config.rc_strict_mode {
                    panic!(
                        "\"{:?}\" {}",
                        target,
                        self.game_text.game_text["error_variable_not_f32_type"]
                            [self.config.language as usize]
                            .clone()
                    );
                } else {
                    self.problem_report(
                        &format!(
                            "\"{:?}\" {}",
                            target,
                            self.game_text.game_text["error_variable_not_f32_type"]
                                [self.config.language as usize]
                                .clone()
                        ),
                        SeverityLevel::SevereWarning,
                        &self.game_text.game_text["error_variable_wrong_type_annotation"]
                            [self.config.language as usize]
                            .clone(),
                    );
                    0_f32
                }
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
                if self.config.rc_strict_mode {
                    panic!(
                        "\"{:?}\" {}",
                        target,
                        self.game_text.game_text["error_variable_not_string_type"]
                            [self.config.language as usize]
                            .clone()
                    );
                } else {
                    self.problem_report(
                        &format!(
                            "\"{:?}\" {}",
                            target,
                            self.game_text.game_text["error_variable_not_string_type"]
                                [self.config.language as usize]
                                .clone()
                        ),
                        SeverityLevel::SevereWarning,
                        &self.game_text.game_text["error_variable_wrong_type_annotation"]
                            [self.config.language as usize]
                            .clone(),
                    );
                    String::new()
                }
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
            image_id.push(self.track_resource(self.resource_image.clone(), &i));
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
                self.resource_image[image_id[count]].origin_position =
                    [temp_position, size_position_boundary[3]];
            } else {
                for _j in 0..count {
                    if left_and_top_or_right_and_bottom {
                        temp_position += size_position_boundary[1];
                    } else {
                        temp_position -= size_position_boundary[1];
                    };
                }
                self.resource_image[image_id[count]].origin_position =
                    [size_position_boundary[2], temp_position];
            };
        }
        let resume_point = if horizontal_or_vertical {
            self.resource_image[image_id[image_id.len() - 1]].origin_position[0]
        } else {
            self.resource_image[image_id[image_id.len() - 1]].origin_position[1]
        };
        self.resource_scroll_background.push(ScrollBackground {
            discern_type: "ScrollBackground".to_string(),
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
        let id = self.track_resource(self.resource_scroll_background.clone(), name);
        self.resource_scroll_background[id].reg_render_resource(&mut self.render_resource_list);
        if !check_resource_exist(self.timer.split_time.clone(), name) {
            self.add_split_time(name, false);
        };
        let mut id2;
        for i in 0..self.resource_scroll_background[id].image_name.len() {
            self.image(
                ui,
                &self.resource_scroll_background[id].image_name[i].clone(),
                ctx,
            );
        }
        if self.timer.now_time - self.split_time(name)[0] >= self.vertrefresh {
            self.add_split_time(name, true);
            for i in 0..self.resource_scroll_background[id].image_name.len() {
                id2 = self.track_resource(
                    self.resource_image.clone(),
                    &self.resource_scroll_background[id].image_name[i].clone(),
                );
                if self.resource_scroll_background[id].horizontal_or_vertical {
                    if self.resource_scroll_background[id].left_and_top_or_right_and_bottom {
                        for _j in 0..self.resource_scroll_background[id].scroll_speed {
                            self.resource_image[id2].origin_position[0] -= 1_f32;
                            self.scroll_background_check_boundary(id, id2);
                        }
                    } else {
                        for _j in 0..self.resource_scroll_background[id].scroll_speed {
                            self.resource_image[id2].origin_position[0] += 1_f32;
                            self.scroll_background_check_boundary(id, id2);
                        }
                    };
                } else if self.resource_scroll_background[id].left_and_top_or_right_and_bottom {
                    for _j in 0..self.resource_scroll_background[id].scroll_speed {
                        self.resource_image[id2].origin_position[1] -= 1_f32;
                        self.scroll_background_check_boundary(id, id2);
                    }
                } else {
                    for _j in 0..self.resource_scroll_background[id].scroll_speed {
                        self.resource_image[id2].origin_position[1] += 1_f32;
                        self.scroll_background_check_boundary(id, id2);
                    }
                };
            }
        };
    }

    fn scroll_background_check_boundary(&mut self, id: usize, id2: usize) {
        if self.resource_scroll_background[id].horizontal_or_vertical {
            if self.resource_scroll_background[id].left_and_top_or_right_and_bottom {
                if self.resource_image[id2].origin_position[0]
                    <= self.resource_scroll_background[id].boundary
                {
                    self.resource_image[id2].origin_position[0] =
                        self.resource_scroll_background[id].resume_point;
                };
            } else if self.resource_image[id2].origin_position[0]
                >= self.resource_scroll_background[id].boundary
            {
                self.resource_image[id2].origin_position[0] =
                    self.resource_scroll_background[id].resume_point;
            };
        } else if self.resource_scroll_background[id].left_and_top_or_right_and_bottom {
            if self.resource_image[id2].origin_position[1]
                <= self.resource_scroll_background[id].boundary
            {
                self.resource_image[id2].origin_position[1] =
                    self.resource_scroll_background[id].resume_point;
            };
        } else if self.resource_image[id2].origin_position[1]
            >= self.resource_scroll_background[id].boundary
        {
            self.resource_image[id2].origin_position[1] =
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
                discern_type: "ImageTexture".to_string(),
                name: name.to_string(),
                texture: image_texture,
                cite_path: path.to_string(),
            });
        } else {
            let id = self.track_resource(self.resource_image_texture.clone(), name);
            self.resource_image_texture[id].texture = image_texture;
            self.resource_image_texture[id].cite_path = path.to_string();
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
        let id = self.track_resource(self.resource_image_texture.clone(), image_texture_name);
        self.resource_image.push(Image {
            discern_type: "Image".to_string(),
            name: name.to_string(),
            image_texture: self.resource_image_texture[id].texture.clone(),
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
            origin_cite_texture: image_texture_name.to_string(),
        });
    }

    pub fn image(&mut self, ui: &Ui, name: &str, ctx: &egui::Context) {
        let id = self.track_resource(self.resource_image.clone(), name);
        self.resource_image[id].reg_render_resource(&mut self.render_resource_list);
        self.resource_image[id].image_position[0] = match self.resource_image[id].x_grid[1] {
            0 => self.resource_image[id].origin_position[0],
            _ => {
                (ctx.available_rect().width() as f64 / self.resource_image[id].x_grid[1] as f64
                    * self.resource_image[id].x_grid[0] as f64) as f32
                    + self.resource_image[id].origin_position[0]
            }
        };
        self.resource_image[id].image_position[1] = match self.resource_image[id].y_grid[1] {
            0 => self.resource_image[id].origin_position[1],
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
        mut appearance: Vec<SwitchData>,
        enable_hover_click_image_and_use_overlay: [bool; 3],
        switch_amounts_state: u32,
        click_method: Vec<SwitchClickAction>,
    ) {
        let mut count = 1;
        if enable_hover_click_image_and_use_overlay[0] {
            count += 1;
        };
        if enable_hover_click_image_and_use_overlay[1] {
            count += 1;
        };
        if appearance.len() as u32 != count * switch_amounts_state {
            if self.config.rc_strict_mode {
                panic!(
                    "{}{}:{}",
                    name_and_switch_image_name[0],
                    self.game_text.game_text["error_switch_appearance_mismatch"]
                        [self.config.language as usize],
                    count * switch_amounts_state - appearance.len() as u32
                );
            } else {
                self.problem_report(
                    &format!(
                        "{}{}:{}",
                        name_and_switch_image_name[0],
                        self.game_text.game_text["error_switch_appearance_mismatch"]
                            [self.config.language as usize],
                        count * switch_amounts_state - appearance.len() as u32
                    ),
                    SeverityLevel::MildWarning,
                    &self.game_text.game_text["error_switch_appearance_mismatch_annotation"]
                        [self.config.language as usize]
                        .clone(),
                );
                for _ in 0..count * switch_amounts_state - appearance.len() as u32 {
                    appearance.push(SwitchData {
                        texture: "Error".to_string(),
                        color: [255, 255, 255, 255],
                    });
                }
            };
        };
        let id = self.track_resource(self.resource_image.clone(), name_and_switch_image_name[1]);
        self.resource_image[id].use_overlay_color = true;
        self.resource_switch.push(Switch {
            discern_type: "Switch".to_string(),
            name: name_and_switch_image_name[0].to_string(),
            appearance,
            switch_image_name: name_and_switch_image_name[1].to_string(),
            enable_hover_click_image: [
                enable_hover_click_image_and_use_overlay[0],
                enable_hover_click_image_and_use_overlay[1],
            ],
            state: 0,
            click_method,
            last_time_clicked: false,
            last_time_clicked_index: 0,
            animation_count: count,
        });
    }

    #[allow(dead_code)]
    pub fn switch(
        &mut self,
        name: &str,
        ui: &mut Ui,
        ctx: &egui::Context,
        enable: bool,
        play_sound: bool,
    ) -> [usize; 2] {
        let mut activated = [5, 0];
        let id = self.track_resource(self.resource_switch.clone(), name);
        self.resource_switch[id].reg_render_resource(&mut self.render_resource_list);
        let id2 = self.track_resource(
            self.resource_image.clone(),
            &self.resource_switch[id].switch_image_name.clone(),
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
                            i.pointer.button_down(
                                self.resource_switch[id].click_method[u as usize].click_method,
                            )
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
                            if self.resource_switch[id].enable_hover_click_image[0] {
                                self.resource_image[id2].overlay_color = self.resource_switch[id]
                                    .appearance[(self
                                    .resource_switch[id]
                                    .state
                                    * self.resource_switch[id].animation_count
                                    + 2) as usize]
                                    .color;
                                id3 = self.track_resource(
                                    self.resource_image_texture.clone(),
                                    &self.resource_switch[id].appearance[(self.resource_switch[id]
                                        .state
                                        * self.resource_switch[id].animation_count
                                        + 2)
                                        as usize]
                                        .texture
                                        .clone(),
                                );
                                self.resource_image[id2].image_texture =
                                    self.resource_image_texture[id3].texture.clone();
                            } else {
                                self.resource_image[id2].overlay_color = self.resource_switch[id]
                                    .appearance[(self
                                    .resource_switch[id]
                                    .state
                                    * self.resource_switch[id].animation_count
                                    + 1) as usize]
                                    .color;
                                id3 = self.track_resource(
                                    self.resource_image_texture.clone(),
                                    &self.resource_switch[id].appearance[(self.resource_switch[id]
                                        .state
                                        * self.resource_switch[id].animation_count
                                        + 1)
                                        as usize]
                                        .texture
                                        .clone(),
                                );
                                self.resource_image[id2].image_texture =
                                    self.resource_image_texture[id3].texture.clone();
                            };
                        } else if !self.resource_switch[id].enable_hover_click_image[0] {
                            self.resource_image[id2].overlay_color =
                                self.resource_switch[id].appearance[(self.resource_switch[id].state
                                    * self.resource_switch[id].animation_count)
                                    as usize]
                                    .color;
                            id3 = self.track_resource(
                                self.resource_image_texture.clone(),
                                &self.resource_switch[id].appearance[(self.resource_switch[id]
                                    .state
                                    * self.resource_switch[id].animation_count)
                                    as usize]
                                    .texture
                                    .clone(),
                            );
                            self.resource_image[id2].image_texture =
                                self.resource_image_texture[id3].texture.clone();
                        };
                    } else {
                        if self.resource_switch[id].last_time_clicked {
                            if play_sound {
                                general_click_feedback();
                            };
                            let mut count = 1;
                            if self.resource_switch[id].enable_hover_click_image[0] {
                                count += 1;
                            };
                            if self.resource_switch[id].enable_hover_click_image[1] {
                                count += 1;
                            };
                            if self.resource_switch[id].click_method
                                [self.resource_switch[id].last_time_clicked_index]
                                .action
                            {
                                if self.resource_switch[id].state
                                    < (self.resource_switch[id].appearance.len() / count - 1) as u32
                                {
                                    self.resource_switch[id].state += 1;
                                } else {
                                    self.resource_switch[id].state = 0;
                                };
                            };
                            activated[0] = self.resource_switch[id].last_time_clicked_index;
                            self.resource_switch[id].last_time_clicked = false;
                        };
                        if self.resource_switch[id].enable_hover_click_image[0] {
                            self.resource_image[id2].overlay_color = self.resource_switch[id]
                                .appearance[(self.resource_switch[id]
                                .state
                                * self.resource_switch[id].animation_count
                                + 1) as usize]
                                .color;
                            id3 = self.track_resource(
                                self.resource_image_texture.clone(),
                                &self.resource_switch[id].appearance[(self.resource_switch[id]
                                    .state
                                    * self.resource_switch[id].animation_count
                                    + 1)
                                    as usize]
                                    .texture
                                    .clone(),
                            );
                            self.resource_image[id2].image_texture =
                                self.resource_image_texture[id3].texture.clone();
                        } else {
                            self.resource_image[id2].overlay_color =
                                self.resource_switch[id].appearance[(self.resource_switch[id].state
                                    * self.resource_switch[id].animation_count)
                                    as usize]
                                    .color;
                            id3 = self.track_resource(
                                self.resource_image_texture.clone(),
                                &self.resource_switch[id].appearance[(self.resource_switch[id]
                                    .state
                                    * self.resource_switch[id].animation_count)
                                    as usize]
                                    .texture
                                    .clone(),
                            );
                            self.resource_image[id2].image_texture =
                                self.resource_image_texture[id3].texture.clone();
                        };
                    };
                } else {
                    self.resource_switch[id].last_time_clicked = false;
                    self.resource_image[id2].overlay_color = self.resource_switch[id].appearance
                        [(self.resource_switch[id].state * self.resource_switch[id].animation_count)
                            as usize]
                        .color;
                    id3 = self.track_resource(
                        self.resource_image_texture.clone(),
                        &self.resource_switch[id].appearance[(self.resource_switch[id].state
                            * self.resource_switch[id].animation_count)
                            as usize]
                            .texture
                            .clone(),
                    );
                    self.resource_image[id2].image_texture =
                        self.resource_image_texture[id3].texture.clone();
                };
            };
        } else {
            self.resource_switch[id].last_time_clicked = false;
            self.resource_image[id2].overlay_color =
                self.resource_switch[id].appearance[(self.resource_switch[id].state
                    * self.resource_switch[id].animation_count)
                    as usize]
                    .color;
            id3 = self.track_resource(
                self.resource_image_texture.clone(),
                &self.resource_switch[id].appearance[(self.resource_switch[id].state
                    * self.resource_switch[id].animation_count)
                    as usize]
                    .texture
                    .clone(),
            );
            self.resource_image[id2].image_texture =
                self.resource_image_texture[id3].texture.clone();
        };
        self.image(ui, &self.resource_switch[id].switch_image_name.clone(), ctx);
        activated[1] = self.resource_switch[id].state as usize;
        activated
    }
}
