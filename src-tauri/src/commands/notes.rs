use tauri::State;

use crate::db;
use crate::AppState;

#[tauri::command]
pub fn get_note(
    state: State<'_, AppState>,
    feature_id: String,
) -> Result<Option<db::notes::Note>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::notes::get_note(&conn, &feature_id)
}

#[tauri::command]
pub fn save_note(
    state: State<'_, AppState>,
    feature_id: String,
    content: String,
) -> Result<db::notes::Note, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::notes::save_note(&conn, &feature_id, &content)
}
