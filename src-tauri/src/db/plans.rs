use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Plan {
    pub id: String,
    pub feature_id: String,
    pub session_id: Option<String>,
    pub title: String,
    pub body: String,
    pub status: String,
    pub feedback: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

const SELECT_COLS: &str =
    "id, feature_id, session_id, title, body, status, feedback, created_at, resolved_at";

fn row_to_plan(row: &rusqlite::Row) -> rusqlite::Result<Plan> {
    Ok(Plan {
        id: row.get(0)?,
        feature_id: row.get(1)?,
        session_id: row.get(2)?,
        title: row.get(3)?,
        body: row.get(4)?,
        status: row.get(5)?,
        feedback: row.get(6)?,
        created_at: row.get(7)?,
        resolved_at: row.get(8)?,
    })
}

pub fn get_plans(conn: &Connection, feature_id: &str) -> Result<Vec<Plan>, String> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {} FROM plans WHERE feature_id = ?1 ORDER BY created_at DESC",
            SELECT_COLS
        ))
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![feature_id], |row| row_to_plan(row))
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn get_plan(conn: &Connection, id: &str) -> Result<Plan, String> {
    conn.query_row(
        &format!("SELECT {} FROM plans WHERE id = ?1", SELECT_COLS),
        params![id],
        |row| row_to_plan(row),
    )
    .map_err(|e| e.to_string())
}

pub fn create_plan(
    conn: &Connection,
    feature_id: &str,
    session_id: Option<&str>,
    title: &str,
    body: &str,
) -> Result<Plan, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO plans (id, feature_id, session_id, title, body, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'pending', ?6)",
        params![id, feature_id, session_id, title, body, now],
    )
    .map_err(|e| e.to_string())?;

    // Touch feature updated_at
    conn.execute(
        "UPDATE features SET updated_at = ?1 WHERE id = ?2",
        params![now, feature_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(Plan {
        id,
        feature_id: feature_id.to_string(),
        session_id: session_id.map(|s| s.to_string()),
        title: title.to_string(),
        body: body.to_string(),
        status: "pending".to_string(),
        feedback: None,
        created_at: now,
        resolved_at: None,
    })
}

pub fn resolve_plan(
    conn: &Connection,
    id: &str,
    status: &str,
    feedback: Option<&str>,
) -> Result<Plan, String> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE plans SET status = ?1, feedback = ?2, resolved_at = ?3 WHERE id = ?4",
        params![status, feedback, now, id],
    )
    .map_err(|e| e.to_string())?;

    // Touch parent feature
    conn.execute(
        "UPDATE features SET updated_at = ?1 WHERE id = (SELECT feature_id FROM plans WHERE id = ?2)",
        params![now, id],
    )
    .map_err(|e| e.to_string())?;

    get_plan(conn, id)
}

pub fn update_plan(
    conn: &Connection,
    id: &str,
    title: Option<&str>,
    body: Option<&str>,
) -> Result<Plan, String> {
    let now = Utc::now().to_rfc3339();

    if let Some(t) = title {
        conn.execute(
            "UPDATE plans SET title = ?1 WHERE id = ?2",
            params![t, id],
        )
        .map_err(|e| e.to_string())?;
    }

    if let Some(b) = body {
        conn.execute(
            "UPDATE plans SET body = ?1 WHERE id = ?2",
            params![b, id],
        )
        .map_err(|e| e.to_string())?;
    }

    // Reset status to pending if it was rejected (revised plan)
    conn.execute(
        "UPDATE plans SET status = 'pending', feedback = NULL, resolved_at = NULL WHERE id = ?1 AND status = 'rejected'",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    // Touch parent feature
    conn.execute(
        "UPDATE features SET updated_at = ?1 WHERE id = (SELECT feature_id FROM plans WHERE id = ?2)",
        params![now, id],
    )
    .map_err(|e| e.to_string())?;

    get_plan(conn, id)
}

pub fn delete_plan(conn: &Connection, id: &str) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();

    // Touch parent feature
    conn.execute(
        "UPDATE features SET updated_at = ?1 WHERE id = (SELECT feature_id FROM plans WHERE id = ?2)",
        params![now, id],
    )
    .map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM plans WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}
