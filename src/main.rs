use chrono::Local;
use directories::ProjectDirs;
use eframe::egui::{
    self, Align, Button, Color32, ComboBox, Frame, Image, ImageSource, Layout, Margin,
    RichText, ScrollArea, ThemePreference, TextEdit, Vec2,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

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
    create_modal_open: bool,
    create_modal_step: CreateModalStep,
    selected_framework: ProjectType,
    search_text: String,
    projects_file_path: Option<PathBuf>,
    projects: Vec<ProjectRecord>,
    create_form: CreateProjectForm,
    form_error: Option<String>,
    status_message: Option<String>,
    storage_error: Option<String>,
    theme_popup_open: bool,
}

impl Default for ProjectDashboardApp {
    fn default() -> Self {
        let (projects_file_path, projects, storage_error) = init_storage();
        Self {
            zoom_applied: false,
            mica_attempted: false,
            mica_error: None,
            nav: Nav::Home,
            sidebar_visible: true,
            sidebar_width: 260.0,
            sidebar_animated_width: 260.0,
            create_modal_open: false,
            create_modal_step: CreateModalStep::Framework,
            selected_framework: ProjectType::Android,
            search_text: String::new(),
            projects_file_path,
            projects,
            create_form: CreateProjectForm::default(),
            form_error: None,
            status_message: None,
            storage_error,
            theme_popup_open: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Nav {
    Home,
    Archived,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ProjectType {
    Android,
    Flutter,
    DotNet,
    Python,
    ReactNative,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CreateModalStep {
    Framework,
    Form,
}

impl ProjectType {
    fn label(self) -> &'static str {
        match self {
            Self::Android => "Android Studio",
            Self::Flutter => "Flutter",
            Self::DotNet => ".NET",
            Self::Python => "Python",
            Self::ReactNative => "React Native",
        }
    }

    fn storage_value(self) -> &'static str {
        self.label()
    }

    fn all() -> [Self; 5] {
        [
            Self::Android,
            Self::Flutter,
            Self::DotNet,
            Self::Python,
            Self::ReactNative,
        ]
    }

    fn from_storage(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "android" | "android studio" => Some(Self::Android),
            "flutter" => Some(Self::Flutter),
            ".net" | "dotnet" => Some(Self::DotNet),
            "python" => Some(Self::Python),
            "react native" | "react-native" => Some(Self::ReactNative),
            _ => None,
        }
    }
}

#[derive(Default)]
struct CreateProjectForm {
    name: String,
    main_path: String,
    project_type: ProjectType,
}

#[derive(Serialize, Deserialize, Clone)]
struct BuildEntry {
    name: String,
    path: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct ProjectRecord {
    name: String,
    #[serde(rename = "type")]
    project_type: String,
    main_path: String,
    builds: Vec<BuildEntry>,
    status: String,
    created_on: String,
    edited_on: String,
}

impl Default for ProjectType {
    fn default() -> Self {
        Self::Android
    }
}

impl eframe::App for ProjectDashboardApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if !self.zoom_applied {
            self.zoom_applied = true;
            ctx.set_zoom_factor(1.20);
        }

        if self.create_modal_open && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.create_modal_open = false;
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
                    ui.horizontal(|ui| {
                        ui.add(icon_image(themed_icon(dark, IconKind::Theme), 18.0));
                        if ui.button("Theme").clicked() {
                            self.theme_popup_open = true;
                        }
                    });
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
                        if ui.add(brand_button("Create")).clicked() {
                            self.form_error = None;
                            self.create_form = CreateProjectForm::default();
                            self.create_modal_step = CreateModalStep::Framework;
                            self.selected_framework = self.create_form.project_type;
                            self.create_modal_open = true;
                        }
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
                ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                    for project in self.filtered_projects() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(&project.name);

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let _ = ui.add(icon_button(themed_icon(dark, IconKind::Edit), 22.0));
                                    ui.add_space(4.0);
                                    let _ = ui.add(brand_button("Serve"));
                                });
                            });
                            ui.horizontal_wrapped(|ui| {
                                let framework_label = map_framework_label(&project.project_type);
                                ui.label(egui::RichText::new(framework_label).strong());
                                ui.label("•");
                                ui.label(egui::RichText::new(&project.main_path).italics());
                            });
                        });
                        ui.add_space(6.0);
                    }
                });

                if let Some(message) = &self.status_message {
                    ui.colored_label(Color32::from_rgb(130, 210, 130), message);
                }

                if let Some(storage_error) = &self.storage_error {
                    ui.colored_label(Color32::LIGHT_RED, format!("Storage error: {storage_error}"));
                }

                if let Some(err) = &self.mica_error {
                    ui.add_space(8.0);
                    ui.colored_label(Color32::LIGHT_RED, format!("Mica unavailable: {err}"));
                }
            });
        });

        if self.theme_popup_open {
            let mut open = self.theme_popup_open;
            let mut close_theme_popup = false;
            egui::Window::new("Theme")
                .collapsible(false)
                .resizable(false)
                .default_size(Vec2::new(220.0, 130.0))
                .open(&mut open)
                .show(ctx, |ui| {
                    if ui.button("System").clicked() {
                        ctx.set_theme(ThemePreference::System);
                        close_theme_popup = true;
                    }
                    if ui.button("Light").clicked() {
                        ctx.set_theme(ThemePreference::Light);
                        close_theme_popup = true;
                    }
                    if ui.button("Dark").clicked() {
                        ctx.set_theme(ThemePreference::Dark);
                        close_theme_popup = true;
                    }
                });
            if close_theme_popup {
                open = false;
            }
            self.theme_popup_open = open;
        }

        if self.create_modal_open {
            // Dim the page behind the modal.
            let overlay_painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Middle,
                egui::Id::new("create_modal_overlay"),
            ));
            overlay_painter.rect_filled(
                ctx.viewport_rect(),
                0.0,
                Color32::from_rgba_premultiplied(0, 0, 0, 160),
            );

            let mut close_modal = false;
            let mut open = self.create_modal_open;
            egui::Window::new("Create Project")
                .order(egui::Order::Foreground)
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .movable(true)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .default_size(Vec2::new(360.0, 220.0))
                .min_size(Vec2::new(320.0, 210.0))
                .show(ctx, |ui| {
                    ui.spacing_mut().item_spacing.y = 8.0;

                    match self.create_modal_step {
                        CreateModalStep::Framework => {
                            ui.label("Select Framework");
                            ui.horizontal_centered(|ui| {
                                let android_response = framework_card(
                                    ui,
                                    self.selected_framework == ProjectType::Android,
                                    "📱",
                                    "Android Studio",
                                );
                                if android_response.clicked() {
                                    self.selected_framework = ProjectType::Android;
                                }

                                ui.add_space(8.0);

                                let flutter_response = framework_card(
                                    ui,
                                    self.selected_framework == ProjectType::Flutter,
                                    "🦋",
                                    "Flutter",
                                );
                                if flutter_response.clicked() {
                                    self.selected_framework = ProjectType::Flutter;
                                }
                            });

                            ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                if ui.button("Cancel").clicked() {
                                    close_modal = true;
                                }
                                if ui.button("Next").clicked() {
                                    self.create_form.project_type = self.selected_framework;
                                    self.create_modal_step = CreateModalStep::Form;
                                }
                            });
                        }
                        CreateModalStep::Form => {
                            ui.label("Name");
                            let _ =
                                ui.add(TextEdit::singleline(&mut self.create_form.name).desired_width(280.0));

                            ui.label("Path");
                            ui.horizontal(|ui| {
                                let _ = ui.add(
                                    TextEdit::singleline(&mut self.create_form.main_path)
                                        .desired_width(240.0),
                                );
                                if ui.button("Browse").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                        self.create_form.main_path = path.display().to_string();
                                    }
                                }
                            });

                            ui.label("Type");
                            ComboBox::from_id_salt("project_type_select")
                                .selected_text(self.create_form.project_type.label())
                                .show_ui(ui, |ui| {
                                    for project_type in ProjectType::all() {
                                        ui.selectable_value(
                                            &mut self.create_form.project_type,
                                            project_type,
                                            project_type.label(),
                                        );
                                    }
                                });

                            if let Some(form_error) = &self.form_error {
                                ui.colored_label(Color32::LIGHT_RED, form_error);
                            }

                            ui.horizontal(|ui| {
                                if ui.button("Back").clicked() {
                                    self.selected_framework = self.create_form.project_type;
                                    self.create_modal_step = CreateModalStep::Framework;
                                }
                                if ui.button("Cancel").clicked() {
                                    close_modal = true;
                                }
                                if ui.add(brand_button("Create")).clicked() {
                                    match self.create_project() {
                                        Ok(success_message) => {
                                            self.status_message = Some(success_message);
                                            self.form_error = None;
                                            close_modal = true;
                                        }
                                        Err(err) => {
                                            self.form_error = Some(err);
                                        }
                                    };
                                }
                            });
                        }
                    }
                });

            if close_modal {
                open = false;
            }
            self.create_modal_open = open;
        }
    }
}

