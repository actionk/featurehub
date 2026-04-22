use serde::Serialize;
use std::path::Path;
use tauri::State;

use crate::extensions::manifest::ExtensionManifest;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct RequiresStatusInfo {
    pub name: String,
    pub found: bool,
    pub path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExtensionInfo {
    pub manifest: ExtensionManifest,
    pub enabled: bool,
    pub dir: String,
    pub requires_status: Vec<RequiresStatusInfo>,
}

#[tauri::command]
pub fn get_extensions(state: State<'_, AppState>) -> Result<Vec<ExtensionInfo>, String> {
    let registry = state.extensions.lock().map_err(|e| e.to_string())?;
    Ok(registry
        .extensions
        .iter()
        .map(|ext| ExtensionInfo {
            manifest: ext.manifest.clone(),
            enabled: ext.enabled,
            dir: ext.dir.to_string_lossy().to_string(),
            requires_status: ext
                .requires_status
                .iter()
                .map(|r| RequiresStatusInfo {
                    name: r.name.clone(),
                    found: r.found,
                    path: r.path.clone(),
                })
                .collect(),
        })
        .collect())
}

#[tauri::command]
pub fn get_extension_badge(
    state: State<'_, AppState>,
    extension_id: String,
    tab_id: String,
    feature_id: String,
) -> Result<i64, String> {
    let registry = state.extensions.lock().map_err(|e| e.to_string())?;
    let ext = registry
        .extensions
        .iter()
        .find(|e| e.manifest.id == extension_id && e.enabled)
        .ok_or_else(|| format!("Extension '{}' not found or disabled", extension_id))?;
    let tab = ext
        .manifest
        .tabs
        .iter()
        .find(|t| t.id == tab_id)
        .ok_or_else(|| format!("Tab '{}' not found in extension '{}'", tab_id, extension_id))?;
    let query = match &tab.badge_query {
        Some(q) => q.clone(),
        None => return Ok(0),
    };
    drop(registry); // Release extension lock before acquiring DB lock
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let count: i64 = conn
        .query_row(&query, rusqlite::params![feature_id], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    Ok(count)
}

pub fn get_extension_settings_inner(
    storage_path: &Path,
    key: &str,
) -> Result<serde_json::Value, String> {
    let path = storage_path.join("settings.json");
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    Ok(v.get("extension_settings")
        .and_then(|e| e.get(key))
        .cloned()
        .unwrap_or_else(|| serde_json::json!({})))
}

pub fn set_extension_settings_inner(
    storage_path: &Path,
    key: &str,
    value: serde_json::Value,
) -> Result<(), String> {
    let path = storage_path.join("settings.json");
    let mut root: serde_json::Value = if path.exists() {
        let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&text).map_err(|e| e.to_string())?
    } else {
        serde_json::json!({})
    };
    if !root.is_object() {
        root = serde_json::json!({});
    }
    let extension_settings = root
        .as_object_mut()
        .unwrap()
        .entry("extension_settings".to_string())
        .or_insert_with(|| serde_json::json!({}));
    if !extension_settings.is_object() {
        *extension_settings = serde_json::json!({});
    }
    extension_settings
        .as_object_mut()
        .unwrap()
        .insert(key.to_string(), value);
    let text = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?;
    std::fs::write(&path, text).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_extension_settings(
    state: State<'_, AppState>,
    key: String,
) -> Result<serde_json::Value, String> {
    let sp_guard = state.storage_path.lock().map_err(|e| e.to_string())?;
    let sp = sp_guard.as_ref().ok_or("No active storage")?;
    get_extension_settings_inner(sp, &key)
}

#[tauri::command]
pub fn set_extension_settings(
    state: State<'_, AppState>,
    key: String,
    value: serde_json::Value,
) -> Result<(), String> {
    let sp_guard = state.storage_path.lock().map_err(|e| e.to_string())?;
    let sp = sp_guard.as_ref().ok_or("No active storage")?;
    set_extension_settings_inner(sp, &key, value)
}

#[tauri::command]
pub async fn invoke_extension_tool(
    state: tauri::State<'_, AppState>,
    extension_id: String,
    tool_name: String,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let (handler_path, timeout, ext_id) = {
        let registry = state.extensions.lock().map_err(|e| e.to_string())?;
        let ext = registry
            .extensions
            .iter()
            .find(|e| e.manifest.id == extension_id && e.enabled)
            .ok_or_else(|| format!("Extension '{}' not found or disabled", extension_id))?;
        let tool = ext
            .manifest
            .tools
            .iter()
            .find(|t| t.name == tool_name)
            .ok_or_else(|| format!("Tool '{}' not found in '{}'", tool_name, extension_id))?;
        (
            ext.dir.join(&tool.handler),
            tool.timeout_secs.unwrap_or(10),
            ext.manifest.id.clone(),
        )
    };
    let (db_path, storage_path) = {
        let sp_guard = state.storage_path.lock().map_err(|e| e.to_string())?;
        let sp = sp_guard.as_ref().ok_or("No active storage")?;
        (
            sp.join("feature-hub.db").to_string_lossy().to_string(),
            sp.to_string_lossy().to_string(),
        )
    };
    let params_obj = params.as_object().cloned().unwrap_or_default();
    let input = crate::extensions::script_runner::ScriptInput {
        params: params_obj,
        db_path,
        storage_path,
        feature_id: None,
    };
    let result = tokio::task::spawn_blocking(move || {
        crate::extensions::script_runner::run_blocking_with_notifications(
            &handler_path,
            &input,
            timeout,
        )
    })
    .await
    .map_err(|e| e.to_string())??;
    crate::extensions::script_runner::forward_notifications(&ext_id, result.notifications);
    Ok(result.data)
}

#[tauri::command]
pub fn shell_open_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| format!("Failed to open URL: {}", e))
}

