use tauri::State;

use crate::db;
use crate::files::manager as file_manager;
use crate::AppState;

#[tauri::command]
pub fn get_files(
    state: State<'_, AppState>,
    feature_id: String,
) -> Result<Vec<db::files::FileEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::files::get_files(&conn, &feature_id)
}

#[tauri::command]
pub fn add_files(
    state: State<'_, AppState>,
    feature_id: String,
    paths: Vec<String>,
    folder_id: Option<String>,
) -> Result<Vec<db::files::FileEntry>, String> {
    let (base, subfolder) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
        let base = storage
            .as_deref()
            .ok_or("No active storage path")?
            .to_path_buf();
        let subfolder = match &folder_id {
            Some(fid) => Some(db::folders::get_folder_path(&conn, fid)?),
            None => None,
        };
        (base, subfolder)
    }; // Both locks released before file copies

    // Copy files to storage (disk I/O without holding locks)
    let mut copied = Vec::new();
    for source_path in &paths {
        let (filename, dest_path, size) = file_manager::copy_file_to_storage(
            &base,
            &feature_id,
            source_path,
            subfolder.as_deref(),
        )?;
        copied.push((filename, source_path.clone(), dest_path, size));
    }

    // Re-acquire DB lock to insert records
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut entries = Vec::new();
    for (filename, source_path, dest_path, size) in &copied {
        let entry = db::files::add_file(
            &conn,
            &feature_id,
            filename,
            source_path,
            dest_path,
            *size,
            folder_id.as_deref(),
        )?;
        entries.push(entry);
    }

    Ok(entries)
}

#[tauri::command]
pub fn delete_file(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let (stored_path_raw, storage_base) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
        let base = storage.as_deref().map(|b| b.to_path_buf());
        (db::files::delete_file(&conn, &id)?, base)
    }; // Locks released before disk I/O

    // Resolve relative path and delete the actual file from disk
    let resolved = match &storage_base {
        Some(base) => crate::paths::resolve_path_string(&stored_path_raw, base),
        None => stored_path_raw,
    };
    if std::path::Path::new(&resolved).exists() {
        std::fs::remove_file(&resolved)
            .map_err(|e| format!("Failed to delete file from disk: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub fn open_file(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let resolved = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let raw = db::files::open_file(&conn, &id)?;
        let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
        match storage.as_deref() {
            Some(base) => crate::paths::resolve_path_string(&raw, base),
            None => raw,
        }
    };

    open::that(&resolved).map_err(|e| format!("Failed to open file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn get_files_directory(
    state: State<'_, AppState>,
    feature_id: String,
) -> Result<String, String> {
    let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
    let base = storage.as_deref().ok_or("No active storage path")?;
    let dir = file_manager::get_storage_path(base, &feature_id);
    Ok(dir.to_string_lossy().to_string())
}

#[tauri::command]
pub fn open_files_directory(state: State<'_, AppState>, feature_id: String) -> Result<(), String> {
    let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
    let base = storage.as_deref().ok_or("No active storage path")?;
    let dir = file_manager::get_storage_path(base, &feature_id);
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create directory: {}", e))?;
    open::that(&dir).map_err(|e| format!("Failed to open directory: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn sync_workspace_files(
    state: State<'_, AppState>,
    feature_id: String,
) -> Result<Vec<db::files::FileEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
    let base = storage.as_deref().ok_or("No active storage path")?;

    let workspace_dir = file_manager::get_storage_path(base, &feature_id);
    if !workspace_dir.exists() {
        return Ok(vec![]);
    }

    // Collect all stored_paths already tracked in the DB for this feature
    // Normalize to forward slashes for comparison
    let existing_files = db::files::get_files(&conn, &feature_id)?;
    let tracked: std::collections::HashMap<String, (String, i64)> = existing_files
        .into_iter()
        .map(|f| (f.stored_path.replace('\\', "/"), (f.id, f.size)))
        .collect();

    let mut new_entries = Vec::new();

    // Walk workspace dir (top-level only, skip hidden files/dirs)
    let read_dir = std::fs::read_dir(&workspace_dir)
        .map_err(|e| format!("Failed to read workspace directory: {}", e))?;

    for entry in read_dir {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();

        // Skip directories and hidden files
        if path.is_dir() {
            continue;
        }
        let file_name = match path.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };
        if file_name.starts_with('.') {
            continue;
        }

        // Store as relative path
        let stored_path_rel = crate::paths::to_storage_relative(&path.to_string_lossy(), base);
        let normalized = stored_path_rel.replace('\\', "/");

        if let Some((file_id, db_size)) = tracked.get(&normalized) {
            // File already tracked — update size if it changed on disk
            let disk_size = path.metadata().map(|m| m.len() as i64).unwrap_or(0);
            if disk_size != *db_size {
                db::files::update_file_size(&conn, file_id, disk_size).ok();
            }
            continue;
        }

        let size = path.metadata().map(|m| m.len() as i64).unwrap_or(0);
        let file_entry = db::files::add_file(
            &conn,
            &feature_id,
            &file_name,
            &stored_path_rel,
            &stored_path_rel,
            size,
            None,
        )?;
        new_entries.push(file_entry);
    }

    Ok(new_entries)
}

#[tauri::command]
pub fn reveal_file(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let resolved = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let raw = db::files::open_file(&conn, &id)?;
        let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
        match storage.as_deref() {
            Some(base) => crate::paths::resolve_path_string(&raw, base),
            None => raw,
        }
    };

    let path = std::path::Path::new(&resolved);
    if !path.exists() {
        return Err(format!("File does not exist: {}", resolved));
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .args(["/select,", &resolved])
            .spawn()
            .map_err(|e| format!("Failed to reveal file: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &resolved])
            .spawn()
            .map_err(|e| format!("Failed to reveal file: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(parent) = path.parent() {
            open::that(parent).map_err(|e| format!("Failed to reveal file: {}", e))?;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn get_file_path(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let raw = db::files::open_file(&conn, &id)?;
    let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
    Ok(match storage.as_deref() {
        Some(base) => crate::paths::resolve_path_string(&raw, base),
        None => raw,
    })
}

#[tauri::command]
pub fn open_path(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    open::that(p).map_err(|e| format!("Failed to open path: {}", e))?;
    Ok(())
}
