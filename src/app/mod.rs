mod bridge_status;
mod project_page;
mod sidebar;
mod terminal;
mod theme_popup;

use crate::config::{init_app_config, init_preferences, save_preferences, AppConfig, Preferences};

use crate::icons::{icon_button, icon_image, themed_icon, IconKind};
use crate::models::{BuildEntry, CreateProjectForm, ProjectRecord, ProjectType};
use crate::storage::{current_date, init_storage, save_projects};
use chrono::{DateTime, Local};
use directories::UserDirs;
use eframe::egui::{
    self, Button, Color32, ComboBox, RichText, TextEdit, ThemePreference, TopBottomPanel, Vec2,
};
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

const REALTIME_SCAN_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Nav {
    Home,
    Archived,
    Bin,
    About,
    Feedback,
    PrivacyPolicy,
    Debug,
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
enum ProjectSortBy {
    Title,
    DateCreated,
    ProjectType,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ProjectSortOrder {
    Asc,
    Desc,
}

#[derive(Clone, PartialEq, Eq)]
enum ProjectConfirmAction {
    MoveToBin { project_name: String },
    PermanentDelete { project_name: String },
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

impl ProjectSortBy {
    fn from_pref(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "date created" | "date_created" | "created" => Self::DateCreated,
            "project type" | "project_type" | "type" => Self::ProjectType,
            _ => Self::Title,
        }
    }

    fn as_pref(self) -> &'static str {
        match self {
            Self::Title => "Title",
            Self::DateCreated => "Date Created",
            Self::ProjectType => "Project Type",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Title => "Project Title",
            Self::DateCreated => "Date Created",
            Self::ProjectType => "Project Type",
        }
    }
}

impl ProjectSortOrder {
    fn from_pref(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "desc" | "descending" => Self::Desc,
            _ => Self::Asc,
        }
    }

    fn as_pref(self) -> &'static str {
        match self {
            Self::Asc => "Asc",
            Self::Desc => "Desc",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Asc => "Ascending",
            Self::Desc => "Descending",
        }
    }
}

pub struct ProjectDashboardApp {
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
    status_message_tint: Option<Color32>,
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
    pending_project_action: Option<ProjectConfirmAction>,
    selected_project_name: Option<String>,
    selected_build_index: Option<usize>,
    selected_artifact_type: String,
    bridge_status_expanded: bool,
    real_time_enabled: bool,
    terminal_link_popup_open: bool,
    terminal_link_target: Option<String>,
    build_location_popup_open: bool,
    build_location_popup_path: Option<String>,
    project_path_popup_open: bool,
    project_path_popup_path: Option<String>,
    project_sort_by: ProjectSortBy,
    project_sort_order: ProjectSortOrder,
    last_realtime_scan: Option<Instant>,
    app_config: AppConfig,
    app_config_file_path: Option<PathBuf>,
    app_config_error: Option<String>,
    preferences: Preferences,
    preferences_file_path: Option<PathBuf>,
    terminal_lines: Vec<String>,
    terminal_rx: Option<Receiver<String>>,
    serve_child: Option<Child>,
}

