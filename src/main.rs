use eframe::egui::{
    self, Align, Button, Color32, Frame, Image, ImageSource, Layout, Margin, RichText, TextEdit,
    Vec2,
};

fn main() -> Result<(), eframe::Error> {
    let viewport = egui::ViewportBuilder::default()
        .with_title("BuildBridge")
        .with_inner_size([1024.0, 768.0])
        .with_min_inner_size([920.0, 620.0])
        .with_resizable(true)
        .with_decorations(true);

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "BuildBridge",
        native_options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<ProjectDashboardApp>::default())
        }),
    )
}

struct ProjectDashboardApp {
    zoom_applied: bool,
    mica_attempted: bool,
    mica_error: Option<String>,
    nav: Nav,
    sidebar_visible: bool,
    sidebar_width: f32,
    sidebar_animated_width: f32,
    search_text: String,
    project_name: String,
}

impl Default for ProjectDashboardApp {
    fn default() -> Self {
        Self {
            zoom_applied: false,
            mica_attempted: false,
            mica_error: None,
            nav: Nav::Home,
            sidebar_visible: true,
            sidebar_width: 260.0,
            sidebar_animated_width: 260.0,
            search_text: String::new(),
            project_name: "MyAndroidProject-1".to_owned(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Nav {
    Home,
    Archived,
    Theme,
}

impl eframe::App for ProjectDashboardApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if !self.zoom_applied {
            self.zoom_applied = true;
            ctx.set_zoom_factor(1.20);
        }

        if !self.mica_attempted {
            self.mica_attempted = true;
            apply_native_mica(frame, &mut self.mica_error);
        }

        let dark = ctx.style().visuals.dark_mode;

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
            let fully_open = self.sidebar_visible && (self.sidebar_animated_width - self.sidebar_width).abs() < 1.0;
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
                    self.sidebar_width = ui.available_width().clamp(200.0, 360.0);
                    self.sidebar_animated_width = self.sidebar_width;
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
                    nav_item(ui, dark, &mut self.nav, Nav::Home, "Home", IconKind::Home);
                    nav_item(
                        ui,
                        dark,
                        &mut self.nav,
                        Nav::Archived,
                        "Archived",
                        IconKind::Archive,
                    );
                    nav_item(ui, dark, &mut self.nav, Nav::Theme, "Theme", IconKind::Theme);
                }
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            Frame::new().inner_margin(Margin::same(12)).show(ui, |ui| {
                ui.horizontal(|ui| {
                    if !self.sidebar_visible {
                        if ui
                            .add(icon_button(themed_icon(dark, IconKind::PanelShow), 26.0))
                            .on_hover_text("Show sidebar")
                            .clicked()
                        {
                            self.sidebar_visible = true;
                        }
                        ui.add_space(8.0);
                    }

                    ui.heading("Your Projects");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let _ = ui.add(brand_button("Create"));
                    });
                });
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    let sort_base = ui.spacing().interact_size.y;
                    let search_height = sort_base + 5.0;
                    let space = ui.spacing().item_spacing.x;
                    let search_width = (ui.available_width() - search_height - space).max(120.0);

                    let search = TextEdit::singleline(&mut self.search_text)
                        .hint_text("Search")
                        .desired_width(search_width);
                    let _ = ui.add_sized([search_width, search_height], search);

                    let _ = ui.add(
                        icon_button(themed_icon(dark, IconKind::Sort), search_height)
                            .min_size(Vec2::splat(search_height)),
                    );
                });

                ui.add_space(8.0);
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(&self.project_name);

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let _ = ui.add(icon_button(themed_icon(dark, IconKind::Edit), 22.0));
                            ui.add_space(4.0);
                            let _ = ui.add(brand_button("Serve"));
                        });
                    });
                });

                if let Some(err) = &self.mica_error {
                    ui.add_space(8.0);
                    ui.colored_label(Color32::LIGHT_RED, format!("Mica unavailable: {err}"));
                }
            });
        });
    }
}

fn brand_button(label: &str) -> Button<'_> {
    Button::new(RichText::new(label).color(Color32::WHITE)).fill(Color32::from_rgb(2, 110, 193))
}

fn nav_item(ui: &mut egui::Ui, dark: bool, nav: &mut Nav, value: Nav, label: &str, icon: IconKind) {
    ui.horizontal(|ui| {
        ui.add(icon_image(themed_icon(dark, icon), 18.0));
        ui.selectable_value(nav, value, label);
    });
}

fn icon_button(source: ImageSource<'static>, size: f32) -> Button<'static> {
    Button::image(icon_image(source, size))
}

fn icon_image(source: ImageSource<'static>, size: f32) -> Image<'static> {
    Image::new(source).fit_to_exact_size(Vec2::splat(size))
}

#[derive(Clone, Copy)]
enum IconKind {
    Home,
    Archive,
    Theme,
    PanelHide,
    PanelShow,
    Sort,
    Edit,
}

fn themed_icon(dark: bool, icon: IconKind) -> ImageSource<'static> {
    match (dark, icon) {
        (true, IconKind::Home) => egui::include_image!("../assets/icons/home_dark.svg"),
        (false, IconKind::Home) => egui::include_image!("../assets/icons/home_light.svg"),
        (true, IconKind::Archive) => egui::include_image!("../assets/icons/archive_dark.svg"),
        (false, IconKind::Archive) => egui::include_image!("../assets/icons/archive_light.svg"),
        (true, IconKind::Theme) => egui::include_image!("../assets/icons/theme_dark.svg"),
        (false, IconKind::Theme) => egui::include_image!("../assets/icons/theme_light.svg"),
        (true, IconKind::PanelHide) => egui::include_image!("../assets/icons/panel_hide_dark.svg"),
        (false, IconKind::PanelHide) => egui::include_image!("../assets/icons/panel_hide_light.svg"),
        (true, IconKind::PanelShow) => egui::include_image!("../assets/icons/panel_show_dark.svg"),
        (false, IconKind::PanelShow) => egui::include_image!("../assets/icons/panel_show_light.svg"),
        (true, IconKind::Sort) => egui::include_image!("../assets/icons/sort_dark.svg"),
        (false, IconKind::Sort) => egui::include_image!("../assets/icons/sort_light.svg"),
        (true, IconKind::Edit) => egui::include_image!("../assets/icons/edit_dark.svg"),
        (false, IconKind::Edit) => egui::include_image!("../assets/icons/edit_light.svg"),
    }
}

fn apply_native_mica(frame: &eframe::Frame, mica_error: &mut Option<String>) {
    #[cfg(target_os = "windows")]
    {
        if let Err(err) = window_vibrancy::apply_mica(frame, Some(true)) {
            *mica_error = Some(err.to_string());
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = frame;
        let _ = mica_error;
    }
}
