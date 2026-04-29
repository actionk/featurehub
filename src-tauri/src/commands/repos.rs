use tauri::State;

use crate::db;
use crate::AppState;

#[tauri::command]
pub fn add_directory(
    state: State<'_, AppState>,
    feature_id: String,
    path: String,
    label: Option<String>,
) -> Result<db::directories::Directory, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::directories::add_directory(&conn, &feature_id, &path, label, None, None)
}

#[tauri::command]
pub fn remove_directory(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let storage = state.storage_path.lock().map_err(|e| e.to_string())?;

    // If this directory was a cloned repo, delete the folder from disk
    if let Ok(dir) = db::directories::get_directory(&conn, &id) {
        if dir.repo_url.is_some() {
            let resolved = if let Some(ref base) = *storage {
                crate::paths::resolve_path(&dir.path, base)
            } else {
                std::path::PathBuf::from(&dir.path)
            };
            if resolved.exists() {
                let _ = std::fs::remove_dir_all(&resolved);
            }
        }
    }

    db::directories::remove_directory(&conn, &id)
}

/// Clone a repository into a feature's workspace directory.
/// Returns the directory record immediately; cloning happens in background via Tauri async.
#[tauri::command]
pub async fn clone_repository(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    feature_id: String,
    repo_url: String,
    name: Option<String>,
) -> Result<db::directories::Directory, String> {
    // Derive name from URL if not provided
    let repo_name = name.unwrap_or_else(|| {
        repo_url
            .rsplit('/')
            .next()
            .unwrap_or("repo")
            .trim_end_matches(".git")
            .to_string()
    });

    // Compute target path: workspaces/<feature-id>/<name>/
    let (target_path, target_str) = {
        let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
        let base = storage.as_ref().ok_or("No active storage")?;
        let abs_path = base.join("workspaces").join(&feature_id).join(&repo_name);
        let rel_str = crate::paths::to_storage_relative(&abs_path.to_string_lossy(), base);
        (abs_path, rel_str)
    };

    // Ensure parent dir exists
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create workspace dir: {}", e))?;
    }

    // Insert directory record with clone_status="cloning"
    let dir = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::directories::add_directory(
            &conn,
            &feature_id,
            &target_str,
            Some(repo_name.clone()),
            Some(repo_url.clone()),
            Some("cloning".to_string()),
        )?
    };

    // Spawn blocking clone task
    let dir_id = dir.id.clone();
    let url = repo_url.clone();
    let target = target_path.clone();
    let app_handle = app.clone();
    let state_db_path = {
        let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
        let base = storage.as_ref().ok_or("No active storage")?;
        base.join("feature-hub.db")
    };

    tauri::async_runtime::spawn_blocking(move || {
        let result = crate::git::clone_repo(&url, &target);

        // Open a fresh DB connection for the background task
        if let Ok(conn) = rusqlite::Connection::open(&state_db_path) {
            match result {
                Ok(()) => {
                    let _ = db::directories::update_clone_status(&conn, &dir_id, "ready", None);
                    use tauri::Emitter;
                    let _ = app_handle.emit("clone-complete", &dir_id);
                }
                Err(ref e) => {
                    let _ = db::directories::update_clone_status(&conn, &dir_id, "failed", Some(e));
                    use tauri::Emitter;
                    let _ = app_handle.emit(
                        "clone-failed",
                        serde_json::json!({ "id": dir_id, "error": e }),
                    );
                }
            }
        }
    });

    // Return with resolved absolute path for the frontend
    let mut resolved_dir = dir;
    {
        let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
        if let Some(ref base) = *storage {
            resolved_dir.path = crate::paths::resolve_path_string(&resolved_dir.path, base);
        }
    }
    Ok(resolved_dir)
}

/// Retry a failed clone — deletes the failed directory and re-runs the clone.
#[tauri::command]
pub async fn retry_clone(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    directory_id: String,
) -> Result<db::directories::Directory, String> {
    let (dir, state_db_path) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let dir = db::directories::get_directory(&conn, &directory_id)?;
        let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
        let base = storage.as_ref().ok_or("No active storage")?;
        (dir, base.join("feature-hub.db"))
    };

    let repo_url = dir.repo_url.clone().ok_or("Not a cloned repository")?;

    // Clean up failed directory on disk — resolve relative path
    let target = {
        let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
        let base = storage.as_ref().ok_or("No active storage")?;
        crate::paths::resolve_path(&dir.path, base)
    };
    if target.exists() {
        let _ = std::fs::remove_dir_all(&target);
    }

    // Reset status to cloning
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::directories::update_clone_status(&conn, &directory_id, "cloning", None)?;
    }

    // Spawn blocking clone task
    let dir_id = directory_id.clone();
    let app_handle = app.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let result = crate::git::clone_repo(&repo_url, &target);

        if let Ok(conn) = rusqlite::Connection::open(&state_db_path) {
            match result {
                Ok(()) => {
                    let _ = db::directories::update_clone_status(&conn, &dir_id, "ready", None);
                    use tauri::Emitter;
                    let _ = app_handle.emit("clone-complete", &dir_id);
                }
                Err(ref e) => {
                    let _ = db::directories::update_clone_status(&conn, &dir_id, "failed", Some(e));
                    use tauri::Emitter;
                    let _ = app_handle.emit(
                        "clone-failed",
                        serde_json::json!({ "id": dir_id, "error": e }),
                    );
                }
            }
        }
    });

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut dir = db::directories::get_directory(&conn, &directory_id)?;
    let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
    if let Some(ref base) = *storage {
        dir.path = crate::paths::resolve_path_string(&dir.path, base);
    }
    Ok(dir)
}

/// Remove cloned repo directories and workspace for a feature without deleting the feature itself.
#[tauri::command]
pub fn cleanup_feature_repos(state: State<'_, AppState>, feature_id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let storage = state.storage_path.lock().map_err(|e| e.to_string())?;

    // Delete cloned repo directories from disk
    let dirs = db::directories::get_directories(&conn, &feature_id)?;
    for dir in &dirs {
        if dir.repo_url.is_some() {
            let resolved = if let Some(ref base) = *storage {
                crate::paths::resolve_path(&dir.path, base)
            } else {
                std::path::PathBuf::from(&dir.path)
            };
            if resolved.exists() {
                let _ = std::fs::remove_dir_all(&resolved);
            }
        }
    }

    // Delete workspace directory
    if let Some(ref base) = *storage {
        let workspace_dir = base.join("workspaces").join(&feature_id);
        if workspace_dir.exists() {
            let _ = std::fs::remove_dir_all(&workspace_dir);
        }
    }

    Ok(())
}
