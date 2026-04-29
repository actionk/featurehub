use tauri::State;

use crate::db;
use crate::AppState;

#[tauri::command]
pub fn get_tasks(
    state: State<'_, AppState>,
    feature_id: String,
) -> Result<Vec<db::tasks::Task>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::tasks::get_tasks(&conn, &feature_id)
}

#[tauri::command]
pub fn create_task(
    state: State<'_, AppState>,
    feature_id: String,
    title: String,
    source: Option<String>,
    external_key: Option<String>,
    external_url: Option<String>,
    external_status: Option<String>,
    description: Option<String>,
) -> Result<db::tasks::Task, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::tasks::create_task(
        &conn,
        &feature_id,
        &title,
        source.as_deref(),
        external_key.as_deref(),
        external_url.as_deref(),
        external_status.as_deref(),
        description.as_deref(),
    )
}

#[tauri::command]
pub fn update_task(
    state: State<'_, AppState>,
    id: String,
    title: Option<String>,
    done: Option<bool>,
    external_status: Option<String>,
    description: Option<String>,
) -> Result<db::tasks::Task, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::tasks::update_task(
        &conn,
        &id,
        title.as_deref(),
        done,
        external_status.as_deref(),
        description.as_deref(),
    )
}

#[tauri::command]
pub fn delete_task(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::tasks::delete_task(&conn, &id)
}
