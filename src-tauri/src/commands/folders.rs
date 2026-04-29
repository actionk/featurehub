use tauri::State;

use crate::db;
use crate::files::manager as file_manager;
use crate::files::preview as file_preview;
use crate::AppState;

#[tauri::command]
pub fn get_folders(
    state: State<'_, AppState>,
    feature_id: String,
) -> Result<Vec<db::folders::Folder>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::folders::get_folders(&conn, &feature_id)
}

#[tauri::command]
pub fn create_folder(
    state: State<'_, AppState>,
    feature_id: String,
    parent_id: Option<String>,
    name: String,
) -> Result<db::folders::Folder, String> {
    let (folder, base, relative_path) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
        let base = storage
            .as_deref()
            .ok_or("No active storage path")?
            .to_path_buf();

        let folder = db::folders::create_folder(&conn, &feature_id, parent_id.as_deref(), &name)?;
        let relative_path = db::folders::get_folder_path(&conn, &folder.id)?;
        (folder, base, relative_path)
    }; // Locks released before disk I/O

    file_manager::create_folder_on_disk(&base, &feature_id, &relative_path)?;

    Ok(folder)
}

#[tauri::command]
pub fn rename_folder(
    state: State<'_, AppState>,
    id: String,
    new_name: String,
) -> Result<db::folders::Folder, String> {
    let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
    let base = storage
        .as_deref()
        .ok_or("No active storage path")?
        .to_path_buf();
    drop(storage);

    // Phase 1: DB operations — get old path, rename in DB, get new path
    let (folder, feature_id, old_disk_path, new_disk_path) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;

        let feature_id: String = conn
            .query_row(
                "SELECT feature_id FROM folders WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        let old_relative_path = db::folders::get_folder_path(&conn, &id)?;
        let old_disk_path =
            file_manager::get_storage_path(&base, &feature_id).join(&old_relative_path);

        let folder = db::folders::rename_folder(&conn, &id, &new_name)?;

        // Compute new path from old path (just swap the last component) instead of re-walking
        let new_disk_path = old_disk_path.with_file_name(&folder.name);

        (folder, feature_id, old_disk_path, new_disk_path)
    }; // DB lock released

    // Phase 2: Disk I/O without holding the lock
    file_manager::rename_folder_on_disk(
        &old_disk_path.to_string_lossy(),
        &new_disk_path.to_string_lossy(),
    )?;

    // Phase 3: Re-acquire lock to update stored paths
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        update_stored_paths_after_rename(&conn, &base, &feature_id, &id)?;
    }

    Ok(folder)
}

