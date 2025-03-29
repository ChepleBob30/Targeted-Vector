//! pages.rs is the core part of the page of the Targeted Vector, mainly the page content.
use crate::function::{check_file_exists, create_pretty_json, read_from_json, track_resource, value_to_bool, App, User, Value};
use chrono::{Local, Timelike};
use eframe::egui;
use eframe::epaint::Rounding;
use egui::{Frame, Shadow, Stroke};
use json::object;
use std::process::exit;

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if Local::now().hour() >= 18 {
            ctx.set_visuals(egui::Visuals::dark());
            self.frame = Frame {
                inner_margin: egui::Margin::same(10.0),
                outer_margin: egui::Margin::same(0.0),
                rounding: Rounding::same(10.0),
                shadow: Shadow {
                    offset: egui::Vec2::new(1.0, 2.0),
                    color: egui::Color32::from_rgb(0, 0, 0),
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
                    color: egui::Color32::from_rgb(0, 0, 0),
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
                if !self.resource_page[track_resource(self.resource_page.clone(), "Launch", "page")].change_page_updated {
                    self.launch_page_preload(ctx);
                    self.new_page_update(track_resource(self.resource_page.clone(), "Launch", "page") as i32);
                    self.add_var("progress", 0);
                    self.add_split_time("0");
                };
                let mut id = track_resource(self.resource_image.clone(), "RC_Logo", "image");
                let mut id2 = track_resource(self.resource_text.clone(), "Powered", "text");
                let id3 = track_resource(self.variables.clone(), "progress", "variables");
                if self.var_i("progress") >= 2 && self.var_i("progress") < 4 {
                    id = track_resource(self.resource_image.clone(), "Binder_Logo", "image");
                    id2 = track_resource(self.resource_text.clone(), "Organize", "text");
                } else if self.var_i("progress") >= 4 {
                    id = track_resource(self.resource_image.clone(), "Mouse", "image");
                    id2 = track_resource(self.resource_text.clone(), "Mouse", "text");
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
                                        self.add_split_time("1");
                                    };
                                }
                                1 => {
                                    if self.resource_image[id].alpha == 0
                                        && self.resource_text[id2].rgba[3] == 0
                                        && (self.timer.now_time - self.split_time("1")[0]) >= 1.0
                                    {
                                        self.variables[id3].value = Value::Int(2);
                                        self.add_split_time("2");
                                    };
                                }
                                2 => {
                                    if self.resource_image[id].alpha == 255
                                        && self.resource_text[id2].rgba[3] == 255
                                        && (self.timer.now_time - self.split_time("2")[0]) >= 2.0
                                    {
                                        self.variables[id3].value = Value::Int(3);
                                        self.add_split_time("3");
                                    };
                                }
                                3 => {
                                    if self.resource_image[id].alpha == 0
                                        && self.resource_text[id2].rgba[3] == 0
                                        && (self.timer.now_time - self.split_time("3")[0]) >= 1.0
                                    {
                                        self.variables[id3].value = Value::Int(4);
                                        self.add_split_time("4");
                                    };
                                }
                                4 => {
                                    if self.resource_image[id].alpha == 255
                                        && self.resource_text[id2].rgba[3] == 255
                                        && (self.timer.now_time - self.split_time("4")[0]) >= 2.0
                                    {
                                        self.variables[id3].value = Value::Int(5);
                                        self.add_split_time("5");
                                    };
                                }
                                5 => {
                                    if self.resource_image[id].alpha == 0
                                        && self.resource_text[id2].rgba[3] == 0
                                        && (self.timer.now_time - self.split_time("5")[0]) >= 1.0
                                    {
                                        self.switch_page("Login");
                                    };
                                }
                                _ => {}
                            };
                            if self.var_i("progress") != 0
                                && self.var_i("progress") != 2
                                && self.var_i("progress") != 4
                                && self.resource_image[id].alpha != 0
                            {
                                self.resource_image[id].alpha -= 1;
                            };
                            if self.var_i("progress") != 0
                                && self.var_i("progress") != 2
                                && self.var_i("progress") != 4
                                && self.resource_text[id2].rgba[3] != 0
                            {
                                self.resource_text[id2].rgba[3] -= 1;
                            };
                            if self.var_i("progress") != 1
                                && self.var_i("progress") != 3
                                && self.var_i("progress") != 5
                                && self.resource_image[id].alpha != 255
                            {
                                self.resource_image[id].alpha += 1;
                            };
                            if self.var_i("progress") != 1
                                && self.var_i("progress") != 3
                                && self.var_i("progress") != 5
                                && self.resource_text[id2].rgba[3] != 255
                            {
                                self.resource_text[id2].rgba[3] += 1;
                            };
                        }
                    };
                });
            }
            "Login" => {
                let scroll_background = track_resource(
                    self.resource_scroll_background.clone(),
                    "ScrollWallpaper",
                    "scroll_background",
                );
                if !self.resource_page[track_resource(self.resource_page.clone(), "Login", "page")].change_page_updated {
                    self.variables.remove(track_resource(
                        self.variables.clone(),
                        "progress",
                        "variables",
                    ));
                    self.new_page_update(track_resource(self.resource_page.clone(), "Login", "page") as i32);
                    self.add_var("title_float_status", true);
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
                    self.resource_scroll_background[scroll_background].resume_point =
                        ctx.available_rect().width();
                    for i in 0..self.resource_scroll_background[scroll_background]
                        .image_name
                        .len()
                    {
                        let id = track_resource(
                            self.resource_image.clone(),
                            &self.resource_scroll_background[scroll_background].image_name[i]
                                .clone(),
                            "image",
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
                let window_free = !value_to_bool(self.page_windows_amount as i32);
                let mut input3 = self.var_s("reg_account_name_str");
                let mut input4 = self.var_s("reg_account_password_str");
                let mut input5 = self.var_s("reg_account_check_password_str");
                egui::CentralPanel::default().show(ctx, |ui| {
                    if self.last_window_size[0] != ctx.available_rect().width()
                        || self.last_window_size[1] != ctx.available_rect().height()
                    {
                        self.resource_scroll_background[scroll_background].resume_point =
                            ctx.available_rect().width();
                        for i in 0..self.resource_scroll_background[scroll_background]
                            .image_name
                            .len()
                        {
                            let id = track_resource(
                                self.resource_image.clone(),
                                &self.resource_scroll_background[scroll_background].image_name[i]
                                    .clone(),
                                "image",
                            );
                            self.resource_image[id].image_size =
                                [ctx.available_rect().width(), ctx.available_rect().height()];
                            self.resource_image[id].image_position[0] =
                                i as f32 * self.resource_image[id].image_size[0];
                            self.resource_scroll_background[scroll_background].boundary =
                                -ctx.available_rect().width();
                        }
                    };
                    // 将加载的图片作为参数
                    self.image(ui, "Background", ctx);
                    self.scroll_background(ui, "ScrollWallpaper", ctx);
                    egui::Area::new("Login".into())
                        .fixed_pos(egui::Pos2::new(
                            ctx.available_rect().width() / 2_f32 - 100_f32,
                            ctx.available_rect().height() / 8_f32 * 6_f32,
                        ))
                        .show(ui.ctx(), |ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut input1)
                                    .cursor_at_end(true)
                                    .desired_width(200_f32)
                                    .char_limit(20)
                                    .interactive(window_free)
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
                                    .interactive(window_free)
                                    .hint_text(game_text["account_password"][self.config.language as usize].clone())
                                    .password(true)
                                    .font(egui::FontId::proportional(16.0)), // 字体大小
                            );
                            if self.var_b("login_enable_password_error_message") {
                                ui.colored_label(egui::Color32::RED, game_text["login_password_error"][self.config.language as usize].clone());
                            };
                        });
                    if self.switch("Power", ui, ctx, window_free)[0] == 0 {
                        exit(0);
                    };
                    if self.switch("Login", ui, ctx, window_free)[0] == 0 {
                        if check_file_exists(format!("Resources/config/user_{}.json", input1.replace(" ", "").replace("/", "").replace("\\", ""))) {
                            let mut user = User {
                                name: "".to_string(),
                                password: "".to_string(),
                            };
                            if let Ok(json_value) = read_from_json(format!("Resources/config/user_{}.json", input1.replace(" ", "").replace("/", "").replace("\\", ""))) {
                                if let Some(read_user) = User::from_json_value(&json_value) {
                                    user = read_user;
                                }
                            };
                            if user.password == input2 {
                                self.login_user_name = user.name;
                                self.modify_var("login_enable_password_error_message", false);
                                self.switch_page("Home_Page");
                            };
                            self.modify_var("login_enable_password_error_message", user.password != input2);
                        };
                        self.modify_var("login_enable_name_error_message", !check_file_exists(format!("Resources/config/user_{}.json", input1.replace(" ", "").replace("/", "").replace("\\", ""))));
                    };
                    if self.switch("Register", ui, ctx, window_free)[0] == 0 {
                        self.page_windows_amount += 1;
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
                                            self.page_windows_amount -= 1;
                                            self.modify_var("open_reg_window", false);
                                        };
                                        if ui.button(game_text["continue"][self.config.language as usize].clone()).clicked() {
                                            self.modify_var("reg_enable_name_error_message", false);
                                            self.modify_var("reg_enable_password_error_message", false);
                                            self.modify_var("reg_status", Value::UInt(1));
                                        };
                                    } else if self.var_u("reg_status") == 1 {
                                        if ui.button(game_text["cancel"][self.config.language as usize].clone()).clicked() {
                                            self.modify_var("reg_status", Value::UInt(0));
                                        };
                                        if ui.button(game_text["continue"][self.config.language as usize].clone()).clicked() {
                                            self.modify_var("reg_enable_password_error_message", input4 != input5);
                                            self.modify_var("reg_enable_name_error_message", input3.replace(" ", "").replace("/", "").replace("\\", "").is_empty() || check_file_exists(format!("Resources/config/user_{}.json", input3.replace(" ", "").replace("/", "")).replace("\\", "")));
                                            if input4 == input5 {
                                                if !check_file_exists(format!("Resources/config/user_{}.json", input3.replace(" ", "").replace("/", "")).replace("\\", "")) && !input3.replace(" ", "").replace("/", "").replace("\\", "").is_empty() {
                                                    let user_data = object! {
                                                        "name": input3.replace(" ", "").replace("/", "").replace("\\", "").clone(),
                                                        "password": input4.clone()
                                                    };
                                                    let _ = create_pretty_json(format!("Resources/config/user_{}.json", input3.replace(" ", "").replace("/", "").replace("\\", "")), user_data);
                                                    self.modify_var("reg_status", Value::UInt(2));
                                                };
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
                                            self.modify_var("reg_status", Value::UInt(0));
                                        };
                                        if ui.button(game_text["reg_complete"][self.config.language as usize].clone()).clicked() {
                                            self.page_windows_amount -= 1;
                                            self.modify_var("open_reg_window", false);
                                        };
                                    };
                            });
                        });
                    self.image(ui, "Title", ctx);
                    self.modify_var("account_name_str", input1);
                    self.modify_var("account_password_str", input2);
                    self.modify_var("reg_account_name_str", input3);
                    self.modify_var("reg_account_password_str", input4);
                    self.modify_var("reg_account_check_password_str", input5);
                    let id = track_resource(self.resource_image.clone(), "Title", "image");
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
                });
                self.last_window_size =
                    [ctx.available_rect().width(), ctx.available_rect().height()];
            },
            "Home_Page" => {
                if !self.resource_page[track_resource(self.resource_page.clone(), "Home_Page", "page")].change_page_updated {
                    self.variables.remove(track_resource(
                        self.variables.clone(),
                        "title_float_status",
                        "variables",
                    ));
                    self.variables.remove(track_resource(
                        self.variables.clone(),
                        "account_name_str",
                        "variables",
                    ));
                    self.variables.remove(track_resource(
                        self.variables.clone(),
                        "account_password_str",
                        "variables",
                    ));
                    self.variables.remove(track_resource(
                        self.variables.clone(),
                        "open_reg_window",
                        "variables",
                    ));
                    self.variables.remove(track_resource(
                        self.variables.clone(),
                        "reg_status",
                        "variables",
                    ));
                    self.variables.remove(track_resource(
                        self.variables.clone(),
                        "reg_account_name_str",
                        "variables",
                    ));
                    self.variables.remove(track_resource(
                        self.variables.clone(),
                        "reg_account_password_str",
                        "variables",
                    ));
                    self.variables.remove(track_resource(
                        self.variables.clone(),
                        "reg_account_description_str",
                        "variables",
                    ));
                    self.variables.remove(track_resource(
                        self.variables.clone(),
                        "reg_account_check_password_str",
                        "variables",
                    ));
                    self.variables.remove(track_resource(
                        self.variables.clone(),
                        "reg_enable_password_error_message",
                        "variables",
                    ));
                    self.variables.remove(track_resource(
                        self.variables.clone(),
                        "reg_enable_name_error_message",
                        "variables",
                    ));
                    self.variables.remove(track_resource(
                        self.variables.clone(),
                        "login_enable_name_error_message",
                        "variables",
                    ));
                    self.variables.remove(track_resource(
                        self.variables.clone(),
                        "login_enable_password_error_message",
                        "variables",
                    ));
                    self.new_page_update(track_resource(self.resource_page.clone(), "Home_Page", "page") as i32);
                };
            },
            _ => panic!(
                "RustConstructor Error[Page load failed]: Page not found: \"{}\"",
                self.page
            ),
        };
        let check_is_home = self.page.find("Home_");
        if let Some(_) = check_is_home {
            egui::CentralPanel::default().show(ctx, |ui| {
                if self.switch("Home", ui, ctx, true)[0] == 0 {
                    exit(0);
                };
                self.switch("Settings", ui, ctx, true);
            });
        };
        if self.resource_page[self.page_id as usize].forced_update {
            // 请求重新绘制界面
            ctx.request_repaint();
        };
    }
}
