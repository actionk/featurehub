use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Structs ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnowledgeFolder {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub sort_order: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnowledgeEntry {
    pub id: String,
    pub folder_id: Option<String>,
    pub feature_id: Option<String>,
    pub title: String,
    pub description: String,
    pub content: String,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

// ─── Folder CRUD ────────────────────────────────────────────────────────────

pub fn list_folders(conn: &Connection) -> Result<Vec<KnowledgeFolder>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, parent_id, name, sort_order, created_at
             FROM knowledge_folders ORDER BY sort_order ASC, name ASC",
        )
        .map_err(|e| e.to_string())?;

    let folders = stmt
        .query_map([], |row| {
            Ok(KnowledgeFolder {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                name: row.get(2)?,
                sort_order: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(folders)
}

fn get_folder(conn: &Connection, id: &str) -> Result<KnowledgeFolder, String> {
    conn.query_row(
        "SELECT id, parent_id, name, sort_order, created_at
         FROM knowledge_folders WHERE id = ?1",
        params![id],
        |row| {
            Ok(KnowledgeFolder {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                name: row.get(2)?,
                sort_order: row.get(3)?,
                created_at: row.get(4)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

fn get_folder_depth(conn: &Connection, folder_id: &str) -> Result<usize, String> {
    let mut depth = 1;
    let mut current_id = Some(folder_id.to_string());
    while let Some(id) = current_id {
        let folder = get_folder(conn, &id)?;
        current_id = folder.parent_id;
        if current_id.is_some() {
            depth += 1;
        }
    }
    Ok(depth)
}

pub fn get_folder_path(conn: &Connection, folder_id: &str) -> Result<String, String> {
    let mut parts = Vec::new();
    let mut current_id = Some(folder_id.to_string());
    while let Some(id) = current_id {
        let folder = get_folder(conn, &id)?;
        parts.push(folder.name.clone());
        current_id = folder.parent_id;
    }
    parts.reverse();
    Ok(parts.join("/"))
}

pub fn create_folder(
    conn: &Connection,
    name: &str,
    parent_id: Option<&str>,
) -> Result<KnowledgeFolder, String> {
    if name.is_empty() {
        return Err("Folder name must not be empty".to_string());
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err("Folder name must not contain path separators or '..'".to_string());
    }

    if let Some(pid) = parent_id {
        let depth = get_folder_depth(conn, pid)?;
        if depth >= 5 {
            return Err("Maximum folder nesting depth of 5 exceeded".to_string());
        }
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO knowledge_folders (id, parent_id, name, sort_order, created_at)
         VALUES (?1, ?2, ?3, 0, ?4)",
        params![id, parent_id, name, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(KnowledgeFolder {
        id,
        parent_id: parent_id.map(|s| s.to_string()),
        name: name.to_string(),
        sort_order: 0,
        created_at: now,
    })
}

pub fn rename_folder(
    conn: &Connection,
    id: &str,
    new_name: &str,
) -> Result<KnowledgeFolder, String> {
    if new_name.is_empty() {
        return Err("Folder name must not be empty".to_string());
    }
    if new_name.contains('/') || new_name.contains('\\') || new_name.contains("..") {
        return Err("Folder name must not contain path separators or '..'".to_string());
    }

    conn.execute(
        "UPDATE knowledge_folders SET name = ?1 WHERE id = ?2",
        params![new_name, id],
    )
    .map_err(|e| e.to_string())?;

    get_folder(conn, id)
}

pub fn delete_folder(conn: &Connection, id: &str) -> Result<(), String> {
    // Move child entries to parent folder (or root) before deleting
    let folder = get_folder(conn, id)?;
    conn.execute(
        "UPDATE knowledge_entries SET folder_id = ?1 WHERE folder_id = ?2",
        params![folder.parent_id, id],
    )
    .map_err(|e| e.to_string())?;

    // CASCADE will handle child folders; their entries also get reparented
    // by the ON DELETE SET NULL FK on knowledge_entries.folder_id
    conn.execute("DELETE FROM knowledge_folders WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ─── Entry CRUD ─────────────────────────────────────────────────────────────

fn read_entry(row: &rusqlite::Row) -> rusqlite::Result<KnowledgeEntry> {
    Ok(KnowledgeEntry {
        id: row.get(0)?,
        folder_id: row.get(1)?,
        feature_id: row.get(2)?,
        title: row.get(3)?,
        description: row.get(4)?,
        content: row.get(5)?,
        sort_order: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

const ENTRY_COLUMNS: &str =
    "id, folder_id, feature_id, title, description, content, sort_order, created_at, updated_at";

pub fn list_entries(conn: &Connection) -> Result<Vec<KnowledgeEntry>, String> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {} FROM knowledge_entries ORDER BY sort_order ASC, title ASC",
            ENTRY_COLUMNS
        ))
        .map_err(|e| e.to_string())?;

    let entries = stmt
        .query_map([], |row| read_entry(row))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(entries)
}

pub fn list_entries_in_folder(
    conn: &Connection,
    folder_id: Option<&str>,
) -> Result<Vec<KnowledgeEntry>, String> {
    if let Some(fid) = folder_id {
        let mut stmt = conn
            .prepare(&format!(
                "SELECT {} FROM knowledge_entries WHERE folder_id = ?1 ORDER BY sort_order ASC, title ASC",
                ENTRY_COLUMNS
            ))
            .map_err(|e| e.to_string())?;

        let entries = stmt
            .query_map(params![fid], |row| read_entry(row))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(entries)
    } else {
        let mut stmt = conn
            .prepare(&format!(
                "SELECT {} FROM knowledge_entries WHERE folder_id IS NULL ORDER BY sort_order ASC, title ASC",
                ENTRY_COLUMNS
            ))
            .map_err(|e| e.to_string())?;

        let entries = stmt
            .query_map([], |row| read_entry(row))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(entries)
    }
}

pub fn get_entry(conn: &Connection, id: &str) -> Result<KnowledgeEntry, String> {
    conn.query_row(
        &format!(
            "SELECT {} FROM knowledge_entries WHERE id = ?1",
            ENTRY_COLUMNS
        ),
        params![id],
        |row| read_entry(row),
    )
    .map_err(|e| e.to_string())
}

pub fn create_entry(
    conn: &Connection,
    title: &str,
    content: &str,
    description: Option<&str>,
    folder_id: Option<&str>,
    feature_id: Option<&str>,
) -> Result<KnowledgeEntry, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let desc = description.unwrap_or("");

    conn.execute(
        "INSERT INTO knowledge_entries (id, folder_id, feature_id, title, description, content, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, ?7)",
        params![id, folder_id, feature_id, title, desc, content, now],
    )
    .map_err(|e| e.to_string())?;

    super::search::index_knowledge_entry(conn, &id, title, desc, content).ok();

    get_entry(conn, &id)
}

pub fn update_entry(
    conn: &Connection,
    id: &str,
    title: Option<&str>,
    content: Option<&str>,
    description: Option<&str>,
    folder_id: Option<Option<&str>>,
    feature_id: Option<Option<&str>>,
) -> Result<KnowledgeEntry, String> {
    let now = Utc::now().to_rfc3339();
    let existing = get_entry(conn, id)?;

    let new_title = title.unwrap_or(&existing.title);
    let new_content = content.unwrap_or(&existing.content);
    let new_desc = description.unwrap_or(&existing.description);
    let new_folder_id = folder_id.unwrap_or(existing.folder_id.as_deref());
    let new_feature_id = feature_id.unwrap_or(existing.feature_id.as_deref());

    conn.execute(
        "UPDATE knowledge_entries SET title = ?1, content = ?2, description = ?3,
         folder_id = ?4, feature_id = ?5, updated_at = ?6 WHERE id = ?7",
        params![
            new_title,
            new_content,
            new_desc,
            new_folder_id,
            new_feature_id,
            now,
            id
        ],
    )
    .map_err(|e| e.to_string())?;

    super::search::index_knowledge_entry(conn, id, new_title, new_desc, new_content).ok();

    get_entry(conn, id)
}

pub fn delete_entry(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM knowledge_entries WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    super::search::delete_knowledge_entry(conn, id).ok();
    Ok(())
}
