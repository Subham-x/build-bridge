use super::{nav_item, support_page_row, ProjectDashboardApp};
use crate::icons::{icon_button, icon_image, themed_icon, IconKind};
use eframe::egui::{self, Align, Button, Color32, Layout};

impl ProjectDashboardApp {
    pub(super) fn render_sidebar(&mut self, ctx: &egui::Context, dark: bool) {
        let target_width = if self.sidebar_visible {
            self.sidebar_width
        } else {
            0.0
        };
        let delta = target_width - self.sidebar_animated_width;
        if delta.abs() > 0.5 {
            self.sidebar_animated_width += delta * 0.22;
            ctx.request_repaint();
        } else {
            self.sidebar_animated_width = target_width;
        }

        if self.sidebar_animated_width > 1.0 {
            let fully_open =
                self.sidebar_visible && (self.sidebar_animated_width - self.sidebar_width).abs() < 1.0;
            let panel = if fully_open {
                egui::SidePanel::left("sidebar")
                    .resizable(true)
                    .default_width(self.sidebar_width)
                    .min_width(200.0)
                    .max_width(360.0)
            } else {
                egui::SidePanel::left("sidebar")
                    .resizable(false)
                    .exact_width(self.sidebar_animated_width)
            };

            panel.show(ctx, |ui| {
                if fully_open {
                    let current_width = ui.available_width().clamp(200.0, 360.0);
                    let panel_rect = ui.max_rect();
                    let drag_margin = 6.0;
                    let dragging_width = ctx.input(|i| {
                        if let Some(pos) = i.pointer.interact_pos() {
                            let near_edge = (panel_rect.right() - pos.x).abs() <= drag_margin;
                            near_edge && i.pointer.primary_down() && i.pointer.delta().x.abs() > 0.0
                        } else {
                            false
                        }
                    });
                    if dragging_width || current_width < self.sidebar_width {
                        self.sidebar_width = current_width;
                        self.sidebar_animated_width = self.sidebar_width;
                    }
                }

                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    if ui
                        .add(icon_button(themed_icon(dark, IconKind::PanelHide), 26.0))
                        .on_hover_text("Hide sidebar")
                        .clicked()
                    {
                        self.sidebar_visible = false;
                    }
                });
                ui.add_space(8.0);

                if self.sidebar_animated_width >= 120.0 {
                    ui.label("General");
                    let home_response = nav_item(
                        ui,
                        dark,
                        &mut self.nav,
                        super::Nav::Home,
                        "Projects",
                        IconKind::Briefcase,
                    );
                    if home_response.clicked() && self.selected_project_name.is_some() {
                        self.close_project_details();
                    }
                    let archived_response = nav_item(
                        ui,
                        dark,
                        &mut self.nav,
                        super::Nav::Archived,
                        "Archived",
                        IconKind::Archive,
                    );
                    if archived_response.clicked() && self.selected_project_name.is_some() {
                        self.close_project_details();
                    }
                    let bin_response = nav_item(
                        ui,
                        dark,
                        &mut self.nav,
                        super::Nav::Bin,
                        "Bin",
                        IconKind::Trash,
                    );
                    if bin_response.clicked() && self.selected_project_name.is_some() {
                        self.close_project_details();
                    }
                    ui.horizontal(|ui| {
                        ui.add(icon_image(themed_icon(dark, IconKind::Palette), 18.0));
                        if ui
                            .add(Button::new("Theme").frame(false).fill(Color32::TRANSPARENT))
                            .clicked()
                        {
                            self.theme_popup_open = true;
                        }
                    });

                    ui.add_space(8.0);
                    ui.label("Support");
                    if support_page_row(ui, dark, IconKind::About, "About").clicked() {
                        self.nav = super::Nav::About;
                    }
                    if support_page_row(ui, dark, IconKind::Feedback, "Feedback").clicked() {
                        self.nav = super::Nav::Feedback;
                    }
                    if support_page_row(ui, dark, IconKind::Privacy, "Privacy Policy").clicked() {
                        self.nav = super::Nav::PrivacyPolicy;
                    }

                    if self.app_config.debug_page {
                        if support_page_row(ui, dark, IconKind::Bug, "Debug").clicked() {
                            self.nav = super::Nav::Debug;
                        }
                    }

                    if self.selected_project_name.is_some() {
                        self.render_terminal_panel(ui, dark);
                    }
                }
            });
        }
    }
}