impl Default for ProjectDashboardApp {
    fn default() -> Self {
        let (projects_file_path, projects, storage_error) = init_storage();
        let (app_config_file_path, app_config, mut app_config_error) = init_app_config();
        let (preferences_file_path, preferences, pref_error) = init_preferences();

        if let Some(err) = pref_error {
            let combined = match app_config_error {
                Some(existing) => format!("{existing}\n{err}"),
                None => err,
            };
            app_config_error = Some(combined);
        }

        let theme_mode = match preferences.settings.theme.to_lowercase().as_str() {
            "dark" => AppThemeMode::Dark,
            "light" => AppThemeMode::Light,
            _ => AppThemeMode::System,
        };

        let sidebar_width = preferences.config.side_pane.width.unwrap_or(260.0);
        let sidebar_visible = !preferences.config.side_pane.collapsed;
        let bridge_status_expanded = !preferences.project_settings.build_status_collapse;
        let real_time_enabled = preferences.project_settings.real_time;
        let project_sort_by =
            ProjectSortBy::from_pref(&preferences.config.project_list.sort.sort_by);
        let project_sort_order =
            ProjectSortOrder::from_pref(&preferences.config.project_list.sort.order);

        Self {
            zoom_applied: false,
            mica_attempted: false,
            mica_error: None,
            nav: Nav::Home,
            sidebar_visible,
            sidebar_width,
            sidebar_animated_width: sidebar_width,
            create_modal_open: false,
            create_modal_step: CreateModalStep::Framework,
            selected_framework: ProjectType::Android,
            search_text: String::new(),
            projects_file_path,
            projects,
            create_form: CreateProjectForm::default(),
            form_error: None,
            status_message: None,
            status_message_tint: None,
            storage_error,
            theme_popup_open: false,
            theme_mode,
            project_action_error: None,
            modal_mode: ModalMode::Create,
            archive_select_mode: false,
            archive_selected: HashSet::new(),
            bin_select_mode: false,
            bin_selected: HashSet::new(),
            empty_bin_confirm_open: false,
            pending_project_action: None,
            selected_project_name: None,
            selected_build_index: None,
            selected_artifact_type: "Type".to_owned(),
            bridge_status_expanded,
            real_time_enabled,
            terminal_link_popup_open: false,
            terminal_link_target: None,
            build_location_popup_open: false,
            build_location_popup_path: None,
            project_path_popup_open: false,
            project_path_popup_path: None,
            project_sort_by,
            project_sort_order,
            last_realtime_scan: None,
            app_config,
            app_config_file_path,
            app_config_error,
            preferences,
            preferences_file_path,
            terminal_lines: Vec::new(),
            terminal_rx: None,
            serve_child: None,
        }
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

        if !matches!(self.nav, Nav::Home | Nav::Archived | Nav::Bin | Nav::Debug)
            && self.selected_project_name.is_some()
        {
            self.close_project_details();
        }
        if self.selected_project_name.is_some() && self.selected_project().is_none() {
            self.close_project_details();
        }

        self.maybe_refresh_realtime_builds();
        self.poll_terminal_output();

        let dark = ctx.style().visuals.dark_mode;

        self.render_sidebar(ctx, dark);
        self.render_status_bar(ctx, dark);
        self.render_bridge_panel(ctx, dark);

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.nav {
                Nav::Home | Nav::Archived | Nav::Bin => {
                    self.render_project_page(ctx, dark);
                }
                Nav::Debug => {
                    self.render_debug_page(ui, dark);
                }
                _ => {
                    self.render_support_page(ui, dark);
                }
            }
        });

        self.render_theme_popup(ctx);
        self.render_create_modal(ctx);
        self.render_empty_bin_confirm(ctx);
        self.render_project_action_confirm(ctx);
        self.render_build_location_popup(ctx);
        self.render_project_path_popup(ctx);

        if let Some(err) = self.app_config_error.clone() {
            self.render_error_toast(ctx, &err);
        }

        // Sync and persist preferences
        let theme_str = match self.theme_mode {
            AppThemeMode::System => "system",
            AppThemeMode::Dark => "dark",
            AppThemeMode::Light => "light",
        };

        let mut changed = false;
        if self.preferences.settings.theme != theme_str {
            self.preferences.settings.theme = theme_str.to_owned();
            changed = true;
        }
        if self.preferences.config.side_pane.width != Some(self.sidebar_width) {
            self.preferences.config.side_pane.width = Some(self.sidebar_width);
            changed = true;
        }
        if self.preferences.config.side_pane.collapsed != !self.sidebar_visible {
            self.preferences.config.side_pane.collapsed = !self.sidebar_visible;
            changed = true;
        }
        if self.preferences.project_settings.build_status_collapse != !self.bridge_status_expanded {
            self.preferences.project_settings.build_status_collapse = !self.bridge_status_expanded;
            changed = true;
        }
        if self.preferences.project_settings.real_time != self.real_time_enabled {
            self.preferences.project_settings.real_time = self.real_time_enabled;
            changed = true;
        }
        if self.preferences.config.project_list.sort.sort_by != self.project_sort_by.as_pref() {
            self.preferences.config.project_list.sort.sort_by =
                self.project_sort_by.as_pref().to_owned();
            changed = true;
        }
        if self.preferences.config.project_list.sort.order != self.project_sort_order.as_pref() {
            self.preferences.config.project_list.sort.order =
                self.project_sort_order.as_pref().to_owned();
            changed = true;
        }

        if changed {
            if let Err(err) = self.persist_preferences() {
                self.app_config_error = Some(err);
            }
        }
    }
}

