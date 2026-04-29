use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::storage::{StorageConfig, StorageEntry};

/// Returns the standalone config directory (no Tauri dependency).
/// Uses the same path Tauri would resolve for `com.littlebrushgames.feature-hub`.
pub fn config_dir() -> Result<PathBuf, String> {
    let data_dir = dirs::data_dir().ok_or("Could not find data directory")?;
    let app_dir = data_dir.join("com.littlebrushgames.feature-hub");
    std::fs::create_dir_all(&app_dir).map_err(|e| format!("Failed to create config dir: {}", e))?;
    Ok(app_dir)
}

/// Returns the path to config.json.
pub fn config_path() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("config.json"))
}

/// Loads StorageConfig from disk without requiring a Tauri AppHandle.
pub fn load_config() -> Result<StorageConfig, String> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(StorageConfig::default());
    }
    let data =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read config: {}", e))?;
    serde_json::from_str(&data).map_err(|e| format!("Failed to parse config: {}", e))
}

/// Returns the active storage entry, if one is configured.
pub fn get_active_storage() -> Result<Option<StorageEntry>, String> {
    let config = load_config()?;
    match &config.active_storage_id {
        Some(id) => Ok(config.storages.iter().find(|s| s.id == *id).cloned()),
        None => Ok(None),
    }
}

/// Returns the path to the active storage directory.
pub fn get_active_storage_path() -> Result<PathBuf, String> {
    let entry = get_active_storage()?
        .ok_or("No active storage configured. Open FeatureHub to set one up.")?;
    Ok(PathBuf::from(&entry.path))
}

/// Returns the path to the active storage's `feature-hub.db`.
pub fn get_active_db_path() -> Result<PathBuf, String> {
    Ok(get_active_storage_path()?.join("feature-hub.db"))
}

// ─── Shared Types ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpServer {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
    #[serde(default = "default_true")]
    pub default_enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Repository {
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Extension {
    pub id: String,
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp_server: Option<McpServer>,
    #[serde(default)]
    pub instructions: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub content: String,
    #[serde(default = "default_true")]
    pub default_enabled: bool,
}

// ─── Global App Settings (per-machine) ───────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppSettings {
    pub fh_cli_path: Option<String>,
    #[serde(default)]
    pub mermaid_diagrams: bool,
    #[serde(default)]
    pub openfga_highlighting: bool,
    #[serde(default)]
    pub show_tab_emojis: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui_font: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mono_font: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui_font_size: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_font_size: Option<u32>,
    #[serde(default)]
    pub preferred_ides: Vec<String>,
}

fn settings_path() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("settings.json"))
}

pub fn load_settings() -> Result<AppSettings, String> {
    let path = settings_path()?;
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    let data =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read settings: {}", e))?;
    serde_json::from_str(&data).map_err(|e| format!("Failed to parse settings: {}", e))
}

pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let path = settings_path()?;
    let data = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    std::fs::write(&path, data).map_err(|e| format!("Failed to write settings: {}", e))
}

// ─── Storage Settings (per-storage) ─────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct StorageSettings {
    #[serde(default)]
    pub mcp_servers: Vec<McpServer>,
    #[serde(
        default,
        alias = "default_directories",
        deserialize_with = "deserialize_default_repositories"
    )]
    pub default_repositories: Vec<Repository>,
    #[serde(default)]
    pub extensions: Vec<Extension>,
    #[serde(default)]
    pub skills: Vec<Skill>,
}

impl StorageSettings {
    /// Returns all MCP servers: user-configured + enabled extension servers.
    pub fn all_mcp_servers(&self) -> Vec<McpServer> {
        let mut servers = self.mcp_servers.clone();
        for ext in &self.extensions {
            if ext.enabled {
                if let Some(ref mcp) = ext.mcp_server {
                    let mut server = mcp.clone();
                    server.name = ext.id.clone();
                    servers.push(server);
                }
            }
        }
        servers
    }
}

fn storage_settings_path(storage_path: &Path) -> PathBuf {
    storage_path.join("settings.json")
}

