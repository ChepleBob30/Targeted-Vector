//! pages.rs is the core part of the page of the Targeted Vector, mainly the page content.
use crate::function;
use eframe::egui;
use function::App;
use function::Value;
use std::process::exit;
use chrono::{Local, Timelike};
use eframe::epaint::Rounding;
use egui::{Frame, Shadow, Stroke};
use crate::function::value_to_bool;

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
        let game_text = self.game_text.clone();
        self.update_timer();
        match &*self.page.clone() {
            "Launch" => {
                if !self.page_status[0].change_page_updated {
                    self.launch_page_preload(ctx);
                    self.new_page_update(0);
                    self.add_var("progress", 0);
                    self.add_split_time("0");
                };
                let mut id = self.track_resource("image", "RC_Logo");
                let mut id2 = self.track_resource("text", "Powered");
                let id3 = self.track_resource("variables", "progress");
                if self.var_i("progress") >= 2 && self.var_i("progress") < 4 {
                    id = self.track_resource("image", "Binder_Logo");
                    id2 = self.track_resource("text", "Organize");
                } else if self.var_i("progress") >= 4 {
                    id = self.track_resource("image", "Mouse");
                    id2 = self.track_resource("text", "Mouse");
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
                                        self.switch_page("Home");
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
            "Home" => {
                let scroll_background = self.track_resource("scroll_background", "ScrollWallpaper");
                if !self.page_status[1].change_page_updated {
                    let id = self.track_resource("variables", "progress");
                    self.variables.remove(id);
                    self.new_page_update(1);
                    self.add_var("title_float_status", true);
                    self.add_var("account_name_str", "".to_string());
                    self.add_var("account_password_str", "".to_string());
                    self.add_var("open_reg_window", false);
                    self.resource_scroll_background[scroll_background].resume_point =
                        ctx.available_rect().width();
                    for i in 0..self.resource_scroll_background[scroll_background]
                        .image_name
                        .len()
                    {
                        let id = self.track_resource(
                            "image",
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
                let window_free = !value_to_bool(self.page_windows_account as i32);
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
                            let id = self.track_resource(
                                "image",
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
                    // 将加载的图片作为参数
                    self.image(ui, "Background", ctx);
                    self.scroll_background(ui, "ScrollWallpaper", ctx);
                    egui::Area::new("".into())
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
                                    .text_color(egui::Color32::from_rgb(20, 20, 20)) // 文字颜色
                                    .hint_text(game_text.game_text["account_name"][self.config.language as usize].clone())
                                    .background_color(egui::Color32::from_rgb(150, 150, 150))
                                    .font(egui::FontId::proportional(16.0)), // 字体大小
                            );
                            ui.add(
                                egui::TextEdit::singleline(&mut input2)
                                    .cursor_at_end(true)
                                    .desired_width(200_f32)
                                    .char_limit(20)
                                    .interactive(window_free)
                                    .text_color(egui::Color32::from_rgb(20, 20, 20)) // 文字颜色
                                    .hint_text(game_text.game_text["account_password"][self.config.language as usize].clone())
                                    .password(true)
                                    .background_color(egui::Color32::from_rgb(150, 150, 150))
                                    .font(egui::FontId::proportional(16.0)), // 字体大小
                            );
                        });
                    self.switch("Power", ui, ctx, window_free);
                    if self.switch("Register", ui, ctx, window_free)[0] == 1 {
                        self.page_windows_account += 1;
                        self.modify_var("open_reg_window", true);
                    }
                    egui::Window::new(game_text.game_text["reg_account"][self.config.language as usize].clone())
                        .open(&mut self.var_b("open_reg_window"))
                        .frame(self.frame)
                        .resizable(false)
                        .title_bar(false)
                        .pivot(egui::Align2::CENTER_CENTER)
                        .scroll(true)
                        .default_size(egui::Vec2::new(600_f32, 300_f32))
                        .default_pos(egui::Pos2::new(
                            ctx.available_rect().width() / 2_f32 - 150_f32,
                            ctx.available_rect().height() / 2_f32 - 50_f32,
                        ))
                        .show(ctx, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.heading(game_text.game_text["welcome"][self.config.language as usize].clone());
                            });
                            ui.label(game_text.game_text["intro"][self.config.language as usize].clone());
                            ui.text_edit_singleline(&mut "");
                            if ui.button(game_text.game_text["cancel"][self.config.language as usize].clone()).clicked() {
                                self.page_windows_account -= 1;
                                self.modify_var("open_reg_window", false);
                            };
                            if ui.button(game_text.game_text["continue"][self.config.language as usize].clone()).clicked() {
                            };
                        });
                    self.image(ui, "Title", ctx);
                    self.modify_var("account_name_str", input1);
                    self.modify_var("account_password_str", input2);
                    let id = self.track_resource("image", "Title");
                    if self.var_b("title_float_status") {
                        if self.resource_image[id].origin_position[1] < 10_f32 {
                            self.resource_image[id].origin_position[1] += 0.1;
                        } else {
                            self.modify_var("title_float_status", false);
                        };
                    } else if self.resource_image[id].origin_position[1] > -10_f32 {
                        self.resource_image[id].origin_position[1] -= 0.1;
                    } else {
                        self.modify_var("title_float_status", true);
                    };
                });
                self.last_window_size =
                    [ctx.available_rect().width(), ctx.available_rect().height()];
            }
            _ => panic!("未找到页面\"{}\"", self.page),
        };
        if self.page_id >= 2 {
            egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    if ui.button("退出").clicked() {
                        exit(0);
                    };
                    if ui.button("首页").clicked() {
                        self.page_status[1].change_page_updated = false;
                        self.switch_page("Home");
                    };
                });
            });
        }
        if self.page_status[self.page_id as usize].forced_update {
            // 请求重新绘制界面
            ctx.request_repaint();
        };
    }
}
