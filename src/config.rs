use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RembyConfig {
    #[serde(default)]
    pub enabled_libraries: Vec<String>,
    #[serde(default)]
    pub latest_libraries: Vec<String>,
}

fn config_path() -> PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("remby");
    dir.join("config.json")
}

pub fn load_config() -> RembyConfig {
    let path = config_path();
    match std::fs::read_to_string(&path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => RembyConfig::default(),
    }
}

pub fn save_config(config: &RembyConfig) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(config)?;
    std::fs::write(&path, data)?;
    Ok(())
}
