use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecentRepo {
    pub path: String,
    pub name: String,
    pub opened_at: String,
}

const MAX_RECENTS: usize = 5;

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("git-tree").join("recent.json"))
}

pub fn load_recent() -> Vec<RecentRepo> {
    let Some(path) = config_path() else {
        return Vec::new();
    };
    let Ok(data) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    serde_json::from_str(&data).unwrap_or_default()
}

pub fn save_recent(path_str: &str, name: &str) -> Result<()> {
    let config_path = config_path().ok_or_else(|| anyhow::anyhow!("Cannot find config path"))?;

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut recents = load_recent();
    //Remove Dub
    recents.retain(|r| r.path != path_str);
    //Prepend Newest
    recents.insert(
        0,
        RecentRepo {
            path: path_str.to_string(),
            name: name.to_string(),
            opened_at: chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string(),
        },
    );
    recents.truncate(MAX_RECENTS);

    std::fs::write(&config_path, serde_json::to_string_pretty(&recents)?)?;
    Ok(())
}
