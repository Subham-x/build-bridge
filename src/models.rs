use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
    Android,
    Flutter,
    DotNet,
    Python,
    ReactNative,
}

impl ProjectType {
    pub fn label(self) -> &'static str {
        match self {
            Self::Android => "Android Studio",
            Self::Flutter => "Flutter",
            Self::DotNet => ".NET",
            Self::Python => "Python",
            Self::ReactNative => "React Native",
        }
    }

    pub fn storage_value(self) -> &'static str {
        self.label()
    }

    pub fn all() -> [Self; 5] {
        [
            Self::Android,
            Self::Flutter,
            Self::DotNet,
            Self::Python,
            Self::ReactNative,
        ]
    }

    pub fn from_storage(value: &str) -> Option<Self> {
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

impl Default for ProjectType {
    fn default() -> Self {
        Self::Android
    }
}

#[derive(Default)]
pub struct CreateProjectForm {
    pub name: String,
    pub main_path: String,
    pub project_type: ProjectType,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct BuildEntry {
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub created_on: Option<String>,
    #[serde(default)]
    pub starred: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProjectRecord {
    pub name: String,
    #[serde(rename = "type")]
    pub project_type: String,
    pub main_path: String,
    pub builds: Vec<BuildEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub star: Option<String>,
    pub status: String,
    pub created_on: String,
    pub edited_on: String,
}
