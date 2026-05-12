use crate::app::ProjectDashboardApp;
use eframe::egui::{self, FontData, FontDefinitions, FontFamily, IconData};
use image::ImageReader;
use std::io::Cursor;
use std::sync::Arc;

pub fn run() -> Result<(), eframe::Error> {
    let icon_data = load_app_icon();

    let viewport = egui::ViewportBuilder::default()
        .with_title("Build Bridge")
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
        "Build Bridge",
        native_options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            configure_fonts(&cc.egui_ctx);
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

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "jetbrains-regular".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/JetBrainsMono-Regular.ttf"
        ))),
    );
    fonts
        .families
        .entry(FontFamily::Name("JetBrainsMono".into()))
        .or_default()
        .push("jetbrains-regular".to_owned());
    ctx.set_fonts(fonts);
}
