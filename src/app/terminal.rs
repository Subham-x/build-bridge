use super::ProjectDashboardApp;
use crate::icons::{icon_button, themed_icon, IconKind};
use eframe::egui::{
    self, Color32, FontFamily, FontId, Frame, Margin, RichText, ScrollArea,
};

#[derive(Clone, Debug)]
struct TerminalToken {
    text: String,
    color: Color32,
    bold: bool,
    is_link: bool,
}

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
                ui.horizontal(|ui| {
                    ui.colored_label(title_text, "Terminal");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let open_icon = themed_icon(dark, IconKind::OpenIn);
                        let btn = icon_button(open_icon, 14.0).frame(false);
                        if ui.add(btn).on_hover_text("Open in System Terminal").clicked() {
                            if let Some(project_name) = self.serve_project.clone() {
                                if let Some(project) = self.projects.iter().find(|p| p.name == project_name) {
                                    let _ = self.open_standalone_terminal(project);
                                }
                            } else {
                                let _ = self.open_system_terminal();
                            }
                        }
                    });
                });
            });

        let bg_color = Color32::from_gray(12); // Slightly lighter than pure black for a "perfect" look
        Frame::new()
            .fill(bg_color)
            .inner_margin(Margin::same(12))
            .corner_radius(4.0)
            .show(ui, |ui| {
                let terminal_font = FontId::new(12.5, FontFamily::Name("JetBrainsMono".into()));
                ui.set_min_height((ui.available_height() - 4.0).max(120.0));
                
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        if self.terminal_lines.is_empty() {
                            self.render_terminal_line(
                                ui,
                                "PS > bridge idle",
                                &terminal_font,
                            );
                            return;
                        }
                        
                        for i in 0..self.terminal_lines.len() {
                            let line = self.terminal_lines[i].clone();
                            self.render_terminal_line(ui, &line, &terminal_font);
                        }
                    });
            });
    }

    fn parse_ansi_line(&self, line: &str) -> Vec<TerminalToken> {
        let mut tokens = Vec::new();
        let mut current_text = String::new();
        let mut current_color = Color32::from_gray(210); // Default White/Silver
        let mut is_bold = false;

        let mut i = 0;
        let bytes = line.as_bytes();
        
        while i < bytes.len() {
            if bytes[i] == 0x1B && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
                if !current_text.is_empty() {
                    self.push_tokens(&mut tokens, &current_text, current_color, is_bold);
                    current_text.clear();
                }

                i += 2;
                let start = i;
                while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b';') {
                    i += 1;
                }

                if i < bytes.len() && bytes[i] == b'm' {
                    let code_str = String::from_utf8_lossy(&bytes[start..i]);
                    for part in code_str.split(';') {
                        match part {
                            "0" => {
                                current_color = Color32::from_gray(210);
                                is_bold = false;
                            }
                            "1" => is_bold = true,
                            "31" | "91" => current_color = Color32::from_rgb(255, 85, 85), // Red (Error)
                            "32" | "92" => current_color = Color32::from_rgb(80, 250, 123), // Green (Success)
                            "33" | "93" => current_color = Color32::from_rgb(241, 250, 140), // Yellow (Warning)
                            "34" | "94" => current_color = Color32::from_rgb(139, 233, 253), // Cyan/Blue
                            "90" => current_color = Color32::from_gray(120), // Dark Grey
                            _ => {}
                        }
                    }
                    i += 1;
                } else {
                    current_text.push_str("\x1B[");
                }
            } else {
                current_text.push(bytes[i] as char);
                i += 1;
            }
        }

        if !current_text.is_empty() {
            self.push_tokens(&mut tokens, &current_text, current_color, is_bold);
        }

        tokens
    }

    fn push_tokens(&self, tokens: &mut Vec<TerminalToken>, text: &str, color: Color32, bold: bool) {
        let mut current_word = String::new();
        
        for c in text.chars() {
            if c.is_whitespace() {
                if !current_word.is_empty() {
                    let is_link = super::is_terminal_link(&current_word);
                    tokens.push(TerminalToken {
                        text: current_word.clone(),
                        color: if is_link { Color32::from_rgb(66, 133, 244) } else { color },
                        bold: bold || is_link,
                        is_link,
                    });
                    current_word.clear();
                }
                tokens.push(TerminalToken {
                    text: c.to_string(),
                    color,
                    bold,
                    is_link: false,
                });
            } else {
                current_word.push(c);
            }
        }

        if !current_word.is_empty() {
            let is_link = super::is_terminal_link(&current_word);
            tokens.push(TerminalToken {
                text: current_word,
                color: if is_link { Color32::from_rgb(66, 133, 244) } else { color },
                bold: bold || is_link,
                is_link,
            });
        }
    }

    pub(super) fn render_terminal_line(&mut self, ui: &mut egui::Ui, line: &str, font: &FontId) {
        let tokens = self.parse_ansi_line(line);
        
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            
            for token in tokens {
                let mut text = RichText::new(&token.text)
                    .font(font.clone())
                    .color(token.color);
                
                if token.bold {
                    text = text.strong();
                }

                if token.is_link {
                    let response = ui.add(egui::Link::new(text));
                    
                    if response.clicked() {
                        let open_directly = ui.input(|i| i.modifiers.ctrl);
                        if open_directly {
                            ui.ctx().open_url(egui::OpenUrl {
                                url: token.text.to_owned(),
                                new_tab: true,
                            });
                        } else {
                            self.terminal_link_target = Some(token.text.to_owned());
                            self.terminal_link_popup_open = true;
                        }
                    }

                    response.context_menu(|ui| {
                        ui.label("Link Action");
                        if ui.button("Copy Link").clicked() {
                            ui.ctx().copy_text(token.text.to_owned());
                            ui.close();
                        }
                        if ui.button("Open (Ctrl + Click)").clicked() {
                            ui.ctx().open_url(egui::OpenUrl {
                                url: token.text.to_owned(),
                                new_tab: true,
                            });
                            ui.close();
                        }
                    });
                } else {
                    ui.add(egui::Label::new(text));
                }
            }
        });
    }
}
