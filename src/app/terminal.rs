use super::{is_terminal_link, ProjectDashboardApp};
use eframe::egui::{
    self, Color32, FontFamily, FontId, Frame, Margin, RichText, ScrollArea, Stroke,
};

impl ProjectDashboardApp {
    pub(super) fn render_terminal_panel(&mut self, ui: &mut egui::Ui, dark: bool) {
        ui.add_space(8.0);
        let title_bg = if dark {
            Color32::from_gray(52)
        } else {
            Color32::from_gray(224)
        };
        let title_text = if dark {
            Color32::from_gray(175)
        } else {
            Color32::from_gray(80)
        };
        Frame::new()
            .fill(title_bg)
            .inner_margin(Margin::same(6))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.colored_label(title_text, "Terminal");
            });
        Frame::new()
            .fill(Color32::BLACK)
            .inner_margin(Margin::same(8))
            .show(ui, |ui| {
                let terminal_font = FontId::new(12.5, FontFamily::Name("JetBrainsMono".into()));
                ui.set_min_height((ui.available_height() - 4.0).max(120.0));
                if let Some(project_name) = self.selected_project_name.clone() {
                    let serve_line = format!("PS > serve \"{project_name}\" --mode bridge");
                    ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        self.render_terminal_line(ui, &serve_line, &terminal_font);
                        self.render_terminal_line(ui, "Bridge status: connected", &terminal_font);
                        self.render_terminal_line(
                            ui,
                            "Listening on http://127.1.1.0:4000",
                            &terminal_font,
                        );
                    });
                }
            });
    }

    pub(super) fn render_terminal_line(&mut self, ui: &mut egui::Ui, line: &str, font: &FontId) {
        ui.horizontal_wrapped(|ui| {
            let mut first = true;
            for token in line.split_whitespace() {
                if !first {
                    ui.label(RichText::new(" ").font(font.clone()).color(Color32::WHITE));
                }
                first = false;

                if is_terminal_link(token) {
                    let response = ui.add(
                        egui::Label::new(
                            RichText::new(token)
                                .font(font.clone())
                                .color(Color32::from_rgb(66, 133, 244)),
                        )
                        .sense(egui::Sense::click()),
                    );
                    if response.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        ui.painter().hline(
                            response.rect.x_range(),
                            response.rect.bottom() - 1.0,
                            Stroke::new(1.0, Color32::from_rgb(66, 133, 244)),
                        );
                    }
                    if response.clicked() {
                        let open_directly = ui.input(|i| i.modifiers.ctrl);
                        if open_directly {
                            ui.ctx().open_url(egui::OpenUrl {
                                url: token.to_owned(),
                                new_tab: true,
                            });
                        } else {
                            self.terminal_link_target = Some(token.to_owned());
                            self.terminal_link_popup_open = true;
                        }
                    }

                    response.context_menu(|ui| {
                        // ui.label("Actions");
                        if ui.button("Copy (Ctrl + Click)").clicked() {
                            ui.ctx().copy_text(token.to_owned());
                            ui.close();
                        }
                        if ui.button("Open").clicked() {
                            ui.ctx().open_url(egui::OpenUrl {
                                url: token.to_owned(),
                                new_tab: true,
                            });
                            ui.close();
                        }
                    });
                } else {
                    ui.label(RichText::new(token).font(font.clone()).color(Color32::WHITE));
                }
            }
        });
    }
}
