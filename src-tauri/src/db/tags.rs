use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String,
}

pub fn get_tags(conn: &Connection) -> Result<Vec<Tag>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, color FROM tags ORDER BY name ASC")
        .map_err(|e| e.to_string())?;

    let tags = stmt
        .query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(tags)
}

pub fn create_tag(conn: &Connection, name: &str, color: &str) -> Result<Tag, String> {
    let id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO tags (id, name, color) VALUES (?1, ?2, ?3)",
        params![id, name, color],
    )
    .map_err(|e| e.to_string())?;

    Ok(Tag {
        id,
        name: name.to_string(),
        color: color.to_string(),
    })
}

pub fn delete_tag(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM tags WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn toggle_tag(conn: &Connection, feature_id: &str, tag_id: &str) -> Result<(), String> {
    // Check if the association exists
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM feature_tags WHERE feature_id = ?1 AND tag_id = ?2",
            params![feature_id, tag_id],
            |row| {
                let count: i64 = row.get(0)?;
                Ok(count > 0)
            },
        )
        .map_err(|e| e.to_string())?;

    if exists {
        conn.execute(
            "DELETE FROM feature_tags WHERE feature_id = ?1 AND tag_id = ?2",
            params![feature_id, tag_id],
        )
        .map_err(|e| e.to_string())?;
    } else {
        conn.execute(
            "INSERT INTO feature_tags (feature_id, tag_id) VALUES (?1, ?2)",
            params![feature_id, tag_id],
        )
        .map_err(|e| e.to_string())?;
    }

    // Touch feature updated_at
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE features SET updated_at = ?1 WHERE id = ?2",
        params![now, feature_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
