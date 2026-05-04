use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
// -------- Edit App Config Here --------
const DEFAULT_DEBUG_PAGE: bool = true;

// ---- LOGIC ----

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct AppConfig {
    #[serde(rename = "debug-page")]
    pub debug_page: bool,
}

pub fn init_app_config() -> (Option<PathBuf>, AppConfig, Option<String>) {
    match resolve_app_config_file_path() {
        Ok(path) => match load_or_create_app_config(&path) {
            Ok(config) => (Some(path), config, None),
            Err(err) => (Some(path), AppConfig::default(), Some(err)),
        },
        Err(err) => (None, AppConfig::default(), Some(err)),
    }
}

pub fn resolve_app_config_file_path() -> Result<PathBuf, String> {
    let project_dirs = ProjectDirs::from("com", "BuildBridge", "BuildBridge")
        .ok_or_else(|| "Failed to locate a writable config folder for this OS.".to_owned())?;
    let config_dir = project_dirs.config_dir();
    fs::create_dir_all(config_dir).map_err(|err| {
        format!(
            "Failed to create config folder '{}': {err}",
            config_dir.display()
        )
    })?;
    Ok(config_dir.join("app-config"))
}

pub fn load_or_create_app_config(path: &Path) -> Result<AppConfig, String> {
    if !path.exists() {
        let default_config = AppConfig {
            debug_page: DEFAULT_DEBUG_PAGE,
        };
        save_app_config(path, &default_config)?;
        return Ok(default_config);
    }

    let raw = fs::read_to_string(path)
        .map_err(|err| format!("Failed to read '{}': {err}", path.display()))?;

    serde_json::from_str::<AppConfig>(&raw).map_err(|err| {
        format!(
            "app-config is invalid JSON at '{}': {err}",
            path.display()
        )
    })
}

pub fn save_app_config(path: &Path, config: &AppConfig) -> Result<(), String> {
    let json = serde_json::to_string_pretty(config)
        .map_err(|err| format!("Failed to serialize app config to JSON: {err}"))?;
    fs::write(path, json).map_err(|err| format!("Failed to write '{}': {err}", path.display()))
}
