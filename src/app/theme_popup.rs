use super::ProjectDashboardApp;
use eframe::egui::{self, Align, Vec2};

impl ProjectDashboardApp {
    pub(super) fn render_theme_popup(&mut self, ctx: &egui::Context) {
        if !self.theme_popup_open {
            return;
        }

        let mut open = self.theme_popup_open;
        let mut close_theme_popup = false;
        egui::Window::new("Theme")
            .collapsible(false)
            .resizable(false)
            .default_size(Vec2::new(360.0, 170.0))
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Theme");
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("Close").clicked() {
                            close_theme_popup = true;
                        }
                    });
                });

                ui.add_space(6.0);
                ui.horizontal_centered(|ui| {
                    let system_response = super::framework_card(
                        ui,
                        self.theme_mode == super::AppThemeMode::System,
                        "🖥",
                        "System",
                    );
                    if system_response.clicked() {
                        self.theme_mode = super::AppThemeMode::System;
                    }

                    ui.add_space(6.0);

                    let dark_response = super::framework_card(
                        ui,
                        self.theme_mode == super::AppThemeMode::Dark,
                        "🌙",
                        "Dark",
                    );
                    if dark_response.clicked() {
                        self.theme_mode = super::AppThemeMode::Dark;
                    }

                    ui.add_space(6.0);

                    let light_response = super::framework_card(
                        ui,
                        self.theme_mode == super::AppThemeMode::Light,
                        "☀",
                        "Light",
                    );
                    if light_response.clicked() {
                        self.theme_mode = super::AppThemeMode::Light;
                    }
                });
            });
        if close_theme_popup {
            open = false;
        }
        self.theme_popup_open = open;
    }
}
