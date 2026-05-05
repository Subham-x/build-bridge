use super::ProjectDashboardApp;
use crate::models::ProjectRecord;
use crate::icons::{icon_button, themed_icon, IconKind};
use eframe::egui::{self, Align, Color32, FontFamily, FontId, Layout, RichText, Vec2};

impl ProjectDashboardApp {
    pub(super) fn render_bridge_status(
        &mut self,
        ui: &mut egui::Ui,
        dark: bool,
        project: &ProjectRecord,
    ) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Bridge Status"));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let toggle_icon = if self.bridge_status_expanded {
                    IconKind::BridgeStatusCollapse
                } else {
                    IconKind::BridgeStatusExpand
                };
                if ui
                    .add(
                        icon_button(themed_icon(dark, toggle_icon), 16.0)
                            .min_size(Vec2::new(24.0, 20.0)),
                    )
                    .clicked()
                {
                    self.bridge_status_expanded = !self.bridge_status_expanded;
                }
            });
        });

        if !self.bridge_status_expanded {
            return;
        }

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let detail_font = FontId::new(13.0, FontFamily::Name("JetBrainsMono".into()));
            let label_color = if dark {
                Color32::WHITE
            } else {
                Color32::from_gray(40)
            };
            let value_color = if dark {
                Color32::from_gray(175)
            } else {
                Color32::from_gray(90)
            };
            ui.add_sized([88.0, 88.0], egui::Label::new(RichText::new("QR").size(54.0).strong()));
            ui.add_space(10.0);
            ui.vertical(|ui| {
                let is_online = project.status == "active";
                let status = if is_online { "Online" } else { "Offline" };
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Status : ")
                            .font(detail_font.clone())
                            .color(label_color),
                    );
                    ui.label(
                        RichText::new(status)
                            .font(detail_font.clone())
                            .color(value_color),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Port : ")
                            .font(detail_font.clone())
                            .color(label_color),
                    );
                    ui.label(
                        RichText::new("127.1.1.0:4000")
                            .font(detail_font.clone())
                            .color(value_color),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Device : ")
                            .font(detail_font.clone())
                            .color(label_color),
                    );
                    ui.label(
                        RichText::new("Web")
                            .font(detail_font.clone())
                            .color(value_color),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Type : ")
                            .font(detail_font.clone())
                            .color(label_color),
                    );
                    ui.label(
                        RichText::new(&self.selected_artifact_type)
                            .font(detail_font.clone())
                            .color(value_color),
                    );
                });
            });
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Edit").clicked() {
                        self.close_project_details();
                        self.begin_edit_project(&project.name);
                    }
                    if ui.button("Restart").clicked() {
                        self.set_status_message(format!("Restart clicked for '{}'.", project.name));
                    }
                    if ui.button("Stop").clicked() {
                        self.set_status_message(format!("Stop clicked for '{}'.", project.name));
                    }
                });
            });
        });
    }
}
