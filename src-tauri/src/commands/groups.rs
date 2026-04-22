use tauri::State;

use crate::db;
use crate::AppState;

#[tauri::command]
pub fn get_feature_groups(state: State<'_, AppState>) -> Result<Vec<db::feature_groups::FeatureGroup>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::feature_groups::get_feature_groups(&conn)
}

#[tauri::command]
pub fn create_feature_group(
    state: State<'_, AppState>,
    name: String,
    color: Option<String>,
) -> Result<db::feature_groups::FeatureGroup, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::feature_groups::create_feature_group(&conn, &name, color)
}

#[tauri::command]
pub fn update_feature_group(
    state: State<'_, AppState>,
    id: String,
    name: Option<String>,
    color: Option<String>,
) -> Result<db::feature_groups::FeatureGroup, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::feature_groups::update_feature_group(&conn, &id, name, color)
}

#[tauri::command]
pub fn delete_feature_group(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::feature_groups::delete_feature_group(&conn, &id)
}

#[tauri::command]
pub fn reorder_feature_groups(state: State<'_, AppState>, ids: Vec<String>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::feature_groups::reorder_feature_groups(&conn, &ids)
}

#[tauri::command]
pub fn set_feature_group(
    state: State<'_, AppState>,
    feature_id: String,
    group_id: Option<String>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::feature_groups::set_feature_group(&conn, &feature_id, group_id)
}
