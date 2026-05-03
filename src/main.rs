use chrono::Local;
use directories::ProjectDirs;
use eframe::egui::{
    self, Align, Button, Color32, ComboBox, CornerRadius, Frame, Image, ImageSource, Layout,
    IconData, Margin, RichText, ScrollArea, Stroke, StrokeKind,
    ThemePreference, TextEdit, TopBottomPanel, Vec2,
};
use image::ImageReader;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> Result<(), eframe::Error> {
    let icon_data = load_app_icon();

    let viewport = egui::ViewportBuilder::default()
        .with_title("Build Stream")
        .with_inner_size([1024.0, 538.0])
        .with_min_inner_size([920.0, 434.0])
        .with_icon(icon_data)
        .with_resizable(true)
        .with_decorations(true);

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Build Stream",
        native_options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<ProjectDashboardApp>::default())
        }),
    )
}

fn load_app_icon() -> IconData {
    let icon_bytes = include_bytes!("../icon.ico");
    let decoded = ImageReader::new(Cursor::new(icon_bytes))
        .with_guessed_format()
        .expect("Failed to detect format for icon.ico")
        .decode()
        .expect("Failed to decode icon.ico");
    let rgba = decoded.to_rgba8();
    let (width, height) = rgba.dimensions();

    IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    }
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
    theme_mode: AppThemeMode,
    project_action_error: Option<String>,
    modal_mode: ModalMode,
    archive_select_mode: bool,
    archive_selected: HashSet<String>,
    bin_select_mode: bool,
    bin_selected: HashSet<String>,
    empty_bin_confirm_open: bool,
    project_details_open: bool,
    selected_project_name: Option<String>,
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
            theme_mode: AppThemeMode::System,
            project_action_error: None,
            modal_mode: ModalMode::Create,
            archive_select_mode: false,
            archive_selected: HashSet::new(),
            bin_select_mode: false,
            bin_selected: HashSet::new(),
            empty_bin_confirm_open: false,
            project_details_open: false,
            selected_project_name: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Nav {
    Home,
    Archived,
    Bin,
    About,
    Feedback,
    PrivacyPolicy,
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum AppThemeMode {
    System,
    Dark,
    Light,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SupportPage {
    About,
    Feedback,
    PrivacyPolicy,
}

#[derive(Clone, PartialEq, Eq)]
enum ModalMode {
    Create,
    Edit { original_name: String },
}

impl AppThemeMode {
    fn to_pref(self) -> ThemePreference {
        match self {
            Self::System => ThemePreference::System,
            Self::Dark => ThemePreference::Dark,
            Self::Light => ThemePreference::Light,
        }
    }
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

        ctx.set_theme(self.theme_mode.to_pref());

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
                    ui.label("General");
                    nav_item(ui, dark, &mut self.nav, Nav::Home, "Home", IconKind::Home);
                    nav_item(
                        ui,
                        dark,
                        &mut self.nav,
                        Nav::Archived,
                        "Archived",
                        IconKind::Archive,
                    );
                    nav_item(ui, dark, &mut self.nav, Nav::Bin, "Bin", IconKind::Bin);
                    ui.horizontal(|ui| {
                        ui.add(icon_image(themed_icon(dark, IconKind::Theme), 18.0));
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
                        self.nav = Nav::About;
                    }
                    if support_page_row(ui, dark, IconKind::Feedback, "Feedback").clicked() {
                        self.nav = Nav::Feedback;
                    }
                    if support_page_row(ui, dark, IconKind::Privacy, "Privacy Policy").clicked() {
                        self.nav = Nav::PrivacyPolicy;
                    }
                }
            });
        }

        TopBottomPanel::bottom("status_bar")
            .exact_height(24.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some(message) = &self.status_message {
                        ui.colored_label(Color32::from_rgb(2, 110, 193), message);
                    }
                });
            });

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

                    let heading = if self.nav == Nav::Bin {
                        "Bin"
                    } else {
                        "Your Projects"
                    };
                    ui.heading(heading);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        match self.nav {
                            Nav::Bin => {
                                let select_label = if self.bin_select_mode { "Done" } else { "Select" };
                                if ui.button(select_label).clicked() {
                                    self.bin_select_mode = !self.bin_select_mode;
                                    if !self.bin_select_mode {
                                        self.bin_selected.clear();
                                    }
                                }
                                if !self.bin_select_mode && ui.button("Empty Bin").clicked() {
                                    self.empty_bin_confirm_open = true;
                                }
                            }
                            Nav::Archived => {
                                let select_label = if self.archive_select_mode { "Done" } else { "Select" };
                                if ui.button(select_label).clicked() {
                                    self.archive_select_mode = !self.archive_select_mode;
                                    if !self.archive_select_mode {
                                        self.archive_selected.clear();
                                    }
                                }
                            }
                            Nav::Home => {
                                if ui.add(brand_button("Create")).clicked() {
                                    self.form_error = None;
                                    self.create_form = CreateProjectForm::default();
                                    self.create_modal_step = CreateModalStep::Framework;
                                    self.selected_framework = self.create_form.project_type;
                                    self.modal_mode = ModalMode::Create;
                                    self.create_modal_open = true;
                                }
                            }
                            Nav::About | Nav::Feedback | Nav::PrivacyPolicy => {}
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
                        .frame(false)
                        .hint_text("Search")
                        .desired_width(search_width);
                    let search_response = ui.add_sized([search_width, search_height], search);
                    ui.painter().rect_stroke(
                        search_response.rect,
                        CornerRadius::same(8),
                        Stroke::new(1.0, Color32::from_gray(95)),
                        StrokeKind::Outside,
                    );

                    if search_response.has_focus() && ctx.input(|i| i.key_pressed(egui::Key::Delete)) {
                        self.search_text.clear();
                    }

                    if !self.search_text.is_empty() {
                        let clear_size = (search_height - 6.0).max(16.0);
                        let clear_rect = egui::Rect::from_min_size(
                            egui::pos2(
                                search_response.rect.right() - clear_size - 4.0,
                                search_response.rect.center().y - clear_size * 0.5,
                            ),
                            egui::Vec2::splat(clear_size),
                        );
                        let clear_response = ui.put(clear_rect, icon_button(themed_icon(dark, IconKind::Clear), clear_size).frame(false));
                        if clear_response.clicked() {
                            self.search_text.clear();
                        }
                    }

                    let _ = ui.add(
                        icon_button(themed_icon(dark, IconKind::Sort), search_height)
                            .min_size(Vec2::splat(search_height)),
                    );
                });

                if self.nav == Nav::Archived && self.archive_select_mode {
                    ui.add_space(6.0);
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Select all").clicked() {
                            let names: HashSet<String> = self
                                .filtered_projects()
                                .into_iter()
                                .map(|p| p.name)
                                .collect();
                            self.archive_selected = names;
                        }
                        if ui.button("Unselect all").clicked() {
                            self.archive_selected.clear();
                        }
                        if ui.button("Bin").clicked() {
                            let selected = self.archive_selected.clone();
                            match self.bulk_bin_projects(&selected) {
                                Ok(count) => {
                                    self.status_message = Some(format!("Moved {count} project(s) to Bin."));
                                    self.archive_selected.clear();
                                }
                                Err(err) => {
                                    self.project_action_error = Some(err);
                                }
                            }
                        }
                        if ui.add(brand_button("Unarchive")).clicked() {
                            let selected = self.archive_selected.clone();
                            match self.bulk_unarchive_projects(&selected) {
                                Ok(count) => {
                                    self.status_message = Some(format!("Unarchived {count} project(s)."));
                                    self.archive_selected.clear();
                                }
                                Err(err) => {
                                    self.project_action_error = Some(err);
                                }
                            }
                        }
                    });
                }

                if self.nav == Nav::Bin && self.bin_select_mode {
                    ui.add_space(6.0);
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Select all").clicked() {
                            let names: HashSet<String> = self
                                .filtered_projects()
                                .into_iter()
                                .map(|p| p.name)
                                .collect();
                            self.bin_selected = names;
                        }
                        if ui.button("Unselect all").clicked() {
                            self.bin_selected.clear();
                        }
                        if ui
                            .add(Button::new(RichText::new("Permanent delete").color(Color32::from_rgb(229, 57, 53))))
                            .clicked()
                        {
                            let selected = self.bin_selected.clone();
                            match self.bulk_permanent_delete_projects(&selected) {
                                Ok(count) => {
                                    self.status_message = Some(format!("Permanently deleted {count} project(s)."));
                                    self.bin_selected.clear();
                                }
                                Err(err) => {
                                    self.project_action_error = Some(err);
                                }
                            }
                        }
                        if ui.add(brand_button("Restore")).clicked() {
                            let selected = self.bin_selected.clone();
                            match self.bulk_restore_projects(&selected) {
                                Ok(count) => {
                                    self.status_message = Some(format!("Restored {count} project(s)."));
                                    self.bin_selected.clear();
                                }
                                Err(err) => {
                                    self.project_action_error = Some(err);
                                }
                            }
                        }
                    });
                }

                ui.add_space(8.0);
                match self.nav {
                    Nav::Home | Nav::Archived | Nav::Bin => {
                        let list_height = (ui.available_height() - 6.0).max(180.0);
                        ScrollArea::vertical().max_height(list_height).show(ui, |ui| {
                            for project in self.filtered_projects() {
                                let card_response = ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        if (self.nav == Nav::Archived && self.archive_select_mode)
                                            || (self.nav == Nav::Bin && self.bin_select_mode)
                                        {
                                            let mut checked = if self.nav == Nav::Archived {
                                                self.archive_selected.contains(&project.name)
                                            } else {
                                                self.bin_selected.contains(&project.name)
                                            };
                                            if ui.checkbox(&mut checked, "").changed() {
                                                if self.nav == Nav::Archived {
                                                    if checked {
                                                        self.archive_selected.insert(project.name.clone());
                                                    } else {
                                                        self.archive_selected.remove(&project.name);
                                                    }
                                                } else {
                                                    if checked {
                                                        self.bin_selected.insert(project.name.clone());
                                                    } else {
                                                        self.bin_selected.remove(&project.name);
                                                    }
                                                }
                                            }
                                        }

                                        let name_color = if dark { Color32::WHITE } else { Color32::BLACK };
                                        ui.label(RichText::new(&project.name).strong().color(name_color));

                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            ui.menu_image_button(icon_image(themed_icon(dark, IconKind::MoreVert), 16.0), |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.add(icon_image(themed_icon(dark, IconKind::ActionEdit), 14.0));
                                                    if ui.button("Edit").clicked() {
                                                        self.begin_edit_project(&project.name);
                                                        ui.close();
                                                    }
                                                });
                                                if self.nav != Nav::Archived {
                                                    ui.horizontal(|ui| {
                                                        ui.add(icon_image(themed_icon(dark, IconKind::ActionArchive), 14.0));
                                                        if ui.button("Archive").clicked() {
                                                            if let Err(err) = self.archive_project(&project.name) {
                                                                self.project_action_error = Some(err);
                                                            }
                                                            ui.close();
                                                        }
                                                    });
                                                } else {
                                                    ui.horizontal(|ui| {
                                                        ui.add(icon_image(themed_icon(dark, IconKind::ActionArchive), 14.0));
                                                        if ui.button("Unarchive").clicked() {
                                                            if let Err(err) = self.unarchive_project(&project.name) {
                                                                self.project_action_error = Some(err);
                                                            }
                                                            ui.close();
                                                        }
                                                    });
                                                }

                                                if self.nav != Nav::Bin {
                                                    ui.horizontal(|ui| {
                                                        ui.add(icon_image(themed_icon(dark, IconKind::Bin), 14.0));
                                                        let bin_button = if self.nav == Nav::Home {
                                                            Button::new(
                                                                RichText::new("Bin").color(Color32::from_rgb(229, 57, 53)),
                                                            )
                                                        } else {
                                                            Button::new("Bin")
                                                        };
                                                        if ui.add(bin_button).clicked()
                                                        {
                                                            if let Err(err) = self.bin_project(&project.name) {
                                                                self.project_action_error = Some(err);
                                                            }
                                                            ui.close();
                                                        }
                                                    });
                                                }

                                                if self.nav == Nav::Bin {
                                                    ui.horizontal(|ui| {
                                                        ui.add(icon_image(themed_icon(dark, IconKind::ActionArchive), 14.0));
                                                        if ui.button("Restore").clicked() {
                                                            if let Err(err) = self.restore_project(&project.name) {
                                                                self.project_action_error = Some(err);
                                                            }
                                                            ui.close();
                                                        }
                                                    });
                                                    ui.horizontal(|ui| {
                                                        ui.add(icon_image(themed_icon(dark, IconKind::ActionDelete), 14.0));
                                                        if ui
                                                            .add(Button::new(RichText::new("Permanent Delete").color(Color32::from_rgb(229, 57, 53))))
                                                            .clicked()
                                                        {
                                                            if let Err(err) = self.permanent_delete_project(&project.name) {
                                                                self.project_action_error = Some(err);
                                                            }
                                                            ui.close();
                                                        }
                                                    });
                                                }

                                            });
                                            if ui
                                                .add(
                                                    icon_button(themed_icon(dark, IconKind::Broadcast), 14.0)
                                                        .frame(true)
                                                        .min_size(Vec2::new(28.0, 24.0)),
                                                )
                                                .on_hover_text("Serve settings")
                                                .clicked()
                                            {
                                                self.status_message = Some(format!(
                                                    "Serve settings clicked for '{}'.",
                                                    project.name
                                                ));
                                            }
                                            ui.add_space(2.0);
                                            if self.nav == Nav::Archived {
                                                if ui.add(brand_button("Unarchive")).clicked() {
                                                    if let Err(err) = self.unarchive_project(&project.name) {
                                                        self.project_action_error = Some(err);
                                                    }
                                                }
                                            } else if self.nav == Nav::Bin {
                                                if ui.add(brand_button("Restore")).clicked() {
                                                    if let Err(err) = self.restore_project(&project.name) {
                                                        self.project_action_error = Some(err);
                                                    }
                                                }
                                            } else if project.status == "active" {
                                                let _ = ui.add(brand_button("Serve"));
                                            }
                                        });
                                    });
                                    ui.horizontal_wrapped(|ui| {
                                        let framework_label = map_framework_label(&project.project_type);
                                        ui.label(egui::RichText::new(framework_label).strong());
                                        ui.label("•");
                                        ui.label(egui::RichText::new(&project.main_path).italics());
                                    });
                                });
                                if card_response.response.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if card_response.response.clicked() {
                                    self.open_project_details(&project.name);
                                }
                                ui.add_space(6.0);
                            }
                        });
                    }
                    Nav::About => {
                        ui.label(support_page_body(SupportPage::About));
                    }
                    Nav::Feedback => {
                        ui.label(support_page_body(SupportPage::Feedback));
                    }
                    Nav::PrivacyPolicy => {
                        ui.label(support_page_body(SupportPage::PrivacyPolicy));
                    }
                }

                if let Some(action_error) = &self.project_action_error {
                    ui.colored_label(Color32::LIGHT_RED, action_error);
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
                .default_size(Vec2::new(360.0, 170.0))
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Theme");
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button("Close").clicked() {
                                close_theme_popup = true;
                            }
                        });
                    });

                    ui.add_space(6.0);
                    ui.horizontal_centered(|ui| {
                        let system_response = framework_card(
                            ui,
                            self.theme_mode == AppThemeMode::System,
                            "🖥",
                            "System",
                        );
                        if system_response.clicked() {
                            self.theme_mode = AppThemeMode::System;
                        }

                        ui.add_space(6.0);

                        let dark_response = framework_card(
                            ui,
                            self.theme_mode == AppThemeMode::Dark,
                            "🌙",
                            "Dark",
                        );
                        if dark_response.clicked() {
                            self.theme_mode = AppThemeMode::Dark;
                        }

                        ui.add_space(6.0);

                        let light_response = framework_card(
                            ui,
                            self.theme_mode == AppThemeMode::Light,
                            "☀",
                            "Light",
                        );
                        if light_response.clicked() {
                            self.theme_mode = AppThemeMode::Light;
                        }
                    });
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
            let modal_size = if self.create_modal_step == CreateModalStep::Framework {
                Vec2::new(360.0, 212.0)
            } else {
                Vec2::new(360.0, 250.0)
            };
            egui::Window::new("Create Project")
                .order(egui::Order::Foreground)
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .movable(true)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .default_size(modal_size)
                .min_size(modal_size)
                .max_size(modal_size)
                .show(ctx, |ui| {
                    ui.spacing_mut().item_spacing.y = 6.0;

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

                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                if ui.button("Cancel").clicked() {
                                    close_modal = true;
                                }
                                if ui.add(brand_button("Next")).clicked() {
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
                                let submit_label = if matches!(self.modal_mode, ModalMode::Edit { .. }) {
                                    "Save"
                                } else {
                                    "Create"
                                };
                                if ui.add(brand_button(submit_label)).clicked() {
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
            if !self.create_modal_open {
                self.modal_mode = ModalMode::Create;
            }
        }

        if self.project_details_open {
            let mut open = self.project_details_open;
            egui::Window::new("Project Details")
                .order(egui::Order::Foreground)
                .open(&mut open)
                .collapsible(false)
                .resizable(true)
                .default_size(Vec2::new(520.0, 360.0))
                .min_size(Vec2::new(440.0, 280.0))
                .show(ctx, |ui| {
                    if let Some(project) = self.selected_project() {
                        ui.heading(&project.name);
                        ui.add_space(6.0);
                        ui.label(format!("Type: {}", map_framework_label(&project.project_type)));
                        ui.label(format!("Path: {}", project.main_path));
                        ui.label(format!("Status: {}", project.status));
                        ui.label(format!("Created: {}", project.created_on));
                        ui.label(format!("Edited: {}", project.edited_on));
                        ui.add_space(8.0);
                        ui.label(RichText::new("Builds").strong());
                        if project.builds.is_empty() {
                            ui.label("No builds yet.");
                        } else {
                            for build in &project.builds {
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(RichText::new(&build.name).strong());
                                    ui.label("•");
                                    ui.label(&build.path);
                                });
                            }
                        }
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if ui.add(brand_button("Serve")).clicked() {
                                self.status_message =
                                    Some(format!("Serve clicked for '{}'.", project.name));
                            }
                            if ui.button("Edit").clicked() {
                                self.project_details_open = false;
                                self.begin_edit_project(&project.name);
                            }
                        });
                    } else {
                        ui.label("Project not found.");
                    }
                });
            self.project_details_open = open && self.selected_project().is_some();
            if !self.project_details_open {
                self.selected_project_name = None;
            }
        }

        if self.empty_bin_confirm_open {
            let mut open = self.empty_bin_confirm_open;
            let mut confirm_empty = false;
            let mut close_confirm = false;
            egui::Window::new("Confirm Empty Bin")
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .default_size(Vec2::new(420.0, 120.0))
                .show(ctx, |ui| {
                    ui.label("This will permanently delete all projects currently in Bin. This action cannot be undone.");
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() {
                            confirm_empty = true;
                        }
                        if ui.add(brand_button("No")).clicked() {
                            close_confirm = true;
                        }
                    });
                });

            if confirm_empty {
                if let Err(err) = self.empty_bin() {
                    self.project_action_error = Some(err);
                }
                open = false;
            }

            if close_confirm {
                open = false;
            }

            self.empty_bin_confirm_open = open;
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

    let content = format!("{}\n\n{}", icon, label);
    let response = ui.add_sized(
        card_size,
        Button::new(RichText::new(content).size(16.0).strong())
            .fill(ui.style().visuals.panel_fill)
            .stroke(stroke),
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
                    Nav::Home => project.status == "active",
                    Nav::Archived => project.status == "archived",
                    Nav::Bin => project.status == "deleted",
                    Nav::About | Nav::Feedback | Nav::PrivacyPolicy => false,
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

        let editing_original = match &self.modal_mode {
            ModalMode::Edit { original_name } => Some(original_name.as_str()),
            ModalMode::Create => None,
        };

        if self.projects.iter().any(|project| {
            project.name.eq_ignore_ascii_case(name)
                && Some(project.name.as_str()) != editing_original
        }) {
            return Err(format!("A project named '{name}' already exists."));
        }

        let today = current_date();
        match &self.modal_mode {
            ModalMode::Create => {
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
            }
            ModalMode::Edit { original_name } => {
                let project = self
                    .projects
                    .iter_mut()
                    .find(|project| project.name == *original_name)
                    .ok_or_else(|| format!("Project '{}' not found for edit.", original_name))?;
                project.name = name.to_owned();
                project.project_type = self.create_form.project_type.storage_value().to_owned();
                project.main_path = main_path.to_owned();
                project.edited_on = today;
            }
        }

        self.persist_projects()?;
        if matches!(self.modal_mode, ModalMode::Edit { .. }) {
            Ok(format!("Project '{name}' updated."))
        } else {
            Ok(format!("Project '{name}' created."))
        }
    }

    fn begin_edit_project(&mut self, project_name: &str) {
        if let Some(project) = self.projects.iter().find(|project| project.name == project_name) {
            self.create_form.name = project.name.clone();
            self.create_form.main_path = project.main_path.clone();
            self.create_form.project_type =
                ProjectType::from_storage(&project.project_type).unwrap_or(ProjectType::Android);
            self.selected_framework = self.create_form.project_type;
            self.create_modal_step = CreateModalStep::Form;
            self.modal_mode = ModalMode::Edit {
                original_name: project.name.clone(),
            };
            self.form_error = None;
            self.create_modal_open = true;
        }
    }

    fn open_project_details(&mut self, project_name: &str) {
        self.selected_project_name = Some(project_name.to_owned());
        self.project_details_open = true;
    }

    fn selected_project(&self) -> Option<ProjectRecord> {
        let selected_name = self.selected_project_name.as_ref()?;
        self.projects
            .iter()
            .find(|project| project.name == *selected_name)
            .cloned()
    }

    fn archive_project(&mut self, project_name: &str) -> Result<(), String> {
        let project = self
            .projects
            .iter_mut()
            .find(|project| project.name == project_name)
            .ok_or_else(|| format!("Project '{project_name}' not found."))?;
        project.status = "archived".to_owned();
        project.edited_on = current_date();
        self.persist_projects()?;
        self.status_message = Some(format!("Project '{project_name}' archived."));
        Ok(())
    }

    fn unarchive_project(&mut self, project_name: &str) -> Result<(), String> {
        let project = self
            .projects
            .iter_mut()
            .find(|project| project.name == project_name)
            .ok_or_else(|| format!("Project '{project_name}' not found."))?;
        project.status = "active".to_owned();
        project.edited_on = current_date();
        self.persist_projects()?;
        self.status_message = Some(format!("Project '{project_name}' unarchived."));
        Ok(())
    }

    fn bin_project(&mut self, project_name: &str) -> Result<(), String> {
        let project = self
            .projects
            .iter_mut()
            .find(|project| project.name == project_name)
            .ok_or_else(|| format!("Project '{project_name}' not found."))?;
        project.status = "deleted".to_owned();
        project.edited_on = current_date();
        self.persist_projects()?;
        self.status_message = Some(format!("Project '{project_name}' moved to Bin."));
        Ok(())
    }

    fn restore_project(&mut self, project_name: &str) -> Result<(), String> {
        let project = self
            .projects
            .iter_mut()
            .find(|project| project.name == project_name)
            .ok_or_else(|| format!("Project '{project_name}' not found."))?;
        project.status = "active".to_owned();
        project.edited_on = current_date();
        self.persist_projects()?;
        self.status_message = Some(format!("Project '{project_name}' restored from Bin."));
        Ok(())
    }

    fn permanent_delete_project(&mut self, project_name: &str) -> Result<(), String> {
        let before = self.projects.len();
        self.projects.retain(|project| project.name != project_name);
        if self.projects.len() == before {
            return Err(format!("Project '{project_name}' not found."));
        }
        self.persist_projects()?;
        self.status_message = Some(format!("Project '{project_name}' permanently deleted."));
        Ok(())
    }

    fn bulk_bin_projects(&mut self, project_names: &HashSet<String>) -> Result<usize, String> {
        if project_names.is_empty() {
            return Ok(0);
        }

        let today = current_date();
        let mut count = 0;
        for project in &mut self.projects {
            if project_names.contains(&project.name) {
                project.status = "deleted".to_owned();
                project.edited_on = today.clone();
                count += 1;
            }
        }

        self.persist_projects()?;
        Ok(count)
    }

    fn bulk_unarchive_projects(&mut self, project_names: &HashSet<String>) -> Result<usize, String> {
        if project_names.is_empty() {
            return Ok(0);
        }

        let today = current_date();
        let mut count = 0;
        for project in &mut self.projects {
            if project_names.contains(&project.name) {
                project.status = "active".to_owned();
                project.edited_on = today.clone();
                count += 1;
            }
        }

        self.persist_projects()?;
        Ok(count)
    }

    fn bulk_restore_projects(&mut self, project_names: &HashSet<String>) -> Result<usize, String> {
        if project_names.is_empty() {
            return Ok(0);
        }

        let today = current_date();
        let mut count = 0;
        for project in &mut self.projects {
            if project_names.contains(&project.name) {
                project.status = "active".to_owned();
                project.edited_on = today.clone();
                count += 1;
            }
        }

        self.persist_projects()?;
        Ok(count)
    }

    fn bulk_permanent_delete_projects(
        &mut self,
        project_names: &HashSet<String>,
    ) -> Result<usize, String> {
        if project_names.is_empty() {
            return Ok(0);
        }

        let before = self.projects.len();
        self.projects
            .retain(|project| !project_names.contains(&project.name));
        let count = before.saturating_sub(self.projects.len());
        self.persist_projects()?;
        Ok(count)
    }

    fn empty_bin(&mut self) -> Result<(), String> {
        let before = self.projects.len();
        self.projects.retain(|project| project.status != "deleted");
        let removed = before.saturating_sub(self.projects.len());
        if removed == 0 {
            self.status_message = Some("Bin is already empty.".to_owned());
            return Ok(());
        }
        self.persist_projects()?;
        self.status_message = Some(format!("Removed {removed} project(s) from Bin."));
        Ok(())
    }

    fn persist_projects(&self) -> Result<(), String> {
        let path = self
            .projects_file_path
            .as_ref()
            .ok_or_else(|| "Cannot determine config directory for Projects.json".to_owned())?;
        save_projects(path, &self.projects)
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

fn support_page_row(ui: &mut egui::Ui, dark: bool, icon: IconKind, label: &str) -> egui::Response {
    ui.horizontal(|ui| {
        ui.add(icon_image(themed_icon(dark, icon), 16.0));
        ui.add(
            Button::new(label)
                .frame(false)
                .fill(Color32::TRANSPARENT),
        )
    })
    .inner
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
    Bin,
    MoreVert,
    Theme,
    PanelHide,
    PanelShow,
    Sort,
    Clear,
    About,
    Feedback,
    Privacy,
    Broadcast,
    ActionEdit,
    ActionArchive,
    ActionDelete,
}

fn themed_icon(dark: bool, icon: IconKind) -> ImageSource<'static> {
    match (dark, icon) {
        (true, IconKind::Home) => egui::include_image!("../assets/icons/home_dark.svg"),
        (false, IconKind::Home) => egui::include_image!("../assets/icons/home_light.svg"),
        (true, IconKind::Archive) => egui::include_image!("../assets/icons/archive_dark.svg"),
        (false, IconKind::Archive) => egui::include_image!("../assets/icons/archive_light.svg"),
        (true, IconKind::Bin) => egui::include_image!("../assets/icons/bin_dark.svg"),
        (false, IconKind::Bin) => egui::include_image!("../assets/icons/bin_light.svg"),
        (true, IconKind::MoreVert) => egui::include_image!("../assets/icons/more_vert_dark.svg"),
        (false, IconKind::MoreVert) => egui::include_image!("../assets/icons/more_vert_light.svg"),
        (true, IconKind::Theme) => egui::include_image!("../assets/icons/theme_dark.svg"),
        (false, IconKind::Theme) => egui::include_image!("../assets/icons/theme_light.svg"),
        (true, IconKind::PanelHide) => egui::include_image!("../assets/icons/panel_hide_dark.svg"),
        (false, IconKind::PanelHide) => egui::include_image!("../assets/icons/panel_hide_light.svg"),
        (true, IconKind::PanelShow) => egui::include_image!("../assets/icons/panel_show_dark.svg"),
        (false, IconKind::PanelShow) => egui::include_image!("../assets/icons/panel_show_light.svg"),
        (true, IconKind::Sort) => egui::include_image!("../assets/icons/sort_dark.svg"),
        (false, IconKind::Sort) => egui::include_image!("../assets/icons/sort_light.svg"),
        (true, IconKind::Clear) => egui::include_image!("../assets/icons/clear_dark.svg"),
        (false, IconKind::Clear) => egui::include_image!("../assets/icons/clear_light.svg"),
        (true, IconKind::About) => egui::include_image!("../assets/icons/about_dark.svg"),
        (false, IconKind::About) => egui::include_image!("../assets/icons/about_light.svg"),
        (true, IconKind::Feedback) => egui::include_image!("../assets/icons/feedback_dark.svg"),
        (false, IconKind::Feedback) => egui::include_image!("../assets/icons/feedback_light.svg"),
        (true, IconKind::Privacy) => egui::include_image!("../assets/icons/privacy_dark.svg"),
        (false, IconKind::Privacy) => egui::include_image!("../assets/icons/privacy_light.svg"),
        (true, IconKind::Broadcast) => egui::include_image!("../assets/icons/broadcast_dark.svg"),
        (false, IconKind::Broadcast) => egui::include_image!("../assets/icons/broadcast_light.svg"),
        (true, IconKind::ActionEdit) => egui::include_image!("../assets/icons/action_edit_dark.svg"),
        (false, IconKind::ActionEdit) => egui::include_image!("../assets/icons/action_edit_light.svg"),
        (true, IconKind::ActionArchive) => egui::include_image!("../assets/icons/action_archive_dark.svg"),
        (false, IconKind::ActionArchive) => egui::include_image!("../assets/icons/action_archive_light.svg"),
        (_, IconKind::ActionDelete) => egui::include_image!("../assets/icons/action_delete_red.svg"),
    }
}

fn support_page_body(page: SupportPage) -> &'static str {
    match page {
        SupportPage::About => {
            "BuildBridge is a native desktop app for organizing project build outputs."
        }
        SupportPage::Feedback => {
            "Feedback: please share bugs and feature ideas in your issue tracker."
        }
        SupportPage::PrivacyPolicy => {
            "Privacy Policy: project data is saved locally in your OS config folder."
        }
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
