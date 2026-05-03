mod bridge_status;
mod project_page;
mod sidebar;
mod terminal;
mod theme_popup;

use crate::icons::{icon_image, themed_icon, IconKind};
use crate::models::{CreateProjectForm, ProjectRecord, ProjectType};
use crate::storage::{current_date, init_storage, save_projects};
use eframe::egui::{
    self, Button, Color32, ComboBox, RichText, TextEdit, ThemePreference, TopBottomPanel, Vec2,
};
use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

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
    selected_project_name: Option<String>,
    selected_build_index: Option<usize>,
    selected_artifact_type: String,
    bridge_status_expanded: bool,
    terminal_link_popup_open: bool,
    terminal_link_target: Option<String>,
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
            selected_project_name: None,
            selected_build_index: None,
            selected_artifact_type: "Type".to_owned(),
            bridge_status_expanded: false,
            terminal_link_popup_open: false,
            terminal_link_target: None,
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

        if !matches!(self.nav, Nav::Home | Nav::Archived | Nav::Bin)
            && self.selected_project_name.is_some()
        {
            self.close_project_details();
        }
        if self.selected_project_name.is_some() && self.selected_project().is_none() {
            self.close_project_details();
        }

        let dark = ctx.style().visuals.dark_mode;

        self.render_sidebar(ctx, dark);
        self.render_status_bar(ctx, dark);
        self.render_bridge_panel(ctx, dark);
        self.render_project_page(ctx, dark);
        self.render_theme_popup(ctx);
        self.render_create_modal(ctx);
        self.render_empty_bin_confirm(ctx);
    }
}

impl ProjectDashboardApp {
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
                    ui.add(icon_image(themed_icon(dark, icon), 16.0));
                    ui.add_space(6.0);
                    ui.colored_label(Color32::from_rgb(2, 110, 193), text);
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
        self.selected_build_index = None;
        self.selected_artifact_type = "Type".to_owned();
        self.bridge_status_expanded = false;
    }

    fn close_project_details(&mut self) {
        self.selected_project_name = None;
        self.selected_build_index = None;
        self.selected_artifact_type = "Type".to_owned();
        self.bridge_status_expanded = false;
        self.terminal_link_popup_open = false;
        self.terminal_link_target = None;
    }

    fn selected_project(&self) -> Option<ProjectRecord> {
        let selected_name = self.selected_project_name.as_ref()?;
        self.projects
            .iter()
            .find(|project| project.name == *selected_name)
            .cloned()
    }

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
        self.status_message = Some(format!(
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

fn is_terminal_link(token: &str) -> bool {
    token.starts_with("http://") || token.starts_with("https://")
}

fn build_timestamp(index: usize) -> String {
    let minute = (index * 7 + 12) % 60;
    let hour_24 = 9 + (index % 8);
    let (hour_12, suffix) = if hour_24 >= 12 {
        let hour = if hour_24 == 12 { 12 } else { hour_24 - 12 };
        (hour, "PM")
    } else {
        (hour_24, "AM")
    };
    format!("{hour_12:02}:{minute:02} {suffix}")
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
