use tauri::State;

use crate::db;
use crate::AppState;

#[tauri::command]
pub fn add_link(
    state: State<'_, AppState>,
    feature_id: String,
    title: String,
    url: String,
    link_type: Option<String>,
    description: Option<String>,
) -> Result<db::links::Link, String> {
    let link = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::links::add_link(&conn, &feature_id, &title, &url, link_type, description)?
    };
    // Fire extension event hooks for link_created (fire-and-forget)
    if let (Ok(registry), Ok(sp_guard)) = (state.extensions.lock(), state.storage_path.lock()) {
        if let Some(ref storage_path) = *sp_guard {
            let db_path = storage_path
                .join("feature-hub.db")
                .to_string_lossy()
                .to_string();
            let sp_str = storage_path.to_string_lossy().to_string();
            let payload = serde_json::json!({
                "link_type": link.link_type,
                "feature_id": feature_id,
                "link_id": link.id,
                "url": link.url,
                "title": link.title,
            });
            crate::extensions::dispatch_event(
                &registry,
                "link_created",
                payload,
                db_path,
                sp_str,
                Some(feature_id.clone()),
            );
        }
    }
    Ok(link)
}

#[tauri::command]
pub fn update_link(
    state: State<'_, AppState>,
    id: String,
    title: Option<String>,
    url: Option<String>,
    link_type: Option<String>,
    description: Option<Option<String>>,
) -> Result<db::links::Link, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::links::update_link(&conn, &id, title, url, link_type, description)
}

#[tauri::command]
pub fn delete_link_by_url(
    state: State<'_, AppState>,
    feature_id: String,
    url: String,
) -> Result<(), String> {
    let id = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let id: Option<String> = conn
            .query_row(
                "SELECT id FROM links WHERE feature_id = ?1 AND url = ?2",
                rusqlite::params![feature_id, url],
                |row| row.get(0),
            )
            .ok();
        id
    };
    match id {
        Some(id) => delete_link(state, id),
        None => Ok(()),
    }
}

#[tauri::command]
pub fn delete_link(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let link = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let link = db::links::get_link(&conn, &id).ok();
        db::links::delete_link(&conn, &id)?;
        link
    };
    if let Some(link) = link {
        if let (Ok(registry), Ok(sp_guard)) = (state.extensions.lock(), state.storage_path.lock()) {
            if let Some(ref storage_path) = *sp_guard {
                let db_path = storage_path
                    .join("feature-hub.db")
                    .to_string_lossy()
                    .to_string();
                let sp_str = storage_path.to_string_lossy().to_string();
                let payload = serde_json::json!({
                    "link_type": link.link_type,
                    "feature_id": link.feature_id,
                    "link_id": link.id,
                    "url": link.url,
                });
                crate::extensions::dispatch_event(
                    &registry,
                    "link_deleted",
                    payload,
                    db_path,
                    sp_str,
                    Some(link.feature_id.clone()),
                );
            }
        }
    }
    Ok(())
}
