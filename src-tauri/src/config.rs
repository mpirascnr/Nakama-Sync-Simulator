use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedDeckRecord {
    pub filename: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub simulator_decks_folder: Option<String>,
    #[serde(default = "default_site_url")]
    pub site_url: String,
    #[serde(default = "default_backend_url")]
    pub backend_api_url: String,
    #[serde(default)]
    pub autostart_enabled: bool,
    /// Sync as soon as OP TCG Simulator is launched, so decks are in place
    /// before the user opens the deck menu.
    #[serde(default = "default_true")]
    pub sync_on_simulator_launch: bool,
    #[serde(default)]
    pub synced_decks: HashMap<String, SyncedDeckRecord>,
    /// Set once the machine has been paired. The token itself never lands
    /// here — it lives in the Windows Credential Manager (see credentials.rs).
    #[serde(default)]
    pub linked: bool,
}

fn default_site_url() -> String {
    "https://nakamacompanion.com".to_string()
}

/// The desktop app talks to Nakama Companion's public web API, never
/// directly to its application backend, which has no public URL and is
/// unreachable from a user's PC.
fn default_backend_url() -> String {
    "https://nakamacompanion.com/api".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            simulator_decks_folder: None,
            site_url: default_site_url(),
            backend_api_url: default_backend_url(),
            autostart_enabled: false,
            sync_on_simulator_launch: true,
            synced_decks: HashMap::new(),
            linked: false,
        }
    }
}

fn config_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("config_dir_unavailable:{e}"))?;
    fs::create_dir_all(&dir).map_err(|e| format!("config_dir_failed:{e}"))?;
    Ok(dir.join("config.json"))
}

pub fn load(app: &AppHandle) -> AppConfig {
    let path = match config_path(app) {
        Ok(p) => p,
        Err(_) => return AppConfig::default(),
    };
    match fs::read_to_string(&path) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
        Err(_) => AppConfig::default(),
    }
}

pub fn save(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let path = config_path(app)?;
    let raw = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, raw).map_err(|e| format!("config_write_failed:{e}"))
}