#[tauri::command]
pub fn restart_extension_schedules(state: State<'_, AppState>) -> Result<(), String> {
    let storage_path = {
        let sp_guard = state.storage_path.lock().map_err(|e| e.to_string())?;
        sp_guard.as_ref().ok_or("No active storage")?.clone()
    };
    let new_handles = {
        let registry = state.extensions.lock().map_err(|e| e.to_string())?;
        crate::spawn_extension_schedules(&registry, &storage_path)
    };
    let mut handles = state.schedule_handles.lock().map_err(|e| e.to_string())?;
    handles.clear();
    handles.extend(new_handles);
    Ok(())
}

#[cfg(test)]
mod settings_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn roundtrip_settings() {
        let tmp = TempDir::new().unwrap();
        let sp = tmp.path();
        std::fs::write(sp.join("settings.json"), r#"{"extension_settings": {}}"#).unwrap();

        set_extension_settings_inner(
            sp,
            "github_prs",
            serde_json::json!({"poll_enabled": true, "poll_interval_secs": 180}),
        )
        .unwrap();
        let got = get_extension_settings_inner(sp, "github_prs").unwrap();
        assert_eq!(got["poll_enabled"], true);
        assert_eq!(got["poll_interval_secs"], 180);
    }

    #[test]
    fn returns_empty_object_for_unknown_key() {
        let tmp = TempDir::new().unwrap();
        let sp = tmp.path();
        std::fs::write(sp.join("settings.json"), r#"{"extension_settings": {}}"#).unwrap();
        let got = get_extension_settings_inner(sp, "nonexistent").unwrap();
        assert!(got.is_object());
        assert_eq!(got.as_object().unwrap().len(), 0);
    }

    #[test]
    fn set_creates_file_and_keys_when_missing() {
        let tmp = TempDir::new().unwrap();
        let sp = tmp.path();
        // No settings.json exists
        set_extension_settings_inner(
            sp,
            "github_prs",
            serde_json::json!({"poll_enabled": false}),
        )
        .unwrap();
        let got = get_extension_settings_inner(sp, "github_prs").unwrap();
        assert_eq!(got["poll_enabled"], false);
    }

    #[test]
    fn set_preserves_existing_top_level_keys() {
        let tmp = TempDir::new().unwrap();
        let sp = tmp.path();
        std::fs::write(
            sp.join("settings.json"),
            r#"{"extensions": [{"id":"jira","enabled":true,"instructions":""}], "extension_settings": {}}"#,
        )
        .unwrap();
        set_extension_settings_inner(
            sp,
            "github_prs",
            serde_json::json!({"poll_enabled": true}),
        )
        .unwrap();
        let text = std::fs::read_to_string(sp.join("settings.json")).unwrap();
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert!(v.get("extensions").is_some(), "extensions array preserved");
        assert_eq!(v["extension_settings"]["github_prs"]["poll_enabled"], true);
    }
}
