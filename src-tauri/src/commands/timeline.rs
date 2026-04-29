use tauri::State;

use crate::AppState;

#[derive(serde::Serialize)]
pub struct TimelineEvent {
    pub event_type: String,
    pub title: String,
    pub detail: Option<String>,
    pub timestamp: String,
}

#[derive(serde::Serialize)]
pub struct GlobalTimelineEvent {
    pub event_type: String,
    pub title: String,
    pub detail: Option<String>,
    pub timestamp: String,
    pub feature_id: String,
    pub feature_title: String,
}

#[tauri::command]
pub fn get_timeline(
    state: State<'_, AppState>,
    feature_id: String,
) -> Result<Vec<TimelineEvent>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let sql = "
        SELECT 'feature_created' as event_type, title, NULL as detail, created_at as ts FROM features WHERE id = ?1
        UNION ALL
        SELECT 'link_added', title, link_type, created_at FROM links WHERE feature_id = ?1
        UNION ALL
        SELECT 'session_linked', COALESCE(title, 'Untitled session'), branch, COALESCE(started_at, linked_at) FROM sessions WHERE feature_id = ?1
        UNION ALL
        SELECT CASE WHEN done = 1 THEN 'task_completed' ELSE 'task_added' END, title, source, created_at FROM tasks WHERE feature_id = ?1
        UNION ALL
        SELECT 'note_updated', 'Notes updated', NULL, updated_at FROM notes WHERE feature_id = ?1
        UNION ALL
        SELECT 'file_added', filename, NULL, created_at FROM files WHERE feature_id = ?1
        UNION ALL
        SELECT 'directory_linked', COALESCE(label, path), path, created_at FROM directories WHERE feature_id = ?1
        ORDER BY ts DESC
    ";

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let events = stmt
        .query_map(rusqlite::params![feature_id], |row| {
            Ok(TimelineEvent {
                event_type: row.get(0)?,
                title: row.get(1)?,
                detail: row.get(2)?,
                timestamp: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(events)
}

#[tauri::command]
pub fn get_global_timeline(state: State<'_, AppState>) -> Result<Vec<GlobalTimelineEvent>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let sql = "
        SELECT 'feature_created', f.title, NULL, f.created_at, f.id, f.title
        FROM features f WHERE f.archived = 0
        UNION ALL
        SELECT 'link_added', l.title, l.link_type, l.created_at, l.feature_id, f.title
        FROM links l JOIN features f ON f.id = l.feature_id WHERE f.archived = 0
        UNION ALL
        SELECT 'session_linked', COALESCE(s.title, 'Untitled session'), s.branch,
               COALESCE(s.started_at, s.linked_at), s.feature_id, f.title
        FROM sessions s JOIN features f ON f.id = s.feature_id WHERE f.archived = 0
        UNION ALL
        SELECT CASE WHEN t.done = 1 THEN 'task_completed' ELSE 'task_added' END,
               t.title, t.source, t.created_at, t.feature_id, f.title
        FROM tasks t JOIN features f ON f.id = t.feature_id WHERE f.archived = 0
        UNION ALL
        SELECT 'note_updated', 'Notes updated', NULL, n.updated_at, n.feature_id, f.title
        FROM notes n JOIN features f ON f.id = n.feature_id WHERE f.archived = 0
        UNION ALL
        SELECT 'file_added', fi.filename, NULL, fi.created_at, fi.feature_id, f.title
        FROM files fi JOIN features f ON f.id = fi.feature_id WHERE f.archived = 0
        ORDER BY 4 DESC
        LIMIT 300
    ";

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let events = stmt
        .query_map([], |row| {
            Ok(GlobalTimelineEvent {
                event_type: row.get(0)?,
                title: row.get(1)?,
                detail: row.get(2)?,
                timestamp: row.get(3)?,
                feature_id: row.get(4)?,
                feature_title: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(events)
}
