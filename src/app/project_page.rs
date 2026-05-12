use super::{format_build_timestamp, map_framework_label, project_artifact_type_options, support_page_body};
use crate::models::{CreateProjectForm, ProjectRecord, ProjectType};
use super::ProjectDashboardApp;
use crate::icons::{icon_button, icon_image, themed_icon, IconKind};
use eframe::egui::{
    self, Align, Align2, Button, Color32, CornerRadius, FontId, Frame, Label, Layout, Margin,
    RichText, ScrollArea, Sense, Stroke, StrokeKind, TextEdit, TextStyle, Vec2,
};

impl ProjectDashboardApp {
    pub(super) fn render_project_page(&mut self, ctx: &egui::Context, dark: bool) {
        egui::CentralPanel::default().show(ctx, |ui| {
            Frame::new().inner_margin(Margin::same(12)).show(ui, |ui| {
                let in_project_page = matches!(self.nav, super::Nav::Home | super::Nav::Archived | super::Nav::Bin)
                    && self.selected_project_name.is_some();
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
                    if in_project_page {
                        let back_response = ui.add(
                            icon_button(themed_icon(dark, IconKind::Back), 18.0)
                                .frame(true)
                                .min_size(Vec2::new(26.0, 26.0)),
                        );
                        if back_response.clicked() {
                            self.close_project_details();
                        }
                        if back_response.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        ui.add_space(4.0);
                    }

                    let heading = if in_project_page {
                        self.selected_project_name.as_deref().unwrap_or("Project Details")
                    } else if self.nav == super::Nav::Bin {
                        "Bin"
                    } else {
                        "Your Projects"
                    };
                    ui.heading(heading);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if in_project_page {
                        } else {
                            match self.nav {
                                super::Nav::Bin => {
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
                                super::Nav::Archived => {
                                    let select_label = if self.archive_select_mode { "Done" } else { "Select" };
                                    if ui.button(select_label).clicked() {
                                        self.archive_select_mode = !self.archive_select_mode;
                                        if !self.archive_select_mode {
                                            self.archive_selected.clear();
                                        }
                                    }
                                }
                                super::Nav::Home => {
                                    if ui.add(super::brand_button("Create")).clicked() {
                                        self.form_error = None;
                                        self.create_form = CreateProjectForm::default();
                                        self.create_modal_step = super::CreateModalStep::Framework;
                                        self.selected_framework = self.create_form.project_type;
                                        self.modal_mode = super::ModalMode::Create;
                                        self.create_modal_open = true;
                                    }
                                }
                                super::Nav::About | super::Nav::Feedback | super::Nav::PrivacyPolicy | super::Nav::Debug => {}
                            }
                        }
                    });
                });
                ui.add_space(8.0);

                let mut sort_menu_open = false;
                if !in_project_page {
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
                            let clear_response = ui.put(
                                clear_rect,
                                icon_button(themed_icon(dark, IconKind::Clear), clear_size).frame(false),
                            );
                            if clear_response.clicked() {
                                self.search_text.clear();
                            }
                        }

                        let sort_menu = ui.menu_image_button(
                            icon_image(themed_icon(dark, IconKind::Sort), search_height),
                            |ui| {
                                ui.label("Sort by");
                                ui.separator();
                                if ui
                                    .selectable_value(
                                        &mut self.project_sort_by,
                                        super::ProjectSortBy::Title,
                                        super::ProjectSortBy::Title.label(),
                                    )
                                    .clicked()
                                {
                                    ui.close_menu();
                                }
                                if ui
                                    .selectable_value(
                                        &mut self.project_sort_by,
                                        super::ProjectSortBy::DateCreated,
                                        super::ProjectSortBy::DateCreated.label(),
                                    )
                                    .clicked()
                                {
                                    ui.close_menu();
                                }
                                if ui
                                    .selectable_value(
                                        &mut self.project_sort_by,
                                        super::ProjectSortBy::ProjectType,
                                        super::ProjectSortBy::ProjectType.label(),
                                    )
                                    .clicked()
                                {
                                    ui.close_menu();
                                }

                                ui.add_space(8.0);
                                // ui.separator();
                                // ui.label("Order");
                                ui.separator();
                                if ui
                                    .selectable_value(
                                        &mut self.project_sort_order,
                                        super::ProjectSortOrder::Asc,
                                        super::ProjectSortOrder::Asc.label(),
                                    )
                                    .clicked()
                                {
                                    ui.close_menu();
                                }
                                if ui
                                    .selectable_value(
                                        &mut self.project_sort_order,
                                        super::ProjectSortOrder::Desc,
                                        super::ProjectSortOrder::Desc.label(),
                                    )
                                    .clicked()
                                {
                                    ui.close_menu();
                                }
                            },
                        );
                        if sort_menu.inner.is_some() {
                            sort_menu_open = true;
                        }
                    });
                }

                if !in_project_page && self.nav == super::Nav::Archived && self.archive_select_mode {
                    ui.add_space(6.0);
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Select all").clicked() {
                            let names: std::collections::HashSet<String> = self
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
                                    self.set_status_message(format!("Moved {count} project(s) to Bin."));
                                    self.archive_selected.clear();
                                }
                                Err(err) => {
                                    self.project_action_error = Some(err);
                                }
                            }
                        }
                        if ui
                            .add(super::brand_button("Unarchive").min_size(Vec2::new(64.0, 26.0)))
                            .clicked()
                        {
                            let selected = self.archive_selected.clone();
                            match self.bulk_unarchive_projects(&selected) {
                                Ok(count) => {
                                    self.set_status_message(format!("Unarchived {count} project(s)."));
                                    self.archive_selected.clear();
                                }
                                Err(err) => {
                                    self.project_action_error = Some(err);
                                }
                            }
                        }
                    });
                }

                if !in_project_page && self.nav == super::Nav::Bin && self.bin_select_mode {
                    ui.add_space(6.0);
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Select all").clicked() {
                            let names: std::collections::HashSet<String> = self
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
                                    self.set_status_message(format!(
                                        "Permanently deleted {count} project(s)."
                                    ));
                                    self.bin_selected.clear();
                                }
                                Err(err) => {
                                    self.project_action_error = Some(err);
                                }
                            }
                        }
                        if ui
                            .add(super::brand_button("Restore").min_size(Vec2::new(64.0, 26.0)))
                            .clicked()
                        {
                            let selected = self.bin_selected.clone();
                            match self.bulk_restore_projects(&selected) {
                                Ok(count) => {
                                    self.set_status_message(format!("Restored {count} project(s)."));
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
                    super::Nav::Home | super::Nav::Archived | super::Nav::Bin => {
                        if let Some(project) = self.selected_project() {
                            self.render_project_details_content(ui, dark, &project);
                        } else {
                            let list_height = (ui.available_height() - 6.0).max(180.0);
                            let projects = self.filtered_projects();
                            if projects.is_empty() {
                                let (empty_icon, empty_label) = match self.nav {
                                    super::Nav::Home => (IconKind::Briefcase, "No Projects"),
                                    super::Nav::Archived => (IconKind::Archive, "No Archived Projects"),
                                    super::Nav::Bin => (IconKind::Trash, "No Projects in Bin"),
                                    super::Nav::About | super::Nav::Feedback | super::Nav::PrivacyPolicy | super::Nav::Debug => {
                                        (IconKind::Briefcase, "No Projects")
                                    }
                                };
                                ui.allocate_ui_with_layout(
                                    Vec2::new(ui.available_width(), list_height),
                                    Layout::top_down(Align::Center),
                                    |ui| {
                                        ui.add_space(12.0);
                                        ui.add(icon_image(themed_icon(dark, empty_icon), 48.0));
                                        ui.add_space(6.0);
                                        ui.label(empty_label);
                                    },
                                );
                            } else {
                                let mut any_menu_open = false;
                                let any_popup_open = self.create_modal_open
                                    || self.theme_popup_open
                                    || self.empty_bin_confirm_open
                                    || self.pending_project_action.is_some()
                                    || self.terminal_link_popup_open
                                    || self.build_location_popup_open
                                    || self.project_path_popup_open
                                    || sort_menu_open;
                                ScrollArea::vertical().max_height(list_height).show(ui, |ui| {
                                    for project in projects {
                                        let mut block_rects = Vec::new();
                                        let mut menu_action_clicked = false;
                                        let card_response = ui
                                            .group(|ui| {
                                        ui.horizontal(|ui| {
                                            if (self.nav == super::Nav::Archived && self.archive_select_mode)
                                                || (self.nav == super::Nav::Bin && self.bin_select_mode)
                                            {
                                                let mut checked = if self.nav == super::Nav::Archived {
                                                    self.archive_selected.contains(&project.name)
                                                } else {
                                                    self.bin_selected.contains(&project.name)
                                                };
                                                let checkbox_response = ui.checkbox(&mut checked, "");
                                                if checkbox_response.changed() {
                                                    if self.nav == super::Nav::Archived {
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
                                                block_rects.push(checkbox_response.rect);
                                            }

                                            let name_color = if dark { Color32::WHITE } else { Color32::BLACK };
                                            ui.label(RichText::new(&project.name).strong().color(name_color));

                                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                let button_font = ui
                                                    .style()
                                                    .text_styles
                                                    .get(&TextStyle::Button)
                                                    .cloned()
                                                    .unwrap_or_else(|| FontId::proportional(14.0));
                                                let fixed_width_labels = ["abdominoscopy", "Permanent Delete This"];
                                                let max_text_width = fixed_width_labels
                                                    .iter()
                                                    .map(|label| {
                                                        ui.ctx().fonts_mut(|fonts| {
                                                            fonts
                                                                .layout_no_wrap(
                                                                    (*label).to_owned(),
                                                                    button_font.clone(),
                                                                    Color32::WHITE,
                                                                )
                                                                .size()
                                                                .x
                                                        })
                                                    })
                                                    .fold(0.0_f32, f32::max);
                                                let icon_size = 14.0;
                                                let icon_gap = ui.spacing().item_spacing.x;
                                                let button_padding = ui.spacing().button_padding.x * 2.0;
                                                let menu_width =
                                                    (icon_size + icon_gap + max_text_width + button_padding).ceil();

                                                let ctx_style_backup = ui.ctx().style().clone();
                                                let mut ctx_style = (*ctx_style_backup).clone();
                                                ctx_style.spacing.menu_margin = Margin::same(0);
                                                ctx_style.spacing.menu_width = menu_width;
                                                ui.ctx().set_style(ctx_style);

                                                let menu_response = ui
                                                    .scope(|ui| {
                                                        let button_padding_x =
                                                            ui.spacing().button_padding.x;
                                                        ui.spacing_mut().button_padding =
                                                            Vec2::new(button_padding_x, 5.0);
                                                        ui.spacing_mut().interact_size =
                                                            Vec2::new(30.0, 26.0);
                                                        ui.menu_image_button(
                                                            icon_image(
                                                                themed_icon(dark, IconKind::MoreVert),
                                                                16.0,
                                                            ),
                                                            |ui| {
                                                                ui.set_width(menu_width);
                                                                let row_width = ui.available_width();

                                                                let danger_text =
                                                                    Color32::from_rgb(255, 0, 79);
                                                                let danger_bg =
                                                                    Color32::from_rgba_unmultiplied(255, 0, 79, 40);
                                                                let menu_row = |ui: &mut egui::Ui,
                                                                                icon_kind: IconKind,
                                                                                label: RichText,
                                                                                icon_tint: Option<Color32>,
                                                                                hover_bg: Option<Color32>|
                                                                 -> egui::Response {
                                                                    let row_height = ui.spacing().interact_size.y;
                                                                    let (rect, response) =
                                                                        ui.allocate_exact_size(
                                                                            Vec2::new(row_width, row_height),
                                                                            egui::Sense::click(),
                                                                        );
                                                                    let pointer_over = ui
                                                                        .ctx()
                                                                        .pointer_hover_pos()
                                                                        .map_or(false, |pos| rect.contains(pos));
                                                                    let pointer_down =
                                                                        ui.ctx().input(|i| {
                                                                            i.pointer.primary_down()
                                                                                && i.pointer
                                                                                    .interact_pos()
                                                                                    .map_or(false, |pos| {
                                                                                        rect.contains(pos)
                                                                                    })
                                                                        });
                                                                    let visuals =
                                                                        ui.style().interact(&response);
                                                                    if pointer_over || pointer_down {
                                                                        let fill =
                                                                            hover_bg.unwrap_or(visuals.bg_fill);
                                                                        ui.painter().rect_filled(
                                                                            rect,
                                                                            CornerRadius::same(6),
                                                                            fill,
                                                                        );
                                                                    }
                                                                    ui.scope_builder(
                                                                        egui::UiBuilder::new()
                                                                            .max_rect(rect)
                                                                            .layout(
                                                                                Layout::left_to_right(
                                                                                    Align::Center,
                                                                                ),
                                                                            ),
                                                                        |ui| {
                                                                            ui.add_space(
                                                                                ui.spacing().button_padding.x,
                                                                            );
                                                                            let mut icon = icon_image(
                                                                                themed_icon(dark, icon_kind),
                                                                                icon_size,
                                                                            );
                                                                            if let Some(tint) = icon_tint {
                                                                                icon = icon.tint(tint);
                                                                            }
                                                                            ui.add(icon);
                                                                            ui.add_space(icon_gap);
                                                                            ui.add(
                                                                                Label::new(label).selectable(false),
                                                                            );
                                                                        },
                                                                    );
                                                                    if pointer_over {
                                                                        ui.ctx().set_cursor_icon(
                                                                            egui::CursorIcon::PointingHand,
                                                                        );
                                                                    }
                                                                    response
                                                                };

                                                                let edit_response = menu_row(
                                                                    ui,
                                                                    IconKind::ActionEdit,
                                                                    RichText::new("Edit"),
                                                                    None,
                                                                    None,
                                                                );
                                                                if edit_response.clicked() {
                                                                    menu_action_clicked = true;
                                                                    self.begin_edit_project(&project.name);
                                                                    ui.close();
                                                                }
                                                                block_rects.push(edit_response.rect);
                                                                if self.nav != super::Nav::Archived {
                                                                    let archive_response = menu_row(
                                                                        ui,
                                                                        IconKind::ActionArchive,
                                                                        RichText::new("Archive"),
                                                                        None,
                                                                        None,
                                                                    );
                                                                    if archive_response.clicked() {
                                                                        menu_action_clicked = true;
                                                                        if let Err(err) =
                                                                            self.archive_project(&project.name)
                                                                        {
                                                                            self.project_action_error =
                                                                                Some(err);
                                                                        }
                                                                        ui.close();
                                                                    }
                                                                    block_rects.push(archive_response.rect);
                                                                } else {
                                                                    let unarchive_response = menu_row(
                                                                        ui,
                                                                        IconKind::ActionArchive,
                                                                        RichText::new("Unarchive"),
                                                                        None,
                                                                        None,
                                                                    );
                                                                    if unarchive_response.clicked() {
                                                                        menu_action_clicked = true;
                                                                        if let Err(err) =
                                                                            self.unarchive_project(&project.name)
                                                                        {
                                                                            self.project_action_error =
                                                                                Some(err);
                                                                        }
                                                                        ui.close();
                                                                    }
                                                                    block_rects.push(unarchive_response.rect);
                                                                }

                                                                if self.nav != super::Nav::Bin {
                                                                    let bin_label = RichText::new("Move to Bin")
                                                                        .color(danger_text);
                                                                    let bin_response = menu_row(
                                                                        ui,
                                                                        IconKind::Trash,
                                                                        bin_label,
                                                                        Some(danger_text),
                                                                        Some(danger_bg),
                                                                    );
                                                                    if bin_response.clicked() {
                                                                        menu_action_clicked = true;
                                                                        self.pending_project_action = Some(
                                                                            super::ProjectConfirmAction::MoveToBin {
                                                                                project_name: project.name.clone(),
                                                                            },
                                                                        );
                                                                        ui.close();
                                                                    }
                                                                    block_rects.push(bin_response.rect);
                                                                }

                                                                if self.nav == super::Nav::Bin {
                                                                    let restore_response = menu_row(
                                                                        ui,
                                                                        IconKind::ActionArchive,
                                                                        RichText::new("Restore"),
                                                                        None,
                                                                        None,
                                                                    );
                                                                    if restore_response.clicked() {
                                                                        menu_action_clicked = true;
                                                                        if let Err(err) =
                                                                            self.restore_project(&project.name)
                                                                        {
                                                                            self.project_action_error =
                                                                                Some(err);
                                                                        }
                                                                        ui.close();
                                                                    }
                                                                    block_rects.push(restore_response.rect);

                                                                    let delete_response = menu_row(
                                                                        ui,
                                                                        IconKind::ActionDelete,
                                                                        RichText::new("Permanent Delete")
                                                                            .color(danger_text),
                                                                        Some(danger_text),
                                                                        Some(danger_bg),
                                                                    );
                                                                    if delete_response.clicked() {
                                                                        menu_action_clicked = true;
                                                                        self.pending_project_action = Some(
                                                                            super::ProjectConfirmAction::PermanentDelete {
                                                                                project_name: project.name.clone(),
                                                                            },
                                                                        );
                                                                        ui.close();
                                                                    }
                                                                    block_rects.push(delete_response.rect);
                                                                }
                                                            },
                                                        )
                                                    })
                                                    .inner;
                                                if menu_response.inner.is_some() {
                                                    any_menu_open = true;
                                                }
                                                ui.ctx().set_style(ctx_style_backup);
                                                block_rects.push(menu_response.response.rect);
                                                let current_stream_label = project
                                                    .stream_type
                                                    .as_deref()
                                                    .and_then(stream_type_label)
                                                    .unwrap_or("Select stream");
                                                let stream_menu = ui
                                                    .menu_image_button(
                                                        icon_image(
                                                            themed_icon(dark, IconKind::Broadcast),
                                                            16.0,
                                                        ),
                                                        |ui| {
                                                            for (label, value) in stream_type_options() {
                                                                let selected = project
                                                                    .stream_type
                                                                    .as_deref()
                                                                    == Some(value);
                                                                if ui
                                                                    .selectable_label(selected, label)
                                                                    .clicked()
                                                                {
                                                                    menu_action_clicked = true;
                                                                    if let Err(err) =
                                                                        self.set_project_stream_type(
                                                                            &project.name,
                                                                            value,
                                                                        )
                                                                    {
                                                                        self.project_action_error =
                                                                            Some(err);
                                                                    }
                                                                    ui.close();
                                                                }
                                                            }
                                                        },
                                                    );
                                                if stream_menu.inner.is_some() {
                                                    any_menu_open = true;
                                                }
                                                let stream_menu_response = stream_menu
                                                    .response
                                                    .on_hover_text(format!(
                                                        "Stream: {current_stream_label}"
                                                    ));
                                                block_rects.push(stream_menu_response.rect);
                                                ui.add_space(2.0);
                                                if self.nav == super::Nav::Archived {
                                                    let unarchive_response = ui.add(
                                                        super::brand_button("Unarchive")
                                                            .min_size(Vec2::new(64.0, 26.0)),
                                                    );
                                                    if unarchive_response.clicked() {
                                                        if let Err(err) = self.unarchive_project(&project.name) {
                                                            self.project_action_error = Some(err);
                                                        }
                                                    }
                                                    block_rects.push(unarchive_response.rect);
                                                } else if self.nav == super::Nav::Bin {
                                                    let restore_response = ui.add(
                                                        super::brand_button("Restore")
                                                            .min_size(Vec2::new(64.0, 26.0)),
                                                    );
                                                    if restore_response.clicked() {
                                                        if let Err(err) = self.restore_project(&project.name) {
                                                            self.project_action_error = Some(err);
                                                        }
                                                    }
                                                    block_rects.push(restore_response.rect);
                                                } else if project.status == "active" {
                                                    let is_active = self.is_bridge_online(&project.name);
                                                    let button_label = if is_active { "Stop" } else { "Serve" };
                                                    let button_fill = if is_active {
                                                        Color32::from_rgb(220, 68, 55) // Red for Stop
                                                    } else {
                                                        Color32::from_rgb(2, 110, 193) // Blue for Serve
                                                    };
                                                    
                                                    let serve_response = ui.add(
                                                        Button::new(RichText::new(button_label).color(Color32::WHITE))
                                                            .fill(button_fill)
                                                            .min_size(Vec2::new(64.0, 26.0))
                                                    );
                                                    if serve_response.clicked() {
                                                        if is_active {
                                                            self.stop_bridge_serve();
                                                        } else {
                                                            if let Err(err) = self.start_bridge_serve(&project) {
                                                                self.project_action_error = Some(err);
                                                            }
                                                        }
                                                    }
                                                    block_rects.push(serve_response.rect);
                                                }
                                            });
                                        });
                                        ui.horizontal_wrapped(|ui| {
                                            let framework_label = map_framework_label(&project.project_type);
                                            ui.label(egui::RichText::new(framework_label).strong());
                                            ui.label("•");
                                            ui.label(egui::RichText::new(&project.main_path).italics());
                                        });
                                        })
                                        .response;
                                    let pointer_over_control = ui
                                        .ctx()
                                        .input(|i| i.pointer.interact_pos())
                                        .map_or(false, |pos| {
                                            block_rects.iter().any(|rect| rect.contains(pos))
                                        });
                                    let clicked_background = ui.ctx().input(|i| {
                                        i.pointer.primary_clicked()
                                            && i
                                                .pointer
                                                .interact_pos()
                                                .map_or(false, |pos| card_response.rect.contains(pos))
                                    });
                                    if card_response.hovered() && !pointer_over_control {
                                        ui.painter().rect_stroke(
                                            card_response.rect,
                                            CornerRadius::same(8),
                                            Stroke::new(1.0, Color32::from_gray(110)),
                                            StrokeKind::Outside,
                                        );
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                    }
                                    if clicked_background
                                        && !pointer_over_control
                                        && self.nav != super::Nav::Bin
                                        && !menu_action_clicked
                                        && !any_menu_open
                                        && !any_popup_open
                                    {
                                        self.open_project_details(&project.name);
                                    }
                                        ui.add_space(6.0);
                                    }
                                });
                            }
                        }
                    }
                    super::Nav::About => {
                        ui.label(support_page_body(super::SupportPage::About));
                    }
                    super::Nav::Feedback => {
                        ui.label(support_page_body(super::SupportPage::Feedback));
                    }
                    super::Nav::PrivacyPolicy => {
                        ui.label(support_page_body(super::SupportPage::PrivacyPolicy));
                    }
                    super::Nav::Debug => {
                        ui.label("Debug Page - use sidebar to access debug features");
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
    }

    fn render_project_details_content(
        &mut self,
        ui: &mut egui::Ui,
        dark: bool,
        project: &ProjectRecord,
    ) {
        ui.vertical(|ui| {
            ui.horizontal_top(|ui| {
                let left_width = (ui.available_width() * 0.48).max(220.0);
                ui.allocate_ui_with_layout(
                    Vec2::new(left_width, 96.0),
                    Layout::top_down(Align::Min),
                    |ui| {
                        Frame::new()
                            .fill(if dark {
                                Color32::from_gray(52)
                            } else {
                                Color32::from_gray(224)
                            })
                            .inner_margin(Margin::same(10))
                            .show(ui, |ui| {
                                let value_text_color = if dark {
                                    Color32::WHITE
                                } else {
                                    Color32::BLACK
                                };
                                let key_text_color = if dark {
                                    Color32::from_gray(165)
                                } else {
                                    Color32::from_gray(90)
                                };
                                let label_width = 96.0;
                                let label = |ui: &mut egui::Ui, text: &str| {
                                    ui.add_sized(
                                        [label_width, 0.0],
                                        Label::new(RichText::new(text).color(key_text_color)),
                                    );
                                };
                                ui.horizontal(|ui| {
                                    label(ui, "Project:");
                                    ui.colored_label(
                                        value_text_color,
                                        RichText::new(&project.name).strong(),
                                    );
                                });
                                ui.horizontal(|ui| {
                                    label(ui, "Project Type:");
                                    ui.colored_label(
                                        value_text_color,
                                        RichText::new(map_framework_label(&project.project_type)).strong(),
                                    );
                                });
                                ui.horizontal(|ui| {
                                    label(ui, "Location:");
                                    let short_path = shorten_project_path(&project.main_path);
                                    ui.colored_label(
                                        value_text_color,
                                        RichText::new(short_path).strong(),
                                    );
                                    ui.add_space(6.0);
                                    let info_button =
                                        icon_button(themed_icon(dark, IconKind::Info), 14.0)
                                            .frame(true)
                                            .min_size(Vec2::new(24.0, 22.0));
                                    let info_response =
                                        ui.add(info_button).on_hover_text("Show full path");
                                    if info_response.clicked() {
                                        self.project_path_popup_path =
                                            Some(project.main_path.clone());
                                        self.project_path_popup_open = true;
                                    }
                                });
                                ui.horizontal(|ui| {
                                    label(ui, "Feedback:");
                                    if ui
                                        .add(
                                            Button::new(
                                                RichText::new("Feedback folder")
                                                    .color(Color32::BLACK)
                                                    .strong(),
                                            )
                                            .fill(Color32::from_rgb(255, 205, 67)),
                                        )
                                        .clicked()
                                    {
                                        if let Err(err) = self.open_feedback_folder(&project.name) {
                                            self.project_action_error = Some(err);
                                        }
                                    }
                                });
                            });
                    },
                );

                ui.add_space(10.0);
                ui.vertical(|ui| {
                    let artifact_options = project_artifact_type_options(project);
                    if !artifact_options
                        .iter()
                        .any(|item| item == &self.selected_artifact_type)
                    {
                        self.selected_artifact_type = artifact_options[0].clone();
                    }
                    Frame::new()
                        .stroke(Stroke::new(
                            1.0,
                            if dark { Color32::from_gray(95) } else { Color32::from_gray(150) },
                        ))
                        .corner_radius(CornerRadius::same(0))
                        .inner_margin(Margin::same(8))
                        .show(ui, |ui| {
                            let label_width = 96.0;
                            ui.horizontal(|ui| {
                                ui.add_sized([label_width, 0.0], Label::new("Add file"));
                                let add_button =
                                    Button::new("+").min_size(Vec2::new(30.0, 26.0));
                                if ui.add(add_button).clicked() {
                                    let picked_file = rfd::FileDialog::new()
                                        .set_directory(&project.main_path)
                                        .pick_file();
                                    if let Some(path) = picked_file {
                                        match self.add_extra_file(&project.name, path) {
                                            Ok(()) => self.set_status_message(format!(
                                                "Extra file added for '{}'.",
                                                project.name
                                            )),
                                            Err(err) => self.project_action_error = Some(err),
                                        }
                                    }
                                }
                            });

                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.add_sized([label_width, 0.0], Label::new("Real-Time"));
                                toggle_switch(ui, &mut self.real_time_enabled);
                            });

                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.add_sized([label_width, 0.0], Label::new("Start Serve"));
                                let current_stream_label = project
                                    .stream_type
                                    .as_deref()
                                    .and_then(stream_type_label)
                                    .unwrap_or("Select stream");
                                ui.menu_image_button(
                                    icon_image(themed_icon(dark, IconKind::Broadcast), 16.0),
                                    |ui| {
                                        for (label, value) in stream_type_options() {
                                            let selected = project.stream_type.as_deref()
                                                == Some(value);
                                            if ui.selectable_label(selected, label).clicked() {
                                                if let Err(err) = self.set_project_stream_type(
                                                    &project.name,
                                                    value,
                                                ) {
                                                    self.project_action_error = Some(err);
                                                }
                                                ui.close();
                                            }
                                        }
                                    },
                                )
                                .response
                                .on_hover_text(format!("Stream: {current_stream_label}"));
                                ui.add_space(6.0);
                                
                                let is_active = self.is_bridge_online(&project.name);
                                let button_label = if is_active { "Stop" } else { "Serve" };
                                let button_fill = if is_active {
                                    Color32::from_rgb(220, 68, 55) // Red for Stop
                                } else {
                                    Color32::from_rgb(2, 110, 193) // Blue for Serve
                                };
                                
                                let serve_button = Button::new(RichText::new(button_label).color(Color32::WHITE))
                                    .fill(button_fill)
                                    .min_size(Vec2::new(64.0, 26.0));
                                if ui.add(serve_button).clicked() {
                                    if is_active {
                                        self.stop_bridge_serve();
                                    } else {
                                        if let Err(err) = self.start_bridge_serve(project) {
                                            self.project_action_error = Some(err);
                                        }
                                    }
                                }
                            });
                        });
                });
            });

            ui.add_space(10.0);
            let project_type =
                ProjectType::from_storage(&project.project_type).unwrap_or(ProjectType::Android);
            if project_type != ProjectType::Android {
                ui.add_space(6.0);
                ui.label(
                    RichText::new("UPCOMING...")
                        .italics()
                        .color(ui.style().visuals.weak_text_color()),
                );
                return;
            }

            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Total Builds ({})", project.builds.len())).strong());
                let refresh_button = icon_button(themed_icon(dark, IconKind::Refresh), 14.0)
                    .frame(true)
                    .min_size(Vec2::new(26.0, 22.0));
                let refresh_response = ui
                    .add(refresh_button)
                    .on_hover_text("Refresh builds");
                if refresh_response.clicked() {
                    match self.refresh_android_builds_with_result(&project.name) {
                        Ok(count) => {
                            self.set_status_message_tinted(
                                format!("Builds refreshed ({count})."),
                                Color32::from_rgb(46, 160, 67),
                            );
                        }
                        Err(err) => {
                            self.set_status_message_tinted(
                                format!("Build refresh failed: {err}"),
                                Color32::from_rgb(220, 68, 55),
                            );
                        }
                    }
                }
            });
            ui.add_space(6.0);

            ScrollArea::vertical()
                .max_height((ui.available_height() - 4.0).max(120.0))
                .show(ui, |ui| {
                    if project.builds.is_empty() {
                        ui.label("No builds yet.");
                        return;
                    }

                    let mut builds = project.builds.clone();
                    builds.sort_by(|a, b| {
                        b.starred
                            .cmp(&a.starred)
                            .then_with(|| b.created_on.cmp(&a.created_on))
                            .then_with(|| a.name.cmp(&b.name))
                    });

                    let star_size = 14.0;
                    let icon_gap = 6.0;
                    for (index, build) in builds.iter().enumerate() {
                        let selected = self.selected_build_index == Some(index);
                        let row_height = 36.0;
                        let (rect, response) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), row_height),
                            Sense::click(),
                        );
                        let response = response.on_hover_text_at_pointer("Double click to show path");
                        let bg_fill = if selected {
                            Color32::from_rgb(25, 55, 90)
                        } else {
                            ui.style().visuals.panel_fill
                        };
                        ui.painter().rect_filled(rect, CornerRadius::same(6), bg_fill);

                        let text_rect = rect.shrink2(Vec2::new(10.0, 6.0));
                        let created_label =
                            format_build_timestamp(build.created_on.as_deref());
                        let star_rect = egui::Rect::from_min_size(
                            egui::pos2(
                                text_rect.left(),
                                text_rect.center().y - star_size * 0.5,
                            ),
                            Vec2::splat(star_size),
                        );
                        let star_icon = if build.starred {
                            IconKind::StarFilled
                        } else {
                            IconKind::Star
                        };
                        let star_response = ui.put(
                            star_rect,
                            icon_button(themed_icon(dark, star_icon), star_size).frame(false),
                        );
                        let star_clicked = star_response.clicked();
                        if star_clicked {
                            self.toggle_build_star(&project.name, &build.path);
                        }

                        let name_pos = egui::pos2(star_rect.right() + icon_gap, text_rect.center().y);
                        ui.painter().text(
                            name_pos,
                            Align2::LEFT_CENTER,
                            &build.name,
                            FontId::proportional(18.0),
                            ui.style().visuals.text_color(),
                        );
                        ui.painter().text(
                            text_rect.right_center(),
                            Align2::RIGHT_CENTER,
                            created_label,
                            FontId::proportional(14.0),
                            ui.style().visuals.weak_text_color(),
                        );

                        let star_hovered = star_response.hovered();
                        if response.clicked() && !star_clicked {
                            self.selected_build_index = Some(index);
                        }
                        if response.double_clicked() && !star_hovered {
                            self.selected_build_index = Some(index);
                            self.build_location_popup_path = Some(build.path.clone());
                            self.build_location_popup_open = true;
                        }
                        ui.add_space(6.0);
                    }
                });

            ui.add_space(10.0);
            let extra_count = if project.added_file.is_some() { 1 } else { 0 };
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Extra Added ({})", extra_count)).strong());
            });
            ui.add_space(6.0);
            ScrollArea::vertical()
                .id_salt("extra_added_scroll")
                .max_height(80.0)
                .show(ui, |ui| {
                if let Some(extra_path) = &project.added_file {
                    ui.label(RichText::new(extra_path).strong());
                } else {
                    ui.label("No extra files added.");
                }
            });
        });
    }
}

