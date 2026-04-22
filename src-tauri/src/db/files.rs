use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub id: String,
    pub feature_id: String,
    pub filename: String,
    pub original_path: String,
    pub stored_path: String,
    pub size: i64,
    pub folder_id: Option<String>,
    pub created_at: String,
}

pub fn get_files(conn: &Connection, feature_id: &str) -> Result<Vec<FileEntry>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, feature_id, filename, original_path, stored_path, size, folder_id, created_at
             FROM files WHERE feature_id = ?1 ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let files = stmt
        .query_map(params![feature_id], |row| {
            Ok(FileEntry {
                id: row.get(0)?,
                feature_id: row.get(1)?,
                filename: row.get(2)?,
                original_path: row.get(3)?,
                stored_path: row.get(4)?,
                size: row.get(5)?,
                folder_id: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(files)
}

pub fn add_file(
    conn: &Connection,
    feature_id: &str,
    filename: &str,
    original_path: &str,
    stored_path: &str,
    size: i64,
    folder_id: Option<&str>,
) -> Result<FileEntry, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO files (id, feature_id, filename, original_path, stored_path, size, folder_id, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, feature_id, filename, original_path, stored_path, size, folder_id, now],
    )
    .map_err(|e| e.to_string())?;

    // Touch feature updated_at
    conn.execute(
        "UPDATE features SET updated_at = ?1 WHERE id = ?2",
        params![now, feature_id],
    )
    .map_err(|e| e.to_string())?;

    super::search::index_file(conn, &id, feature_id, filename).ok();

    Ok(FileEntry {
        id,
        feature_id: feature_id.to_string(),
        filename: filename.to_string(),
        original_path: original_path.to_string(),
        stored_path: stored_path.to_string(),
        size,
        folder_id: folder_id.map(|s| s.to_string()),
        created_at: now,
    })
}

pub fn update_file_size(conn: &Connection, id: &str, size: i64) -> Result<(), String> {
    conn.execute(
        "UPDATE files SET size = ?1 WHERE id = ?2",
        params![size, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_file(conn: &Connection, id: &str) -> Result<String, String> {
    // Get the stored path before deleting
    let stored_path: String = conn
        .query_row(
            "SELECT stored_path FROM files WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM files WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    super::search::remove_file_from_index(conn, id).ok();

    Ok(stored_path)
}

pub fn open_file(conn: &Connection, id: &str) -> Result<String, String> {
    let stored_path: String = conn
        .query_row(
            "SELECT stored_path FROM files WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(stored_path)
}

pub fn move_file(conn: &Connection, file_id: &str, folder_id: Option<&str>) -> Result<FileEntry, String> {
    conn.execute(
        "UPDATE files SET folder_id = ?1 WHERE id = ?2",
        params![folder_id, file_id],
    )
    .map_err(|e| e.to_string())?;

    let entry = conn.query_row(
        "SELECT id, feature_id, filename, original_path, stored_path, size, folder_id, created_at
         FROM files WHERE id = ?1",
        params![file_id],
        |row| {
            Ok(FileEntry {
                id: row.get(0)?,
                feature_id: row.get(1)?,
                filename: row.get(2)?,
                original_path: row.get(3)?,
                stored_path: row.get(4)?,
                size: row.get(5)?,
                folder_id: row.get(6)?,
                created_at: row.get(7)?,
            })
        },
    )
    .map_err(|e| e.to_string())?;

    Ok(entry)
}

pub fn rename_file(conn: &Connection, file_id: &str, new_filename: &str) -> Result<FileEntry, String> {
    conn.execute(
        "UPDATE files SET filename = ?1 WHERE id = ?2",
        params![new_filename, file_id],
    )
    .map_err(|e| e.to_string())?;

    let entry = conn.query_row(
        "SELECT id, feature_id, filename, original_path, stored_path, size, folder_id, created_at
         FROM files WHERE id = ?1",
        params![file_id],
        |row| {
            Ok(FileEntry {
                id: row.get(0)?,
                feature_id: row.get(1)?,
                filename: row.get(2)?,
                original_path: row.get(3)?,
                stored_path: row.get(4)?,
                size: row.get(5)?,
                folder_id: row.get(6)?,
                created_at: row.get(7)?,
            })
        },
    )
    .map_err(|e| e.to_string())?;

    super::search::index_file(conn, &entry.id, &entry.feature_id, &entry.filename).ok();

    Ok(entry)
}

pub fn update_stored_path(conn: &Connection, file_id: &str, new_stored_path: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE files SET stored_path = ?1 WHERE id = ?2",
        params![new_stored_path, file_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
