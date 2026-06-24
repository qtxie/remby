use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::emby::Library;
use crate::theme::ThemeColors;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RembyConfig {
    #[serde(default)]
    pub enabled_libraries: Vec<String>,
    #[serde(default)]
    pub latest_libraries: Vec<String>,
    #[serde(default)]
    pub following_series: Vec<String>,
    #[serde(default = "default_mpv_path")]
    pub mpv_path: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub preferred_resolution: String,
    #[serde(default)]
    pub preferred_audio_language: String,
    #[serde(default)]
    pub preferred_subtitle_language: String,
}

fn default_mpv_path() -> String {
    "mpv".to_string()
}

fn default_language() -> String {
    crate::i18n::detect_system_lang().to_string()
}

fn default_theme() -> String {
    "default".to_string()
}

impl RembyConfig {
    pub fn sort_libraries(&self, libs: Vec<Library>) -> Vec<Library> {
        if self.enabled_libraries.is_empty() {
            return libs;
        }
        let mut libs = libs;
        libs.sort_by_key(|lib| {
            self.enabled_libraries.iter().position(|id| id == &lib.id)
                .unwrap_or(usize::MAX)
        });
        libs
    }

    pub fn filter_and_sort_libraries(&self, libs: Vec<Library>) -> Vec<Library> {
        let filtered = if self.enabled_libraries.is_empty() {
            libs
        } else {
            libs.into_iter()
                .filter(|lib| self.enabled_libraries.contains(&lib.id))
                .collect()
        };
        self.sort_libraries(filtered)
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub label: String,
    pub server: String,
    pub username: String,
    pub password_enc: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AccountsConfig {
    #[serde(default)]
    pub accounts: Vec<Account>,
    #[serde(default)]
    pub last_account_id: Option<String>,
}

fn accounts_path() -> PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("remby");
    dir.join("accounts.json")
}

pub fn load_accounts() -> AccountsConfig {
    let path = accounts_path();
    match std::fs::read_to_string(&path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => AccountsConfig::default(),
    }
}

pub fn save_accounts(config: &AccountsConfig) -> Result<()> {
    let path = accounts_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(config)?;
    std::fs::write(&path, data)?;
    Ok(())
}

pub fn load_themes() -> HashMap<String, ThemeColors> {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("remby");
    let path = dir.join("theme.json");
    match std::fs::read_to_string(&path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}
