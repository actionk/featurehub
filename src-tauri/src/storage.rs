use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageEntry {
    pub id: String,
    pub path: String,
    pub added_at: String,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct StorageConfig {
    pub storages: Vec<StorageEntry>,
    pub active_storage_id: Option<String>,
}

fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    Ok(data_dir.join("config.json"))
}

pub fn load_config(_app: &tauri::AppHandle) -> Result<StorageConfig, String> {
    // Delegate to standalone config loader — both resolve to the same path
    crate::config::load_config()
}

pub fn save_config(app: &tauri::AppHandle, config: &StorageConfig) -> Result<(), String> {
    let path = config_path(app)?;
    let data = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    std::fs::write(&path, data).map_err(|e| format!("Failed to write config: {}", e))
}

pub fn add_storage(app: &tauri::AppHandle, path: &str) -> Result<StorageEntry, String> {
    let storage_path = Path::new(path);

    // Create the storage folder and files subdirectory
    std::fs::create_dir_all(storage_path)
        .map_err(|e| format!("Failed to create storage directory: {}", e))?;
    std::fs::create_dir_all(storage_path.join("workspaces"))
        .map_err(|e| format!("Failed to create workspaces directory: {}", e))?;

    let mut config = load_config(app)?;

    // Check if already added
    if config.storages.iter().any(|s| s.path == path) {
        return Err("Storage at this path already exists".to_string());
    }

    let entry = StorageEntry {
        id: uuid::Uuid::new_v4().to_string(),
        path: path.to_string(),
        added_at: chrono::Utc::now().to_rfc3339(),
        icon: None,
    };

    config.storages.push(entry.clone());

    // If this is the first storage, make it active
    if config.active_storage_id.is_none() {
        config.active_storage_id = Some(entry.id.clone());
    }

    save_config(app, &config)?;
    Ok(entry)
}

pub fn remove_storage(app: &tauri::AppHandle, id: &str) -> Result<(), String> {
    let mut config = load_config(app)?;
    config.storages.retain(|s| s.id != id);

    // If the active one was removed, clear it
    if config.active_storage_id.as_deref() == Some(id) {
        config.active_storage_id = config.storages.first().map(|s| s.id.clone());
    }

    save_config(app, &config)
}

pub fn storage_name(entry: &StorageEntry) -> String {
    Path::new(&entry.path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| entry.path.clone())
}

pub fn update_storage_icon(
    app: &tauri::AppHandle,
    id: &str,
    icon: Option<&str>,
) -> Result<StorageEntry, String> {
    let mut config = load_config(app)?;
    let entry = config
        .storages
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or("Storage not found")?;
    entry.icon = icon.map(|s| s.to_string());
    let updated = entry.clone();
    save_config(app, &config)?;
    Ok(updated)
}

pub fn rename_storage(
    app: &tauri::AppHandle,
    id: &str,
    new_path: &str,
) -> Result<StorageEntry, String> {
    let mut config = load_config(app)?;
    let entry = config
        .storages
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or("Storage not found")?;

    let old_path = PathBuf::from(&entry.path);
    let new_path_buf = PathBuf::from(new_path);

    // Don't do anything if paths are the same
    if old_path == new_path_buf {
        return Ok(entry.clone());
    }

    // Check the new path doesn't already exist (unless it's the same dir)
    if new_path_buf.exists() {
        return Err(format!(
            "Destination already exists: {}",
            new_path_buf.display()
        ));
    }

    // Ensure the parent directory of the new path exists
    if let Some(parent) = new_path_buf.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create parent directory: {}", e))?;
    }

    // Move the folder — try rename first, fall back to copy+delete
    if old_path.exists() {
        if let Err(_) = std::fs::rename(&old_path, &new_path_buf) {
            // rename() fails on Windows when files were recently open or
            // when moving across drives. Fall back to recursive copy + delete.
            copy_dir_recursive(&old_path, &new_path_buf).map_err(|e| {
                format!(
                    "Failed to copy storage from '{}' to '{}': {}",
                    old_path.display(),
                    new_path_buf.display(),
                    e
                )
            })?;
            // Try to remove old dir — retry a few times in case of lingering handles
            let mut removed = false;
            for attempt in 0..5 {
                if attempt > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
                if std::fs::remove_dir_all(&old_path).is_ok() {
                    removed = true;
                    break;
                }
            }
            if !removed {
                eprintln!(
                    "Warning: could not remove old storage directory '{}' — you may delete it manually",
                    old_path.display()
                );
            }
        }
    } else {
        // Old path doesn't exist, just create at new location
        std::fs::create_dir_all(&new_path_buf)
            .map_err(|e| format!("Failed to create storage directory: {}", e))?;
        std::fs::create_dir_all(new_path_buf.join("workspaces"))
            .map_err(|e| format!("Failed to create workspaces directory: {}", e))?;
    }

    entry.path = new_path.to_string();
    let updated = entry.clone();
    save_config(app, &config)?;

    // Run migration on the DB at the new location to convert any remaining absolute paths
    let db_path = new_path_buf.join("feature-hub.db");
    if db_path.exists() {
        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
            crate::db::migrate_to_relative_paths(&conn, &new_path_buf);
        }
    }

    Ok(updated)
}

pub fn get_active_storage(app: &tauri::AppHandle) -> Result<Option<StorageEntry>, String> {
    let config = load_config(app)?;
    match &config.active_storage_id {
        Some(id) => Ok(config.storages.iter().find(|s| s.id == *id).cloned()),
        None => Ok(None),
    }
}

/// Recursively copy a directory tree.
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

/// Check git status of a storage folder.
/// Returns: "clean" (up to date with remote), "dirty" (unpushed/uncommitted changes), "none" (not a git repo)
pub fn check_git_status(storage_path: &str) -> String {
    let path = Path::new(storage_path);

    // Check if it's a git repo
    let is_git = std::process::Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(path)
        .output();

    match is_git {
        Ok(output) if output.status.success() => {}
        _ => return "none".to_string(),
    }

    // Check for uncommitted changes (staged + unstaged + untracked)
    let status = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(path)
        .output();

    if let Ok(output) = status {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            return "dirty".to_string();
        }
    }

    // Check for unpushed commits
    let ahead = std::process::Command::new("git")
        .args(["rev-list", "--count", "@{upstream}..HEAD"])
        .current_dir(path)
        .output();

    match ahead {
        Ok(output) if output.status.success() => {
            let count = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if count != "0" {
                return "dirty".to_string();
            }
        }
        _ => {
            // No upstream configured — treat as dirty (not pushed)
            return "dirty".to_string();
        }
    }

    "clean".to_string()
}
