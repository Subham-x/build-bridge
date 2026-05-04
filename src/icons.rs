use eframe::egui::{Button, Image, ImageSource, Vec2};

#[derive(Clone, Copy)]
pub enum IconKind {
    Back,
    Briefcase,
    Archive,
    Trash,
    MoreVert,
    Palette,
    PanelHide,
    PanelShow,
    Sort,
    Clear,
    About,
    Feedback,
    Privacy,
    Broadcast,
    Bell,
    BellSlash,
    BridgeStatusExpand,
    BridgeStatusCollapse,
    ActionEdit,
    ActionArchive,
    ActionDelete,
    Bug,
    Star,
    StarFilled,
}

pub fn themed_icon(dark: bool, icon: IconKind) -> ImageSource<'static> {
    match (dark, icon) {
        (true, IconKind::Bug) => egui::include_image!("../assets/icons/bug_dark.svg"),
        (false, IconKind::Bug) => egui::include_image!("../assets/icons/bug_light.svg"),
        (true, IconKind::Back) => egui::include_image!("../assets/icons/caret_left_dark.svg"),
        (false, IconKind::Back) => egui::include_image!("../assets/icons/caret_left_light.svg"),
        (true, IconKind::Briefcase) => egui::include_image!("../assets/icons/briefcase_dark.svg"),
        (false, IconKind::Briefcase) => egui::include_image!("../assets/icons/briefcase_light.svg"),
        (true, IconKind::Archive) => egui::include_image!("../assets/icons/archive_dark.svg"),
        (false, IconKind::Archive) => egui::include_image!("../assets/icons/archive_light.svg"),
        (true, IconKind::Trash) => egui::include_image!("../assets/icons/trash_dark.svg"),
        (false, IconKind::Trash) => egui::include_image!("../assets/icons/trash_light.svg"),
        (true, IconKind::MoreVert) => egui::include_image!("../assets/icons/more_vert_dark.svg"),
        (false, IconKind::MoreVert) => egui::include_image!("../assets/icons/more_vert_light.svg"),
        (true, IconKind::Palette) => egui::include_image!("../assets/icons/palette_dark.svg"),
        (false, IconKind::Palette) => egui::include_image!("../assets/icons/palette_light.svg"),
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
        (true, IconKind::Bell) => egui::include_image!("../assets/icons/bell_dark.svg"),
        (false, IconKind::Bell) => egui::include_image!("../assets/icons/bell_light.svg"),
        (true, IconKind::BellSlash) => {
            egui::include_image!("../assets/icons/bell_slash_dark.svg")
        }
        (false, IconKind::BellSlash) => {
            egui::include_image!("../assets/icons/bell_slash_light.svg")
        }
        (true, IconKind::BridgeStatusExpand) => {
            egui::include_image!("../assets/icons/bridge_status_expand_dark.svg")
        }
        (false, IconKind::BridgeStatusExpand) => {
            egui::include_image!("../assets/icons/bridge_status_expand_light.svg")
        }
        (true, IconKind::BridgeStatusCollapse) => {
            egui::include_image!("../assets/icons/bridge_status_collapse_dark.svg")
        }
        (false, IconKind::BridgeStatusCollapse) => {
            egui::include_image!("../assets/icons/bridge_status_collapse_light.svg")
        }
        (true, IconKind::ActionEdit) => egui::include_image!("../assets/icons/action_edit_dark.svg"),
        (false, IconKind::ActionEdit) => {
            egui::include_image!("../assets/icons/action_edit_light.svg")
        }
        (true, IconKind::ActionArchive) => {
            egui::include_image!("../assets/icons/action_archive_dark.svg")
        }
        (false, IconKind::ActionArchive) => {
            egui::include_image!("../assets/icons/action_archive_light.svg")
        }
        (_, IconKind::ActionDelete) => {
            egui::include_image!("../assets/icons/action_delete_red.svg")
        }
        (true, IconKind::Star) => egui::include_image!("../assets/icons/star_dark.svg"),
        (false, IconKind::Star) => egui::include_image!("../assets/icons/star_light.svg"),
        (_, IconKind::StarFilled) => egui::include_image!("../assets/icons/star-fill.svg"),
    }
}

pub fn icon_button(source: ImageSource<'static>, size: f32) -> Button<'static> {
    Button::image(icon_image(source, size))
}

pub fn icon_image(source: ImageSource<'static>, size: f32) -> Image<'static> {
    Image::new(source).fit_to_exact_size(Vec2::splat(size))
}