fn framework_card(ui: &mut egui::Ui, selected: bool, icon: &str, label: &str) -> egui::Response {
    let card_size = Vec2::new(120.0, 120.0);
    let stroke = if selected {
        egui::Stroke::new(2.0, Color32::from_rgb(2, 110, 193))
    } else {
        egui::Stroke::new(1.0, Color32::from_gray(95))
    };

    let response = ui.add_sized(
        card_size,
        Button::new("")
            .fill(ui.style().visuals.panel_fill)
            .stroke(stroke),
    );

    let rect = response.rect;
    let text_color = ui.style().visuals.text_color();
    ui.painter().text(
        egui::pos2(rect.center().x, rect.center().y - 20.0),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(26.0),
        text_color,
    );
    ui.painter().text(
        egui::pos2(rect.center().x, rect.center().y + 24.0),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(16.0),
        text_color,
    );

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    response
}

impl ProjectDashboardApp {
    fn filtered_projects(&self) -> Vec<ProjectRecord> {
        let query = self.search_text.to_lowercase();
        self.projects
            .iter()
            .filter(|project| {
                let nav_match = match self.nav {
                    Nav::Home => project.status != "archived",
                    Nav::Archived => project.status == "archived",
                };
                let framework = map_framework_label(&project.project_type).to_lowercase();
                let search_match = query.is_empty()
                    || project.name.to_lowercase().contains(&query)
                    || project.main_path.to_lowercase().contains(&query)
                    || framework.contains(&query)
                    || project.status.to_lowercase().contains(&query);
                nav_match && search_match
            })
            .cloned()
            .collect()
    }

