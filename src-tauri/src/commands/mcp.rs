use tauri::State;

use crate::db;
use crate::AppState;

#[tauri::command]
pub fn get_feature_mcp_servers(
    state: State<'_, AppState>,
    feature_id: String,
) -> Result<Vec<db::mcp_servers::FeatureMcpServer>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::mcp_servers::get_feature_mcp_servers(&conn, &feature_id)
}

#[tauri::command]
pub fn set_feature_mcp_server(
    state: State<'_, AppState>,
    feature_id: String,
    server_name: String,
    enabled: bool,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::mcp_servers::set_feature_mcp_server(&conn, &feature_id, &server_name, enabled)
}
