use tauri::State;

use crate::db;
use crate::AppState;

#[tauri::command]
pub fn global_search(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<db::search::SearchResult>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::search::global_search(&conn, &query)
}

#[tauri::command]
pub fn rebuild_search_index(state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::search::rebuild_search_index(&conn)
}
