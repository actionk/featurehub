use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Context {
    pub id: String,
    pub feature_id: String,
    pub content: String,
    pub updated_at: String,
}

pub fn get_context(conn: &Connection, feature_id: &str) -> Result<Option<Context>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, feature_id, content, updated_at
             FROM context WHERE feature_id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let result = stmt
        .query_row(params![feature_id], |row| {
            Ok(Context {
                id: row.get(0)?,
                feature_id: row.get(1)?,
                content: row.get(2)?,
                updated_at: row.get(3)?,
            })
        });

    match result {
        Ok(context) => Ok(Some(context)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn save_context(conn: &Connection, feature_id: &str, content: &str) -> Result<Context, String> {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();

    // Atomic upsert — avoids race condition when two sessions save simultaneously
    conn.execute(
        "INSERT INTO context (id, feature_id, content, updated_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(feature_id) DO UPDATE SET content = excluded.content, updated_at = excluded.updated_at",
        params![id, feature_id, content, now],
    )
    .map_err(|e| e.to_string())?;

    // Touch feature updated_at
    conn.execute(
        "UPDATE features SET updated_at = ?1 WHERE id = ?2",
        params![now, feature_id],
    )
    .map_err(|e| e.to_string())?;

    // Return the context
    get_context(conn, feature_id)?.ok_or_else(|| "Failed to retrieve saved context".to_string())
}
