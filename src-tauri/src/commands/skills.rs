use tauri::State;

use crate::db;
use crate::AppState;

#[tauri::command]
pub fn get_feature_skills(
    state: State<'_, AppState>,
    feature_id: String,
) -> Result<Vec<db::skills::FeatureSkill>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::skills::get_feature_skills(&conn, &feature_id)
}

#[tauri::command]
pub fn set_feature_skill(
    state: State<'_, AppState>,
    feature_id: String,
    skill_id: String,
    enabled: bool,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::skills::set_feature_skill(&conn, &feature_id, &skill_id, enabled)
}
