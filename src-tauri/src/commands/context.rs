use tauri::State;

use crate::db;
use crate::AppState;

#[tauri::command]
pub fn get_context(
    state: State<'_, AppState>,
    feature_id: String,
) -> Result<Option<db::context::Context>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::context::get_context(&conn, &feature_id)
}

#[tauri::command]
pub fn save_context(
    state: State<'_, AppState>,
    feature_id: String,
    content: String,
) -> Result<db::context::Context, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::context::save_context(&conn, &feature_id, &content)
}
