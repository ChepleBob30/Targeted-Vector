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
use std::thread;
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

#[derive(Debug, Clone)]
pub struct Config {
    pub launch_path: String,
    pub language: u8,
    pub login_user_name: String,
    pub amount_languages: u8,
    pub rc_strict_mode: bool,
    pub enable_debug_mode: bool,
}

impl Config {
    pub fn from_json_value(value: &JsonValue) -> Option<Config> {
        Some(Config {
            launch_path: value["launch_path"].as_str()?.to_string(),
            language: value["language"].as_u8()?,
            login_user_name: value["login_user_name"].as_str()?.to_string(),
            amount_languages: value["amount_languages"].as_u8()?,
            rc_strict_mode: value["rc_strict_mode"].as_bool()?,
            enable_debug_mode: value["enable_debug_mode"].as_bool()?,
        })
    }
    pub fn to_json_value(&self) -> JsonValue {
        json::object! {
            launch_path: self.launch_path.clone(),
            language: self.language,
            login_user_name: self.login_user_name.clone(),
            amount_languages: self.amount_languages,
            rc_strict_mode: self.rc_strict_mode,
            enable_debug_mode: self.enable_debug_mode,
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
    pub operation_background: String,
    pub operation_background_expand: String,
    pub operation_start_background: String,
    pub operation_over_background: String,
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
            operation_background: value["operation_background"].as_str()?.to_string(),
            operation_background_expand: value["operation_background_expand"].as_str()?.to_string(),
            operation_start_background: value["operation_start_background"].as_str()?.to_string(),
            operation_over_background: value["operation_over_background"].as_str()?.to_string(),
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
    pub enemy_approach_alpha: u8,
    pub enemy_increase_alpha_speed: u8,
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
            enemy_approach_alpha: value["enemy_approach_alpha"].as_u8()?,
            enemy_increase_alpha_speed: value["enemy_increase_alpha_speed"].as_u8()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct OperationMessageBox {
    pub box_size: [f32; 2],
    pub box_image_path: String,
    pub box_title: Vec<String>,
    pub box_content: Vec<String>,
    pub box_title_color: [u8; 4],
    pub box_content_color: [u8; 4],
    pub box_existing_time: f32,
    pub box_appear_time: f32,
    pub box_enable: bool,
}

impl OperationMessageBox {
    pub fn from_json_value(value: &JsonValue) -> Option<OperationMessageBox> {
        Some(OperationMessageBox {
            box_size: [
                value["box_size"][0].as_f32()?,
                value["box_size"][1].as_f32()?,
            ],
            box_image_path: value["box_image_path"].as_str()?.to_string(),
            box_title: value["box_title"]
                .members()
                .map(|s| s.to_string())
                .collect(),
            box_content: value["box_content"]
                .members()
                .map(|s| s.to_string())
                .collect(),
            box_title_color: [
                value["box_title_color"][0].as_u8()?,
                value["box_title_color"][1].as_u8()?,
                value["box_title_color"][2].as_u8()?,
                value["box_title_color"][3].as_u8()?,
            ],
            box_content_color: [
                value["box_content_color"][0].as_u8()?,
                value["box_content_color"][1].as_u8()?,
                value["box_content_color"][2].as_u8()?,
                value["box_content_color"][3].as_u8()?,
            ],
            box_existing_time: value["box_existing_time"].as_f32()?,
            box_appear_time: value["box_appear_time"].as_f32()?,
            box_enable: true,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub global: OperationGlobal,
    pub target_enemy: Vec<OperationTargetEnemy>,
    pub message_box: Vec<OperationMessageBox>,
}

impl Operation {
    pub fn from_json_value(value: &JsonValue) -> Option<Operation> {
        Some(Operation {
            global: OperationGlobal::from_json_value(&value["global"])?,
            target_enemy: value["target_enemy"]
                .members()
                .map(OperationTargetEnemy::from_json_value)
                .collect::<Option<Vec<_>>>()?,
            message_box: value["message_box"]
                .members()
                .map(OperationMessageBox::from_json_value)
                .collect::<Option<Vec<_>>>()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct MovePath {
    pub move_status: [bool; 4],
    pub move_time: f32,
}

impl MovePath {
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
    pub enemy_name: String,
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
    pub enemy_size: [f32; 2],
    pub enemy_current_walk_status: u32,
    pub enemy_start_walk_time: f32,
    pub enemy_increase_alpha_speed: u8,
    pub enemy_animation_forward: bool,
    pub enemy_current_animation_count: u32,
    pub enemy_walk_interval: f32,
    pub enemy_walk_time: f32,
    pub enemy_animation_interval: f32,
    pub enemy_animation_change_time: f32,
    pub enemy_out: bool,
    pub enemy_hit_time: f32,
    pub enemy_initial_hp: f32,
    pub enemy_memory_hp: f32,
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
    pub enemy_walk_interval: f32,
    pub enemy_animation_interval: f32,
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
            enemy_walk_interval: value["enemy_walk_interval"].as_f32()?,
            enemy_animation_interval: value["enemy_animation_interval"].as_f32()?,
        })
    }
}

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

#[derive(Debug, Clone)]
pub struct PauseMessage {
    pub start_pause_time: f32,
    pub pause_total_time: f32,
    pub mentioned: bool,
}

#[derive(Debug, Clone)]
pub struct Level {
    pub level_name: String,
    pub level_name_expand: Vec<String>,
    pub level_description: Vec<String>,
    pub level_type: String,
    pub level_position: [f32; 2],
    pub level_initial_status: bool,
    pub unlock_map: Vec<UnlockMap>,
    pub unlock_level: Vec<UnlockLevel>,
}

#[derive(Debug, Clone)]
pub struct UnlockMap {
    pub map_name: String,
    pub require_perfect_clear: bool,
}

impl UnlockMap {
    pub fn from_json_value(value: &JsonValue) -> Option<UnlockMap> {
        Some(Self {
            map_name: value["map_name"].as_str()?.to_string(),
            require_perfect_clear: value["require_perfect_clear"].as_bool()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct UnlockLevel {
    pub level_name: String,
    pub level_map: String,
    pub require_perfect_clear: bool,
}

impl UnlockLevel {
    pub fn from_json_value(value: &JsonValue) -> Option<UnlockLevel> {
        Some(Self {
            level_name: value["level_name"].as_str()?.to_string(),
            level_map: value["level_map"].as_str()?.to_string(),
            require_perfect_clear: value["require_perfect_clear"].as_bool()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Map {
    pub map_name: Vec<String>,
    pub map_author: String,
    pub map_image: String,
    pub map_width: f32,
    pub map_scroll_offset: f32,
    pub map_description: Vec<String>,
    pub map_intro: String,
    pub map_content: Vec<Level>,
    pub map_connecting_line: Vec<[String; 2]>,
    pub map_initial_unlock_status: bool,
    pub map_unlock_description: Vec<String>,
    pub map_lock_intro: String,
}

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
                        level_initial_status: v["level_initial_status"].as_bool()?,
                        unlock_map: v["unlock_map"]
                            .members()
                            .filter_map(UnlockMap::from_json_value)
                            .collect(),
                        unlock_level: v["unlock_level"]
                            .members()
                            .filter_map(UnlockLevel::from_json_value)
                            .collect(),
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
            map_initial_unlock_status: value["map_initial_unlock_status"].as_bool()?,
            map_unlock_description: value["map_unlock_description"]
                .members()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            map_lock_intro: value["map_lock_intro"].as_str()?.to_string(),
        })
    }
    pub fn to_json_value(&self) -> JsonValue {
        json::object! {
            map_name: self.map_name.clone(),
            map_author: self.map_author.clone(),
            map_image: self.map_image.clone(),
            map_width: self.map_width,
            map_scroll_offset: self.map_scroll_offset,
            map_description: self.map_description.clone(),
            map_intro: self.map_intro.clone(),
            map_content: self.map_content.iter().map(|l| {
                json::object! {
                    level_name: l.level_name.clone(),
                    level_name_expand: l.level_name_expand.clone(),
                    level_description: l.level_description.clone(),
                    level_type: l.level_type.clone(),
                    level_position: [l.level_position[0], l.level_position[1]],
                    level_initial_status: l.level_initial_status, // 新增缺失字段
                    unlock_map: l.unlock_map.iter().map(|x| json::object! { map_name: x.map_name.clone(), require_perfect_clear: x.require_perfect_clear }).collect::<Vec<_>>(),
                    unlock_level: l.unlock_level.iter().map(|x| json::object! { level_name: x.level_name.clone(), level_map: x.level_map.clone(), require_perfect_clear: x.require_perfect_clear }).collect::<Vec<_>>(),
                }
            }).collect::<Vec<_>>(),
            map_connecting_line: self.map_connecting_line.iter().map(|pair| { // 新增连接线字段
                json::array![pair[0].clone(), pair[1].clone()]
            }).collect::<Vec<_>>(),
            map_initial_unlock_status: self.map_initial_unlock_status,
            map_unlock_description: self.map_unlock_description.clone(),
            map_lock_intro: self.map_lock_intro.clone(),
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
    pub gun_initial_unlock: bool,
    pub gun_no_bullet_shoot_sound: String,
    pub gun_reload_sound: String,
    pub gun_reload_bullet_sound: String,
    pub gun_reload_interval: f32,
    pub gun_overheating_sound: String,
}

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
            gun_initial_unlock: value["gun_initial_unlock"].as_bool()?,
            gun_no_bullet_shoot_sound: value["gun_no_bullet_shoot_sound"].as_str()?.to_string(),
            gun_reload_bullet_sound: value["gun_reload_bullet_sound"].as_str()?.to_string(),
            gun_reload_sound: value["gun_reload_sound"].as_str()?.to_string(),
            gun_reload_interval: value["gun_reload_interval"].as_f32()?,
            gun_overheating_sound: value["gun_overheating_sound"].as_str()?.to_string(),
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

#[derive(Debug, Clone)]
pub struct UserMapStatus {
    pub map_name: String,
    pub map_unlock_status: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct User {
    pub name: String,
    pub password: String,
    pub language: u8,
    pub wallpaper: String,
    pub current_map: String,
    pub level_status: Vec<UserLevelStatus>,
    pub gun_status: Vec<UserGunStatus>,
    pub map_status: Vec<UserMapStatus>,
    pub settings: HashMap<String, String>,
    pub current_level: String,
}

#[allow(dead_code)]
impl User {
    pub fn from_json_value(value: &JsonValue) -> Option<User> {
        let mut parsed = HashMap::new();
        for (key, val) in value["settings"].entries() {
            parsed.insert(key.to_string(), val.to_string());
        }
        Some(User {
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
            map_status: value["map_status"]
                .members()
                .filter_map(|v| {
                    Some(UserMapStatus {
                        map_name: v["map_name"].as_str()?.to_string(),
                        map_unlock_status: v["map_unlock_status"].as_bool()?,
                    })
                })
                .collect(),
            settings: parsed,
            current_level: value["current_level"].as_str()?.to_string(),
        })
    }

    pub fn to_json_value(&self) -> JsonValue {
        json::object! {
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
            map_status: self.map_status.iter().map(|l| json::object! {
                map_name: l.map_name.clone(),
                map_status: l.map_unlock_status,
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
    resource_list.iter().any(|x| x.name() == resource_name)
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

#[derive(Clone, Debug)]
pub struct MessageBox {
    pub discern_type: String,
    pub name: String,
    pub box_size: [f32; 2],
    pub box_content_name: String,
    pub box_title_name: String,
    pub box_image_name: String,
    pub box_keep_existing: bool,
    pub box_existing_time: f32,
    pub box_exist: bool,
    pub box_speed: f32,
    pub box_restore_speed: f32,
    pub box_memory_offset: f32,
}

impl RustConstructorResource for MessageBox {
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
    pub pause_list: Vec<PauseMessage>,
    pub resource_message_box: Vec<MessageBox>,
    pub operation_preload_message_box: Vec<OperationMessageBox>,
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
            enable_debug_mode: false,
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
                name: "".to_string(),
                password: "".to_string(),
                language: 0,
                wallpaper: "".to_string(),
                current_map: "".to_string(),
                level_status: vec![],
                gun_status: vec![],
                map_status: vec![],
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
                    name: "Operation_Result".to_string(),
                    forced_update: true,
                    change_page_updated: false,
                },
                PageData {
                    discern_type: "PageData".to_string(),
                    name: "Error".to_string(),
                    forced_update: true,
                    change_page_updated: false,
                },
                PageData {
                    discern_type: "PageData".to_string(),
                    name: "Editor".to_string(),
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
            pause_list: Vec::new(),
            resource_message_box: Vec::new(),
            operation_preload_message_box: Vec::new(),
        }
    }

    pub fn switch_page(&mut self, page: &str) {
        self.page = page.to_string();
        self.timer.start_time = self.timer.total_time;
        self.update_timer();
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
            [0_f32, 0_f32, 600_f32, ctx.available_rect().height(), 0_f32],
            [1, 1, 1, 2],
            [true, false, false, true],
            [0, 0, 0, 240, 255, 255, 255, 255],
            0.0,
        );
        self.add_text(
            ["Level_Title", ""],
            [-200_f32, 30_f32, 60_f32, 500_f32, 0.0],
            [255, 255, 255, 255, 0, 0, 0],
            [true, true, true, false],
            false,
            [1, 1, 0, 0],
        );
        self.add_text(
            ["Level_Description", ""],
            [-200_f32, 200_f32, 20_f32, 500_f32, 0.0],
            [255, 255, 255, 255, 0, 0, 0],
            [true, true, true, false],
            false,
            [1, 1, 0, 0],
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
        self.add_rect(
            "Operation_Win_Background",
            [0_f32, 0_f32, ctx.available_rect().width(), 300_f32, 0_f32],
            [0, 0, 1, 2],
            [true, false, false, true],
            [0, 0, 0, 255, 255, 255, 255, 255],
            0.0,
        );
        self.add_text(
            ["Operation_Win_Text", ""],
            [0_f32, 0_f32, 80_f32, 1000_f32, 0.0],
            [255, 255, 255, 255, 0, 0, 0],
            [false, false, true, true],
            false,
            [0, 0, 1, 2],
        );
        self.add_rect(
            "Operation_Fail_Background",
            [
                0_f32,
                0_f32,
                ctx.available_rect().width(),
                ctx.available_rect().height(),
                0_f32,
            ],
            [1, 2, 1, 2],
            [false, false, true, true],
            [255, 0, 0, 0, 255, 255, 255, 255],
            0.0,
        );
        self.add_image_texture(
            "Operation_Over",
            "Resources/assets/images/start_operation.png",
            [true, false],
            true,
            ctx,
        );
        self.add_image(
            "Operation_Over",
            [0_f32, 0_f32, 50_f32, 50_f32],
            [1, 2, 3, 4],
            [false, false, true, true, false],
            [255, 0, 0, 0, 0],
            "Operation_Over",
        );
        self.add_switch(
            ["Operation_Over", "Operation_Over"],
            vec![
                SwitchData {
                    texture: "Operation_Over".to_string(),
                    color: [255, 255, 255, 255],
                },
                SwitchData {
                    texture: "Operation_Over".to_string(),
                    color: [180, 180, 180, 255],
                },
                SwitchData {
                    texture: "Operation_Over".to_string(),
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
            "Refresh",
            "Resources/assets/images/refresh.png",
            [true, false],
            true,
            ctx,
        );
        self.add_image(
            "Refresh",
            [150_f32, -150_f32, 50_f32, 50_f32],
            [1, 2, 1, 1],
            [true, false, false, false, false],
            [255, 0, 0, 0, 0],
            "Refresh",
        );
        self.add_switch(
            ["Refresh", "Refresh"],
            vec![
                SwitchData {
                    texture: "Refresh".to_string(),
                    color: [255, 255, 255, 255],
                },
                SwitchData {
                    texture: "Refresh".to_string(),
                    color: [180, 180, 180, 255],
                },
                SwitchData {
                    texture: "Refresh".to_string(),
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
            "Close_Message_Box",
            "Resources/assets/images/close_message_box.png",
            [false, false],
            true,
            ctx,
        );
        self.add_image_texture(
            "Editor",
            "Resources/assets/images/editor.png",
            [false, false],
            true,
            ctx,
        );
        self.add_image(
            "Editor",
            [120_f32, 10_f32, 50_f32, 50_f32],
            [0, 1, 0, 1],
            [true, true, false, false, false],
            [255, 0, 0, 0, 0],
            "Editor",
        );
        self.add_switch(
            ["Editor", "Editor"],
            vec![
                SwitchData {
                    texture: "Editor".to_string(),
                    color: [255, 255, 255, 255],
                },
                SwitchData {
                    texture: "Editor".to_string(),
                    color: [180, 180, 180, 255],
                },
                SwitchData {
                    texture: "Editor".to_string(),
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
            "Editor_Left_Sidebar",
            [0_f32, 0_f32, 300_f32, ctx.available_rect().height(), 0_f32],
            [0, 1, 1, 2],
            [true, false, false, true],
            [100, 100, 100, 255, 0, 0, 0, 255],
            0.5,
        );
        self.add_rect(
            "Editor_Right_Sidebar",
            [0_f32, 0_f32, 300_f32, ctx.available_rect().height(), 0_f32],
            [1, 1, 1, 2],
            [false, false, false, true],
            [100, 100, 100, 255, 0, 0, 0, 255],
            0.5,
        );
        self.add_image_texture(
            "Editor_Background",
            "Resources/assets/images/editor_background.png",
            [false, false],
            true,
            ctx,
        );
        self.add_image(
            "Editor_Background",
            [0_f32, 0_f32, 300_f32, 300_f32],
            [1, 2, 1, 2],
            [false, false, true, true, false],
            [255, 0, 0, 0, 0],
            "Editor_Background",
        );
        self.add_rect(
            "Editor_Background",
            [
                0_f32,
                0_f32,
                ctx.available_rect().width(),
                ctx.available_rect().height(),
                0_f32,
            ],
            [1, 2, 1, 2],
            [false, false, true, true],
            [58, 58, 58, 255, 255, 255, 255, 255],
            0.0,
        );
        self.add_text(
            ["Operation_Over_Text2", ""],
            [0_f32, -50_f32, 60_f32, 1000_f32, 0.0],
            [255, 255, 255, 255, 0, 0, 0],
            [true, true, false, false],
            false,
            [1, 6, 1, 3],
        );
        self.add_rect(
            "Operation_Runtime",
            [10_f32, 10_f32, 200_f32, 70_f32, 20_f32],
            [0, 1, 0, 1],
            [true, true, false, false],
            [100, 100, 100, 125, 240, 255, 255, 255],
            0.0,
        );
        self.add_image_texture(
            "Operation_Runtime",
            "Resources/assets/images/operation_runtime.png",
            [false, false],
            true,
            ctx,
        );
        self.add_image(
            "Operation_Runtime",
            [20_f32, 45_f32, 50_f32, 50_f32],
            [0, 1, 0, 1],
            [true, false, false, true, false],
            [120, 0, 0, 0, 0],
            "Operation_Runtime",
        );
        self.add_text(
            ["Operation_Runtime", ""],
            [80_f32, 45_f32, 30_f32, 300_f32, 0.0],
            [255, 255, 255, 120, 0, 0, 0],
            [true, false, false, true],
            false,
            [0, 1, 0, 1],
        );
    }

    pub fn fade(
        &mut self,
        fade_in_or_out: bool,
        ctx: &egui::Context,
        ui: &mut Ui,
        split_time_name: &str,
        resource_name: &str,
        fade_speed: u8,
    ) -> u8 {
        let cut_to_rect_id = self.track_resource(self.resource_rect.clone(), resource_name);
        self.resource_rect[cut_to_rect_id].size =
            [ctx.available_rect().width(), ctx.available_rect().height()];
        if self.timer.now_time - self.split_time(split_time_name)[0] >= self.vertrefresh {
            self.add_split_time(split_time_name, true);
            if fade_in_or_out {
                self.resource_rect[cut_to_rect_id].color[3] =
                    if self.resource_rect[cut_to_rect_id].color[3] > 255 - fade_speed {
                        255
                    } else {
                        self.resource_rect[cut_to_rect_id].color[3] + fade_speed
                    };
            } else {
                self.resource_rect[cut_to_rect_id].color[3] =
                    self.resource_rect[cut_to_rect_id].color[3].saturating_sub(fade_speed)
            };
        };
        self.rect(ui, resource_name, ctx);
        self.resource_rect[cut_to_rect_id].color[3]
    }

    pub fn find_pause_index(&mut self, time: f32) -> i32 {
        let mut index = -1;
        for i in 0..self.pause_list.len() {
            if time <= self.pause_list[i].start_pause_time {
                index = i as i32;
                self.pause_list[i].mentioned = true;
                break;
            };
        }
        index
    }

    pub fn count_pause_time(&self, index: usize) -> f32 {
        let mut time = 0_f32;
        for i in 0..self.pause_list.len() - index {
            time += self.pause_list[index + i].pause_total_time;
        }
        time
    }

    pub fn add_enemy(
        &mut self,
        enemy_hp_def_speed_invincible_time_position_activated_time_size_walk_interval_and_animation_interval: [f32; 11],
        enemy_image_count_minus_target_point_alpha_and_increase_alpha_speed: [u32; 4],
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
            enemy_name: format!("Enemy_{}", enemy_name_image_and_type[0]),
            enemy_hp: enemy_hp_def_speed_invincible_time_position_activated_time_size_walk_interval_and_animation_interval[0],
            enemy_def: enemy_hp_def_speed_invincible_time_position_activated_time_size_walk_interval_and_animation_interval[1],
            enemy_speed: enemy_hp_def_speed_invincible_time_position_activated_time_size_walk_interval_and_animation_interval[2],
            enemy_invincible_time:
                enemy_hp_def_speed_invincible_time_position_activated_time_size_walk_interval_and_animation_interval[3],
            enemy_image_count: enemy_image_count_minus_target_point_alpha_and_increase_alpha_speed[0],
            enemy_tag: enemy_tag_and_move_path[0].clone(),
            enemy_image: enemy_name_image_and_type[1].clone(),
            enemy_image_type: enemy_name_image_and_type[2].clone(),
            enemy_minus_target_point: enemy_image_count_minus_target_point_alpha_and_increase_alpha_speed[1],
            enemy_position: [
                enemy_hp_def_speed_invincible_time_position_activated_time_size_walk_interval_and_animation_interval[4],
                enemy_hp_def_speed_invincible_time_position_activated_time_size_walk_interval_and_animation_interval[5],
            ],
            enemy_move_path: move_path,
            enemy_detected: enemy_detected_and_activated[0],
            enemy_activated: enemy_detected_and_activated[1],
            enemy_activated_time:
                enemy_hp_def_speed_invincible_time_position_activated_time_size_walk_interval_and_animation_interval[6],
            enemy_size: [
                enemy_hp_def_speed_invincible_time_position_activated_time_size_walk_interval_and_animation_interval[7],
                enemy_hp_def_speed_invincible_time_position_activated_time_size_walk_interval_and_animation_interval[8],
            ],
            enemy_current_walk_status: 0,
            enemy_start_walk_time: 0_f32,
            enemy_increase_alpha_speed: enemy_image_count_minus_target_point_alpha_and_increase_alpha_speed[3] as u8,
            enemy_animation_forward: true,
            enemy_current_animation_count: 0,
            enemy_walk_interval: enemy_hp_def_speed_invincible_time_position_activated_time_size_walk_interval_and_animation_interval[9],
            enemy_walk_time: 0_f32,
            enemy_animation_interval: enemy_hp_def_speed_invincible_time_position_activated_time_size_walk_interval_and_animation_interval[10],
            enemy_animation_change_time: 0_f32,
            enemy_out: false,
            enemy_hit_time: 0_f32,
            enemy_initial_hp: enemy_hp_def_speed_invincible_time_position_activated_time_size_walk_interval_and_animation_interval[0],
            enemy_memory_hp: enemy_hp_def_speed_invincible_time_position_activated_time_size_walk_interval_and_animation_interval[0]
        });
        for i in 0..enemy_image_count_minus_target_point_alpha_and_increase_alpha_speed[0] {
            if !check_resource_exist(
                self.resource_image_texture.clone(),
                &format!("Enemy_{}_{}", enemy_name_image_and_type[0], i),
            ) {
                self.add_image_texture(
                    &format!("Enemy_{}_{}", enemy_name_image_and_type[0], i),
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
            &format!("Enemy_{}", enemy_name_image_and_type[0]),
            [
                enemy_hp_def_speed_invincible_time_position_activated_time_size_walk_interval_and_animation_interval[4],
                enemy_hp_def_speed_invincible_time_position_activated_time_size_walk_interval_and_animation_interval[5],
                enemy_hp_def_speed_invincible_time_position_activated_time_size_walk_interval_and_animation_interval[7],
                enemy_hp_def_speed_invincible_time_position_activated_time_size_walk_interval_and_animation_interval[8],
            ],
            [0, 0, 0, 0],
            [false, false, true, false, true],
            [enemy_image_count_minus_target_point_alpha_and_increase_alpha_speed[2] as u8, 0, 0, 0, 255],
            &format!("Enemy_{}_0", enemy_name_image_and_type[0]),
        );
    }

    pub fn operation_message_box_display(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        for i in 0..self.operation_preload_message_box.len() {
            if self.var_f("operation_runtime")
                >= self.operation_preload_message_box[i].box_appear_time
                && self.operation_preload_message_box[i].box_enable
            {
                self.operation_preload_message_box[i].box_enable = false;
                self.add_image_texture(
                    &format!("Json_MessageBox{}", i),
                    &self.operation_preload_message_box[i].box_image_path.clone(),
                    [false, false],
                    true,
                    ctx,
                );
                self.add_image(
                    &format!("Json_MessageBox{}", i),
                    [0_f32, 0_f32, 100_f32, 100_f32],
                    [1, 2, 1, 2],
                    [true, true, true, false, false],
                    [255, 0, 0, 0, 0],
                    &format!("Json_MessageBox{}", i),
                );
                self.add_text(
                    [
                        &format!("Json_MessageBox{}_Title", i),
                        &self.operation_preload_message_box[i].box_title
                            [self.login_user_config.language as usize]
                            .clone(),
                    ],
                    [0_f32, 0_f32, 20_f32, 325_f32, 0.0],
                    [
                        self.operation_preload_message_box[i].box_title_color[0],
                        self.operation_preload_message_box[i].box_title_color[1],
                        self.operation_preload_message_box[i].box_title_color[2],
                        self.operation_preload_message_box[i].box_title_color[3],
                        0,
                        0,
                        0,
                    ],
                    [true, true, false, false],
                    false,
                    [0, 0, 0, 0],
                );
                self.add_text(
                    [
                        &format!("Json_MessageBox{}_Content", i),
                        &self.operation_preload_message_box[i].box_content
                            [self.login_user_config.language as usize]
                            .clone(),
                    ],
                    [0_f32, 0_f32, 15_f32, 325_f32, 0.0],
                    [
                        self.operation_preload_message_box[i].box_content_color[0],
                        self.operation_preload_message_box[i].box_content_color[1],
                        self.operation_preload_message_box[i].box_content_color[2],
                        self.operation_preload_message_box[i].box_content_color[3],
                        0,
                        0,
                        0,
                    ],
                    [true, true, false, false],
                    false,
                    [0, 0, 0, 0],
                );
                self.add_message_box(
                    [
                        &format!("Json_MessageBox{}", i),
                        &format!("Json_MessageBox{}_Title", i),
                        &format!("Json_MessageBox{}_Content", i),
                        &format!("Json_MessageBox{}", i),
                    ],
                    self.operation_preload_message_box[i].box_size,
                    self.operation_preload_message_box[i].box_existing_time == 0_f32,
                    self.operation_preload_message_box[i].box_existing_time,
                    [30_f32, 10_f32],
                );
            };
        }
        self.message_box_display(ctx, ui);
    }

    pub fn enemy_refresh(&mut self, ctx: &egui::Context, ui: &Ui, refresh: bool) {
        if let Ok(json_value) = read_from_json(self.login_user_config.current_level.clone()) {
            if let Some(read_operation) = Operation::from_json_value(&json_value) {
                for i in 0..self.enemy_list.len() {
                    let id = self.track_resource(
                        self.resource_image.clone(),
                        &self.enemy_list[i].enemy_name.clone(),
                    );
                    if self.enemy_list[i].enemy_activated {
                        self.resource_image[id].origin_position = [
                            (ctx.available_rect().width() - 1280_f32) / 2_f32
                                + self.enemy_list[i].enemy_position[0],
                            (ctx.available_rect().height() - 720_f32) / 2_f32
                                + self.enemy_list[i].enemy_position[1],
                        ];
                        if refresh {
                            if self.resource_image[id].alpha == 255 {
                                let enemy_rect = egui::Rect::from_min_size(
                                    Pos2 {
                                        x: self.resource_image[id].image_position[0],
                                        y: self.resource_image[id].image_position[1],
                                    },
                                    Vec2 {
                                        x: self.resource_image[id].image_size[0],
                                        y: self.resource_image[id].image_size[1],
                                    },
                                );
                                for u in 0..read_operation.global.target_line.len() - 1 {
                                    if self.line_intersects_rect(
                                        &enemy_rect,
                                        Pos2 {
                                            x: read_operation.global.target_line[u][0]
                                                + (ctx.available_rect().width() - 1280_f32) / 2_f32,
                                            y: read_operation.global.target_line[u][1]
                                                + (ctx.available_rect().height() - 720_f32) / 2_f32,
                                        },
                                        Pos2 {
                                            x: read_operation.global.target_line[u + 1][0]
                                                + (ctx.available_rect().width() - 1280_f32) / 2_f32,
                                            y: read_operation.global.target_line[u + 1][1]
                                                + (ctx.available_rect().height() - 720_f32) / 2_f32,
                                        },
                                    ) {
                                        self.modify_var("perfect_clear", false);
                                        self.enemy_list[i].enemy_out = true;
                                        self.enemy_list[i].enemy_activated = false;
                                        self.resource_image[id].overlay_color =
                                            [255, 255, 255, 255];
                                        let target_point = self.var_u("target_point");
                                        if target_point
                                            > self.enemy_list[i].enemy_minus_target_point
                                        {
                                            self.modify_var(
                                                "target_point",
                                                Value::UInt(
                                                    target_point
                                                        - self.enemy_list[i]
                                                            .enemy_minus_target_point,
                                                ),
                                            );
                                        } else {
                                            self.modify_var("target_point", Value::UInt(0));
                                        };
                                        if self.enemy_list[i].enemy_detected {
                                            let current_killed_target_enemy =
                                                self.var_u("current_killed_target_enemy");
                                            self.modify_var(
                                                "current_killed_target_enemy",
                                                Value::UInt(current_killed_target_enemy + 1),
                                            );
                                        };
                                        std::thread::spawn(|| {
                                            kira_play_wav("Resources/assets/sounds/Alert.wav")
                                                .unwrap();
                                        });
                                        return;
                                    };
                                }
                                let id2_id = self.var_u("gun_selected") as usize;
                                let id2 = self.track_resource(
                                    self.resource_image.clone(),
                                    &format!(
                                        "Gun_{}",
                                        self.storage_gun_content[id2_id]
                                            .gun_recognition_name
                                            .clone()
                                    ),
                                );
                                let gun_id = self.track_resource(
                                    self.resource_switch.clone(),
                                    &format!(
                                        "Gun_{}",
                                        self.storage_gun_content[id2_id]
                                            .gun_recognition_name
                                            .clone()
                                    ),
                                );
                                let gun_rect = egui::Rect::from_center_size(
                                    egui::Pos2 {
                                        x: self.resource_image[id2].origin_position[0],
                                        y: self.resource_image[id2].origin_position[1],
                                    },
                                    Vec2 {
                                        x: self.resource_image[id2].image_size[0],
                                        y: self.resource_image[id2].image_size[1],
                                    },
                                );
                                if self.var_f("operation_runtime")
                                    - self.enemy_list[i].enemy_hit_time
                                    < self.enemy_list[i].enemy_invincible_time
                                {
                                    self.resource_image[id].overlay_color = [125, 125, 125, 255];
                                } else {
                                    self.resource_image[id].overlay_color = [255, 255, 255, 255];
                                    if self.rect_intersects_rect(&gun_rect, &enemy_rect)
                                        && self.resource_switch[gun_id].state == 1
                                    {
                                        self.enemy_list[i].enemy_hit_time =
                                            self.var_f("operation_runtime");
                                        if self.storage_gun_content[id2_id].gun_basic_damage
                                            > self.enemy_list[i].enemy_def
                                        {
                                            self.enemy_list[i].enemy_hp -=
                                                self.storage_gun_content[id2_id].gun_basic_damage
                                                    - self.enemy_list[i].enemy_def;
                                            thread::spawn(|| {
                                                kira_play_wav("Resources/assets/sounds/Hit.wav")
                                                    .unwrap();
                                            });
                                        } else {
                                            thread::spawn(|| {
                                                kira_play_wav(
                                                    "Resources/assets/sounds/Hit_No_Damage.wav",
                                                )
                                                .unwrap();
                                            });
                                        };
                                    };
                                };
                                if self.enemy_list[i].enemy_hp <= 0_f32 {
                                    self.enemy_list[i].enemy_out = true;
                                    self.enemy_list[i].enemy_activated = false;
                                    self.resource_image[id].overlay_color = [255, 255, 255, 255];
                                    if self.enemy_list[i].enemy_detected {
                                        let current_killed_target_enemy =
                                            self.var_u("current_killed_target_enemy");
                                        self.modify_var(
                                            "current_killed_target_enemy",
                                            Value::UInt(current_killed_target_enemy + 1),
                                        );
                                    };
                                    std::thread::spawn(|| {
                                        kira_play_wav("Resources/assets/sounds/Enemy_Death.wav")
                                            .unwrap();
                                    });
                                    return;
                                };
                                if self.var_f("operation_runtime")
                                    - self.enemy_list[i].enemy_walk_time
                                    >= self.enemy_list[i].enemy_walk_interval
                                {
                                    self.enemy_list[i].enemy_walk_time =
                                        self.var_f("operation_runtime");
                                    if self.var_f("operation_runtime")
                                        - self.enemy_list[i].enemy_start_walk_time
                                        >= self.enemy_list[i].enemy_move_path
                                            [self.enemy_list[i].enemy_current_walk_status as usize]
                                            .move_time
                                    {
                                        if self.enemy_list[i].enemy_current_walk_status
                                            < (self.enemy_list[i].enemy_move_path.len() - 1) as u32
                                        {
                                            self.enemy_list[i].enemy_current_walk_status += 1;
                                        } else {
                                            self.enemy_list[i].enemy_current_walk_status = 0;
                                        };
                                        self.enemy_list[i].enemy_start_walk_time =
                                            self.var_f("operation_runtime");
                                    };
                                    if self.enemy_list[i].enemy_move_path
                                        [self.enemy_list[i].enemy_current_walk_status as usize]
                                        .move_status[0]
                                    {
                                        self.enemy_list[i].enemy_position[1] -=
                                            self.enemy_list[i].enemy_speed;
                                    };
                                    if self.enemy_list[i].enemy_move_path
                                        [self.enemy_list[i].enemy_current_walk_status as usize]
                                        .move_status[1]
                                    {
                                        self.enemy_list[i].enemy_position[1] +=
                                            self.enemy_list[i].enemy_speed;
                                    };
                                    if self.enemy_list[i].enemy_move_path
                                        [self.enemy_list[i].enemy_current_walk_status as usize]
                                        .move_status[2]
                                    {
                                        self.enemy_list[i].enemy_position[0] -=
                                            self.enemy_list[i].enemy_speed;
                                    };
                                    if self.enemy_list[i].enemy_move_path
                                        [self.enemy_list[i].enemy_current_walk_status as usize]
                                        .move_status[3]
                                    {
                                        self.enemy_list[i].enemy_position[0] +=
                                            self.enemy_list[i].enemy_speed;
                                    };
                                    if self.enemy_list[i].enemy_move_path
                                        [self.enemy_list[i].enemy_current_walk_status as usize]
                                        .move_status
                                        .iter()
                                        .any(|&x| x)
                                        && self.var_f("operation_runtime")
                                            - self.enemy_list[i].enemy_animation_change_time
                                            >= self.enemy_list[i].enemy_animation_interval
                                    {
                                        self.enemy_list[i].enemy_animation_change_time =
                                            self.var_f("operation_runtime");
                                        if self.enemy_list[i].enemy_animation_forward {
                                            if self.enemy_list[i].enemy_current_animation_count
                                                < self.enemy_list[i].enemy_image_count
                                            {
                                                self.enemy_list[i].enemy_current_animation_count +=
                                                    1;
                                            } else {
                                                self.enemy_list[i].enemy_animation_forward = false;
                                                if self.enemy_list[i].enemy_current_animation_count
                                                    > 0
                                                {
                                                    self.enemy_list[i]
                                                        .enemy_current_animation_count -= 1;
                                                };
                                            };
                                        } else if self.enemy_list[i].enemy_current_animation_count
                                            > 0
                                        {
                                            self.enemy_list[i].enemy_current_animation_count -= 1;
                                        } else {
                                            self.enemy_list[i].enemy_animation_forward = true;
                                            if self.enemy_list[i].enemy_current_animation_count
                                                < self.enemy_list[i].enemy_image_count
                                            {
                                                self.enemy_list[i].enemy_current_animation_count +=
                                                    1;
                                            };
                                        };
                                    };
                                };
                            } else {
                                if self.enemy_list[i].enemy_increase_alpha_speed
                                    > 255 - self.resource_image[id].alpha
                                {
                                    self.resource_image[id].alpha = 255;
                                    self.resource_image[id].overlay_color = [255, 255, 255, 255];
                                } else {
                                    self.resource_image[id].alpha +=
                                        self.enemy_list[i].enemy_increase_alpha_speed;
                                    self.resource_image[id].overlay_color[0] +=
                                        self.enemy_list[i].enemy_increase_alpha_speed;
                                    self.resource_image[id].overlay_color[1] +=
                                        self.enemy_list[i].enemy_increase_alpha_speed;
                                    self.resource_image[id].overlay_color[2] +=
                                        self.enemy_list[i].enemy_increase_alpha_speed;
                                };
                                if self.resource_image[id].alpha == 255 {
                                    self.enemy_list[i].enemy_start_walk_time =
                                        self.var_f("operation_runtime");
                                };
                            };
                        };
                        if let Some(index) = self.resource_image_texture.iter().position(|x| {
                            x.name
                                == format!(
                                    "{}_{}",
                                    self.enemy_list[i].enemy_name,
                                    self.enemy_list[i].enemy_current_animation_count
                                )
                        }) {
                            self.resource_image[id].image_texture =
                                self.resource_image_texture[index].texture.clone();
                        };
                    } else if self.var_f("operation_runtime")
                        >= self.enemy_list[i].enemy_activated_time
                        && refresh
                    {
                        if self.enemy_list[i].enemy_out {
                            if self.resource_image[id].alpha
                                < self.enemy_list[i].enemy_increase_alpha_speed
                            {
                                self.resource_image[id].alpha = 0;
                                self.resource_image[id].overlay_color = [0, 0, 0, 255];
                            } else {
                                self.resource_image[id].alpha -=
                                    self.enemy_list[i].enemy_increase_alpha_speed;
                                self.resource_image[id].overlay_color[0] -=
                                    self.enemy_list[i].enemy_increase_alpha_speed;
                                self.resource_image[id].overlay_color[1] -=
                                    self.enemy_list[i].enemy_increase_alpha_speed;
                                self.resource_image[id].overlay_color[2] -=
                                    self.enemy_list[i].enemy_increase_alpha_speed;
                            };
                        } else {
                            self.enemy_list[i].enemy_activated = true;
                        };
                    };
                    if self.resource_image[id].alpha != 0 {
                        self.image(ui, &self.enemy_list[i].enemy_name.clone(), ctx);
                    };
                    if self.enemy_list[i].enemy_activated {
                        ui.painter().line(
                            vec![
                                Pos2 {
                                    x: self.resource_image[id].image_position[0],
                                    y: self.resource_image[id].image_position[1] - 15_f32,
                                },
                                Pos2 {
                                    x: self.resource_image[id].image_position[0]
                                        + self.resource_image[id].image_size[0],
                                    y: self.resource_image[id].image_position[1] - 15_f32,
                                },
                            ],
                            Stroke {
                                width: 10.0,
                                color: Color32::from_rgba_unmultiplied(
                                    0,
                                    0,
                                    0,
                                    self.resource_image[id].alpha,
                                ),
                            },
                        );
                        let enemy_hp_multiple = if self.enemy_list[i].enemy_hp
                            / self.enemy_list[i].enemy_initial_hp
                            < 0_f32
                        {
                            0_f32
                        } else {
                            self.enemy_list[i].enemy_hp / self.enemy_list[i].enemy_initial_hp
                        };
                        let enemy_hp_bar_rgb = if enemy_hp_multiple > 0.6 {
                            [94, 203, 118]
                        } else if 0.2 < enemy_hp_multiple && enemy_hp_multiple <= 0.6 {
                            [255, 240, 59]
                        } else {
                            [255, 52, 40]
                        };
                        if self.enemy_list[i].enemy_memory_hp != self.enemy_list[i].enemy_hp {
                            if refresh {
                                if self.enemy_list[i].enemy_memory_hp - self.enemy_list[i].enemy_hp
                                    <= 0.5
                                {
                                    self.enemy_list[i].enemy_memory_hp =
                                        self.enemy_list[i].enemy_hp;
                                } else {
                                    self.enemy_list[i].enemy_memory_hp -= 0.5;
                                };
                            };
                            let enemy_memory_hp_multiple = if self.enemy_list[i].enemy_memory_hp
                                / self.enemy_list[i].enemy_initial_hp
                                < 0_f32
                            {
                                0_f32
                            } else {
                                self.enemy_list[i].enemy_memory_hp
                                    / self.enemy_list[i].enemy_initial_hp
                            };
                            ui.painter().line(
                                vec![
                                    Pos2 {
                                        x: self.resource_image[id].image_position[0] + 3_f32,
                                        y: self.resource_image[id].image_position[1] - 15_f32,
                                    },
                                    Pos2 {
                                        x: self.resource_image[id].image_position[0]
                                            + 3_f32
                                            + (self.resource_image[id].image_size[0] - 6_f32)
                                                * enemy_memory_hp_multiple,
                                        y: self.resource_image[id].image_position[1] - 15_f32,
                                    },
                                ],
                                Stroke {
                                    width: 5.0,
                                    color: Color32::from_rgba_unmultiplied(
                                        91,
                                        0,
                                        0,
                                        self.resource_image[id].alpha,
                                    ),
                                },
                            );
                        };
                        ui.painter().line(
                            vec![
                                Pos2 {
                                    x: self.resource_image[id].image_position[0] + 3_f32,
                                    y: self.resource_image[id].image_position[1] - 15_f32,
                                },
                                Pos2 {
                                    x: self.resource_image[id].image_position[0]
                                        + 3_f32
                                        + (self.resource_image[id].image_size[0] - 6_f32)
                                            * enemy_hp_multiple,
                                    y: self.resource_image[id].image_position[1] - 15_f32,
                                },
                            ],
                            Stroke {
                                width: 5.0,
                                color: Color32::from_rgba_unmultiplied(
                                    enemy_hp_bar_rgb[0],
                                    enemy_hp_bar_rgb[1],
                                    enemy_hp_bar_rgb[2],
                                    self.resource_image[id].alpha,
                                ),
                            },
                        );
                    };
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
            resource_list
                .iter()
                .position(|x| x.name() == resource_name)
                .unwrap()
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
                self.switch_page("Home_Page");
                self.add_split_time("dock_animation", true);
                self.add_split_time("title_animation", true);
            };
            if self.switch("Home_Settings", ui, ctx, true, true)[0] == 0 {
                self.switch_page("Home_Setting");
                self.add_split_time("dock_animation", true);
            };
            let id2 = self.track_resource(self.resource_switch.clone(), "Home_Power");
            if self.switch("Home_Power", ui, ctx, true, true)[0] == 0 {
                write_to_json(
                    format!("Resources/config/user_{}.json", self.config.login_user_name),
                    self.login_user_config.to_json_value(),
                )
                .unwrap();
                if self.resource_switch[id2].state == 1 {
                    self.config.login_user_name = "".to_string();
                };
                write_to_json(
                    "Resources/config/Preferences.json",
                    self.config.to_json_value(),
                )
                .unwrap();
                exit(0);
            };
            if self.switch("Home_Journey", ui, ctx, true, true)[0] == 0 {
                self.switch_page("Home_Select_Map");
                self.add_split_time("dock_animation", true);
                if check_resource_exist(self.timer.split_time.clone(), "map_select_animation") {
                    self.add_split_time("map_select_animation", true);
                };
            };
        };
        self.resource_rect[id].size[0] = ctx.available_rect().width() - 100_f32;
    }

    pub fn rect_intersects_rect(&self, rect1: &Rect, rect2: &Rect) -> bool {
        // 检查X轴重叠
        let x_overlap = rect1.max.x > rect2.min.x && rect1.min.x < rect2.max.x;

        // 检查Y轴重叠
        let y_overlap = rect1.max.y > rect2.min.y && rect1.min.y < rect2.max.y;

        // 两个轴都有重叠时矩形相交
        x_overlap && y_overlap
    }

    pub fn line_intersects_rect(&self, rect: &Rect, start: Pos2, end: Pos2) -> bool {
        // 检查线段端点是否在矩形内
        if rect.contains(start) || rect.contains(end) {
            return true;
        }

        // 检查线段是否与矩形的四条边相交
        let top_left = rect.min;
        let top_right = Pos2::new(rect.max.x, rect.min.y);
        let bottom_left = Pos2::new(rect.min.x, rect.max.y);
        let bottom_right = rect.max;

        // 检查与上边相交
        if self.line_segments_intersect(start, end, top_left, top_right) {
            return true;
        }

        // 检查与右边相交
        if self.line_segments_intersect(start, end, top_right, bottom_right) {
            return true;
        }

        // 检查与下边相交
        if self.line_segments_intersect(start, end, bottom_right, bottom_left) {
            return true;
        }

        // 检查与左边相交
        if self.line_segments_intersect(start, end, bottom_left, top_left) {
            return true;
        }

        false
    }

    pub fn line_segments_intersect(&self, a1: Pos2, a2: Pos2, b1: Pos2, b2: Pos2) -> bool {
        let v1 = (b1.x - a1.x) * (a2.y - a1.y) - (b1.y - a1.y) * (a2.x - a1.x);
        let v2 = (b2.x - a1.x) * (a2.y - a1.y) - (b2.y - a1.y) * (a2.x - a1.x);
        let v3 = (a1.x - b1.x) * (b2.y - b1.y) - (a1.y - b1.y) * (b2.x - b1.x);
        let v4 = (a2.x - b1.x) * (b2.y - b1.y) - (a2.y - b1.y) * (b2.x - b1.x);

        (v1 * v2 < 0.0) && (v3 * v4 < 0.0)
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
            Color32::from_rgba_unmultiplied(
                self.resource_rect[id].color[0],
                self.resource_rect[id].color[1],
                self.resource_rect[id].color[2],
                self.resource_rect[id].color[3],
            ),
            Stroke {
                width: self.resource_rect[id].border_width,
                color: Color32::from_rgba_unmultiplied(
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
        let image_texture = Some(ctx.load_texture(name, color_image, TextureOptions::LINEAR));
        if !create_new_resource && check_resource_exist(self.resource_image_texture.clone(), name) {
            let id = self.track_resource(self.resource_image_texture.clone(), name);
            self.resource_image_texture[id].texture = image_texture;
            self.resource_image_texture[id].cite_path = path.to_string();
        } else {
            self.resource_image_texture.push(ImageTexture {
                discern_type: "ImageTexture".to_string(),
                name: name.to_string(),
                texture: image_texture,
                cite_path: path.to_string(),
            });
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
                Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                color,
            );
        };
    }

    pub fn add_message_box(
        &mut self,
        box_itself_title_content_image_name: [&str; 4],
        box_size: [f32; 2],
        box_keep_existing: bool,
        box_existing_time: f32,
        box_normal_and_restore_speed: [f32; 2],
    ) {
        if !check_resource_exist(
            self.resource_message_box.clone(),
            box_itself_title_content_image_name[0],
        ) {
            let id = self
                .resource_image
                .iter()
                .position(|x| x.name == box_itself_title_content_image_name[3])
                .unwrap();
            self.resource_image[id].image_size = [box_size[1] - 15_f32, box_size[1] - 15_f32];
            self.resource_image[id].center_display = [true, false, false, true];
            self.resource_image[id].x_grid = [1, 1];
            self.resource_image[id].y_grid = [0, 1];
            let id2 = self
                .resource_text
                .iter()
                .position(|x| x.name == box_itself_title_content_image_name[1])
                .unwrap();
            let id3 = self
                .resource_text
                .iter()
                .position(|x| x.name == box_itself_title_content_image_name[2])
                .unwrap();
            self.resource_text[id2].center_display = [true, true, false, false];
            self.resource_text[id3].center_display = [true, true, false, false];
            self.resource_text[id2].x_grid = [1, 1];
            self.resource_text[id2].y_grid = [0, 1];
            self.resource_text[id3].x_grid = [1, 1];
            self.resource_text[id3].y_grid = [0, 1];
            self.resource_text[id2].wrap_width = box_size[0] - box_size[1] + 5_f32;
            self.resource_text[id3].wrap_width = box_size[0] - box_size[1] + 5_f32;
            self.resource_image[id].name = format!("MessageBox_{}", self.resource_image[id].name);
            self.resource_text[id2].name = format!("MessageBox_{}", self.resource_text[id2].name);
            self.resource_text[id3].name = format!("MessageBox_{}", self.resource_text[id3].name);
            self.resource_message_box.push(MessageBox {
                discern_type: "MessageBox".to_string(),
                name: box_itself_title_content_image_name[0].to_string(),
                box_size,
                box_title_name: format!("MessageBox_{}", box_itself_title_content_image_name[1]),
                box_content_name: format!("MessageBox_{}", box_itself_title_content_image_name[2]),
                box_image_name: format!("MessageBox_{}", box_itself_title_content_image_name[3]),
                box_keep_existing,
                box_existing_time,
                box_exist: true,
                box_speed: box_normal_and_restore_speed[0],
                box_restore_speed: box_normal_and_restore_speed[1],
                box_memory_offset: 0_f32,
            });
            if !box_keep_existing {
                self.add_split_time(
                    &format!("MessageBox_{}", box_itself_title_content_image_name[0]),
                    false,
                );
            };
            self.add_split_time(
                &format!(
                    "MessageBox_{}_animation",
                    box_itself_title_content_image_name[0]
                ),
                false,
            );
            self.add_rect(
                &format!("MessageBox_{}", box_itself_title_content_image_name[0]),
                [0_f32, 0_f32, box_size[0], box_size[1], 20_f32],
                [1, 1, 0, 1],
                [true, true, false, false],
                [100, 100, 100, 125, 240, 255, 255, 255],
                0.0,
            );
            self.add_image(
                &format!(
                    "MessageBox_{}_Close",
                    box_itself_title_content_image_name[0]
                ),
                [0_f32, 0_f32, 30_f32, 30_f32],
                [0, 0, 0, 0],
                [false, false, true, true, false],
                [255, 0, 0, 0, 0],
                "Close_Message_Box",
            );
            self.add_switch(
                [
                    &format!(
                        "MessageBox_{}_Close",
                        box_itself_title_content_image_name[0]
                    ),
                    &format!(
                        "MessageBox_{}_Close",
                        box_itself_title_content_image_name[0]
                    ),
                ],
                vec![
                    SwitchData {
                        texture: "Close_Message_Box".to_string(),
                        color: [255, 255, 255, 0],
                    },
                    SwitchData {
                        texture: "Close_Message_Box".to_string(),
                        color: [180, 180, 180, 200],
                    },
                    SwitchData {
                        texture: "Close_Message_Box".to_string(),
                        color: [255, 255, 255, 200],
                    },
                    SwitchData {
                        texture: "Close_Message_Box".to_string(),
                        color: [180, 180, 180, 200],
                    },
                ],
                [false, true, true],
                2,
                vec![SwitchClickAction {
                    click_method: PointerButton::Primary,
                    action: true,
                }],
            );
        } else if self.config.rc_strict_mode {
            panic!(
                "{}{}",
                box_itself_title_content_image_name[0],
                self.game_text.game_text["error_message_box_already_exists"]
                    [self.config.language as usize]
            );
        } else {
            self.problem_report(
                &format!(
                    "{}{}",
                    box_itself_title_content_image_name[0],
                    self.game_text.game_text["error_message_box_already_exists"]
                        [self.config.language as usize]
                ),
                SeverityLevel::SevereWarning,
                &self.game_text.game_text["error_message_box_already_exists_annotation"]
                    [self.config.language as usize]
                    .clone(),
            );
        };
    }

    pub fn message_box_display(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        let mut offset = 0_f32;
        let mut delete_count = 0;
        for u in 0..self.resource_message_box.len() {
            let mut deleted = false;
            let i = u - delete_count;
            let id = self
                .resource_image
                .iter()
                .position(|x| x.name == self.resource_message_box[i].box_image_name)
                .unwrap();
            let id2 = self
                .resource_rect
                .iter()
                .position(|x| x.name == format!("MessageBox_{}", self.resource_message_box[i].name))
                .unwrap();
            let id3 = self
                .resource_text
                .iter()
                .position(|x| x.name == self.resource_message_box[i].box_title_name)
                .unwrap();
            let id4 = self
                .resource_text
                .iter()
                .position(|x| x.name == self.resource_message_box[i].box_content_name)
                .unwrap();
            let id5 = self
                .resource_switch
                .iter()
                .position(|x| {
                    x.name == format!("MessageBox_{}_Close", self.resource_message_box[i].name)
                })
                .unwrap();
            let id6 = self
                .resource_image
                .iter()
                .position(|x| {
                    x.name == format!("MessageBox_{}_Close", self.resource_message_box[i].name)
                })
                .unwrap();
            if self.resource_message_box[i].box_size[1]
                < self.get_text_size(&self.resource_message_box[i].box_title_name.clone(), ui)[1]
                    + self.get_text_size(&self.resource_message_box[i].box_content_name.clone(), ui)
                        [1]
                    + 10_f32
            {
                self.resource_message_box[i].box_size[1] = self
                    .get_text_size(&self.resource_message_box[i].box_title_name.clone(), ui)[1]
                    + self
                        .get_text_size(&self.resource_message_box[i].box_content_name.clone(), ui)
                        [1]
                    + 10_f32;
                self.resource_rect[id2].size[1] = self.resource_message_box[i].box_size[1];
                self.resource_image[id].image_size = [
                    self.resource_message_box[i].box_size[1] - 15_f32,
                    self.resource_message_box[i].box_size[1] - 15_f32,
                ];
                self.resource_text[id3].wrap_width = self.resource_message_box[i].box_size[0]
                    - self.resource_message_box[i].box_size[1]
                    + 5_f32;
                self.resource_text[id4].wrap_width = self.resource_message_box[i].box_size[0]
                    - self.resource_message_box[i].box_size[1]
                    + 5_f32;
            };
            if self.timer.total_time
                - self.split_time(&format!(
                    "MessageBox_{}_animation",
                    self.resource_message_box[i].name
                ))[1]
                >= self.vertrefresh
            {
                self.add_split_time(
                    &format!("MessageBox_{}_animation", self.resource_message_box[i].name),
                    true,
                );
                if offset != self.resource_message_box[i].box_memory_offset {
                    if self.resource_message_box[i].box_memory_offset < offset {
                        if self.resource_message_box[i].box_memory_offset
                            + self.resource_message_box[i].box_restore_speed
                            >= offset
                        {
                            self.resource_message_box[i].box_memory_offset = offset;
                        } else {
                            self.resource_message_box[i].box_memory_offset +=
                                self.resource_message_box[i].box_restore_speed;
                        };
                    } else if self.resource_message_box[i].box_memory_offset
                        - self.resource_message_box[i].box_restore_speed
                        <= offset
                    {
                        self.resource_message_box[i].box_memory_offset = offset;
                    } else {
                        self.resource_message_box[i].box_memory_offset -=
                            self.resource_message_box[i].box_restore_speed;
                    };
                };
                if self.resource_rect[id2].origin_position[0]
                    != -self.resource_message_box[i].box_size[0] - 5_f32
                {
                    if self.resource_message_box[i].box_exist {
                        if self.resource_rect[id2].origin_position[0]
                            - self.resource_message_box[i].box_speed
                            <= -self.resource_message_box[i].box_size[0] - 5_f32
                        {
                            self.resource_rect[id2].origin_position[0] =
                                -self.resource_message_box[i].box_size[0] - 5_f32;
                            self.add_split_time(
                                &format!("MessageBox_{}", self.resource_message_box[i].name),
                                true,
                            );
                        } else {
                            self.resource_rect[id2].origin_position[0] -=
                                self.resource_message_box[i].box_speed;
                        };
                    } else if self.resource_rect[id2].origin_position[0]
                        + self.resource_message_box[i].box_speed
                        >= 15_f32
                    {
                        self.resource_rect[id2].origin_position[0] = 15_f32;
                        delete_count += 1;
                        deleted = true;
                    } else {
                        self.resource_rect[id2].origin_position[0] +=
                            self.resource_message_box[i].box_speed;
                    };
                };
            };
            self.resource_rect[id2].origin_position[1] =
                self.resource_message_box[i].box_memory_offset + 20_f32;
            self.resource_image[id].origin_position = [
                self.resource_rect[id2].origin_position[0] + 5_f32,
                self.resource_rect[id2].origin_position[1]
                    + self.resource_message_box[i].box_size[1] / 2_f32,
            ];
            self.resource_text[id3].origin_position = [
                self.resource_image[id].origin_position[0]
                    + self.resource_image[id].image_size[0]
                    + 5_f32,
                self.resource_rect[id2].origin_position[1] + 5_f32,
            ];
            self.resource_text[id4].origin_position = [
                self.resource_image[id].origin_position[0]
                    + self.resource_image[id].image_size[0]
                    + 5_f32,
                self.resource_text[id3].origin_position[1]
                    + self.get_text_size(&self.resource_message_box[i].box_title_name.clone(), ui)
                        [1],
            ];
            self.resource_image[id6].origin_position = self.resource_rect[id2].position;
            if !self.resource_message_box[i].box_keep_existing
                && self.timer.total_time
                    - self.split_time(&format!("MessageBox_{}", self.resource_message_box[i].name))
                        [1]
                    >= self.resource_message_box[i].box_existing_time
                && self.resource_rect[id2].origin_position[0]
                    == -self.resource_message_box[i].box_size[0] - 5_f32
            {
                self.resource_message_box[i].box_exist = false;
                if self.resource_rect[id2].origin_position[0]
                    + self.resource_message_box[i].box_speed
                    >= 15_f32
                {
                    self.resource_rect[id2].origin_position[0] = 15_f32;
                } else {
                    self.resource_rect[id2].origin_position[0] +=
                        self.resource_message_box[i].box_speed;
                };
            };
            self.rect(
                ui,
                &format!("MessageBox_{}", self.resource_message_box[i].name),
                ctx,
            );
            self.image(
                ui,
                &self.resource_message_box[i].box_image_name.clone(),
                ctx,
            );
            self.text(ui, &self.resource_text[id3].name.clone(), ctx);
            self.text(ui, &self.resource_text[id4].name.clone(), ctx);
            if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                let rect = egui::Rect::from_min_size(
                    Pos2 {
                        x: self.resource_image[id6].image_position[0],
                        y: self.resource_image[id6].image_position[1],
                    },
                    Vec2 {
                        x: self.resource_rect[id2].size[0] + 25_f32,
                        y: self.resource_rect[id2].size[1] + 25_f32,
                    },
                );
                if rect.contains(mouse_pos) {
                    self.resource_switch[id5].appearance[0].color[3] = 200;
                } else {
                    self.resource_switch[id5].appearance[0].color[3] = 0;
                };
            };
            if self.switch(
                &format!("MessageBox_{}_Close", self.resource_message_box[i].name),
                ui,
                ctx,
                self.resource_switch[id5].state == 0 && self.resource_message_box[i].box_exist,
                true,
            )[0] == 0
            {
                self.resource_message_box[i].box_exist = false;
                if self.resource_rect[id2].origin_position[0]
                    + self.resource_message_box[i].box_speed
                    >= 15_f32
                {
                    self.resource_rect[id2].origin_position[0] = 15_f32;
                } else {
                    self.resource_rect[id2].origin_position[0] +=
                        self.resource_message_box[i].box_speed;
                };
            };
            if deleted {
                self.resource_switch.remove(
                    self.resource_switch
                        .iter()
                        .position(|x| {
                            x.name
                                == format!("MessageBox_{}_Close", self.resource_message_box[i].name)
                        })
                        .unwrap(),
                );
                self.resource_image.remove(
                    self.resource_image
                        .iter()
                        .position(|x| x.name == self.resource_message_box[i].box_image_name)
                        .unwrap(),
                );
                self.resource_image.remove(
                    self.resource_image
                        .iter()
                        .position(|x| {
                            x.name
                                == format!("MessageBox_{}_Close", self.resource_message_box[i].name)
                        })
                        .unwrap(),
                );
                self.resource_text.remove(
                    self.resource_text
                        .iter()
                        .position(|x| x.name == self.resource_message_box[i].box_title_name)
                        .unwrap(),
                );
                self.resource_text.remove(
                    self.resource_text
                        .iter()
                        .position(|x| x.name == self.resource_message_box[i].box_content_name)
                        .unwrap(),
                );
                self.resource_rect.remove(
                    self.resource_rect
                        .iter()
                        .position(|x| {
                            x.name == format!("MessageBox_{}", self.resource_message_box[i].name)
                        })
                        .unwrap(),
                );
                self.timer.split_time.remove(
                    self.timer
                        .split_time
                        .iter()
                        .position(|x| {
                            x.name
                                == format!(
                                    "MessageBox_{}_animation",
                                    self.resource_message_box[i].name
                                )
                        })
                        .unwrap(),
                );
                self.timer.split_time.remove(
                    self.timer
                        .split_time
                        .iter()
                        .position(|x| {
                            x.name == format!("MessageBox_{}", self.resource_message_box[i].name)
                        })
                        .unwrap(),
                );
                self.resource_message_box.remove(i);
            } else {
                offset += self.resource_message_box[i].box_size[1] + 15_f32;
            };
        }
    }

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
