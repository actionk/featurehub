use tauri::State;
use crate::db;
use crate::AppState;

// ─── Folder commands ────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_knowledge_folders(
    state: State<'_, AppState>,
) -> Result<Vec<db::knowledge::KnowledgeFolder>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::knowledge::list_folders(&conn)
}

#[tauri::command]
pub fn create_knowledge_folder(
    state: State<'_, AppState>,
    name: String,
    parent_id: Option<String>,
) -> Result<db::knowledge::KnowledgeFolder, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::knowledge::create_folder(&conn, &name, parent_id.as_deref())
}

#[tauri::command]
pub fn rename_knowledge_folder(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> Result<db::knowledge::KnowledgeFolder, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::knowledge::rename_folder(&conn, &id, &name)
}

#[tauri::command]
pub fn delete_knowledge_folder(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::knowledge::delete_folder(&conn, &id)
}

// ─── Entry commands ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_knowledge_entries(
    state: State<'_, AppState>,
    folder_id: Option<String>,
) -> Result<Vec<db::knowledge::KnowledgeEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::knowledge::list_entries_in_folder(&conn, folder_id.as_deref())
}

#[tauri::command]
pub fn get_all_knowledge_entries(
    state: State<'_, AppState>,
) -> Result<Vec<db::knowledge::KnowledgeEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::knowledge::list_entries(&conn)
}

#[tauri::command]
pub fn get_knowledge_entry(
    state: State<'_, AppState>,
    id: String,
) -> Result<db::knowledge::KnowledgeEntry, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::knowledge::get_entry(&conn, &id)
}

#[tauri::command]
pub fn create_knowledge_entry(
    state: State<'_, AppState>,
    title: String,
    content: String,
    description: Option<String>,
    folder_id: Option<String>,
    feature_id: Option<String>,
) -> Result<db::knowledge::KnowledgeEntry, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::knowledge::create_entry(
        &conn,
        &title,
        &content,
        description.as_deref(),
        folder_id.as_deref(),
        feature_id.as_deref(),
    )
}

#[tauri::command]
pub fn update_knowledge_entry(
    state: State<'_, AppState>,
    id: String,
    title: Option<String>,
    content: Option<String>,
    description: Option<String>,
    folder_id: Option<Option<String>>,
    feature_id: Option<Option<String>>,
) -> Result<db::knowledge::KnowledgeEntry, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::knowledge::update_entry(
        &conn,
        &id,
        title.as_deref(),
        content.as_deref(),
        description.as_deref(),
        folder_id.as_ref().map(|o| o.as_deref()),
        feature_id.as_ref().map(|o| o.as_deref()),
    )
}

#[tauri::command]
pub fn delete_knowledge_entry(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::knowledge::delete_entry(&conn, &id)
}
