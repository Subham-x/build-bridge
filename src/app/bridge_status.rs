use super::ProjectDashboardApp;
use crate::models::ProjectRecord;
use crate::icons::{icon_button, themed_icon, IconKind};
use eframe::egui::{
    self, Align, Color32, ColorImage, FontFamily, FontId, Layout, RichText, Vec2,
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
            ui.label(RichText::new("Bridge Status").strong());
            if let Some(version) = &self.serve_version {
                ui.add_space(8.0);
                ui.label(RichText::new(format!("Build Stream Version : {}", version))
                    .size(11.0)
                    .color(ui.style().visuals.weak_text_color()));
            }
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

        if let Some(url) = self.active_serve_url(&project.name) {
            let needs_refresh = self.bridge_qr_texture.is_none()
                || self.bridge_qr_url.as_deref() != Some(url.as_str());
            if needs_refresh {
                let image = build_qr_image(&url, QR_SCALE, QR_OUTLINE_PX);
                self.bridge_qr_texture = Some(ui.ctx().load_texture(
                    "bridge_qr",
                    image,
                    egui::TextureOptions::LINEAR,
                ));
                self.bridge_qr_url = Some(url);
            }
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

            // QR Code display on the left
            if let Some(texture) = &self.bridge_qr_texture {
                ui.image((texture.id(), Vec2::new(96.0, 96.0)));
                ui.add_space(16.0);
            }

            ui.vertical(|ui| {
                let is_online = self.is_bridge_online(&project.name);
                let status_text = if is_online { "Online" } else { "Offline" };
                
                // Status Row
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Status .").font(detail_font.clone()).color(label_color));
                    let color = if is_online { Color32::GREEN } else { Color32::from_rgb(255, 0, 79) };
                    ui.label(RichText::new(status_text).font(detail_font.clone()).color(color));
                });

                // Link Row
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Link   .").font(detail_font.clone()).color(label_color));
                    if let Some(url) = self.serve_url.as_ref() {
                        ui.scope(|ui| {
                            ui.visuals_mut().button_frame = false;
                            
                            let hover_id = ui.id().with("link_hover");
                            let is_hovered: bool = ui.data_mut(|d| d.get_temp(hover_id).unwrap_or(false));
                            
                            let mut btn_text = RichText::new(url).font(detail_font.clone());
                            if is_hovered {
                                btn_text = btn_text.color(Color32::LIGHT_BLUE).underline();
                            } else {
                                btn_text = btn_text.color(value_color);
                            }

                            let response = ui.menu_button(btn_text, |ui| {
                                if ui.button("Open").clicked() {
                                    ui.ctx().open_url(egui::OpenUrl::new_tab(url));
                                    ui.close();
                                }
                                if ui.button("Copy").clicked() {
                                    ui.ctx().copy_text(url.clone());
                                    ui.close();
                                }
                            }).response;
                            
                            ui.data_mut(|d| d.insert_temp(hover_id, response.hovered()));
                        });
                    } else {
                        ui.label(RichText::new("---").font(detail_font.clone()).color(value_color));
                    }
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
                    
                    let is_online = self.is_bridge_online(&project.name);
                    if is_online {
                        if ui.button("Restart").clicked() {
                            let _ = self.start_bridge_serve(project);
                        }
                        
                        let stop_btn = egui::Button::new(RichText::new("Stop").color(Color32::WHITE))
                            .fill(Color32::from_rgb(220, 68, 55));
                        if ui.add(stop_btn).clicked() {
                            self.stop_bridge_serve();
                        }
                    } else {
                        if ui.button("Start").clicked()
                            && let Err(err) = self.start_bridge_serve(project) {
                                self.project_action_error = Some(err);
                        }
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