impl ProjectDashboardApp {
    fn clear_status_message(&mut self) {
        self.status_message = None;
        self.status_message_tint = None;
    }

    fn set_status_message(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
        self.status_message_tint = None;
    }

    fn set_status_message_tinted(&mut self, message: impl Into<String>, tint: Color32) {
        self.status_message = Some(message.into());
        self.status_message_tint = Some(tint);
    }

    fn poll_terminal_output(&mut self) {
        let Some(rx) = self.terminal_rx.as_ref() else {
            return;
        };

        while let Ok(line) = rx.try_recv() {
            self.terminal_lines.push(line);
        }
    }

    fn start_bridge_serve(&mut self, project: &ProjectRecord) -> Result<(), String> {
        let stream_type = project
            .stream_type
            .as_deref()
            .unwrap_or("localhost-token");

        if !stream_type.starts_with("localhost") {
            self.set_status_message("Serve mode not implemented yet.");
            return Ok(());
        }

        if let Some(mut child) = self.serve_child.take() {
            let _ = child.kill();
        }

        self.terminal_lines.clear();
        self.terminal_rx = None;
        self.terminal_lines.push(format!(
            "PS > serve \"{}\" --mode bridge",
            project.name
        ));

        let projects_path = self
            .projects_file_path
            .as_ref()
            .ok_or_else(|| "Projects file path unavailable.".to_owned())?;
        let agent_path = locate_bridge_agent()?;
        let token = if stream_type.contains("token") {
            Some(generate_token())
        } else {
            None
        };

        let mut command = Command::new(agent_path);
        command
            .arg("--projects")
            .arg(projects_path)
            .arg("--project")
            .arg(&project.name)
            .arg("--port")
            .arg("4000")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(token) = token.as_deref() {
            command.arg("--token").arg(token);
            self.terminal_lines
                .push(format!("Token: {token}"));
        }

        let mut child = command.spawn().map_err(|err| err.to_string())?;
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let (tx, rx) = mpsc::channel();
        if let Some(stdout) = stdout {
            spawn_reader_thread(stdout, tx.clone());
        }
        if let Some(stderr) = stderr {
            spawn_reader_thread(stderr, tx.clone());
        }

        self.serve_child = Some(child);
        self.terminal_rx = Some(rx);
        Ok(())
    }

