use tauri::State;

use crate::db;
use crate::AppState;

#[tauri::command]
pub fn get_tags(state: State<'_, AppState>) -> Result<Vec<db::tags::Tag>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::tags::get_tags(&conn)
}

#[tauri::command]
pub fn create_tag(
    state: State<'_, AppState>,
    name: String,
    color: String,
) -> Result<db::tags::Tag, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::tags::create_tag(&conn, &name, &color)
}

#[tauri::command]
pub fn delete_tag(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::tags::delete_tag(&conn, &id)
}

#[tauri::command]
pub fn toggle_tag(
    state: State<'_, AppState>,
    feature_id: String,
    tag_id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::tags::toggle_tag(&conn, &feature_id, &tag_id)
}