fn toggle_switch(ui: &mut egui::Ui, value: &mut bool) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * Vec2::new(2.0, 1.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());
    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }

    let how_on = ui.ctx().animate_bool(response.id, *value);
    let visuals = ui.style().interact(&response);
    let radius = rect.height() * 0.5;
    let bg_fill = if *value {
        Color32::from_rgb(36, 136, 230)
    } else {
        visuals.bg_fill
    };
    ui.painter().rect_filled(rect, radius, bg_fill);

    let knob_radius = radius - 2.0;
    let left_x = rect.left() + radius;
    let right_x = rect.right() - radius;
    let knob_x = egui::lerp(left_x..=right_x, how_on);
    ui.painter().circle_filled(
        egui::pos2(knob_x, rect.center().y),
        knob_radius,
        Color32::WHITE,
    );

    response
}

fn stream_type_options() -> [(&'static str, &'static str); 4] {
    [
        ("Zrok", "zrok"),
        ("LocalTunnel", "localtunnel"),
        ("LocalHost : token", "localhost-token"),
        ("LocalHost : allowlist + token", "localhost-allowlist-token"),
    ]
}

fn stream_type_label(value: &str) -> Option<&'static str> {
    stream_type_options()
        .iter()
        .find(|(_, key)| *key == value)
        .map(|(label, _)| *label)
}

fn shorten_project_path(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }

    let separator = if path.contains('\\') { '\\' } else { '/' };
    let parts: Vec<&str> = path.split(separator).filter(|part| !part.is_empty()).collect();
    if parts.len() <= 3 {
        return path.to_owned();
    }

    let prefix = format!("{}{}{}", parts[0], separator, parts[1]);
    let suffix = parts[parts.len() - 1];
    format!("{prefix}{separator}....{separator}{suffix}")
}