    fn create_project(&mut self) -> Result<String, String> {
        let name = self.create_form.name.trim();
        let main_path = self.create_form.main_path.trim();

        if name.is_empty() {
            return Err("Project name is required.".to_owned());
        }

        if main_path.is_empty() {
            return Err("Project path is required. Paste a path or use Browse.".to_owned());
        }

        if self
            .projects
            .iter()
            .any(|project| project.name.eq_ignore_ascii_case(name))
        {
            return Err(format!("A project named '{name}' already exists."));
        }

        let today = current_date();
        let project = ProjectRecord {
            name: name.to_owned(),
            project_type: self.create_form.project_type.storage_value().to_owned(),
            main_path: main_path.to_owned(),
            builds: Vec::new(),
            status: "active".to_owned(),
            created_on: today.clone(),
            edited_on: today,
        };

        self.projects.push(project);

        let path = self
            .projects_file_path
            .as_ref()
            .ok_or_else(|| "Cannot determine config directory for Projects.json".to_owned())?;

        save_projects(path, &self.projects)?;
        Ok(format!("Project '{name}' saved to {}", path.display()))
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

fn init_storage() -> (Option<PathBuf>, Vec<ProjectRecord>, Option<String>) {
    match resolve_projects_file_path() {
        Ok(path) => match load_or_create_projects(&path) {
            Ok(projects) => (Some(path), projects, None),
            Err(err) => (Some(path), Vec::new(), Some(err)),
        },
        Err(err) => (None, Vec::new(), Some(err)),
    }
}

fn resolve_projects_file_path() -> Result<PathBuf, String> {
    let project_dirs = ProjectDirs::from("com", "BuildBridge", "BuildBridge")
        .ok_or_else(|| "Failed to locate a writable config folder for this OS.".to_owned())?;
    let config_dir = project_dirs.config_dir();
    fs::create_dir_all(config_dir).map_err(|err| {
        format!(
            "Failed to create config folder '{}': {err}",
            config_dir.display()
        )
    })?;
    Ok(config_dir.join("Projects.json"))
}

fn load_or_create_projects(path: &Path) -> Result<Vec<ProjectRecord>, String> {
    if !path.exists() {
        let sample = sample_projects();
        save_projects(path, &sample)?;
        return Ok(sample);
    }

    let raw = fs::read_to_string(path)
        .map_err(|err| format!("Failed to read '{}': {err}", path.display()))?;

    serde_json::from_str::<Vec<ProjectRecord>>(&raw).map_err(|err| {
        format!(
            "Projects.json is invalid JSON at '{}': {err}",
            path.display()
        )
    })
}

fn save_projects(path: &Path, projects: &[ProjectRecord]) -> Result<(), String> {
    let json = serde_json::to_string_pretty(projects)
        .map_err(|err| format!("Failed to serialize projects to JSON: {err}"))?;
    fs::write(path, json).map_err(|err| format!("Failed to write '{}': {err}", path.display()))
}

fn current_date() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

fn sample_projects() -> Vec<ProjectRecord> {
    vec![
        ProjectRecord {
            name: "Demo1".to_owned(),
            project_type: "Android Studio".to_owned(),
            main_path: "/projects/demo1".to_owned(),
            builds: vec![
                BuildEntry {
                    name: "debug".to_owned(),
                    path: "app/build/debug.apk".to_owned(),
                },
                BuildEntry {
                    name: "release".to_owned(),
                    path: "app/build/release.apk".to_owned(),
                },
            ],
            status: "active".to_owned(),
            created_on: "2026-04-20".to_owned(),
            edited_on: "2026-04-20".to_owned(),
        },
        ProjectRecord {
            name: "Demo2".to_owned(),
            project_type: "Android Studio".to_owned(),
            main_path: "/projects/demo2".to_owned(),
            builds: vec![BuildEntry {
                name: "debug".to_owned(),
                path: "app/build/debug.apk".to_owned(),
            }],
            status: "archived".to_owned(),
            created_on: "2026-04-18".to_owned(),
            edited_on: "2026-04-19".to_owned(),
        },
        ProjectRecord {
            name: "Demo3".to_owned(),
            project_type: "Android Studio".to_owned(),
            main_path: "/projects/demo3".to_owned(),
            builds: vec![
                BuildEntry {
                    name: "flavor1".to_owned(),
                    path: "app/build/flavor1.apk".to_owned(),
                },
                BuildEntry {
                    name: "flavor2".to_owned(),
                    path: "app/build/flavor2.apk".to_owned(),
                },
            ],
            status: "active".to_owned(),
            created_on: "2026-04-15".to_owned(),
            edited_on: "2026-04-20".to_owned(),
        },
    ]
}

fn map_framework_label(stored: &str) -> String {
    ProjectType::from_storage(stored)
        .map(|project_type| project_type.label().to_owned())
        .unwrap_or_else(|| stored.to_owned())
}
