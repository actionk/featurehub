use tauri::State;

use crate::db;
use crate::storage;
use crate::AppState;

#[derive(serde::Serialize)]
pub struct StorageInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub is_active: bool,
    pub git_status: String,
    pub icon: Option<String>,
}

#[tauri::command]
pub fn get_storages(app: tauri::AppHandle) -> Result<Vec<StorageInfo>, String> {
    let config = storage::load_config(&app)?;
    let active_id = config.active_storage_id.as_deref();
    Ok(config
        .storages
        .iter()
        .map(|s| StorageInfo {
            id: s.id.clone(),
            name: storage::storage_name(s),
            path: s.path.clone(),
            is_active: active_id == Some(&s.id),
            git_status: storage::check_git_status(&s.path),
            icon: s.icon.clone(),
        })
        .collect())
}

#[tauri::command]
pub fn get_active_storage(app: tauri::AppHandle) -> Result<Option<StorageInfo>, String> {
    let config = storage::load_config(&app)?;
    match &config.active_storage_id {
        Some(active_id) => Ok(config
            .storages
            .iter()
            .find(|s| s.id == *active_id)
            .map(|s| StorageInfo {
                id: s.id.clone(),
                name: storage::storage_name(s),
                path: s.path.clone(),
                is_active: true,
                git_status: storage::check_git_status(&s.path),
                icon: s.icon.clone(),
            })),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn create_storage(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<StorageInfo, String> {
    let entry = storage::add_storage(&app, &path)?;
    let name = storage::storage_name(&entry);

    // If this is the first/only storage, switch to it now
    let config = storage::load_config(&app)?;
    let is_only = config.storages.len() == 1;
    if is_only {
        do_switch_storage(&app, &state, &entry.id)?;
    }

    Ok(StorageInfo {
        id: entry.id,
        name,
        path: entry.path.clone(),
        is_active: is_only,
        git_status: storage::check_git_status(&entry.path),
        icon: entry.icon,
    })
}

#[tauri::command]
pub fn switch_storage(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    do_switch_storage(&app, &state, &id)
}

fn do_switch_storage(
    app: &tauri::AppHandle,
    state: &State<'_, AppState>,
    id: &str,
) -> Result<(), String> {
    let mut config = storage::load_config(app)?;
    let entry = config
        .storages
        .iter()
        .find(|s| s.id == id)
        .ok_or("Storage not found")?
        .clone();

    let path = std::path::PathBuf::from(&entry.path);
    let db_path = path.join("feature-hub.db");

    std::fs::create_dir_all(&path)
        .map_err(|e| format!("Failed to create storage directory: {}", e))?;
    std::fs::create_dir_all(path.join("workspaces"))
        .map_err(|e| format!("Failed to create workspaces directory: {}", e))?;

    let new_conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    db::initialize(&new_conn).map_err(|e| format!("Failed to initialize database: {}", e))?;
    db::migrate_to_relative_paths(&new_conn, &path);

    // Load and provision extensions into new_conn BEFORE installing it in AppState,
    // so no other thread can observe the new connection with unprovisioned tables.
    let ext_dir = path.join("extensions");
    let new_registry = crate::extensions::ExtensionRegistry::load_from_dir(&ext_dir);
    for ext in &new_registry.extensions {
        for table in &ext.manifest.tables {
            if let Err(e) = crate::extensions::table_provisioner::provision_table(&new_conn, table)
            {
                eprintln!(
                    "[extensions] Table provisioning failed on storage switch: {}",
                    e
                );
            }
        }
    }

    {
        let mut db_guard = state.db.lock().map_err(|e| e.to_string())?;
        *db_guard = new_conn;
    }
    {
        let mut sp = state.storage_path.lock().map_err(|e| e.to_string())?;
        *sp = Some(path.clone());
    }
    {
        let mut ext_guard = state.extensions.lock().map_err(|e| e.to_string())?;
        *ext_guard = new_registry;
    }
    state.stats_cache.lock().map_err(|e| e.to_string())?.clear();
    state
        .jsonl_path_cache
        .lock()
        .map_err(|e| e.to_string())?
        .clear();
    state
        .status_hint_cache
        .lock()
        .map_err(|e| e.to_string())?
        .clear();

    config.active_storage_id = Some(id.to_string());
    storage::save_config(app, &config)
}

#[tauri::command]
pub fn rename_storage(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
    new_path: String,
) -> Result<StorageInfo, String> {
    let config = storage::load_config(&app)?;
    let is_active = config.active_storage_id.as_deref() == Some(&id);

    // If this is the active storage, fully close the DB connection so the
    // folder isn't locked by open SQLite file handles (incl. WAL/SHM).
    if is_active {
        let old_conn;
        {
            let mut db_guard = state.db.lock().map_err(|e| e.to_string())?;
            let tmp_conn = rusqlite::Connection::open_in_memory()
                .map_err(|e| format!("Failed to create temporary connection: {}", e))?;
            // Take ownership of the old connection so we can close it explicitly.
            old_conn = std::mem::replace(&mut *db_guard, tmp_conn);
        }
        {
            let mut sp = state.storage_path.lock().map_err(|e| e.to_string())?;
            *sp = None;
        }
        // Checkpoint WAL and switch journal mode BEFORE closing so the
        // -wal/-shm files are cleaned up.
        let _ =
            old_conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE); PRAGMA journal_mode=DELETE;");
        // Explicitly close — unlike Drop, this ensures all handles are released.
        // close() consumes the connection and returns any unfinalized-statement errors.
        if let Err((_conn, e)) = old_conn.close() {
            eprintln!("Warning: failed to close old DB connection: {}", e);
        }
    }

    let entry = match storage::rename_storage(&app, &id, &new_path) {
        Ok(e) => e,
        Err(err) => {
            // Move failed — restore the original DB connection if we closed it
            if is_active {
                if let Some(old_entry) = config.storages.iter().find(|s| s.id == id) {
                    let old_path = std::path::PathBuf::from(&old_entry.path);
                    if let Ok(conn) = rusqlite::Connection::open(old_path.join("feature-hub.db")) {
                        let _ = db::initialize(&conn);
                        if let Ok(mut db_guard) = state.db.lock() {
                            *db_guard = conn;
                        }
                        if let Ok(mut sp) = state.storage_path.lock() {
                            *sp = Some(old_path);
                        }
                    }
                }
            }
            return Err(err);
        }
    };

    // Re-open the DB at the new location
    if is_active {
        let path = std::path::PathBuf::from(&entry.path);
        let db_path = path.join("feature-hub.db");

        let new_conn = rusqlite::Connection::open(&db_path)
            .map_err(|e| format!("Failed to open database at new location: {}", e))?;
        db::initialize(&new_conn).map_err(|e| format!("Failed to initialize database: {}", e))?;

        {
            let mut db_guard = state.db.lock().map_err(|e| e.to_string())?;
            *db_guard = new_conn;
        }
        {
            let mut sp = state.storage_path.lock().map_err(|e| e.to_string())?;
            *sp = Some(path);
        }
        // Note: state.extensions is intentionally NOT reloaded here. Extension
        // handler paths will be stale until the next app restart, but this is
        // acceptable: rename_storage is infrequent and extension event hooks will
        // silently fail-and-log rather than crash. A restart corrects the paths.
    }

    let name = storage::storage_name(&entry);
    let git_status = storage::check_git_status(&entry.path);
    let icon = entry.icon;
    Ok(StorageInfo {
        id: entry.id,
        name,
        path: entry.path,
        is_active,
        git_status,
        icon,
    })
}

#[tauri::command]
pub fn remove_storage(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let config = storage::load_config(&app)?;
    let was_active = config.active_storage_id.as_deref() == Some(&id);

    storage::remove_storage(&app, &id)?;

    if was_active {
        let new_config = storage::load_config(&app)?;
        if let Some(next) = new_config.storages.first() {
            do_switch_storage(&app, &state, &next.id.clone())?;
        } else {
            let new_conn = rusqlite::Connection::open_in_memory()
                .map_err(|e| format!("Failed to open in-memory database: {}", e))?;
            db::initialize(&new_conn)
                .map_err(|e| format!("Failed to initialize database: {}", e))?;
            {
                let mut db_guard = state.db.lock().map_err(|e| e.to_string())?;
                *db_guard = new_conn;
            }
            {
                let mut sp = state.storage_path.lock().map_err(|e| e.to_string())?;
                *sp = None;
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub fn update_storage_icon(
    app: tauri::AppHandle,
    id: String,
    icon: Option<String>,
) -> Result<StorageInfo, String> {
    let entry = storage::update_storage_icon(&app, &id, icon.as_deref())?;
    let config = storage::load_config(&app)?;
    let is_active = config.active_storage_id.as_deref() == Some(id.as_str());
    let name = storage::storage_name(&entry);
    let git_status = storage::check_git_status(&entry.path);
    Ok(StorageInfo {
        id: entry.id,
        name,
        path: entry.path,
        is_active,
        git_status,
        icon: entry.icon,
    })
}

#[tauri::command]
pub async fn get_storage_git_status(path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || Ok(storage::check_git_status(&path)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn pick_storage_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file().pick_folder(|folder| {
        let _ = tx.send(folder);
    });
    let result = rx.await.map_err(|e| e.to_string())?;
    Ok(result.map(|f| f.to_string()))
}
