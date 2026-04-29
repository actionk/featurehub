use tauri::State;

use crate::db;
use crate::AppState;

#[tauri::command]
pub fn get_plans(
    state: State<'_, AppState>,
    feature_id: String,
) -> Result<Vec<db::plans::Plan>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::plans::get_plans(&conn, &feature_id)
}

#[tauri::command]
pub fn get_plan(state: State<'_, AppState>, id: String) -> Result<db::plans::Plan, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::plans::get_plan(&conn, &id)
}

#[tauri::command]
pub fn resolve_plan(
    state: State<'_, AppState>,
    id: String,
    status: String,
    feedback: Option<String>,
) -> Result<db::plans::Plan, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::plans::resolve_plan(&conn, &id, &status, feedback.as_deref())
}

#[tauri::command]
pub fn delete_plan(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::plans::delete_plan(&conn, &id)
}
