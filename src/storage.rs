use crate::models::ProjectRecord;
use chrono::Local;
use directories::ProjectDirs;
use std::fs;
use std::path::{Path, PathBuf};

pub fn init_storage() -> (Option<PathBuf>, Vec<ProjectRecord>, Option<String>) {
    match resolve_projects_file_path() {
        Ok(path) => match load_or_create_projects(&path) {
            Ok(projects) => (Some(path), projects, None),
            Err(err) => (Some(path), Vec::new(), Some(err)),
        },
        Err(err) => (None, Vec::new(), Some(err)),
    }
}

pub fn resolve_projects_file_path() -> Result<PathBuf, String> {
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

pub fn load_or_create_projects(path: &Path) -> Result<Vec<ProjectRecord>, String> {
    if !path.exists() {
        let empty: Vec<ProjectRecord> = Vec::new();
        save_projects(path, &empty)?;
        return Ok(empty);
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

pub fn save_projects(path: &Path, projects: &[ProjectRecord]) -> Result<(), String> {
    let json = serde_json::to_string_pretty(projects)
        .map_err(|err| format!("Failed to serialize projects to JSON: {err}"))?;
    fs::write(path, json).map_err(|err| format!("Failed to write '{}': {err}", path.display()))
}

pub fn current_date() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}