/// Load storage-specific settings. On first access, migrates from global settings if needed.
pub fn load_storage_settings(storage_path: &Path) -> Result<StorageSettings, String> {
    let path = storage_settings_path(storage_path);
    if path.exists() {
        let data = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read storage settings: {}", e))?;
        let settings: StorageSettings = serde_json::from_str(&data)
            .map_err(|e| format!("Failed to parse storage settings: {}", e))?;
        return Ok(settings);
    }

    // Migration: extract storage-specific fields from old global settings.json
    let global_path = settings_path().ok();
    let mut migrated = StorageSettings::default();

    if let Some(ref gp) = global_path {
        if gp.exists() {
            if let Ok(data) = std::fs::read_to_string(gp) {
                if let Ok(raw) = serde_json::from_str::<serde_json::Value>(&data) {
                    // Check if global has storage-specific fields to migrate
                    let has_storage_fields = raw.get("mcp_servers").is_some()
                        || raw.get("default_repositories").is_some()
                        || raw.get("default_directories").is_some()
                        || raw.get("extensions").is_some()
                        || raw.get("skills").is_some();

                    if has_storage_fields {
                        if let Ok(from_global) = serde_json::from_str::<StorageSettings>(&data) {
                            migrated = from_global;
                        }

                        if let Ok(global) = load_settings() {
                            let _ = save_settings(&global);
                        }
                    }
                }
            }
        }
    }

    let _ = save_storage_settings(storage_path, &migrated);
    Ok(migrated)
}

pub fn save_storage_settings(
    storage_path: &Path,
    settings: &StorageSettings,
) -> Result<(), String> {
    let path = storage_settings_path(storage_path);
    let data = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize storage settings: {}", e))?;
    std::fs::write(&path, data).map_err(|e| format!("Failed to write storage settings: {}", e))
}

/// Backwards-compatible deserializer: accepts old DefaultDirectory format (path+description),
/// simple strings, or new Repository format (url+name+description). Old path-based entries are dropped.
fn deserialize_default_repositories<'de, D>(deserializer: D) -> Result<Vec<Repository>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum RepoEntry {
        Full(Repository),
        Legacy {
            path: String,
            description: Option<String>,
        },
        Simple(String),
    }

    let entries: Vec<RepoEntry> = Vec::deserialize(deserializer)?;
    Ok(entries
        .into_iter()
        .filter_map(|e| match e {
            RepoEntry::Full(r) => Some(r),
            // Legacy entries with local paths are dropped; if it looks like a URL, migrate it
            RepoEntry::Legacy { path, description } => {
                if path.contains("://") || path.ends_with(".git") {
                    Some(Repository {
                        url: path,
                        name: None,
                        description,
                    })
                } else {
                    None // Drop old local-path entries
                }
            }
            RepoEntry::Simple(s) => {
                if s.contains("://") || s.ends_with(".git") {
                    Some(Repository {
                        url: s,
                        name: None,
                        description: None,
                    })
                } else {
                    None
                }
            }
        })
        .collect())
}

// ─── Notifications ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppNotification {
    pub message: String,
    pub feature_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_id: Option<String>,
    pub timestamp: String,
}

pub fn notifications_path() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("notifications.jsonl"))
}

/// Append a notification to the shared notifications file.
pub fn push_notification(message: &str, feature_id: Option<&str>) -> Result<(), String> {
    push_notification_ex(message, feature_id, None)
}

/// Append a notification with optional plan_id to the shared notifications file.
pub fn push_notification_ex(
    message: &str,
    feature_id: Option<&str>,
    plan_id: Option<&str>,
) -> Result<(), String> {
    use std::io::Write;
    let path = notifications_path()?;
    let notif = AppNotification {
        message: message.to_string(),
        feature_id: feature_id.map(|s| s.to_string()),
        plan_id: plan_id.map(|s| s.to_string()),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    let mut line = serde_json::to_string(&notif)
        .map_err(|e| format!("Failed to serialize notification: {}", e))?;
    line.push('\n');
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("Failed to open notifications file: {}", e))?;
    file.write_all(line.as_bytes())
        .map_err(|e| format!("Failed to write notification: {}", e))?;
    Ok(())
}
