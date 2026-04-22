use tauri::State;

use crate::db;
use crate::AppState;

use super::FeatureData;

#[tauri::command]
pub fn get_features(
    state: State<'_, AppState>,
    filter: Option<String>,
    sort: Option<String>,
) -> Result<Vec<db::features::FeatureSummary>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::features::get_features(&conn, filter, sort)
}

#[tauri::command]
pub fn get_feature(state: State<'_, AppState>, id: String) -> Result<db::features::Feature, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut feature = db::features::get_feature(&conn, &id)?;
    let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
    if let Some(ref base) = *storage {
        resolve_feature_dirs(&mut feature, base);
    }
    Ok(feature)
}

/// Combined endpoint: fetches feature + tags + tasks + plans in a single IPC call / single DB lock.
/// Sessions are excluded because they do expensive disk I/O for title scanning.
#[tauri::command]
pub fn get_feature_data(
    state: State<'_, AppState>,
    id: String,
) -> Result<FeatureData, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut feature = db::features::get_feature(&conn, &id)?;
    let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
    if let Some(ref base) = *storage {
        resolve_feature_dirs(&mut feature, base);
    }
    let all_tags = db::tags::get_tags(&conn)?;
    let tasks = db::tasks::get_tasks(&conn, &id)?;
    let plans = db::plans::get_plans(&conn, &id)?;
    let note = db::notes::get_note(&conn, &id)?;
    Ok(FeatureData {
        feature,
        all_tags,
        tasks,
        plans,
        note,
    })
}

/// Resolve relative directory paths to absolute for frontend consumption.
fn resolve_feature_dirs(feature: &mut db::features::Feature, storage_base: &std::path::Path) {
    for dir in &mut feature.directories {
        dir.path = crate::paths::resolve_path_string(&dir.path, storage_base);
    }
}

#[tauri::command]
pub fn create_feature(
    state: State<'_, AppState>,
    title: String,
    ticket_id: Option<String>,
    status: Option<String>,
    description: Option<String>,
    parent_id: Option<String>,
) -> Result<db::features::Feature, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut feature = db::features::create_feature(&conn, &title, ticket_id, status, description, parent_id)?;
    let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
    if let Some(ref base) = *storage {
        resolve_feature_dirs(&mut feature, base);
    }
    Ok(feature)
}

#[tauri::command]
pub fn update_feature(
    state: State<'_, AppState>,
    id: String,
    title: Option<String>,
    ticket_id: Option<String>,
    status: Option<String>,
    sort_order: Option<i64>,
    description: Option<String>,
) -> Result<db::features::Feature, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut feature = db::features::update_feature(&conn, &id, title, ticket_id, status, sort_order, description)?;
    let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
    if let Some(ref base) = *storage {
        resolve_feature_dirs(&mut feature, base);
    }
    Ok(feature)
}

#[tauri::command]
pub fn delete_feature(
    state: State<'_, AppState>,
    id: String,
    cleanup_repos: Option<bool>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let storage = state.storage_path.lock().map_err(|e| e.to_string())?;

    // If cleanup_repos is true, delete the feature's workspace directory (cloned repos)
    if cleanup_repos.unwrap_or(false) {
        if let Some(ref base) = *storage {
            let workspace_dir = base.join("workspaces").join(&id);
            if workspace_dir.exists() {
                let _ = std::fs::remove_dir_all(&workspace_dir);
            }
        }
    }

    db::features::delete_feature(&conn, &id, storage.as_deref())
}

#[tauri::command]
pub fn reorder_features(state: State<'_, AppState>, ids: Vec<String>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::features::reorder_features(&conn, &ids)
}

#[tauri::command]
pub fn duplicate_feature(state: State<'_, AppState>, id: String) -> Result<db::features::Feature, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
    let base = storage.as_deref();
    let mut feature = db::features::duplicate_feature(&conn, &id, base)?;
    if let Some(base) = base {
        resolve_feature_dirs(&mut feature, base);
    }
    Ok(feature)
}

#[tauri::command]
pub fn toggle_pin_feature(state: State<'_, AppState>, id: String) -> Result<db::features::Feature, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut feature = db::features::toggle_pin_feature(&conn, &id)?;
    let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
    if let Some(ref base) = *storage {
        resolve_feature_dirs(&mut feature, base);
    }
    Ok(feature)
}

#[tauri::command]
pub fn set_feature_archived(state: State<'_, AppState>, id: String, archived: bool) -> Result<db::features::Feature, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut feature = db::features::set_archived(&conn, &id, archived)?;
    let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
    if let Some(ref base) = *storage {
        resolve_feature_dirs(&mut feature, base);
    }
    Ok(feature)
}

#[tauri::command]
pub fn set_feature_parent(
    state: State<'_, AppState>,
    id: String,
    parent_id: Option<String>,
) -> Result<db::features::Feature, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut feature = db::features::set_feature_parent(&conn, &id, parent_id)?;
    let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
    if let Some(ref base) = *storage {
        resolve_feature_dirs(&mut feature, base);
    }
    Ok(feature)
}
