use super::{is_terminal_link, ProjectDashboardApp};
use crate::icons::{icon_button, themed_icon, IconKind};
use eframe::egui::{
    self, Color32, FontFamily, FontId, Frame, Margin, RichText, ScrollArea,
};

struct AnsiSpan {
    text: String,
    color: Color32,
    bold: bool,
}

struct AnsiParser {
    current_color: Color32,
    current_bold: bool,
}

impl AnsiParser {
    fn new() -> Self {
        Self {
            current_color: Color32::WHITE,
            current_bold: false,
        }
    }

    fn parse(&mut self, text: &str) -> Vec<AnsiSpan> {
        let mut spans = Vec::new();
        let mut current_text = String::new();
        let bytes = text.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
                if !current_text.is_empty() {
                    spans.push(AnsiSpan {
                        text: current_text.clone(),
                        color: self.current_color,
                        bold: self.current_bold,
                    });
                    current_text.clear();
                }

                i += 2;
                let mut start = i;
                while i < bytes.len() && !((bytes[i] >= b'a' && bytes[i] <= b'z') || (bytes[i] >= b'A' && bytes[i] <= b'Z')) {
                    i += 1;
                }

                if i < bytes.len() {
                    let code_str = String::from_utf8_lossy(&bytes[start..i]);
                    let command = bytes[i] as char;
                    if command == 'm' {
                        for part in code_str.split(';') {
                            match part.parse::<u8>().unwrap_or(0) {
                                0 => {
                                    self.current_color = Color32::WHITE;
                                    self.current_bold = false;
                                }
                                1 => self.current_bold = true,
                                30 => self.current_color = Color32::from_rgb(0, 0, 0),
                                31 => self.current_color = Color32::from_rgb(220, 68, 55), // Red
                                32 => self.current_color = Color32::from_rgb(15, 157, 88),  // Green
                                33 => self.current_color = Color32::from_rgb(244, 180, 0),  // Yellow
                                34 => self.current_color = Color32::from_rgb(66, 133, 244), // Blue
                                35 => self.current_color = Color32::from_rgb(171, 71, 188), // Magenta
                                36 => self.current_color = Color32::from_rgb(0, 172, 193),  // Cyan
                                37 => self.current_color = Color32::from_rgb(255, 255, 255),
                                90 => self.current_color = Color32::from_gray(128),         // Gray
                                _ => {}
                            }
                        }
                    }
                    i += 1;
                }
            } else {
                current_text.push(bytes[i] as char);
                i += 1;
            }
        }

        if !current_text.is_empty() {
            spans.push(AnsiSpan {
                text: current_text,
                color: self.current_color,
                bold: self.current_bold,
            });
        }
        spans
    }
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
        Frame::new()
            .fill(Color32::from_rgb(15, 15, 15))
            .inner_margin(Margin::same(10))
            .show(ui, |ui| {
                let terminal_font = FontId::new(12.5, FontFamily::Name("JetBrainsMono".into()));
                ui.set_min_height((ui.available_height() - 4.0).max(120.0));
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true) // Always scroll to bottom for real PTY feel
                    .show(ui, |ui| {
                        if self.terminal_lines.is_empty() {
                            self.render_ansi_line(
                                ui,
                                "\x1b[90mPS >\x1b[0m bridge idle",
                                &terminal_font,
                            );
                            return;
                        }
                        let lines = self.terminal_lines.clone();
                        for line in &lines {
                            self.render_ansi_line(ui, line, &terminal_font);
                        }
                    });
            });
    }

    fn render_ansi_line(&mut self, ui: &mut egui::Ui, line: &str, font: &FontId) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0; // Terminal text shouldn't have gaps
            let mut parser = AnsiParser::new();
            let spans = parser.parse(line);
            
            for span in spans {
                let mut rich_text = RichText::new(&span.text)
                    .font(font.clone())
                    .color(span.color);
                
                if span.bold {
                    rich_text = rich_text.strong();
                }

                // Check for links within the text span
                if is_terminal_link(&span.text.trim()) {
                    let link = span.text.trim().to_owned();
                    let response = ui.add(
                        egui::Label::new(
                            rich_text.color(Color32::from_rgb(66, 133, 244)).underline(),
                        )
                        .sense(egui::Sense::click()),
                    );
                    if response.clicked() {
                        self.terminal_link_target = Some(link);
                        self.terminal_link_popup_open = true;
                    }
                } else {
                    ui.label(rich_text);
                }
            }
        });
    }
}
