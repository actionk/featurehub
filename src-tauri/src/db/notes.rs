use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Note {
    pub id: String,
    pub feature_id: String,
    pub content: String,
    pub updated_at: String,
}

pub fn get_note(conn: &Connection, feature_id: &str) -> Result<Option<Note>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, feature_id, content, updated_at
             FROM notes WHERE feature_id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let result = stmt.query_row(params![feature_id], |row| {
        Ok(Note {
            id: row.get(0)?,
            feature_id: row.get(1)?,
            content: row.get(2)?,
            updated_at: row.get(3)?,
        })
    });

    match result {
        Ok(note) => Ok(Some(note)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn save_note(conn: &Connection, feature_id: &str, content: &str) -> Result<Note, String> {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();

    // Atomic upsert — avoids race condition when two sessions save simultaneously
    conn.execute(
        "INSERT INTO notes (id, feature_id, content, updated_at)
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

    // Update search index
    super::search::index_note(conn, feature_id, content).ok();

    // Return the note
    get_note(conn, feature_id)?.ok_or_else(|| "Failed to retrieve saved note".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::features::create_feature;
    use crate::db::test_utils::test_db;

    fn setup() -> (rusqlite::Connection, String) {
        let conn = test_db();
        let feature = create_feature(&conn, "Test Feature", None, None, None, None).unwrap();
        let id = feature.id.clone();
        (conn, id)
    }

    #[test]
    fn get_note_returns_none_when_no_note() {
        let (conn, feature_id) = setup();
        let note = get_note(&conn, &feature_id).unwrap();
        assert!(note.is_none());
    }

    #[test]
    fn save_note_creates_and_returns_note() {
        let (conn, feature_id) = setup();
        let note = save_note(&conn, &feature_id, "Hello world").unwrap();

        assert_eq!(note.feature_id, feature_id);
        assert_eq!(note.content, "Hello world");
    }

    #[test]
    fn save_note_upserts_existing() {
        let (conn, feature_id) = setup();
        save_note(&conn, &feature_id, "Version 1").unwrap();
        let updated = save_note(&conn, &feature_id, "Version 2").unwrap();

        assert_eq!(updated.content, "Version 2");

        let fetched = get_note(&conn, &feature_id).unwrap().unwrap();
        assert_eq!(fetched.content, "Version 2");
    }
}
