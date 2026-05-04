mod app;
mod app_window;
mod config;
mod icons;
mod models;
mod storage;

fn main() -> Result<(), eframe::Error> {
    app_window::run()
}
