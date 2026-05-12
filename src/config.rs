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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Preferences {
    pub settings: Settings,
    #[serde(rename = "project-settings", default)]
    pub project_settings: ProjectSettings,
    pub config: Config,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub theme: String,
    #[serde(rename = "fontSize")]
    pub font_size: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ProjectSettings {
    #[serde(rename = "build-status-collapse", default)]
    pub build_status_collapse: bool,
    #[serde(rename = "real-time", default)]
    pub real_time: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(rename = "sidePane")]
    pub side_pane: SidePane,
    #[serde(rename = "project-list", default)]
    pub project_list: ProjectListConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ProjectListConfig {
    #[serde(default)]
    pub sort: SortConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SortConfig {
    #[serde(rename = "sort-by", default)]
    pub sort_by: String,
    #[serde(default)]
    pub order: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SidePane {
    pub width: Option<f32>,
    pub collapsed: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            settings: Settings {
                theme: "system".to_owned(),
                font_size: "medium".to_owned(),
            },
            project_settings: ProjectSettings {
                build_status_collapse: false,
                real_time: true,
            },
            config: Config {
                side_pane: SidePane {
                    width: None,
                    collapsed: false,
                },
                project_list: ProjectListConfig::default(),
            },
        }
    }
}


impl Default for SortConfig {
    fn default() -> Self {
        Self {
            sort_by: "Title".to_owned(),
            order: "Asc".to_owned(),
        }
    }
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

pub fn init_preferences() -> (Option<PathBuf>, Preferences, Option<String>) {
    let project_dirs = match ProjectDirs::from("com", "BuildBridge", "BuildBridge")
        .ok_or_else(|| "Failed to locate a writable config folder for this OS.".to_owned())
    {
        Ok(dirs) => dirs,
        Err(err) => return (None, Preferences::default(), Some(err)),
    };

    let config_dir = project_dirs.config_dir();
    if let Err(err) = fs::create_dir_all(config_dir) {
        return (
            None,
            Preferences::default(),
            Some(format!(
                "Failed to create config folder '{}': {err}",
                config_dir.display()
            )),
        );
    }

    let path = config_dir.join("preferences.json");
    match load_or_create_preferences(&path) {
        Ok(prefs) => (Some(path), prefs, None),
        Err(err) => (Some(path), Preferences::default(), Some(err)),
    }
}

pub fn load_or_create_preferences(path: &Path) -> Result<Preferences, String> {
    if !path.exists() {
        let default_prefs = Preferences::default();
        save_preferences(path, &default_prefs)?;
        return Ok(default_prefs);
    }

    let raw = fs::read_to_string(path)
        .map_err(|err| format!("Failed to read '{}': {err}", path.display()))?;

    serde_json::from_str::<Preferences>(&raw).map_err(|err| {
        format!(
            "preferences.json is invalid JSON at '{}': {err}",
            path.display()
        )
    })
}

pub fn save_preferences(path: &Path, prefs: &Preferences) -> Result<(), String> {
    let json = serde_json::to_string_pretty(prefs)
        .map_err(|err| format!("Failed to serialize preferences to JSON: {err}"))?;
    fs::write(path, json).map_err(|err| format!("Failed to write '{}': {err}", path.display()))
}
