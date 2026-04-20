use eframe::egui::{
    self, Align, Color32, CornerRadius, Frame, Layout, Margin, RichText, Stroke, ThemePreference,
};

fn main() -> Result<(), eframe::Error> {
    let viewport = egui::ViewportBuilder::default()
        .with_title("Creator Deck")
        .with_inner_size([1024.0, 680.0])
        .with_min_inner_size([760.0, 520.0])
        .with_transparent(true)
        .with_decorations(true);

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Creator Deck",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_theme(ThemePreference::System);
            Ok(Box::<CreatorDeckApp>::default())
        }),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ThemeMode {
    FollowSystem,
    Light,
    Dark,
}

impl ThemeMode {
    fn apply(self, ctx: &egui::Context) {
        let preference = match self {
            Self::FollowSystem => ThemePreference::System,
            Self::Light => ThemePreference::Light,
            Self::Dark => ThemePreference::Dark,
        };
        ctx.set_theme(preference);
    }
}

struct ProjectCard {
    title: &'static str,
    description: &'static str,
    accent: Color32,
}

struct CreatorDeckApp {
    theme_mode: ThemeMode,
    selected: usize,
    create_counter: usize,
    cards: Vec<ProjectCard>,
}

impl Default for CreatorDeckApp {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::FollowSystem,
            selected: 0,
            create_counter: 0,
            cards: vec![
                ProjectCard {
                    title: "Landing Page",
                    description: "A modern one-page website starter with responsive sections.",
                    accent: Color32::from_rgb(72, 130, 255),
                },
                ProjectCard {
                    title: "Inventory Tool",
                    description: "Desktop-first CRUD shell for stock, suppliers, and export.",
                    accent: Color32::from_rgb(21, 199, 155),
                },
                ProjectCard {
                    title: "Portfolio",
                    description: "A visual portfolio layout with case studies and contact CTA.",
                    accent: Color32::from_rgb(252, 129, 44),
                },
            ],
        }
    }
}

impl eframe::App for CreatorDeckApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        Color32::TRANSPARENT.to_normalized_gamma_f32()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.theme_mode.apply(ctx);

        egui::SidePanel::left("sidebar")
            .exact_width(340.0)
            .frame(
                Frame::new()
                    .fill(Color32::from_rgba_premultiplied(36, 38, 48, 96))
                    .inner_margin(Margin::same(16))
                    .stroke(Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 24))),
            )
            .show(ctx, |ui| {
                sidebar_header(ui, self.theme_mode, &mut self.theme_mode);
                ui.add_space(8.0);
                ui.label(RichText::new("Templates").size(18.0).strong());
                ui.add_space(10.0);

                for (idx, card) in self.cards.iter().enumerate() {
                    let active = self.selected == idx;
                    let bg = if active {
                        Color32::from_rgba_premultiplied(
                            card.accent.r(),
                            card.accent.g(),
                            card.accent.b(),
                            210,
                        )
                    } else {
                        Color32::from_rgba_premultiplied(255, 255, 255, 26)
                    };

                    let text = if active {
                        Color32::from_rgb(245, 246, 250)
                    } else {
                        Color32::from_rgb(222, 224, 230)
                    };

                    let button = egui::Button::new(
                        RichText::new(format!("{}\n{}", card.title, card.description))
                            .size(17.0)
                            .color(text),
                    )
                    .fill(bg)
                    .corner_radius(CornerRadius::same(16))
                    .stroke(Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 30)));

                    if ui.add_sized([304.0, 86.0], button).clicked() {
                        self.selected = idx;
                    }

                    ui.add_space(8.0);
                }
            });

        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(Color32::from_rgba_premultiplied(18, 20, 27, 108))
                    .inner_margin(Margin::same(24)),
            )
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.heading(RichText::new("Create Workspace").size(36.0));
                });
                ui.add_space(10.0);

                let selected_card = &self.cards[self.selected];
                ui.label(
                    RichText::new(format!(
                        "Selected template: {}\n{}",
                        selected_card.title, selected_card.description
                    ))
                    .size(20.0)
                    .color(Color32::from_rgb(228, 230, 236)),
                );
                ui.add_space(24.0);

                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    let create_button = egui::Button::new(
                        RichText::new("Create")
                            .size(24.0)
                            .strong()
                            .color(Color32::from_rgb(245, 248, 255)),
                    )
                    .fill(Color32::from_rgb(47, 122, 251))
                    .corner_radius(CornerRadius::same(14))
                    .stroke(Stroke::NONE);

                    if ui.add_sized([180.0, 52.0], create_button).clicked() {
                        self.create_counter += 1;
                    }

                    ui.add_space(14.0);
                    ui.label(
                        RichText::new(format!("Created: {}", self.create_counter))
                            .size(20.0)
                            .color(Color32::from_rgb(210, 214, 224)),
                    );
                });
            });
    }
}

fn sidebar_header(ui: &mut egui::Ui, current: ThemeMode, theme_mode: &mut ThemeMode) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("Creator Deck").size(32.0).strong());
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            egui::ComboBox::from_id_salt("theme_picker")
                .selected_text(match current {
                    ThemeMode::FollowSystem => "System",
                    ThemeMode::Light => "Light",
                    ThemeMode::Dark => "Dark",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(theme_mode, ThemeMode::FollowSystem, "System");
                    ui.selectable_value(theme_mode, ThemeMode::Light, "Light");
                    ui.selectable_value(theme_mode, ThemeMode::Dark, "Dark");
                });
        });
    });
}
