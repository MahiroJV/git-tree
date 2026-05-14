// settings_store.rs — Persist user preferences to ~/.config/git-tree/settings.toml
use crate::components::tree_canvas::{BranchStyle, TreeDirection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme_name: String,
    pub font_size: u32,
    pub node_spacing: f64,
    pub show_merges: bool,
    pub crt_overlay: bool,
    pub tree_direction: TreeDirection,
    pub branch_style: BranchStyle,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme_name: "Terminal".to_string(),
            font_size: 13,
            node_spacing: 120.0,
            show_merges: true,
            crt_overlay: false,
            tree_direction: TreeDirection::Horizontal,
            branch_style: BranchStyle::Curved,
        }
    }
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("git-tree").join("settings.toml"))
}

pub fn load_settings() -> AppSettings {
    let Some(path) = config_path() else {
        return AppSettings::default();
    };
    let Ok(data) = std::fs::read_to_string(&path) else {
        return AppSettings::default();
    };
    toml::from_str(&data).unwrap_or_default()
}

pub fn save_settings(settings: &AppSettings) {
    let Some(path) = config_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(data) = toml::to_string(settings) {
        let _ = std::fs::write(&path, data);
    }
}
