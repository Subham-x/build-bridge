use super::ProjectDashboardApp;
use crate::models::ProjectRecord;
use crate::icons::{icon_button, themed_icon, IconKind};
use eframe::egui::{
    self, Align, Align2, Color32, ColorImage, FontFamily, FontId, Layout, RichText, Vec2,
};
use qrcode::{Color, QrCode};

const QR_SCALE: usize = 4;
const QR_OUTLINE_PX: usize = 10;

impl ProjectDashboardApp {
    fn active_serve_url(&self, project_name: &str) -> Option<String> {
        match self.serve_project.as_deref() {
            Some(active) if active == project_name => self.serve_url.clone(),
            _ => None,
        }
    }

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

            ui.vertical(|ui| {
                let is_online = self.is_bridge_online(&project.name);
                let status_text = if is_online { "Online" } else { "Offline" };
                
                // Status Row
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Status .").font(detail_font.clone()).color(label_color));
                    ui.label(RichText::new(status_text).font(detail_font.clone()).color(value_color));
                });

                // Link Row
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Link   .").font(detail_font.clone()).color(label_color));
                    let link_text = if let Some(url) = self.serve_url.as_ref() {
                        url
                    } else {
                        "8080"
                    };
                    ui.label(RichText::new(link_text).font(detail_font.clone()).color(value_color));
                });

                // Type Row
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Type   .").font(detail_font.clone()).color(label_color));
                    ui.label(RichText::new("Localhost").font(detail_font.clone()).color(value_color));
                });
            });

            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Edit").clicked() {
                        self.close_project_details();
                        self.begin_edit_project(&project.name);
                    }
                    if ui.button("Restart").clicked() {
                        self.stop_bridge_serve();
                        if let Err(err) = self.start_bridge_serve(project) {
                            self.project_action_error = Some(err);
                        }
                    }
                    if ui.button("Stop").clicked() {
                        self.stop_bridge_serve();
                    }
                });
            });
        });
    }
}

fn build_qr_image(url: &str, scale: usize, outline_px: usize) -> ColorImage {
    let Ok(code) = QrCode::new(url.as_bytes()) else {
        return ColorImage::new([1, 1], vec![Color32::BLACK]);
    };
    let width = code.width();
    let colors = code.to_colors();
    let size = width * scale + outline_px * 2;
    let mut pixels = vec![Color32::WHITE; size * size];

    for y in 0..width {
        for x in 0..width {
            let idx = y * width + x;
            let color = if colors[idx] == Color::Dark {
                Color32::BLACK
            } else {
                Color32::WHITE
            };
            for dy in 0..scale {
                for dx in 0..scale {
                    let px = outline_px + x * scale + dx;
                    let py = outline_px + y * scale + dy;
                    pixels[py * size + px] = color;
                }
            }
        }
    }

    ColorImage::new([size, size], pixels)
}
