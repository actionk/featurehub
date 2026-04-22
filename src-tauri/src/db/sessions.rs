use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub id: String,
    pub feature_id: String,
    pub claude_session_id: String,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub project_path: Option<String>,
    pub branch: Option<String>,
    pub linked_at: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    #[serde(default)]
    pub title_manual: bool,
}

pub fn get_sessions(conn: &Connection, feature_id: &str) -> Result<Vec<Session>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, feature_id, claude_session_id, title, summary, project_path, branch, linked_at, started_at, ended_at, title_manual
             FROM sessions WHERE feature_id = ?1 ORDER BY COALESCE(started_at, linked_at) DESC",
        )
        .map_err(|e| e.to_string())?;

    let sessions = stmt
        .query_map(params![feature_id], |row| {
            Ok(Session {
                id: row.get(0)?,
                feature_id: row.get(1)?,
                claude_session_id: row.get(2)?,
                title: row.get(3)?,
                summary: row.get(4)?,
                project_path: row.get(5)?,
                branch: row.get(6)?,
                linked_at: row.get(7)?,
                started_at: row.get(8)?,
                ended_at: row.get(9)?,
                title_manual: row.get::<_, i32>(10).unwrap_or(0) != 0,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(sessions)
}

pub fn get_session(conn: &Connection, id: &str) -> Result<Session, String> {
    conn.query_row(
        "SELECT id, feature_id, claude_session_id, title, summary, project_path, branch, linked_at, started_at, ended_at, title_manual
         FROM sessions WHERE id = ?1",
        params![id],
        |row| {
            Ok(Session {
                id: row.get(0)?,
                feature_id: row.get(1)?,
                claude_session_id: row.get(2)?,
                title: row.get(3)?,
                summary: row.get(4)?,
                project_path: row.get(5)?,
                branch: row.get(6)?,
                linked_at: row.get(7)?,
                started_at: row.get(8)?,
                ended_at: row.get(9)?,
                title_manual: row.get::<_, i32>(10).unwrap_or(0) != 0,
            })
        },
    )
    .map_err(|e| format!("Session not found: {}", e))
}

pub fn link_session(
    conn: &Connection,
    feature_id: &str,
    claude_session_id: &str,
    title: Option<String>,
    summary: Option<String>,
    project_path: Option<String>,
    branch: Option<String>,
) -> Result<Session, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO sessions (id, feature_id, claude_session_id, title, summary, project_path, branch, linked_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, feature_id, claude_session_id, title, summary, project_path, branch, now],
    )
    .map_err(|e| e.to_string())?;

    // Touch feature updated_at
    conn.execute(
        "UPDATE features SET updated_at = ?1 WHERE id = ?2",
        params![now, feature_id],
    )
    .map_err(|e| e.to_string())?;

    // Update search index
    if let Some(ref t) = title {
        super::search::index_session(conn, &id, feature_id, t, summary.as_deref()).ok();
    }

    Ok(Session {
        id,
        feature_id: feature_id.to_string(),
        claude_session_id: claude_session_id.to_string(),
        title,
        summary,
        project_path,
        branch,
        linked_at: now,
        started_at: None,
        ended_at: None,
        title_manual: false,
    })
}

/// Create a session record for a CLI-launched Claude session (with timing).
/// The `claude_session_id` is generated upfront and passed to Claude via `--session-id`.
pub fn create_cli_session(
    conn: &Connection,
    feature_id: &str,
    project_path: Option<String>,
    claude_session_id: &str,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO sessions (id, feature_id, claude_session_id, title, summary, project_path, branch, linked_at, started_at)
         VALUES (?1, ?2, ?3, NULL, NULL, ?4, NULL, ?5, ?6)",
        params![id, feature_id, claude_session_id, project_path, now, now],
    )
    .map_err(|e| e.to_string())?;

    // Touch feature updated_at
    conn.execute(
        "UPDATE features SET updated_at = ?1 WHERE id = ?2",
        params![now, feature_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(id)
}

/// Finalize a CLI session with the discovered claude_session_id and end time.
pub fn finish_session(
    conn: &Connection,
    id: &str,
    claude_session_id: Option<&str>,
    title: Option<&str>,
    summary: Option<&str>,
) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();

    // Only update claude_session_id if provided; keep existing value otherwise
    if let Some(session_id) = claude_session_id {
        conn.execute(
            "UPDATE sessions SET claude_session_id = ?1, ended_at = ?2, title = ?3, summary = ?4 WHERE id = ?5",
            params![session_id, now, title, summary, id],
        )
    } else {
        conn.execute(
            "UPDATE sessions SET ended_at = ?1, title = ?2, summary = ?3 WHERE id = ?4",
            params![now, title, summary, id],
        )
    }
    .map_err(|e| e.to_string())?;

    // Update search index if we have a title
    if let Some(t) = title {
        let feature_id: String = conn
            .query_row(
                "SELECT feature_id FROM sessions WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        super::search::index_session(conn, id, &feature_id, t, summary).ok();
    }

    Ok(())
}

/// Update title and summary without changing other fields (used for auto-fill from disk).
/// Only updates sessions that weren't manually renamed by the user.
pub fn update_session_title_summary(
    conn: &Connection,
    id: &str,
    title: Option<&str>,
    summary: Option<&str>,
) -> Result<(), String> {
    conn.execute(
        "UPDATE sessions SET title = ?1, summary = ?2 WHERE id = ?3 AND title_manual = 0",
        params![title, summary, id],
    )
    .map_err(|e| e.to_string())?;

    // Update search index if we have a title
    if let Some(t) = title {
        let feature_id: String = conn
            .query_row(
                "SELECT feature_id FROM sessions WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        super::search::index_session(conn, id, &feature_id, t, summary).ok();
    }

    Ok(())
}

pub fn rename_session(conn: &Connection, id: &str, title: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE sessions SET title = ?1, title_manual = 1 WHERE id = ?2",
        params![title, id],
    )
    .map_err(|e| e.to_string())?;

    // Update search index
    let feature_id: String = conn
        .query_row(
            "SELECT feature_id FROM sessions WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let summary: Option<String> = conn
        .query_row(
            "SELECT summary FROM sessions WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    super::search::index_session(conn, id, &feature_id, title, summary.as_deref()).ok();

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecentSession {
    pub id: String,
    pub feature_id: String,
    pub feature_title: String,
    pub claude_session_id: String,
    pub title: Option<String>,
    pub project_path: Option<String>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
}

pub fn get_recent_sessions(conn: &Connection, limit: usize) -> Result<Vec<RecentSession>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.feature_id, f.title, s.claude_session_id, s.title, s.project_path, s.started_at, s.ended_at
             FROM sessions s
             JOIN features f ON f.id = s.feature_id
             WHERE s.claude_session_id != ''
             ORDER BY COALESCE(s.ended_at, s.started_at, s.linked_at) DESC
             LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;

    let sessions = stmt
        .query_map(params![limit], |row| {
            Ok(RecentSession {
                id: row.get(0)?,
                feature_id: row.get(1)?,
                feature_title: row.get(2)?,
                claude_session_id: row.get(3)?,
                title: row.get(4)?,
                project_path: row.get(5)?,
                started_at: row.get(6)?,
                ended_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(sessions)
}

/// Get all (feature_id, claude_session_id) pairs for batch active checking.
pub fn get_all_session_ids(conn: &Connection) -> Result<Vec<(String, String)>, String> {
    let mut stmt = conn
        .prepare("SELECT feature_id, claude_session_id FROM sessions WHERE claude_session_id != ''")
        .map_err(|e| e.to_string())?;

    let pairs = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(pairs)
}

pub fn move_session(conn: &Connection, id: &str, target_feature_id: &str) -> Result<Session, String> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE sessions SET feature_id = ?1 WHERE id = ?2",
        params![target_feature_id, id],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE features SET updated_at = ?1 WHERE id = ?2",
        params![now, target_feature_id],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE search_index SET feature_id = ?1 WHERE entity_type = 'session' AND entity_id = ?2",
        params![target_feature_id, id],
    )
    .map_err(|e| e.to_string())?;

    get_session(conn, id)
}

pub fn unlink_session(conn: &Connection, id: &str) -> Result<(), String> {
    // Remove from search index
    conn.execute(
        "DELETE FROM search_index WHERE entity_type = 'session' AND entity_id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM sessions WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(Debug, serde::Serialize)]
pub struct PanelSessionRow {
    pub id: String,
    pub feature_id: String,
    pub feature_name: String,
    pub claude_session_id: String,
    pub branch: Option<String>,
    pub linked_at: String,
    pub title: Option<String>,
}

/// Load all sessions with their feature name for the sessions panel.
/// Excludes sessions with empty `claude_session_id`.
/// Ordered by `linked_at` descending (newest first).
pub fn get_all_sessions_for_panel(conn: &Connection) -> Result<Vec<PanelSessionRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.feature_id, f.title, s.claude_session_id, s.branch, s.linked_at, s.title
             FROM sessions s
             JOIN features f ON f.id = s.feature_id
             WHERE s.claude_session_id != ''
               AND (f.archived = 0 OR f.archived IS NULL)
             ORDER BY s.linked_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(PanelSessionRow {
                id: row.get(0)?,
                feature_id: row.get(1)?,
                feature_name: row.get(2)?,
                claude_session_id: row.get(3)?,
                branch: row.get(4)?,
                linked_at: row.get(5)?,
                title: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_utils::test_db;

    #[test]
    fn test_get_all_sessions_for_panel_joins_feature_names() {
        let conn = test_db();

        conn.execute(
            "INSERT INTO features (id, title, status, sort_order, created_at, updated_at) VALUES ('f1', 'My Feature', 'active', 0, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            [],
        ).unwrap();

        conn.execute(
            "INSERT INTO sessions (id, feature_id, claude_session_id, branch, linked_at)
             VALUES ('s1', 'f1', 'claude-abc', 'main', '2026-01-01T00:00:00Z')",
            [],
        ).unwrap();

        let rows = get_all_sessions_for_panel(&conn).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "s1");
        assert_eq!(rows[0].feature_name, "My Feature");
        assert_eq!(rows[0].claude_session_id, "claude-abc");
        assert_eq!(rows[0].branch.as_deref(), Some("main"));
    }

    #[test]
    fn test_get_all_sessions_for_panel_excludes_empty_claude_session_id() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO features (id, title, status, sort_order, created_at, updated_at) VALUES ('f1', 'F', 'active', 0, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO sessions (id, feature_id, claude_session_id, linked_at)
             VALUES ('s1', 'f1', '', '2026-01-01T00:00:00Z')",
            [],
        ).unwrap();

        let rows = get_all_sessions_for_panel(&conn).unwrap();
        assert!(rows.is_empty());
    }

    #[test]
    fn test_move_session() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO features (id, title, status, sort_order, created_at, updated_at) VALUES ('f1', 'Feature A', 'active', 0, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO features (id, title, status, sort_order, created_at, updated_at) VALUES ('f2', 'Feature B', 'active', 1, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            [],
        ).unwrap();

        let session = link_session(&conn, "f1", "claude-123", Some("Test session".into()), None, None, None).unwrap();
        assert_eq!(session.feature_id, "f1");

        let moved = move_session(&conn, &session.id, "f2").unwrap();
        assert_eq!(moved.feature_id, "f2");
        assert_eq!(moved.claude_session_id, "claude-123");

        let f1_sessions = get_sessions(&conn, "f1").unwrap();
        assert!(f1_sessions.is_empty());

        let f2_sessions = get_sessions(&conn, "f2").unwrap();
        assert_eq!(f2_sessions.len(), 1);
    }
}