fn update_stored_paths_after_rename(
    conn: &rusqlite::Connection,
    base: &std::path::Path,
    feature_id: &str,
    folder_id: &str,
) -> Result<(), String> {
    // Update files directly in this folder
    let relative_path = db::folders::get_folder_path(conn, folder_id)?;
    let disk_dir = file_manager::get_storage_path(base, feature_id).join(&relative_path);

    let mut stmt = conn
        .prepare("SELECT id, filename FROM files WHERE folder_id = ?1")
        .map_err(|e| e.to_string())?;
    let file_rows: Vec<(String, String)> = stmt
        .query_map(rusqlite::params![folder_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for (file_id, filename) in &file_rows {
        // Store as relative path
        let new_stored_path = disk_dir
            .join(filename)
            .strip_prefix(base)
            .map(|r| r.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| disk_dir.join(filename).to_string_lossy().to_string());
        db::files::update_stored_path(conn, file_id, &new_stored_path)?;
    }

    // Recurse into child folders
    let mut child_stmt = conn
        .prepare("SELECT id FROM folders WHERE parent_id = ?1")
        .map_err(|e| e.to_string())?;
    let children: Vec<String> = child_stmt
        .query_map(rusqlite::params![folder_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for child_id in &children {
        update_stored_paths_after_rename(conn, base, feature_id, child_id)?;
    }

    Ok(())
}

#[tauri::command]
pub fn delete_folder(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let (base, feature_id, relative_path) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
        let base = storage
            .as_deref()
            .ok_or("No active storage path")?
            .to_path_buf();

        let feature_id: String = conn
            .query_row(
                "SELECT feature_id FROM folders WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        let relative_path = db::folders::get_folder_path(&conn, &id)?;

        // Delete from DB (also deletes contained files from DB)
        let _stored_paths = db::folders::delete_folder(&conn, &id)?;

        (base, feature_id, relative_path)
    }; // Locks released before disk I/O

    // Delete folder from disk (removes everything inside)
    file_manager::delete_folder_on_disk(&base, &feature_id, &relative_path)?;

    Ok(())
}

#[tauri::command]
pub fn move_folder(
    state: State<'_, AppState>,
    id: String,
    new_parent_id: Option<String>,
) -> Result<db::folders::Folder, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
    let base = storage.as_deref().ok_or("No active storage path")?;

    let feature_id: String = conn
        .query_row(
            "SELECT feature_id FROM folders WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    // Get old disk path
    let old_relative_path = db::folders::get_folder_path(&conn, &id)?;
    let old_disk_path = file_manager::get_storage_path(base, &feature_id).join(&old_relative_path);

    // Move in DB
    let folder = db::folders::move_folder(&conn, &id, new_parent_id.as_deref())?;

    // Get new disk path
    let new_relative_path = db::folders::get_folder_path(&conn, &id)?;
    let new_disk_path = file_manager::get_storage_path(base, &feature_id).join(&new_relative_path);

    // Move on disk
    if old_disk_path != new_disk_path && old_disk_path.exists() {
        std::fs::create_dir_all(new_disk_path.parent().unwrap_or(&new_disk_path))
            .map_err(|e| format!("Failed to create parent dir: {}", e))?;
        std::fs::rename(&old_disk_path, &new_disk_path)
            .map_err(|e| format!("Failed to move folder on disk: {}", e))?;
    }

    // Update stored paths for all files
    update_stored_paths_after_rename(&conn, base, &feature_id, &id)?;

    Ok(folder)
}

#[tauri::command]
pub fn move_file(
    state: State<'_, AppState>,
    id: String,
    folder_id: Option<String>,
) -> Result<db::files::FileEntry, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
    let base = storage.as_deref().ok_or("No active storage path")?;

    // Get current file info
    let old_stored_path_raw: String = conn
        .query_row(
            "SELECT stored_path FROM files WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let feature_id: String = conn
        .query_row(
            "SELECT feature_id FROM files WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    // Resolve relative path for disk operations
    let old_stored_path = crate::paths::resolve_path_string(&old_stored_path_raw, base);

    // Determine target directory
    let target_dir = match &folder_id {
        Some(fid) => {
            let rel_path = db::folders::get_folder_path(&conn, fid)?;
            file_manager::get_storage_path(base, &feature_id).join(rel_path)
        }
        None => file_manager::get_storage_path(base, &feature_id),
    };

    // Move file on disk (uses absolute paths)
    let new_stored_path_abs =
        file_manager::move_file_on_disk(&old_stored_path, &target_dir.to_string_lossy())?;

    // Store as relative path
    let new_stored_path = crate::paths::to_storage_relative(&new_stored_path_abs, base);

    // Update DB
    let entry = db::files::move_file(&conn, &id, folder_id.as_deref())?;

    // Update stored path in DB
    db::files::update_stored_path(&conn, &id, &new_stored_path)?;

    // Return the entry with updated stored_path
    Ok(db::files::FileEntry {
        stored_path: new_stored_path,
        ..entry
    })
}

#[tauri::command]
pub fn rename_file(
    state: State<'_, AppState>,
    id: String,
    new_name: String,
) -> Result<db::files::FileEntry, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
    let base = storage.as_deref().ok_or("No active storage path")?;

    // Get current stored path
    let old_stored_path_raw: String = conn
        .query_row(
            "SELECT stored_path FROM files WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    // Rename in DB
    let entry = db::files::rename_file(&conn, &id, &new_name)?;

    // Validate new_name: reject path separators and parent traversal
    if new_name.contains('/')
        || new_name.contains('\\')
        || new_name.contains("..")
        || new_name.is_empty()
    {
        return Err("Invalid filename: must not contain path separators or be empty".to_string());
    }

    // Resolve for disk operations
    let old_resolved = crate::paths::resolve_path(&old_stored_path_raw, base);
    if old_resolved.exists() {
        let new_path = old_resolved.with_file_name(&new_name);
        std::fs::rename(&old_resolved, &new_path)
            .map_err(|e| format!("Failed to rename file on disk: {}", e))?;
        let new_rel = crate::paths::to_storage_relative(&new_path.to_string_lossy(), base);
        db::files::update_stored_path(&conn, &id, &new_rel)?;
        return Ok(db::files::FileEntry {
            stored_path: new_rel,
            ..entry
        });
    }

    Ok(entry)
}

#[tauri::command]
pub fn preview_file(
    state: State<'_, AppState>,
    id: String,
) -> Result<file_preview::FilePreview, String> {
    let resolved = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let raw = db::files::open_file(&conn, &id)?;
        let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
        match storage.as_deref() {
            Some(base) => crate::paths::resolve_path_string(&raw, base),
            None => raw,
        }
    };
    file_preview::generate_preview(&id, &resolved)
}

#[tauri::command]
pub fn save_file_content(
    state: State<'_, AppState>,
    id: String,
    content: String,
) -> Result<(), String> {
    let resolved = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let raw = db::files::open_file(&conn, &id)?;
        let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
        match storage.as_deref() {
            Some(base) => crate::paths::resolve_path_string(&raw, base),
            None => raw,
        }
    };

    std::fs::write(&resolved, content.as_bytes())
        .map_err(|e| format!("Failed to write file: {}", e))?;

    // Update file size in DB (re-acquire lock)
    let size = std::fs::metadata(&resolved)
        .map_err(|e| format!("Failed to read metadata: {}", e))?
        .len() as i64;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE files SET size = ?1 WHERE id = ?2",
        rusqlite::params![size, id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
