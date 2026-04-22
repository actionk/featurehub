use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Directory {
    pub id: String,
    pub feature_id: String,
    pub path: String,
    pub label: Option<String>,
    pub created_at: String,
    pub repo_url: Option<String>,
    pub clone_status: Option<String>,
    pub clone_error: Option<String>,
}

pub fn get_directories(conn: &Connection, feature_id: &str) -> Result<Vec<Directory>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, feature_id, path, label, created_at, repo_url, clone_status, clone_error
             FROM directories WHERE feature_id = ?1 ORDER BY created_at ASC",
        )
        .map_err(|e| e.to_string())?;

    let dirs = stmt
        .query_map(params![feature_id], |row| {
            Ok(Directory {
                id: row.get(0)?,
                feature_id: row.get(1)?,
                path: row.get(2)?,
                label: row.get(3)?,
                created_at: row.get(4)?,
                repo_url: row.get(5)?,
                clone_status: row.get(6)?,
                clone_error: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(dirs)
}

pub fn add_directory(
    conn: &Connection,
    feature_id: &str,
    path: &str,
    label: Option<String>,
    repo_url: Option<String>,
    clone_status: Option<String>,
) -> Result<Directory, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let status = clone_status.unwrap_or_else(|| "ready".to_string());

    conn.execute(
        "INSERT INTO directories (id, feature_id, path, label, created_at, repo_url, clone_status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, feature_id, path, label, now, repo_url, status],
    )
    .map_err(|e| e.to_string())?;

    // Touch feature updated_at
    conn.execute(
        "UPDATE features SET updated_at = ?1 WHERE id = ?2",
        params![now, feature_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(Directory {
        id,
        feature_id: feature_id.to_string(),
        path: path.to_string(),
        label,
        created_at: now,
        repo_url,
        clone_status: Some(status),
        clone_error: None,
    })
}

pub fn get_directory(conn: &Connection, id: &str) -> Result<Directory, String> {
    conn.query_row(
        "SELECT id, feature_id, path, label, created_at, repo_url, clone_status, clone_error
         FROM directories WHERE id = ?1",
        params![id],
        |row| {
            Ok(Directory {
                id: row.get(0)?,
                feature_id: row.get(1)?,
                path: row.get(2)?,
                label: row.get(3)?,
                created_at: row.get(4)?,
                repo_url: row.get(5)?,
                clone_status: row.get(6)?,
                clone_error: row.get(7)?,
            })
        },
    )
    .map_err(|e| format!("Directory not found: {}", e))
}

pub fn update_clone_status(
    conn: &Connection,
    id: &str,
    status: &str,
    error: Option<&str>,
) -> Result<(), String> {
    conn.execute(
        "UPDATE directories SET clone_status = ?1, clone_error = ?2 WHERE id = ?3",
        params![status, error, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn remove_directory(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM directories WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}
