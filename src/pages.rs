//! pages.rs is the core part of the page of the Targeted Vector, mainly the page content.
use crate::function::{
    check_file_exists, check_resource_exist, count_files_recursive, create_pretty_json,
    general_click_feedback, kira_play_wav, list_files_recursive, read_from_json, write_to_json,
    App, Map, SeverityLevel, SwitchClickAction, SwitchData, User, Value,
};
use chrono::{Local, Timelike};
use eframe::egui;
use eframe::epaint::Rounding;
use egui::{Frame, Shadow, Stroke};
use json::object;
use rfd::FileDialog;
use std::{fs, path::Path, process::exit, vec::Vec};

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_frame_stats(ctx);
        self.render_resource_list = Vec::new();
        if Local::now().hour() >= 18 {
            ctx.set_visuals(egui::Visuals::dark());
            self.frame = Frame {
                inner_margin: egui::Margin::same(10.0),
                outer_margin: egui::Margin::same(0.0),
                rounding: Rounding::same(10.0),
                shadow: Shadow {
                    offset: egui::Vec2::new(1.0, 2.0),
                    color: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 125),
                    blur: 20.0,
                    spread: 5.0,
                },
                fill: egui::Color32::from_rgb(39, 39, 39),
                stroke: Stroke {
                    width: 2.0,
                    color: egui::Color32::from_rgb(13, 14, 115),
                },
            };
        } else {
            ctx.set_visuals(egui::Visuals::light());
            self.frame = Frame {
                inner_margin: egui::Margin::same(10.0),
                outer_margin: egui::Margin::same(0.0),
                rounding: Rounding::same(10.0),
                shadow: Shadow {
                    offset: egui::Vec2::new(1.0, 2.0),
                    color: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 125),
                    blur: 20.0,
                    spread: 5.0,
                },
                fill: egui::Color32::from_rgb(255, 255, 255),
                stroke: Stroke {
                    width: 2.0,
                    color: egui::Color32::from_rgb(200, 200, 200),
                },
            };
        };
        let game_text = self.game_text.game_text.clone();
        self.update_timer();
        match &*self.page.clone() {
            "Launch" => {
                if !self.check_updated(&self.page.clone()) {
                    self.launch_page_preload(ctx);
                    self.add_var("progress", 0);
                    self.add_var("enable_debug_mode", false);
                    self.add_var("debug_fps_window", false);
                    self.add_var("debug_resource_list_window", false);
                    self.add_var("debug_problem_window", false);
                    self.add_var("cut_to", false);
                    self.add_split_time("0", false);
                    self.add_split_time("fade_animation", false);
                    self.add_split_time("cut_to_animation", false);
                };
                let id = self.track_resource(self.resource_rect.clone(), "Background");
                self.resource_rect[id].size =
                    [ctx.available_rect().width(), ctx.available_rect().height()];
                let mut id = self.track_resource(self.resource_image.clone(), "RC_Logo");
                let mut id2 = self.track_resource(self.resource_text.clone(), "Powered");
                let id3 = self.track_resource(self.variables.clone(), "progress");
                if self.var_i("progress") >= 2 && self.var_i("progress") < 4 {
                    id = self.track_resource(self.resource_image.clone(), "Binder_Logo");
                    id2 = self.track_resource(self.resource_text.clone(), "Organize");
                } else if self.var_i("progress") >= 4 {
                    id = self.track_resource(self.resource_image.clone(), "Mouse");
                    id2 = self.track_resource(self.resource_text.clone(), "Mouse");
                };
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.rect(ui, "Background", ctx);
                    if self.timer.now_time >= 1.0 {
                        if self.var_i("progress") < 2 {
                            self.image(ui, "RC_Logo", ctx);
                            self.text(ui, "Powered", ctx);
                        } else if self.var_i("progress") < 4 {
                            self.image(ui, "Binder_Logo", ctx);
                            self.text(ui, "Organize", ctx);
                        } else {
                            self.image(ui, "Mouse", ctx);
                            self.text(ui, "Mouse", ctx);
                        };
                        for _ in 0..10 {
                            match self.var_i("progress") {
                                0 => {
                                    if self.resource_image[id].alpha == 255
                                        && self.resource_text[id2].rgba[3] == 255
                                        && (self.timer.now_time - self.split_time("0")[0]) >= 4.0
                                    {
                                        self.variables[id3].value = Value::Int(1);
                                        self.add_split_time("1", false);
                                    };
                                }
                                1 => {
                                    if self.resource_image[id].alpha == 0
                                        && self.resource_text[id2].rgba[3] == 0
                                        && (self.timer.now_time - self.split_time("1")[0]) >= 3.0
                                    {
                                        self.variables[id3].value = Value::Int(2);
                                        self.add_split_time("2", false);
                                    };
                                }
                                2 => {
                                    if self.resource_image[id].alpha == 255
                                        && self.resource_text[id2].rgba[3] == 255
                                        && (self.timer.now_time - self.split_time("2")[0]) >= 2.0
                                    {
                                        self.variables[id3].value = Value::Int(3);
                                        self.add_split_time("3", false);
                                    };
                                }
                                3 => {
                                    if self.resource_image[id].alpha == 0
                                        && self.resource_text[id2].rgba[3] == 0
                                        && (self.timer.now_time - self.split_time("3")[0]) >= 3.0
                                    {
                                        self.variables[id3].value = Value::Int(4);
                                        self.add_split_time("4", false);
                                    };
                                }
                                4 => {
                                    if self.resource_image[id].alpha == 255
                                        && self.resource_text[id2].rgba[3] == 255
                                        && (self.timer.now_time - self.split_time("4")[0]) >= 2.0
                                    {
                                        self.variables[id3].value = Value::Int(5);
                                        self.add_split_time("5", false);
                                    };
                                }
                                5 => {
                                    if self.resource_image[id].alpha == 0
                                        && self.resource_text[id2].rgba[3] == 0
                                        && (self.timer.now_time - self.split_time("5")[0]) >= 3.0
                                    {
                                        if self.config.login_user_name.is_empty() {
                                            self.switch_page("Login");
                                        } else {
                                            if let Ok(json_value) = read_from_json(format!(
                                                "Resources/config/user_{}.json",
                                                self.config.login_user_name
                                            )) {
                                                if let Some(read_user) =
                                                    User::from_json_value(&json_value)
                                                {
                                                    self.login_user_config = read_user;
                                                };
                                            };
                                            self.config.language = self.login_user_config.language;
                                            self.switch_page("Home_Page");
                                        };
                                    };
                                }
                                _ => {}
                            };
                            if self.var_i("progress") != 0
                                && self.var_i("progress") != 2
                                && self.var_i("progress") != 4
                                && self.resource_image[id].alpha != 0
                                && self.timer.now_time - self.split_time("fade_animation")[0]
                                    >= self.vertrefresh
                            {
                                self.resource_image[id].alpha -= 5;
                                self.add_split_time("fade_animation", true);
                            };
                            if self.var_i("progress") != 0
                                && self.var_i("progress") != 2
                                && self.var_i("progress") != 4
                                && self.resource_text[id2].rgba[3] != 0
                                && self.timer.now_time - self.split_time("fade_animation")[0]
                                    >= self.vertrefresh
                            {
                                self.resource_text[id2].rgba[3] -= 5;
                                self.add_split_time("fade_animation", true);
                            };
                            if self.var_i("progress") != 1
                                && self.var_i("progress") != 3
                                && self.var_i("progress") != 5
                                && self.resource_image[id].alpha != 255
                                && self.timer.now_time - self.split_time("fade_animation")[0]
                                    >= self.vertrefresh
                            {
                                self.resource_image[id].alpha += 5;
                                self.add_split_time("fade_animation", true);
                            };
                            if self.var_i("progress") != 1
                                && self.var_i("progress") != 3
                                && self.var_i("progress") != 5
                                && self.resource_text[id2].rgba[3] != 255
                                && self.timer.now_time - self.split_time("fade_animation")[0]
                                    >= self.vertrefresh
                            {
                                self.resource_text[id2].rgba[3] += 5;
                                self.add_split_time("fade_animation", true);
                            };
                        }
                    };
                });
            }
            "Login" => {
                let scroll_background =
                    self.track_resource(self.resource_scroll_background.clone(), "ScrollWallpaper");
                if !self.check_updated(&self.page.clone()) {
                    self.add_var("account_name_str", "".to_string());
                    self.add_var("account_password_str", "".to_string());
                    self.add_var("open_reg_window", false);
                    self.add_var("reg_status", Value::UInt(0));
                    self.add_var("reg_account_name_str", "".to_string());
                    self.add_var("reg_account_password_str", "".to_string());
                    self.add_var("reg_account_description_str", "".to_string());
                    self.add_var("reg_account_check_password_str", "".to_string());
                    self.add_var("reg_enable_password_error_message", false);
                    self.add_var("reg_enable_name_error_message", false);
                    self.add_var("login_enable_name_error_message", false);
                    self.add_var("login_enable_password_error_message", false);
                    self.add_var("last_window_size", vec![1280.0, 720.0]);
                    self.resource_scroll_background[scroll_background].resume_point =
                        ctx.available_rect().width();
                    for i in 0..self.resource_scroll_background[scroll_background]
                        .image_name
                        .len()
                    {
                        let id = self.track_resource(
                            self.resource_image.clone(),
                            &self.resource_scroll_background[scroll_background].image_name[i]
                                .clone(),
                        );
                        self.resource_image[id].image_size =
                            [ctx.available_rect().width(), ctx.available_rect().height()];
                        self.resource_image[id].image_position[0] =
                            i as f32 * self.resource_image[id].image_size[0];
                        self.resource_scroll_background[scroll_background].boundary =
                            -ctx.available_rect().width();
                    }
                };
                let mut input1 = self.var_s("account_name_str");
                let mut input2 = self.var_s("account_password_str");
                let mut input3 = self.var_s("reg_account_name_str");
                let mut input4 = self.var_s("reg_account_password_str");
                let mut input5 = self.var_s("reg_account_check_password_str");
                egui::CentralPanel::default().show(ctx, |ui| {
                    if self.var_decode_f(self.clone().var_v("last_window_size")[0].clone()) != ctx.available_rect().width()
                        || self.var_decode_f(self.clone().var_v("last_window_size")[1].clone()) != ctx.available_rect().height()
                    {
                        self.resource_scroll_background[scroll_background].resume_point =
                            ctx.available_rect().width();
                        for i in 0..self.resource_scroll_background[scroll_background]
                            .image_name
                            .len()
                        {
                            let id = self.track_resource(
                                self.resource_image.clone(),
                                &self.resource_scroll_background[scroll_background].image_name[i]
                                    .clone()
                            );
                            self.resource_image[id].image_size =
                                [ctx.available_rect().width(), ctx.available_rect().height()];
                            self.resource_image[id].image_position[0] =
                                i as f32 * self.resource_image[id].image_size[0];
                            self.resource_scroll_background[scroll_background].boundary =
                                -ctx.available_rect().width();
                        }
                    };
                    self.scroll_background(ui, "ScrollWallpaper", ctx);
                    let id = self.track_resource(self.resource_text.clone(), "Date");
                    self.resource_text[id].text_content = Local::now().format(&format!("{} {}", &game_text["date"][self.config.language as usize], &game_text["week"][self.config.language as usize])).to_string();
                    if self.config.language == 0 {
                        let week = match Local::now().format("%A").to_string().as_str() {
                            "Monday" => "一",
                            "Tuesday" => "二",
                            "Wednesday" => "三",
                            "Thursday" => "四",
                            "Friday" => "五",
                            "Saturday" => "六",
                            "Sunday" => "日",
                            _ => "一",
                        };
                        self.resource_text[id].text_content = format!("{} {}{}", Local::now().format(&game_text["date"][self.config.language as usize]), game_text["week"][self.config.language as usize], week);
                    }
                    let id2 = self.track_resource(self.resource_text.clone(), "Time");
                    self.resource_text[id2].text_content = Local::now().format(&game_text["time"][self.config.language as usize]).to_string();
                    self.text(ui, "Date", ctx);
                    self.text(ui, "Time", ctx);
                    egui::Area::new("Login".into())
                        .fixed_pos(egui::Pos2::new(
                            ctx.available_rect().width() / 2_f32 - 100_f32,
                            ctx.available_rect().height() / 4_f32 * 3_f32,
                        ))
                        .show(ui.ctx(), |ui| {
                            if !self.var_b("open_reg_window") {
                                egui::ComboBox::from_label("")
                                .selected_text(game_text["language"][self.config.language as usize].clone())
                                .width(200_f32)
                                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                                .show_ui(ui, |ui| {
                                    let lang = self.config.language;
                                    for i in 0..self.config.amount_languages {
                                        ui.selectable_value(&mut self.config.language, i, format!("{}({})", game_text["language"][i as usize].clone(), game_text[&format!("{}_language", lang)][i as usize].clone())); 
                                    };
                                }
                            );
                            };
                            ui.add(
                                egui::TextEdit::singleline(&mut input1)
                                    .cursor_at_end(true)
                                    .desired_width(200_f32)
                                    .char_limit(20)
                                    .interactive(!self.var_b("open_reg_window"))
                                    .hint_text(game_text["account_name"][self.config.language as usize].clone())
                                    .font(egui::FontId::proportional(16.0)), // 字体大小
                            );
                            if self.var_b("login_enable_name_error_message") {
                                ui.colored_label(egui::Color32::RED, game_text["login_name_error"][self.config.language as usize].clone());
                            };
                            ui.add(
                                egui::TextEdit::singleline(&mut input2)
                                    .cursor_at_end(true)
                                    .desired_width(200_f32)
                                    .char_limit(20)
                                    .interactive(!self.var_b("open_reg_window"))
                                    .hint_text(game_text["account_password"][self.config.language as usize].clone())
                                    .password(true)
                                    .font(egui::FontId::proportional(16.0)), // 字体大小
                            );
                            if self.var_b("login_enable_password_error_message") {
                                ui.colored_label(egui::Color32::RED, game_text["login_password_error"][self.config.language as usize].clone());
                            };
                        });
                    let no_window = !self.var_b("open_reg_window");
                    if self.switch("Shutdown", ui, ctx, no_window)[0] != 5 {
                        write_to_json("Resources/config/Preferences.json", self.config.to_json_value()).unwrap();
                        exit(0);
                    };
                    if self.switch("Login", ui, ctx, no_window)[0] != 5 {
                        self.modify_var("login_enable_name_error_message", !check_file_exists(format!("Resources/config/user_{}.json", input1.replace(" ", "").replace("/", "").replace("\\", ""))));
                        if check_file_exists(format!("Resources/config/user_{}.json", input1.replace(" ", "").replace("/", "").replace("\\", ""))) {
                            let mut user = User {
                                version: 0,
                                name: "".to_string(),
                                password: "".to_string(),
                                language: 0,
                                wallpaper: "".to_string(),
                                current_map: "".to_string(),
                            };
                            if let Ok(json_value) = read_from_json(format!("Resources/config/user_{}.json", input1.replace(" ", "").replace("/", "").replace("\\", ""))) {
                                if let Some(read_user) = User::from_json_value(&json_value) {
                                    user = read_user;
                                }
                            };
                            if user.password == input2 {
                                self.config.login_user_name = user.name;
                                self.config.language = user.language;
                                input1 = "".to_string();
                                input2 = "".to_string();
                                self.timer.start_time = self.timer.total_time;
                                self.update_timer();
                                if check_resource_exist(self.timer.split_time.clone(), "dock_animation") {
                                    self.add_split_time("dock_animation", true);
                                    self.add_split_time("title_animation", true);
                                };
                                self.switch_page("Home_Page");
                                if let Ok(json_value) = read_from_json(format!(
                                    "Resources/config/user_{}.json",
                                    self.config.login_user_name
                                )) {
                                    if let Some(read_user) = User::from_json_value(&json_value) {
                                        self.login_user_config = read_user;
                                    };
                                };
                                if check_resource_exist(self.resource_image_texture.clone(), "Home_Wallpaper") {
                                    self.add_image_texture("Home_Wallpaper", &self.login_user_config.wallpaper.clone(), [false, false], false, ctx);
                                    let id = self.track_resource(self.resource_image_texture.clone(), "Home_Wallpaper");
                                    let id2 = self.track_resource(self.resource_image.clone(), "Home_Wallpaper");
                                    self.resource_image[id2].image_texture = self.resource_image_texture[id].texture.clone();
                                };
                            };
                            self.modify_var("login_enable_password_error_message", user.password != input2);
                        };
                    };
                    if self.switch("Register", ui, ctx, no_window)[0] != 5 {
                        self.modify_var("reg_status", Value::UInt(0));
                        self.modify_var("open_reg_window", true);
                    };
                    egui::Window::new("Reg")
                        .open(&mut self.var_b("open_reg_window"))
                        .frame(self.frame)
                        .resizable(false)
                        .title_bar(false)
                        .pivot(egui::Align2::CENTER_CENTER)
                        .scroll(true)
                        .default_size(egui::Vec2::new(200_f32, 300_f32))
                        .fixed_pos(egui::Pos2::new(
                            ctx.available_rect().width() / 2_f32,
                            ctx.available_rect().height() / 2_f32,
                        ))
                        .show(ctx, |ui| {
                            ui.vertical_centered(|ui| {
                                if self.var_u("reg_status") == 0 {
                                    ui.heading(game_text["welcome"][self.config.language as usize].clone());
                                } else if self.var_u("reg_status") == 1 {
                                    ui.heading(game_text["reg_account"][self.config.language as usize].clone());
                                } else if self.var_u("reg_status") == 2 {
                                    ui.heading(game_text["reg_complete"][self.config.language as usize].clone());
                                };
                                ui.separator();
                                if self.var_u("reg_status") == 0 {
                                    self.image(ui, "Gun_Logo", ctx);
                                    ui.label(game_text["intro"][self.config.language as usize].clone());
                                } else if self.var_u("reg_status") == 1 {
                                    ui.add(
                                        egui::TextEdit::singleline(&mut input3)
                                            .cursor_at_end(true)
                                            .desired_width(200_f32)
                                            .char_limit(20)
                                            .hint_text(game_text["reg_account_name"][self.config.language as usize].clone())
                                            .font(egui::FontId::proportional(16.0)),
                                    );
                                    ui.label(format!("{}{}", game_text["reg_name_preview"][self.config.language as usize].clone(), input3.replace(" ", "").replace("/", "")).replace("\\", ""));
                                    ui.add(
                                        egui::TextEdit::singleline(&mut input4)
                                            .cursor_at_end(true)
                                            .desired_width(200_f32)
                                            .char_limit(20)
                                            .password(true)
                                            .hint_text(game_text["reg_account_password"][self.config.language as usize].clone())
                                            .font(egui::FontId::proportional(16.0)),
                                    );
                                    ui.add(
                                        egui::TextEdit::singleline(&mut input5)
                                            .cursor_at_end(true)
                                            .desired_width(200_f32)
                                            .char_limit(20)
                                            .password(true)
                                            .hint_text(game_text["reg_account_check_password"][self.config.language as usize].clone())
                                            .font(egui::FontId::proportional(16.0)),
                                    );
                                } else if self.var_u("reg_status") == 2 {
                                    self.image(ui, "Reg_Complete", ctx);
                                    ui.label(game_text["reg_success"][self.config.language as usize].clone());
                                };
                                    if self.var_u("reg_status") == 0 {
                                        if ui.button(game_text["cancel"][self.config.language as usize].clone()).clicked() {
                                            general_click_feedback();
                                            self.modify_var("open_reg_window", false);
                                        };
                                        if ui.button(game_text["continue"][self.config.language as usize].clone()).clicked() {
                                            general_click_feedback();
                                            self.modify_var("reg_enable_name_error_message", false);
                                            self.modify_var("reg_enable_password_error_message", false);
                                            self.modify_var("reg_status", Value::UInt(1));
                                        };
                                    } else if self.var_u("reg_status") == 1 {
                                        if ui.button(game_text["cancel"][self.config.language as usize].clone()).clicked() {
                                            general_click_feedback();
                                            self.modify_var("reg_status", Value::UInt(0));
                                        };
                                        if ui.button(game_text["continue"][self.config.language as usize].clone()).clicked() {
                                            general_click_feedback();
                                            self.modify_var("reg_enable_password_error_message", input4 != input5);
                                            self.modify_var("reg_enable_name_error_message", input3.replace(" ", "").replace("/", "").replace("\\", "").is_empty() || check_file_exists(format!("Resources/config/user_{}.json", input3.replace(" ", "").replace("/", "")).replace("\\", "")));
                                            if input4 == input5 && !check_file_exists(format!("Resources/config/user_{}.json", input3.replace(" ", "").replace("/", "")).replace("\\", "")) && !input3.replace(" ", "").replace("/", "").replace("\\", "").is_empty(){
                                                    let user_data = object! {
                                                        "version": 10,
                                                        "name": input3.replace(" ", "").replace("/", "").replace("\\", "").clone(),
                                                        "password": input4.clone(),
                                                        "language": self.config.language,
                                                        "wallpaper": "Resources/assets/images/wallpaper.png".to_string(),
                                                        "current_map": "map_tutorial".to_string(),
                                                    };
                                                    create_pretty_json(format!("Resources/config/user_{}.json", input3.replace(" ", "").replace("/", "").replace("\\", "")), user_data).unwrap();
                                                    self.modify_var("reg_status", Value::UInt(2));
                                            };
                                        };
                                        if self.var_b("reg_enable_password_error_message") {
                                            ui.colored_label(egui::Color32::RED, game_text["reg_check_password_error"][self.config.language as usize].clone());
                                        };
                                        if self.var_b("reg_enable_name_error_message") {
                                            ui.colored_label(egui::Color32::RED, game_text["reg_name_error"][self.config.language as usize].clone());
                                        };
                                    } else if self.var_u("reg_status") == 2 {
                                        if ui.button(game_text["re_reg"][self.config.language as usize].clone()).clicked() {
                                            general_click_feedback();
                                            self.modify_var("reg_status", Value::UInt(0));
                                        };
                                        if ui.button(game_text["reg_complete"][self.config.language as usize].clone()).clicked() {
                                            general_click_feedback();
                                            input1 = input3.replace(" ", "").replace("/", "").replace("\\", "");
                                            input2 = input5.clone();
                                            input3 = "".to_string();
                                            input4 = "".to_string();
                                            input5 = "".to_string();
                                            self.modify_var("open_reg_window", false);
                                        };
                                    };
                            });
                        });
                    self.modify_var("account_name_str", input1);
                    self.modify_var("account_password_str", input2);
                    self.modify_var("reg_account_name_str", input3);
                    self.modify_var("reg_account_password_str", input4);
                    self.modify_var("reg_account_check_password_str", input5);
                });
                self.modify_var(
                    "last_window_size",
                    vec![ctx.available_rect().width(), ctx.available_rect().height()],
                );
            }
            "Home_Page" => {
                if !self.check_updated(&self.page.clone()) {
                    self.add_image_texture(
                        "Home_Wallpaper",
                        &self.login_user_config.wallpaper.clone(),
                        [false, false],
                        true,
                        ctx,
                    );
                    self.add_image(
                        "Home_Wallpaper",
                        [
                            0_f32,
                            0_f32,
                            ctx.available_rect().width(),
                            ctx.available_rect().height(),
                        ],
                        [1, 2, 1, 2],
                        [true, true, true, true, false],
                        [255, 0, 0, 0, 0],
                        "Home_Wallpaper",
                    );
                    self.add_var("title_float_status", true);
                    self.add_var("dock_active_status", false);
                    self.add_var("refreshed_map_data", false);
                    self.add_var("selected_map", Value::UInt(0));
                    self.add_split_time("title_animation", false);
                    self.add_split_time("dock_animation", false);
                };
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.wallpaper(ui, ctx);
                    self.image(
                        ui,
                        &format!("{}_Title", self.login_user_config.language),
                        ctx,
                    );
                    let id = self.track_resource(
                        self.resource_image.clone(),
                        &format!("{}_Title", self.login_user_config.language),
                    );
                    if self.timer.now_time - self.split_time("title_animation")[0]
                        >= self.vertrefresh
                    {
                        self.add_split_time("title_animation", true);
                        if self.var_b("title_float_status") {
                            if self.resource_image[id].origin_position[1] < 5_f32 {
                                self.resource_image[id].origin_position[1] += 0.05;
                            } else {
                                self.modify_var("title_float_status", false);
                            };
                        } else if self.resource_image[id].origin_position[1] > -5_f32 {
                            self.resource_image[id].origin_position[1] -= 0.05;
                        } else {
                            self.modify_var("title_float_status", true);
                        };
                    };
                    self.dock(ctx, ui);
                });
            }
            "Home_Setting" => {
                self.check_updated(&self.page.clone());
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.wallpaper(ui, ctx);
                    egui::ScrollArea::vertical()
                        .max_height(ctx.available_rect().height() - 100.0)
                        .max_width(ctx.available_rect().width() / 4_f32 * 3_f32)
                        .auto_shrink(false)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::WidgetText::from(
                                        game_text["game_language"]
                                            [self.login_user_config.language as usize]
                                            .clone()
                                            .to_string(),
                                    )
                                    .text_style(egui::TextStyle::Heading),
                                );
                                ui.separator();
                                egui::ComboBox::from_label("")
                                    .selected_text(
                                        game_text["language"]
                                            [self.login_user_config.language as usize]
                                            .clone(),
                                    )
                                    .width(200_f32)
                                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                                    .show_ui(ui, |ui| {
                                        let lang = self.login_user_config.language;
                                        for i in 0..self.config.amount_languages {
                                            ui.selectable_value(
                                                &mut self.login_user_config.language,
                                                i,
                                                format!(
                                                    "{}({})",
                                                    game_text["language"][i as usize].clone(),
                                                    game_text[&format!("{}_language", lang)]
                                                        [i as usize]
                                                        .clone()
                                                ),
                                            );
                                        }
                                        self.config.language = self.login_user_config.language;
                                    });
                            });
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::WidgetText::from(
                                        game_text["game_version"]
                                            [self.login_user_config.language as usize]
                                            .clone()
                                            .to_string(),
                                    )
                                    .text_style(egui::TextStyle::Heading),
                                );
                                ui.separator();
                                ui.label(
                                    egui::WidgetText::from(
                                        game_text["debug_game_version"]
                                            [self.login_user_config.language as usize]
                                            .clone()
                                            .to_string(),
                                    )
                                    .text_style(egui::TextStyle::Heading),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::WidgetText::from(
                                        game_text["game_wallpaper"]
                                            [self.login_user_config.language as usize]
                                            .clone()
                                            .to_string(),
                                    )
                                    .text_style(egui::TextStyle::Heading),
                                );
                                ui.separator();
                                if ui
                                    .button(
                                        game_text["game_change_wallpaper"]
                                            [self.login_user_config.language as usize]
                                            .clone(),
                                    )
                                    .clicked()
                                {
                                    general_click_feedback();
                                    if let Some(path) = FileDialog::new()
                                        .set_title(
                                            &game_text["choose_image"]
                                                [self.login_user_config.language as usize]
                                                .clone(),
                                        )
                                        .add_filter("", &["png"])
                                        .pick_file()
                                    {
                                        // 复制文件
                                        fs::copy(
                                            &path,
                                            std::path::Path::new(&format!(
                                                "Resources/assets/images/{}_new_wallpaper.png",
                                                self.config.login_user_name
                                            )),
                                        )
                                        .unwrap();
                                        self.add_image_texture(
                                            "Home_Wallpaper",
                                            &format!(
                                                "Resources/assets/images/{}_new_wallpaper.png",
                                                self.config.login_user_name
                                            ),
                                            [false, false],
                                            false,
                                            ctx,
                                        );
                                        let id = self.track_resource(
                                            self.resource_image_texture.clone(),
                                            "Home_Wallpaper",
                                        );
                                        let id2 = self.track_resource(
                                            self.resource_image.clone(),
                                            "Home_Wallpaper",
                                        );
                                        self.resource_image[id2].image_texture =
                                            self.resource_image_texture[id].texture.clone();
                                        self.login_user_config.wallpaper = format!(
                                            "Resources/assets/images/{}_new_wallpaper.png",
                                            self.config.login_user_name
                                        );
                                    };
                                };
                                if ui
                                    .button(
                                        game_text["return_to_default"]
                                            [self.login_user_config.language as usize]
                                            .clone(),
                                    )
                                    .clicked()
                                {
                                    general_click_feedback();
                                    self.add_image_texture(
                                        "Home_Wallpaper",
                                        "Resources/assets/images/wallpaper.png",
                                        [false, false],
                                        false,
                                        ctx,
                                    );
                                    let id = self.track_resource(
                                        self.resource_image_texture.clone(),
                                        "Home_Wallpaper",
                                    );
                                    let id2 = self.track_resource(
                                        self.resource_image.clone(),
                                        "Home_Wallpaper",
                                    );
                                    self.resource_image[id2].image_texture =
                                        self.resource_image_texture[id].texture.clone();
                                    self.login_user_config.wallpaper =
                                        "Resources/assets/images/wallpaper.png".to_string();
                                };
                            });
                        });
                    self.dock(ctx, ui);
                });
            }
            "Home_Select_Map" => {
                if !self.check_updated(&self.page.clone()) {
                    self.add_split_time("map_select_animation", false);
                    self.add_var("fade_in_or_out", true);
                };
                let mut map_information = Map {
                    map_name: vec![],
                    map_author: "".to_string(),
                    map_image: "".to_string(),
                    map_width: 0_f32,
                    map_description: vec![],
                    map_intro: "".to_string(),
                };
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.wallpaper(ui, ctx);
                    let map_list = list_files_recursive(Path::new("Resources/config"), "map_")
                        .unwrap_or_default();
                    let mut map_move_animation = 0;
                    let enable = !self.var_b("cut_to");
                    if self.var_b("refreshed_map_data") {
                        let selected_map = self.var_u("selected_map");
                        let selected_map_id = self.track_resource(
                            self.resource_image.clone(),
                            &format!("Map_{:?}", map_list[selected_map as usize]),
                        );
                        if self.resource_image[selected_map_id].origin_position[0] != 0_f32
                            && (self.timer.now_time - self.split_time("map_select_animation")[0])
                                >= self.vertrefresh
                        {
                            if self.resource_image[selected_map_id].origin_position[0] > 0_f32 {
                                map_move_animation = 1;
                            } else {
                                map_move_animation = 2;
                            };
                            self.add_split_time("map_select_animation", true);
                        };
                    };
                    for (i, _) in map_list.iter().enumerate().take(
                        count_files_recursive(Path::new("Resources/config"), "map_").unwrap_or(0),
                    ) {
                        if let Ok(json_value) =
                            read_from_json(map_list[i].to_string_lossy().to_string())
                        {
                            if let Some(read_map_information) = Map::from_json_value(&json_value) {
                                map_information = read_map_information;
                            }
                            if !check_resource_exist(
                                self.resource_image_texture.clone(),
                                &format!("Map_{:?}", map_list[i]),
                            ) && !self.var_b("refreshed_map_data")
                            {
                                self.add_image_texture(
                                    &format!("Map_{:?}", map_list[i]),
                                    &map_information.map_intro,
                                    [false, false],
                                    true,
                                    ctx,
                                );
                                self.add_image(
                                    &format!("Map_{:?}", map_list[i]),
                                    [(450 * i) as f32, 0_f32, 400_f32, 400_f32],
                                    [1, 2, 1, 2],
                                    [false, false, true, true, true],
                                    [255, 255, 255, 255, 255],
                                    &format!("Map_{:?}", map_list[i]),
                                );
                                self.add_switch(
                                    [
                                        &format!("Map_{:?}", map_list[i]),
                                        &format!("Map_{:?}", map_list[i]),
                                    ],
                                    vec![
                                        SwitchData {
                                            texture: format!("Map_{:?}", map_list[i]),
                                            color: [255, 255, 255, 255],
                                        },
                                        SwitchData {
                                            texture: format!("Map_{:?}", map_list[i]),
                                            color: [180, 180, 180, 255],
                                        },
                                        SwitchData {
                                            texture: format!("Map_{:?}", map_list[i]),
                                            color: [150, 150, 150, 255],
                                        },
                                    ],
                                    [true, true, true],
                                    1,
                                    vec![SwitchClickAction {
                                        click_method: egui::PointerButton::Primary,
                                        action: false,
                                    }],
                                );
                            };
                        };
                        if map_move_animation != 0 {
                            let id = self.track_resource(
                                self.resource_image.clone(),
                                &format!("Map_{:?}", map_list[i]),
                            );
                            if map_move_animation == 1 {
                                self.resource_image[id].origin_position[0] -= 30_f32;
                            } else {
                                self.resource_image[id].origin_position[0] += 30_f32;
                            };
                        };
                        if self.switch(&format!("Map_{:?}", map_list[i]), ui, ctx, enable)[0] == 0 {
                            self.modify_var("cut_to", true);
                            self.modify_var("fade_in_or_out", true);
                            self.login_user_config.current_map =
                                map_list[i].to_string_lossy().to_string();
                        };
                    }
                    self.modify_var("refreshed_map_data", true);
                    if self.switch("Forward", ui, ctx, enable)[0] == 0
                        && self.var_u("selected_map")
                            < (count_files_recursive(Path::new("Resources/config"), "map_")
                                .unwrap_or(0)
                                - 1) as u32
                    {
                        let selected_map = self.var_u("selected_map");
                        self.modify_var("selected_map", Value::UInt(selected_map + 1));
                    };
                    if self.switch("Backward", ui, ctx, enable)[0] == 0
                        && self.var_u("selected_map") > 0
                    {
                        let selected_map = self.var_u("selected_map");
                        self.modify_var("selected_map", Value::UInt(selected_map - 1));
                    };
                    if !self.var_b("cut_to") {
                        self.dock(ctx, ui);
                    } else {
                        let fade_in_or_out = self.var_b("fade_in_or_out");
                        if self.fade(fade_in_or_out, ctx, ui, "cut_to_animation", "Cut_To_Background") == 255 && fade_in_or_out {
                            if let Ok(json_value) =
                                read_from_json(&self.login_user_config.current_map)
                            {
                                if let Some(read_map_information) =
                                    Map::from_json_value(&json_value)
                                {
                                    map_information = read_map_information;
                                }
                            };
                            self.add_image_texture(
                                &format!("{}", map_information.map_image),
                                &format!("{}", map_information.map_image),
                                [false, false],
                                true,
                                ctx,
                            );
                            self.add_image(
                                &format!("{}", map_information.map_image),
                                [
                                    0_f32,
                                    0_f32,
                                    map_information.map_width,
                                    ctx.available_rect().height(),
                                ],
                                [0, 0, 0, 0],
                                [true, true, false, false, false],
                                [255, 0, 0, 0, 0],
                                &format!("{}", map_information.map_image),
                            );
                            self.modify_var("fade_in_or_out", false);
                            self.switch_page("Select_Level");
                            self.timer.start_time = self.timer.total_time;
                            self.update_timer();
                            self.add_split_time("cut_to_animation", true);
                            if check_resource_exist(self.timer.split_time.clone(), "scroll_animation") {
                                self.add_split_time("scroll_animation", true);
                            };
                        } else if self.fade(fade_in_or_out, ctx, ui, "cut_to_animation", "Cut_To_Background") == 0 && !fade_in_or_out {
                            self.modify_var("cut_to", false);
                        };
                    };
                });
            }
            "Select_Level" => {
                let mut map_information = Map {
                    map_name: vec![],
                    map_author: "".to_string(),
                    map_image: "".to_string(),
                    map_width: 0_f32,
                    map_description: vec![],
                    map_intro: "".to_string(),
                };
                if let Ok(json_value) = read_from_json(&self.login_user_config.current_map) {
                    if let Some(read_map_information) = Map::from_json_value(&json_value) {
                        map_information = read_map_information;
                    }
                };
                if !self.check_updated(&self.page.clone()) {
                    self.add_split_time("scroll_animation", false);
                };
                egui::CentralPanel::default().show(ctx, |ui| {
                    let map_background_id = self.track_resource(
                        self.resource_image.clone(),
                        &format!("{}", map_information.map_image),
                    );
                    let scroll_remind_id =
                        self.track_resource(self.resource_image.clone(), "Scroll_Forward");
                    let scroll_remind_id2 =
                        self.track_resource(self.resource_image.clone(), "Scroll_Backward");
                    self.resource_image[map_background_id].image_size[1] =
                        ctx.available_rect().height();
                    self.resource_image[scroll_remind_id].image_size[1] =
                        ctx.available_rect().height();
                    self.resource_image[scroll_remind_id2].image_size[1] =
                        ctx.available_rect().height();
                    self.image(ui, &format!("{}", map_information.map_image), ctx);
                    if self.switch("Back", ui, ctx, true)[0] == 0 {
                        self.modify_var("fade_in_or_out", true);
                        self.modify_var("cut_to", true);
                    };
                    self.image(ui, "Scroll_Forward", ctx);
                    self.image(ui, "Scroll_Backward", ctx);
                    if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                        if self.timer.now_time - self.split_time("scroll_animation")[0] >= self.vertrefresh {
                            if mouse_pos.x < 50_f32 && self.resource_image[map_background_id].image_position[0] < 0_f32 {
                                for _ in 0..10 {
                                    if self.resource_image[map_background_id].image_position[0] < 0_f32 {
                                        self.resource_image[map_background_id].image_position[0] += 1_f32;
                                    } else {
                                        break
                                    };
                                };
                            } else if mouse_pos.x > (ctx.available_rect().width() - 50_f32) && self.resource_image[map_background_id].image_position[0] > ctx.available_rect().width() - map_information.map_width {
                                for _ in 0..10 {
                                    if self.resource_image[map_background_id].image_position[0] > ctx.available_rect().width() - map_information.map_width {
                                        self.resource_image[map_background_id].image_position[0] -= 1_f32;
                                    } else {
                                        break
                                    };
                                };
                            };
                            self.add_split_time("scroll_animation", true);
                        };
                    };
                    if self.resource_image[map_background_id].image_position[0] < ctx.available_rect().width() - map_information.map_width {
                        self.resource_image[map_background_id].image_position[0] = ctx.available_rect().width() - map_information.map_width;
                    };
                    if self.var_b("cut_to") {
                        let fade_in_or_out = self.var_b("fade_in_or_out");
                        if self.fade(fade_in_or_out, ctx, ui, "cut_to_animation", "Cut_To_Background") == 255 && fade_in_or_out {
                            self.timer.start_time = self.timer.total_time;
                            self.update_timer();
                            self.add_split_time("cut_to_animation", true);
                            self.add_split_time("dock_animation", true);
                            self.add_split_time("map_select_animation", true);
                            self.modify_var("fade_in_or_out", false);
                            self.switch_page("Home_Select_Map");
                        } else if self.fade(fade_in_or_out, ctx, ui, "cut_to_animation", "Cut_To_Background") == 0 && !fade_in_or_out {
                            self.modify_var("cut_to", false);
                        };
                    };
                });
            }
            "Error" => {
                self.check_updated(&self.page.clone());
                let id = self.track_resource(self.resource_text.clone(), "Error_Pages_Reason");
                let id2 = self.track_resource(self.resource_text.clone(), "Error_Pages_Solution");
                let id3 = self.track_resource(self.resource_rect.clone(), "Error_Pages_Background");
                self.resource_text[id].text_content =
                    game_text["error_pages_reason"][self.config.language as usize].clone();
                self.resource_text[id2].text_content =
                    game_text["error_pages_solution"][self.config.language as usize].clone();
                self.resource_rect[id3].size =
                    [ctx.available_rect().width(), ctx.available_rect().height()];
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.rect(ui, "Error_Pages_Background", ctx);
                    self.text(ui, "Error_Pages_Sorry", ctx);
                    self.text(ui, "Error_Pages_Reason", ctx);
                    self.text(ui, "Error_Pages_Solution", ctx);
                });
            }
            _ => {
                if self.config.rc_strict_mode {
                    panic!(
                        "{}{}",
                        game_text["error_page_not_found"][self.config.language as usize].clone(),
                        self.page
                    );
                };
                self.problem_report(
                    &format!(
                        "{}{}",
                        game_text["error_page_not_found"][self.config.language as usize].clone(),
                        self.page
                    ),
                    SeverityLevel::Error,
                    &game_text["error_page_not_found_annotation"][self.config.language as usize]
                        .clone(),
                );
                std::thread::spawn(|| {
                    kira_play_wav("Resources/assets/sounds/Error.wav").unwrap();
                });
                self.switch_page("Error")
            }
        };
        egui::TopBottomPanel::top("Debug mode")
            .frame(egui::Frame {
                fill: egui::Color32::TRANSPARENT,
                inner_margin: egui::Margin::symmetric(8.0, 4.0), // 按需调整
                ..Default::default()
            })
            .show_separator_line(false)
            .show(ctx, |ui| {
                if ctx.input(|i| i.key_pressed(egui::Key::F3)) {
                    std::thread::spawn(|| {
                        kira_play_wav("Resources/assets/sounds/Notification.wav").unwrap();
                    });
                    let enable_debug_mode = self.var_b("enable_debug_mode");
                    self.modify_var("enable_debug_mode", !enable_debug_mode);
                };
                if self.var_b("enable_debug_mode") {
                    egui::Window::new("performance")
                    .frame(self.frame)
                    .title_bar(false)
                    .open(&mut self.var_b("debug_fps_window"))
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading(game_text["debug_frame_number_details"][self.config.language as usize].clone());
                        });
                        ui.separator();
                        ui.label(format!("{}: {:.1}{}", game_text["debug_fps"][self.config.language as usize].clone(), self.current_fps(), game_text["debug_fps2"][self.config.language as usize].clone()));
                        ui.separator();
                        ui.label(format!("{}:", game_text["debug_last_ten_frames"][self.config.language as usize].clone()));
                        self.frame_times
                            .iter()
                            .rev()
                            .take(10)
                            .enumerate()
                            .for_each(|(i, &t)| {
                                ui.label(format!("{} {}: {:.2}{}", game_text["debug_frame"][self.config.language as usize].clone(), i + 1, t * 1000.0, game_text["debug_game_millisecond"][self.config.language as usize].clone()));
                            });
                    });
                    egui::Window::new("render_resource_list")
                    .frame(self.frame)
                    .title_bar(false)
                    .open(&mut self.var_b("debug_resource_list_window"))
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading(game_text["debug_render_resource_list"][self.config.language as usize].clone());
                        });
                        ui.separator();
                        egui::ScrollArea::vertical()
                        .max_height(ctx.available_rect().height() - 100.0)
                        .max_width(ctx.available_rect().width() - 100.0)
                        .show(ui, |ui| {
                            self.render_resource_list
                                    .iter()
                                    .rev()
                                    .take(self.render_resource_list.len())
                                    .for_each(|t| {
                                        ui.label(format!("{}: {}", game_text["debug_resource_name"][self.config.language as usize].clone(), t.name));
                                        ui.label(format!("{}: {}", game_text["debug_resource_type"][self.config.language as usize].clone(), t.discern_type));
                                        ui.separator();
                                    });
                        })});
                    egui::Window::new("resource_list")
                    .frame(self.frame)
                    .title_bar(false)
                    .open(&mut self.var_b("debug_resource_list_window"))
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading(game_text["debug_all_resource_list"][self.config.language as usize].clone());
                        });
                        ui.separator();
                        egui::ScrollArea::vertical()
                        .max_height(ctx.available_rect().height() - 100.0)
                        .max_width(ctx.available_rect().width() - 100.0)
                        .show(ui, |ui| {
                                self.resource_page
                                    .iter()
                                    .rev()
                                    .take(self.resource_page.len())
                                    .for_each(|t| {
                                        ui.label(format!("{}: {}", game_text["debug_resource_name"][self.config.language as usize].clone(), t.name));
                                        ui.label(format!("{}: {}", game_text["debug_resource_type"][self.config.language as usize].clone(), t.discern_type));
                                        ui.separator();
                                    });
                                self.resource_image
                                    .iter()
                                    .rev()
                                    .take(self.resource_image.len())
                                    .for_each(|t| {
                                        ui.label(format!("{}: {}", game_text["debug_resource_name"][self.config.language as usize].clone(), t.name));
                                        ui.label(format!("{}: {}", game_text["debug_resource_type"][self.config.language as usize].clone(), t.discern_type));
                                        ui.colored_label(egui::Color32::RED, format!("{}: {:?}", game_text["debug_resource_size"][self.config.language as usize].clone(), t.image_size));
                                        ui.colored_label(egui::Color32::RED, format!("{}: {:?}", game_text["debug_resource_position"][self.config.language as usize].clone(), t.image_position));
                                        ui.colored_label(egui::Color32::RED, format!("{}: {:?}", game_text["debug_resource_origin_or_excursion_position"][self.config.language as usize].clone(), t.origin_position));
                                        ui.colored_label(egui::Color32::RED, format!("{}: {:?}", game_text["debug_resource_alpha"][self.config.language as usize].clone(), t.alpha));
                                        if t.use_overlay_color {
                                            ui.colored_label(egui::Color32::RED, format!("{}: {:?}", game_text["debug_resource_image_overlay"][self.config.language as usize].clone(), t.overlay_color));
                                        };
                                        ui.colored_label(egui::Color32::RED, format!("{}: {:?}", game_text["debug_resource_origin_cite_texture"][self.config.language as usize].clone(), t.origin_cite_texture));
                                        ui.separator();
                                    });
                                self.resource_text
                                    .iter()
                                    .rev()
                                    .take(self.resource_text.len())
                                    .for_each(|t| {
                                        ui.label(format!("{}: {}", game_text["debug_resource_name"][self.config.language as usize].clone(), t.name));
                                        ui.label(format!("{}: {}", game_text["debug_resource_type"][self.config.language as usize].clone(), t.discern_type));
                                        ui.colored_label(egui::Color32::BLUE, format!("{}: {:?}", game_text["debug_resource_text_content"][self.config.language as usize].clone(), t.text_content));
                                        ui.colored_label(egui::Color32::BLUE, format!("{}: {:?}", game_text["debug_resource_size"][self.config.language as usize].clone(), t.font_size));
                                        ui.colored_label(egui::Color32::BLUE, format!("{}: {:?}", game_text["debug_resource_position"][self.config.language as usize].clone(), t.position));
                                        ui.colored_label(egui::Color32::BLUE, format!("{}: {:?}", game_text["debug_resource_origin_or_excursion_position"][self.config.language as usize].clone(), t.origin_position));
                                        ui.colored_label(egui::Color32::BLUE, format!("{}: {:?}", game_text["debug_resource_text_wrap_width"][self.config.language as usize].clone(), t.wrap_width));
                                        ui.colored_label(egui::Color32::BLUE, format!("{}: {:?}", game_text["debug_resource_color"][self.config.language as usize].clone(), t.rgba));
                                        if t.write_background {
                                            ui.colored_label(egui::Color32::BLUE, format!("{}: {:?}", game_text["debug_resource_text_background_color"][self.config.language as usize].clone(), t.background_rgb));
                                            ui.colored_label(egui::Color32::BLUE, format!("{}: {:?}", game_text["debug_resource_text_background_rounding"][self.config.language as usize].clone(), t.rounding));
                                        };
                                        ui.separator();
                                    });
                                self.resource_rect
                                    .iter()
                                    .rev()
                                    .take(self.resource_rect.len())
                                    .for_each(|t| {
                                        ui.label(format!("{}: {}", game_text["debug_resource_name"][self.config.language as usize].clone(), t.name));
                                        ui.label(format!("{}: {}", game_text["debug_resource_type"][self.config.language as usize].clone(), t.discern_type));
                                        ui.colored_label(egui::Color32::YELLOW, format!("{}: {:?}", game_text["debug_resource_position"][self.config.language as usize].clone(), t.position));
                                        ui.colored_label(egui::Color32::YELLOW, format!("{}: {:?}", game_text["debug_resource_size"][self.config.language as usize].clone(), t.size));
                                        ui.colored_label(egui::Color32::YELLOW, format!("{}: {:?}", game_text["debug_resource_origin_or_excursion_position"][self.config.language as usize].clone(), t.origin_position));
                                        ui.colored_label(egui::Color32::YELLOW, format!("{}: {:?}", game_text["debug_resource_rect_rounding"][self.config.language as usize].clone(), t.rounding));
                                        ui.colored_label(egui::Color32::YELLOW, format!("{}: {:?}", game_text["debug_resource_color"][self.config.language as usize].clone(), t.color));
                                        ui.colored_label(egui::Color32::YELLOW, format!("{}: {:?}", game_text["debug_resource_rect_border_width"][self.config.language as usize].clone(), t.border_width));
                                        ui.colored_label(egui::Color32::YELLOW, format!("{}: {:?}", game_text["debug_resource_rect_border_color"][self.config.language as usize].clone(), t.border_color));
                                        ui.separator();
                                    });
                                self.resource_scroll_background
                                    .iter()
                                    .rev()
                                    .take(self.resource_scroll_background.len())
                                    .for_each(|t| {
                                        ui.label(format!("{}: {}", game_text["debug_resource_name"][self.config.language as usize].clone(), t.name));
                                        ui.label(format!("{}: {}", game_text["debug_resource_type"][self.config.language as usize].clone(), t.discern_type));
                                        ui.colored_label(egui::Color32::GREEN, format!("{}: {:?}", game_text["debug_resource_all_image_name"][self.config.language as usize].clone(), t.image_name));
                                        ui.colored_label(egui::Color32::GREEN, format!("{}: {:?}", game_text["debug_resource_scroll_horizontal"][self.config.language as usize].clone(), t.horizontal_or_vertical));
                                        if t.horizontal_or_vertical {
                                            ui.colored_label(egui::Color32::GREEN, format!("{}: {:?}", game_text["debug_resource_scroll_left"][self.config.language as usize].clone(), t.left_and_top_or_right_and_bottom));
                                        } else {
                                            ui.colored_label(egui::Color32::GREEN, format!("{}: {:?}", game_text["debug_resource_scroll_top"][self.config.language as usize].clone(), t.left_and_top_or_right_and_bottom));
                                        };
                                        ui.colored_label(egui::Color32::GREEN, format!("{}: {:?}", game_text["debug_resource_scroll_speed"][self.config.language as usize].clone(), t.scroll_speed));
                                        ui.colored_label(egui::Color32::GREEN, format!("{}: {:?}", game_text["debug_resource_scroll_boundary"][self.config.language as usize].clone(), t.boundary));
                                        ui.colored_label(egui::Color32::GREEN, format!("{}: {:?}", game_text["debug_resource_scroll_resume_point"][self.config.language as usize].clone(), t.resume_point));
                                        ui.separator();
                                    });
                                self.timer.split_time
                                    .iter()
                                    .rev()
                                    .take(self.timer.split_time.len())
                                    .for_each(|t| {
                                        ui.label(format!("{}: {}", game_text["debug_resource_name"][self.config.language as usize].clone(), t.name));
                                        ui.label(format!("{}: {}", game_text["debug_resource_type"][self.config.language as usize].clone(), t.discern_type));
                                        ui.colored_label(egui::Color32::KHAKI, format!("{}: {:?}", game_text["debug_resource_split_time_single_page"][self.config.language as usize].clone(), t.time[0]));
                                        ui.colored_label(egui::Color32::KHAKI, format!("{}: {:?}", game_text["debug_resource_split_time_total"][self.config.language as usize].clone(), t.time[1]));
                                        ui.separator();
                                    });
                                self.variables
                                    .iter()
                                    .rev()
                                    .take(self.variables.len())
                                    .for_each(|t| {
                                        ui.label(format!("{}: {}", game_text["debug_resource_name"][self.config.language as usize].clone(), t.name));
                                        ui.label(format!("{}: {}", game_text["debug_resource_type"][self.config.language as usize].clone(), t.discern_type));
                                        ui.colored_label(egui::Color32::GOLD, format!("{}: {:?}", game_text["debug_resource_variable_value"][self.config.language as usize].clone(), t.value));
                                        ui.separator();
                                    });
                                self.resource_image_texture
                                    .iter()
                                    .rev()
                                    .take(self.resource_image_texture.len())
                                    .for_each(|t| {
                                        ui.label(format!("{}: {}", game_text["debug_resource_name"][self.config.language as usize].clone(), t.name));
                                        ui.label(format!("{}: {}", game_text["debug_resource_type"][self.config.language as usize].clone(), t.discern_type));
                                        ui.colored_label(egui::Color32::GRAY, format!("{}: {:?}", game_text["debug_resource_image_path"][self.config.language as usize].clone(), t.cite_path));
                                        ui.separator();
                                    });
                                self.resource_switch
                                    .iter()
                                    .rev()
                                    .take(self.resource_switch.len())
                                    .for_each(|t| {
                                        ui.label(format!("{}: {}", game_text["debug_resource_name"][self.config.language as usize].clone(), t.name));
                                        ui.label(format!("{}: {}", game_text["debug_resource_type"][self.config.language as usize].clone(), t.discern_type));
                                        ui.colored_label(egui::Color32::ORANGE, format!("{}: {:?}", game_text["debug_resource_switch_image_name"][self.config.language as usize].clone(), t.switch_image_name));
                                        ui.colored_label(egui::Color32::ORANGE, format!("{}: {:?}", game_text["debug_resource_switch_enable_hover_animation"][self.config.language as usize].clone(), t.enable_hover_click_image[0]));
                                        ui.colored_label(egui::Color32::ORANGE, format!("{}: {:?}", game_text["debug_resource_switch_enable_click_animation"][self.config.language as usize].clone(), t.enable_hover_click_image[1]));
                                        ui.colored_label(egui::Color32::ORANGE, format!("{}: {:?}", game_text["debug_resource_switch_state"][self.config.language as usize].clone(), t.state));
                                        ui.colored_label(egui::Color32::ORANGE, format!("{}: {:?}", game_text["debug_resource_switch_appearance"][self.config.language as usize].clone(), t.appearance));
                                        ui.colored_label(egui::Color32::ORANGE, format!("{}: {:?}", game_text["debug_resource_switch_click_method"][self.config.language as usize].clone(), t.click_method));
                                        ui.colored_label(egui::Color32::ORANGE, format!("{}: {:?}", game_text["debug_resource_switch_click_state"][self.config.language as usize].clone(), t.last_time_clicked));
                                        if t.last_time_clicked {
                                            ui.colored_label(egui::Color32::ORANGE, format!("{}: {:?}", game_text["debug_resource_switch_clicked_method"][self.config.language as usize].clone(), t.last_time_clicked_index));
                                        };
                                        ui.separator();
                                    });
                        });
                    });
                    egui::Window::new("problem_report")
                    .frame(self.frame)
                    .title_bar(false)
                    .open(&mut self.var_b("debug_problem_window"))
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading(game_text["debug_problem_report"][self.config.language as usize].clone());
                        });
                        ui.separator();
                        egui::ScrollArea::vertical()
                        .max_height(ctx.available_rect().height() - 100.0)
                        .max_width(ctx.available_rect().width() - 100.0)
                        .show(ui, |ui| {
                            self.problem_list
                                    .iter()
                                    .rev()
                                    .take(self.problem_list.len())
                                    .for_each(|t| {
                                        ui.colored_label(match t.severity_level {
                                            SeverityLevel::Error => egui::Color32::RED,
                                            SeverityLevel::SevereWarning => egui::Color32::ORANGE,
                                            SeverityLevel::MildWarning => egui::Color32::YELLOW,
                                        }, format!("{}: {}", game_text["debug_problem"][self.config.language as usize].clone(), t.problem));
                                        ui.colored_label(match t.severity_level {
                                            SeverityLevel::Error => egui::Color32::RED,
                                            SeverityLevel::SevereWarning => egui::Color32::ORANGE,
                                            SeverityLevel::MildWarning => egui::Color32::YELLOW,
                                        }, format!("{}: {}", game_text["debug_severity_level"][self.config.language as usize].clone(), match t.severity_level {
                                            SeverityLevel::Error => game_text["debug_severity_level_error"][self.config.language as usize].clone(),
                                            SeverityLevel::SevereWarning => game_text["debug_severity_level_severe_warning"][self.config.language as usize].clone(),
                                            SeverityLevel::MildWarning => game_text["debug_severity_level_mild_warning"][self.config.language as usize].clone(),
                                        }));
                                        ui.colored_label(match t.severity_level {
                                            SeverityLevel::Error => egui::Color32::RED,
                                            SeverityLevel::SevereWarning => egui::Color32::ORANGE,
                                            SeverityLevel::MildWarning => egui::Color32::YELLOW,
                                        }, format!("{}: {}", game_text["debug_annotation"][self.config.language as usize].clone(), t.annotation));
                                        ui.colored_label(match t.severity_level {
                                            SeverityLevel::Error => egui::Color32::RED,
                                            SeverityLevel::SevereWarning => egui::Color32::ORANGE,
                                            SeverityLevel::MildWarning => egui::Color32::YELLOW,
                                        }, format!("{}: {}", game_text["debug_problem_current_page"][self.config.language as usize].clone(), t.report_state.current_page));
                                        ui.colored_label(match t.severity_level {
                                            SeverityLevel::Error => egui::Color32::RED,
                                            SeverityLevel::SevereWarning => egui::Color32::ORANGE,
                                            SeverityLevel::MildWarning => egui::Color32::YELLOW,
                                        }, format!("{}: {}", game_text["debug_problem_current_page_runtime"][self.config.language as usize].clone(), t.report_state.current_page_runtime));
                                        ui.colored_label(match t.severity_level {
                                            SeverityLevel::Error => egui::Color32::RED,
                                            SeverityLevel::SevereWarning => egui::Color32::ORANGE,
                                            SeverityLevel::MildWarning => egui::Color32::YELLOW,
                                        }, format!("{}: {}", game_text["debug_problem_current_total_runtime"][self.config.language as usize].clone(), t.report_state.current_total_runtime));
                                        ui.separator();
                                    });
                        });
                    });
                    ui.horizontal(|ui| {
                        // 使用WidgetText进行复杂布局
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(game_text["debug_mode"][self.config.language as usize].clone())
                                        .color(egui::Color32::YELLOW)
                                        .text_style(egui::TextStyle::Heading)
                                        .background_color(egui::Color32::from_black_alpha(220)),
                                )
                                .wrap(),
                            );
                            ui.separator();
                                ui.vertical(|ui| {
                                    if ui.button(game_text["debug_frame_number_details"][self.config.language as usize].clone()).clicked()
                                    {
                                        general_click_feedback();
                                        let flip = !self.var_b("debug_fps_window");
                                        self.modify_var("debug_fps_window", flip);
                                    };
                                    if ui.button(game_text["debug_resource_list"][self.config.language as usize].clone()).clicked()
                                    {
                                        general_click_feedback();
                                        let flip = !self.var_b("debug_resource_list_window");
                                        self.modify_var("debug_resource_list_window", flip);
                                    };
                                    if ui.button(game_text["debug_problem_report"][self.config.language as usize].clone()).clicked()
                                    {
                                        general_click_feedback();
                                        let flip = !self.var_b("debug_problem_window");
                                        self.modify_var("debug_problem_window", flip);
                                    };
                                });
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::WidgetText::from(game_text["debug_game_version"][self.config.language as usize].clone().to_string())
                                        .color(egui::Color32::GRAY)
                                        .background_color(egui::Color32::from_black_alpha(220)),
                                    );
                                    ui.label(
                                        egui::WidgetText::from(format!("{}: {}", game_text["debug_game_page"][self.config.language as usize].clone(), self.page))
                                            .color(egui::Color32::GRAY)
                                            .background_color(egui::Color32::from_black_alpha(220)),
                                    );
                                    ui.label(
                                        egui::WidgetText::from(format!("{}: {}", game_text["debug_login_user"][self.config.language as usize].clone(), self.config.login_user_name))
                                            .color(egui::Color32::GRAY)
                                            .background_color(egui::Color32::from_black_alpha(220)),
                                    );
                                    ui.label(
                                        egui::WidgetText::from(format!("{}: {:.0}{}", game_text["debug_fps"][self.config.language as usize].clone(), self.current_fps(), game_text["debug_fps2"][self.config.language as usize].clone()))
                                            .color(egui::Color32::GRAY)
                                            .background_color(egui::Color32::from_black_alpha(220)),
                                    );
                                    ui.label(
                                        egui::WidgetText::from(format!("{}: {:.2}{}", game_text["debug_game_now_time"][self.config.language as usize].clone(), self.timer.now_time, game_text["debug_game_second"][self.config.language as usize].clone()))
                                            .color(egui::Color32::GRAY)
                                            .background_color(egui::Color32::from_black_alpha(220)),
                                    );
                                    ui.label(
                                        egui::WidgetText::from(format!("{}: {:.2}{}", game_text["debug_game_total_time"][self.config.language as usize].clone(), self.timer.total_time, game_text["debug_game_second"][self.config.language as usize].clone()))
                                            .color(egui::Color32::GRAY)
                                            .background_color(egui::Color32::from_black_alpha(220)),
                                    );
                                });
                        });
                    });
                };
            });
        let id = self.track_resource(self.resource_page.clone(), &self.page.clone());
        if self.resource_page[id].forced_update {
            // 请求重新绘制界面
            ctx.request_repaint();
        };
    }
}