    fn render_status_bar(&mut self, ctx: &egui::Context, dark: bool) {
        TopBottomPanel::bottom("status_bar")
            .exact_height(24.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (icon, text) = if let Some(message) = &self.status_message {
                        (IconKind::Bell, message.as_str())
                    } else {
                        (IconKind::BellSlash, "No new notifications")
                    };
                    let message_color = if self.status_message.is_some() {
                        self.status_message_tint
                            .unwrap_or(Color32::from_rgb(2, 110, 193))
                    } else {
                        Color32::from_rgb(2, 110, 193)
                    };
                    ui.add(icon_image(themed_icon(dark, icon), 16.0));
                    ui.add_space(6.0);
                    ui.colored_label(message_color, text);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.status_message.is_some() {
                            let clear_button =
                                icon_button(themed_icon(dark, IconKind::Clear), 14.0).frame(false);
                            let clear_response = ui
                                .add(clear_button)
                                .on_hover_text("Clear notification");
                            if clear_response.clicked() {
                                self.clear_status_message();
                            }
                        }
                    });
                });
            });
    }

    fn render_bridge_panel(&mut self, ctx: &egui::Context, dark: bool) {
        let in_project_page_for_panels = matches!(self.nav, Nav::Home | Nav::Archived | Nav::Bin)
            && self.selected_project_name.is_some();
        if in_project_page_for_panels {
            let panel = if self.bridge_status_expanded {
                TopBottomPanel::bottom("bridge_status")
                    .resizable(true)
                    .min_height(128.0)
                    .max_height(200.0)
                    .default_height(144.0)
            } else {
                TopBottomPanel::bottom("bridge_status")
                    .resizable(false)
                    .exact_height(24.0)
            };
            panel.show(ctx, |ui| {
                if let Some(project) = self.selected_project() {
                    self.render_bridge_status(ui, dark, &project);
                }
            });
        }
    }

    fn maybe_refresh_realtime_builds(&mut self) {
        if !self.real_time_enabled {
            return;
        }

        let project_name = match self.selected_project_name.clone() {
            Some(name) => name,
            None => return,
        };

        let now = Instant::now();
        if let Some(last) = self.last_realtime_scan {
            if now.duration_since(last) < REALTIME_SCAN_INTERVAL {
                return;
            }
        }

        self.last_realtime_scan = Some(now);
        self.refresh_android_builds(&project_name);
    }

    fn render_create_modal(&mut self, ctx: &egui::Context) {
        if !self.create_modal_open {
            return;
        }

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
                                        self.set_status_message(success_message);
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

    fn render_empty_bin_confirm(&mut self, ctx: &egui::Context) {
        if !self.empty_bin_confirm_open {
            return;
        }

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

    fn render_project_action_confirm(&mut self, ctx: &egui::Context) {
        let action = match self.pending_project_action.clone() {
            Some(action) => action,
            None => return,
        };

        let overlay_rect = ctx.viewport_rect();
        egui::Area::new("project_action_confirm_overlay".into())
            .order(egui::Order::Middle)
            .fixed_pos(overlay_rect.left_top())
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(overlay_rect.size(), egui::Sense::click());
                ui.painter().rect_filled(
                    rect,
                    0.0,
                    Color32::from_rgba_premultiplied(0, 0, 0, 140),
                );
            });

        let danger_text = Color32::from_rgb(255, 0, 79);
        let (title, message, confirm_label) = match &action {
            ProjectConfirmAction::MoveToBin { project_name } => (
                "Move to Bin",
                format!("Move '{project_name}' to Bin? You can restore it later."),
                "Move to Bin",
            ),
            ProjectConfirmAction::PermanentDelete { project_name } => (
                "Permanent Delete",
                format!(
                    "This will permanently delete '{project_name}'. This action cannot be undone."
                ),
                "Permanent Delete",
            ),
        };

        let mut open = true;
        let mut confirm = false;
        let mut close_confirm = false;
        egui::Window::new(title)
            .order(egui::Order::Foreground)
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .default_size(Vec2::new(420.0, 130.0))
            .show(ctx, |ui| {
                ui.label(message);
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(Button::new(RichText::new(confirm_label).color(danger_text)))
                        .clicked()
                    {
                        confirm = true;
                    }
                    if ui.add(brand_button("Cancel")).clicked() {
                        close_confirm = true;
                    }
                });
            });

        if confirm {
            let result = match action {
                ProjectConfirmAction::MoveToBin { project_name } => self.bin_project(&project_name),
                ProjectConfirmAction::PermanentDelete { project_name } => {
                    self.permanent_delete_project(&project_name)
                }
            };
            if let Err(err) = result {
                self.project_action_error = Some(err);
            }
        }

        if confirm || close_confirm || !open {
            self.pending_project_action = None;
        }
    }

    fn render_build_location_popup(&mut self, ctx: &egui::Context) {
        if !self.build_location_popup_open {
            return;
        }

        let path = match self.build_location_popup_path.clone() {
            Some(path) => path,
            None => {
                self.build_location_popup_open = false;
                return;
            }
        };

        let mut open = self.build_location_popup_open;
        let mut close_popup = false;
        let mut open_location = false;
        let mut copy_path = false;
        egui::Window::new("Build Location")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .default_size(Vec2::new(520.0, 150.0))
            .show(ctx, |ui| {
                ui.label("Location");
                ui.add_space(6.0);
                ui.label(
                    RichText::new(&path)
                        .italics()
                        .size(12.0)
                        .color(ui.style().visuals.weak_text_color()),
                );
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.add(brand_button("Open location")).clicked() {
                        open_location = true;
                    }
                    if ui.button("Copy path").clicked() {
                        copy_path = true;
                    }
                    if ui.button("Close").clicked() {
                        close_popup = true;
                    }
                });
            });

        if open_location {
            if let Err(err) = self.open_path_location(&path) {
                self.project_action_error = Some(err);
            }
            close_popup = true;
        }

        if copy_path {
            ctx.copy_text(path.clone());
            close_popup = true;
        }

        if close_popup || !open {
            self.build_location_popup_path = None;
            open = false;
        }
        self.build_location_popup_open = open;
    }

    fn render_project_path_popup(&mut self, ctx: &egui::Context) {
        if !self.project_path_popup_open {
            return;
        }

        let path = match self.project_path_popup_path.clone() {
            Some(path) => path,
            None => {
                self.project_path_popup_open = false;
                return;
            }
        };

        let mut open = self.project_path_popup_open;
        let mut close_popup = false;
        let mut open_location = false;
        let mut copy_path = false;
        egui::Window::new("Project Location")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .default_size(Vec2::new(520.0, 160.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Project Location");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("X").on_hover_text("Close").clicked() {
                            close_popup = true;
                        }
                    });
                });
                ui.add_space(6.0);
                ui.label("Location");
                ui.add_space(6.0);
                ui.label(
                    RichText::new(&path)
                        .italics()
                        .size(12.0)
                        .color(ui.style().visuals.weak_text_color()),
                );
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.add(brand_button("Open")).clicked() {
                        open_location = true;
                    }
                    if ui.button("Copy").clicked() {
                        copy_path = true;
                    }
                });
            });

        if open_location {
            if let Err(err) = self.open_folder_path(&path) {
                self.project_action_error = Some(err);
            }
            close_popup = true;
        }

        if copy_path {
            ctx.copy_text(path.clone());
            close_popup = true;
        }

        if close_popup || !open {
            self.project_path_popup_path = None;
            open = false;
        }
        self.project_path_popup_open = open;
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
                    added_file: None,
                    stream_type: Some("localhost-token".to_owned()),
                    star: None,
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

    fn set_project_stream_type(
        &mut self,
        project_name: &str,
        stream_type: &str,
    ) -> Result<(), String> {
        let project = self
            .projects
            .iter_mut()
            .find(|project| project.name == project_name)
            .ok_or_else(|| format!("Project '{project_name}' not found."))?;
        if project.stream_type.as_deref() == Some(stream_type) {
            return Ok(());
        }
        project.stream_type = Some(stream_type.to_owned());
        self.persist_projects()?;
        Ok(())
    }

    fn open_project_details(&mut self, project_name: &str) {
        self.selected_project_name = Some(project_name.to_owned());
        self.selected_build_index = None;
        self.selected_artifact_type = "Type".to_owned();
        self.bridge_status_expanded = false;
        self.refresh_android_builds(project_name);
    }

    fn close_project_details(&mut self) {
        self.selected_project_name = None;
        self.selected_build_index = None;
        self.selected_artifact_type = "Type".to_owned();
        self.bridge_status_expanded = false;
        self.terminal_link_popup_open = false;
        self.terminal_link_target = None;
        self.project_path_popup_open = false;
        self.project_path_popup_path = None;
    }

    fn selected_project(&self) -> Option<ProjectRecord> {
        let selected_name = self.selected_project_name.as_ref()?;
        self.projects
            .iter()
            .find(|project| project.name == *selected_name)
            .cloned()
    }

    fn refresh_android_builds(&mut self, project_name: &str) {
        if let Err(err) = self.refresh_android_builds_with_result(project_name) {
            self.project_action_error = Some(err);
        }
    }

    fn refresh_android_builds_with_result(&mut self, project_name: &str) -> Result<usize, String> {
        let (project_type, main_path, existing_builds, existing_star) = match self
            .projects
            .iter()
            .find(|project| project.name == project_name)
        {
            Some(project) => (
                ProjectType::from_storage(&project.project_type).unwrap_or(ProjectType::Android),
                project.main_path.clone(),
                project.builds.clone(),
                project.star.clone(),
            ),
            None => return Err(format!("Project '{project_name}' not found.")),
        };
        if project_type != ProjectType::Android {
            return Ok(existing_builds.len());
        }

        let builds = self.detect_android_apk_builds(Path::new(&main_path))?;
        let build_count = builds.len();

        let mut starred_path = existing_star.clone();
        if starred_path.is_none() {
            starred_path = existing_builds
                .iter()
                .find(|build| build.starred)
                .map(|build| build.path.clone());
        }
        let mut builds = builds;
        if let Some(path) = &starred_path {
            if let Some(build) = builds.iter_mut().find(|build| build.path == *path) {
                build.starred = true;
            }
        }

        let star_changed = starred_path != existing_star;
        if existing_builds != builds || star_changed {
            if let Some(project) = self
                .projects
                .iter_mut()
                .find(|project| project.name == project_name)
            {
                project.builds = builds;
                project.star = starred_path;
            }
            if let Err(err) = self.persist_projects() {
                return Err(err);
            }
        }
        Ok(build_count)
    }

    fn detect_android_apk_builds(&self, project_root: &Path) -> Result<Vec<BuildEntry>, String> {
        let output_dir = project_root.join("app").join("build").join("outputs");
        if !output_dir.exists() {
            return Ok(Vec::new());
        }

        let mut files = Vec::new();
        self.collect_build_files(&output_dir, &mut files)?;

        let mut builds = Vec::new();
        for path in files {
            let ext = path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            if ext != "apk" {
                continue;
            }

            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("build.apk")
                .to_owned();
            let created_on = file_timestamp(&path);
            builds.push(BuildEntry {
                name,
                path: path.display().to_string(),
                created_on,
                starred: false,
            });
        }

        builds.sort_by(|a, b| b.created_on.cmp(&a.created_on).then_with(|| a.name.cmp(&b.name)));
        Ok(builds)
    }

    fn collect_build_files(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
        for entry in fs::read_dir(dir)
            .map_err(|err| format!("Failed to read '{}': {err}", dir.display()))?
        {
            let entry = entry
                .map_err(|err| format!("Failed to read entry in '{}': {err}", dir.display()))?;
            let path = entry.path();
            if path.is_dir() {
                self.collect_build_files(&path, files)?;
            } else {
                files.push(path);
            }
        }
        Ok(())
    }

    fn open_path_location(&self, path: &str) -> Result<(), String> {
        let folder = Path::new(path)
            .parent()
            .ok_or_else(|| "Could not determine build folder path.".to_owned())?;

        #[cfg(target_os = "windows")]
        {
            Command::new("explorer")
                .arg(folder)
                .spawn()
                .map_err(|err| format!("Failed to open folder: {err}"))?;
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg(folder)
                .spawn()
                .map_err(|err| format!("Failed to open folder: {err}"))?;
        }

        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open")
                .arg(folder)
                .spawn()
                .map_err(|err| format!("Failed to open folder: {err}"))?;
        }

        Ok(())
    }

    fn open_folder_path(&self, path: &str) -> Result<(), String> {
        let folder = Path::new(path);

        #[cfg(target_os = "windows")]
        {
            Command::new("explorer")
                .arg(folder)
                .spawn()
                .map_err(|err| format!("Failed to open folder: {err}"))?;
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg(folder)
                .spawn()
                .map_err(|err| format!("Failed to open folder: {err}"))?;
        }

        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open")
                .arg(folder)
                .spawn()
                .map_err(|err| format!("Failed to open folder: {err}"))?;
        }

        Ok(())
    }

    fn open_feedback_folder(&self, project_name: &str) -> Result<(), String> {
        let user_dirs = UserDirs::new()
            .ok_or_else(|| "Unable to resolve user directories.".to_owned())?;
        let home_dir = user_dirs.home_dir();
        let folder = home_dir
            .join("Build Bridge")
            .join(project_name)
            .join("feedback");
        fs::create_dir_all(&folder).map_err(|err| {
            format!("Failed to create feedback folder '{}': {err}", folder.display())
        })?;
        self.open_folder_path(&folder.to_string_lossy())
    }

    fn add_extra_file(&mut self, project_name: &str, file_path: PathBuf) -> Result<(), String> {
        let project = self
            .projects
            .iter_mut()
            .find(|project| project.name == project_name)
            .ok_or_else(|| format!("Project '{project_name}' not found."))?;
        let path_str = file_path.display().to_string();
        if project.added_file.as_deref() == Some(path_str.as_str()) {
            return Ok(());
        }
        project.added_file = Some(path_str);
        self.persist_projects()?;
        Ok(())
    }

    fn toggle_build_star(&mut self, project_name: &str, build_path: &str) {
        let project = match self
            .projects
            .iter_mut()
            .find(|project| project.name == project_name)
        {
            Some(project) => project,
            None => return,
        };

        let was_starred = project
            .builds
            .iter()
            .find(|build| build.path == build_path)
            .map(|build| build.starred)
            .unwrap_or(false);

        for build in &mut project.builds {
            build.starred = false;
        }

        if !was_starred {
            if let Some(build) = project
                .builds
                .iter_mut()
                .find(|build| build.path == build_path)
            {
                build.starred = true;
                project.star = Some(build.path.clone());
            }
        } else {
            project.star = None;
        }

        if let Err(err) = self.persist_projects() {
            self.project_action_error = Some(err);
        }
    }

    fn filtered_projects(&self) -> Vec<ProjectRecord> {
        let query = self.search_text.to_lowercase();
        let mut filtered: Vec<ProjectRecord> = self.projects
            .iter()
            .filter(|project| {
                let nav_match = match self.nav {
                    Nav::Home => project.status == "active",
                    Nav::Archived => project.status == "archived",
                    Nav::Bin => project.status == "deleted",
                    Nav::About | Nav::Feedback | Nav::PrivacyPolicy | Nav::Debug => false,
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
            .collect();

        filtered.sort_by(|a, b| {
            let cmp = match self.project_sort_by {
                ProjectSortBy::Title => a
                    .name
                    .to_lowercase()
                    .cmp(&b.name.to_lowercase()),
                ProjectSortBy::DateCreated => a.created_on.cmp(&b.created_on),
                ProjectSortBy::ProjectType => map_framework_label(&a.project_type)
                    .to_lowercase()
                    .cmp(&map_framework_label(&b.project_type).to_lowercase()),
            };
            match self.project_sort_order {
                ProjectSortOrder::Asc => cmp,
                ProjectSortOrder::Desc => cmp.reverse(),
            }
        });

        filtered
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
        self.set_status_message(format!("Project '{project_name}' archived."));
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
        self.set_status_message(format!("Project '{project_name}' unarchived."));
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
        self.set_status_message(format!("Project '{project_name}' moved to Bin."));
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
        self.set_status_message(format!("Project '{project_name}' restored from Bin."));
        Ok(())
    }

    fn permanent_delete_project(&mut self, project_name: &str) -> Result<(), String> {
        let before = self.projects.len();
        self.projects.retain(|project| project.name != project_name);
        if self.projects.len() == before {
            return Err(format!("Project '{project_name}' not found."));
        }
        self.persist_projects()?;
        self.set_status_message(format!(
            "Project '{project_name}' permanently deleted."
        ));
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
            self.set_status_message("Bin is already empty.".to_owned());
            return Ok(());
        }
        self.persist_projects()?;
        self.set_status_message(format!("Removed {removed} project(s) from Bin."));
        Ok(())
    }

        fn persist_projects(&self) -> Result<(), String> {
        let path = self
            .projects_file_path
            .as_ref()
            .ok_or_else(|| "Cannot determine config directory for Projects.json".to_owned())?;
        save_projects(path, &self.projects)
        }

    fn persist_preferences(&self) -> Result<(), String> {
        let path = self
            .preferences_file_path
            .as_ref()
            .ok_or_else(|| "Cannot determine preferences file path.".to_owned())?;
        save_preferences(path, &self.preferences)
    }

    fn render_debug_page(&mut self, ui: &mut egui::Ui, _dark: bool) {
        ui.heading("Debug Page");
        ui.add_space(16.0);
        if ui.button("Open Config Folder").clicked() {
            if let Err(err) = self.open_config_folder() {
                self.app_config_error = Some(err);
            }
        }

        if let Some(config_path) = &self.app_config_file_path {
            ui.add_space(8.0);
            ui.label(format!("Config file: {}", config_path.display()));
        }
        }

    fn open_config_folder(&self) -> Result<(), String> {
        if let Some(path) = &self.app_config_file_path {
            let config_dir = path
                .parent()
                .ok_or_else(|| "Could not get config directory path.".to_owned())?;

            #[cfg(target_os = "windows")]
            Command::new("explorer")
                .arg(config_dir)
                .spawn()
                .map_err(|err| format!("Failed to open config folder: {err}"))?;

            #[cfg(target_os = "macos")]
            Command::new("open")
                .arg(config_dir)
                .spawn()
                .map_err(|err| format!("Failed to open config folder: {err}"))?;

            #[cfg(target_os = "linux")]
            Command::new("xdg-open")
                .arg(config_dir)
                .spawn()
                .map_err(|err| format!("Failed to open config folder: {err}"))?;

            Ok(())
        } else {
            Err("App config file path not set.".to_owned())
        }
    }

    fn render_error_toast(&mut self, ctx: &egui::Context, error_message: &str) {
        egui::Window::new("Error")
            .collapsible(false)
            .resizable(false)
            .title_bar(true)
            .anchor(egui::Align2::RIGHT_TOP, [-10.0, 10.0])
            .show(ctx, |ui| {
                ui.set_max_width(300.0);
                ui.add(
                    egui::Label::new(egui::RichText::new(error_message).color(Color32::LIGHT_RED))
                        .wrap(),
                );
                if ui.button("Dismiss").clicked() {
                    self.app_config_error = None;
                }
            });
    }

    fn render_support_page(&mut self, ui: &mut egui::Ui, _dark: bool) {
        let body = match self.nav {
            Nav::About => support_page_body(SupportPage::About),
            Nav::Feedback => support_page_body(SupportPage::Feedback),
            Nav::PrivacyPolicy => support_page_body(SupportPage::PrivacyPolicy),
            _ => "",
        };
        ui.heading(body);
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

fn brand_button(label: &str) -> Button<'_> {
    Button::new(RichText::new(label).color(Color32::WHITE)).fill(Color32::from_rgb(2, 110, 193))
}

fn nav_item(
    ui: &mut egui::Ui,
    dark: bool,
    nav: &mut Nav,
    value: Nav,
    label: &str,
    icon: IconKind,
) -> egui::Response {
    ui.horizontal(|ui| {
        ui.add(icon_image(themed_icon(dark, icon), 18.0));
        ui.selectable_value(nav, value, label)
    })
    .inner
}

fn support_page_row(ui: &mut egui::Ui, dark: bool, icon: IconKind, label: &str) -> egui::Response {
    ui.horizontal(|ui| {
        ui.add(icon_image(themed_icon(dark, icon), 16.0));
        ui.add(Button::new(label).frame(false).fill(Color32::TRANSPARENT))
    })
    .inner
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

fn map_framework_label(stored: &str) -> String {
    ProjectType::from_storage(stored)
        .map(|project_type| project_type.label().to_owned())
        .unwrap_or_else(|| stored.to_owned())
}

fn file_timestamp(path: &Path) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    let timestamp = metadata.created().or_else(|_| metadata.modified()).ok()?;
    let datetime: DateTime<Local> = timestamp.into();
    Some(datetime.format("%Y-%m-%d %H:%M").to_string())
}

fn format_build_timestamp(raw: Option<&str>) -> String {
    let raw = match raw {
        Some(raw) => raw,
        None => return "Unknown time".to_owned(),
    };
    match chrono::NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M") {
        Ok(datetime) => datetime
            .format("%d-%m-%Y | %I:%M %p")
            .to_string()
            .to_lowercase(),
        Err(_) => "Unknown time".to_owned(),
    }
}

fn is_terminal_link(token: &str) -> bool {
    token.starts_with("http://") || token.starts_with("https://")
}

fn locate_bridge_agent() -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe().map_err(|err| err.to_string())?;
    let exe_dir = exe_path
        .parent()
        .ok_or_else(|| "Executable directory unavailable.".to_owned())?;
    let agent_name = if cfg!(target_os = "windows") {
        "bridge_serve_agent.exe"
    } else {
        "bridge_serve_agent"
    };
    let agent_path = exe_dir.join(agent_name);
    if !agent_path.exists() {
        return Err(format!(
            "Serve agent not found at {}",
            agent_path.display()
        ));
    }
    Ok(agent_path)
}

fn generate_token() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{:x}{:x}", now.as_secs(), now.subsec_nanos())
}

fn spawn_reader_thread<T: std::io::Read + Send + 'static>(reader: T, tx: mpsc::Sender<String>) {
    thread::spawn(move || {
        let buffered = BufReader::new(reader);
        for line in buffered.lines().flatten() {
            let _ = tx.send(line);
        }
    });
}

fn project_artifact_type_options(project: &ProjectRecord) -> Vec<String> {
    let mut options: BTreeSet<String> = BTreeSet::new();
    for build in &project.builds {
        if let Some(ext) = Path::new(&build.path).extension().and_then(|ext| ext.to_str()) {
            let ext = ext.trim().to_ascii_lowercase();
            if !ext.is_empty() {
                options.insert(ext);
            }
        }
    }

    if options.is_empty() {
        match ProjectType::from_storage(&project.project_type).unwrap_or(ProjectType::Android) {
            ProjectType::Android => {
                options.insert("apk".to_owned());
                options.insert("aab".to_owned());
            }
            ProjectType::Flutter => {
                options.insert("apk".to_owned());
            }
            _ => {
                options.insert("custom_type".to_owned());
            }
        }
    }
    options.insert("custom_type".to_owned());
    options.into_iter().collect()
}
