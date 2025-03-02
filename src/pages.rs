//! pages.rs is the core part of the page of the Targeted Vector, mainly the page content.
use crate::function;
use eframe::egui;
use function::App;
use function::Page;
use std::process::exit;

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_timer();
        match self.page {
            Page::Launch => {
                if !self.page_status[0].change_page_updated {
                    self.launch_page_preload(ctx);
                    self.new_page_update(0);
                    self.variables.progress = 0;
                    self.add_split_time("0");
                };
                let mut id = self.track_resource("image", "RC_Logo");
                let mut id2 = self.track_resource("text", "Powered");
                if self.variables.progress >= 2 && self.variables.progress < 4 {
                    id = self.track_resource("image", "Binder_Logo");
                    id2 = self.track_resource("text", "Organize");
                } else if self.variables.progress >= 4 {
                    id = self.track_resource("image", "Mouse");
                    id2 = self.track_resource("text", "Mouse");
                };
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.rect(ui, "Background", ctx);
                    if self.timer.now_time >= 1.0 {
                        if self.variables.progress < 2 {
                            self.image(ctx, "RC_Logo", ui);
                            self.text(ui, "Powered", ctx);
                        } else if self.variables.progress < 4 {
                            self.image(ctx, "Binder_Logo", ui);
                            self.text(ui, "Organize", ctx);
                        } else {
                            self.image(ctx, "Mouse", ui);
                            self.text(ui, "Mouse", ctx);
                        };
                        for _ in 0..10 {
                            match self.variables.progress {
                                0 => {
                                    if self.resource_image[id].alpha == 255
                                        && self.resource_text[id2].rgba[3] == 255
                                        && (self.timer.now_time - self.split_time("0")[0]) >= 6.0
                                    {
                                        self.variables.progress = 1;
                                        self.add_split_time("1");
                                    };
                                }
                                1 => {
                                    if self.resource_image[id].alpha == 0
                                        && self.resource_text[id2].rgba[3] == 0
                                        && (self.timer.now_time - self.split_time("1")[0]) >= 3.0
                                    {
                                        self.variables.progress = 2;
                                        self.add_split_time("2");
                                    };
                                }
                                2 => {
                                    if self.resource_image[id].alpha == 255
                                        && self.resource_text[id2].rgba[3] == 255
                                        && (self.timer.now_time - self.split_time("2")[0]) >= 3.0
                                    {
                                        self.variables.progress = 3;
                                        self.add_split_time("3");
                                    };
                                }
                                3 => {
                                    if self.resource_image[id].alpha == 0
                                        && self.resource_text[id2].rgba[3] == 0
                                        && (self.timer.now_time - self.split_time("3")[0]) >= 3.0
                                    {
                                        self.variables.progress = 4;
                                        self.add_split_time("4");
                                    };
                                }
                                4 => {
                                    if self.resource_image[id].alpha == 255
                                        && self.resource_text[id2].rgba[3] == 255
                                        && (self.timer.now_time - self.split_time("4")[0]) >= 3.0
                                    {
                                        self.variables.progress = 5;
                                        self.add_split_time("5");
                                    };
                                }
                                5 => {
                                    if self.resource_image[id].alpha == 0
                                        && self.resource_text[id2].rgba[3] == 0
                                        && (self.timer.now_time - self.split_time("5")[0]) >= 3.0
                                    {
                                        self.page = Page::Home;
                                    };
                                }
                                _ => {}
                            };
                            if self.variables.progress != 0
                                && self.variables.progress != 2
                                && self.variables.progress != 4
                                && self.resource_image[id].alpha != 0
                            {
                                self.resource_image[id].alpha -= 1;
                            };
                            if self.variables.progress != 0
                                && self.variables.progress != 2
                                && self.variables.progress != 4
                                && self.resource_text[id2].rgba[3] != 0
                            {
                                self.resource_text[id2].rgba[3] -= 1;
                            };
                            if self.variables.progress != 1
                                && self.variables.progress != 3
                                && self.variables.progress != 5
                                && self.resource_image[id].alpha != 255
                            {
                                self.resource_image[id].alpha += 1;
                            };
                            if self.variables.progress != 1
                                && self.variables.progress != 3
                                && self.variables.progress != 5
                                && self.resource_text[id2].rgba[3] != 255
                            {
                                self.resource_text[id2].rgba[3] += 1;
                            };
                        }
                    };
                });
            }
            Page::Home => {
                let scroll_background = self.track_resource("scroll_background", "ScrollWallpaper");
                if !self.page_status[1].change_page_updated {
                    self.new_page_update(1);
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
                    self.image(ctx, "Background", ui);
                    self.scroll_background(ui, "ScrollWallpaper", ctx);
                    self.text(ui, "Title", ctx);
                });
                self.last_window_size =
                    [ctx.available_rect().width(), ctx.available_rect().height()];
            }
        };
        if self.page_id != 0 {
            egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    if ui.button("退出").clicked() {
                        exit(0);
                    };
                    if ui.button("首页").clicked() {
                        self.page_status[1].change_page_updated = false;
                        self.page = Page::Home;
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
