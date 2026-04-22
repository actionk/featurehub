use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub entity_type: String,
    pub entity_id: String,
    pub feature_id: String,
    pub title: String,
    pub snippet: String,
}

pub fn global_search(conn: &Connection, query: &str) -> Result<Vec<SearchResult>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    // Sanitize the query for FTS5 - escape double quotes and wrap terms
    let sanitized = query
        .split_whitespace()
        .map(|term| {
            let escaped = term.replace('"', "\"\"");
            format!("\"{}\"*", escaped)
        })
        .collect::<Vec<_>>()
        .join(" ");

    let mut stmt = conn
        .prepare(
            "SELECT entity_type, entity_id, feature_id, title,
                    snippet(search_index, 4, '<mark>', '</mark>', '...', 32) as snippet
             FROM search_index
             WHERE search_index MATCH ?1
             ORDER BY rank
             LIMIT 50",
        )
        .map_err(|e| e.to_string())?;

    let results = stmt
        .query_map(params![sanitized], |row| {
            Ok(SearchResult {
                entity_type: row.get(0)?,
                entity_id: row.get(1)?,
                feature_id: row.get(2)?,
                title: row.get(3)?,
                snippet: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(results)
}

pub fn index_feature(
    conn: &Connection,
    id: &str,
    title: &str,
    ticket_id: Option<&str>,
) -> Result<(), String> {
    // Remove old entry
    conn.execute(
        "DELETE FROM search_index WHERE entity_type = 'feature' AND entity_id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    let content = format!("{} {}", title, ticket_id.unwrap_or(""));

    conn.execute(
        "INSERT INTO search_index (entity_type, entity_id, feature_id, title, content)
         VALUES ('feature', ?1, ?2, ?3, ?4)",
        params![id, id, title, content],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn index_link(
    conn: &Connection,
    id: &str,
    feature_id: &str,
    title: &str,
    url: &str,
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM search_index WHERE entity_type = 'link' AND entity_id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    let content = format!("{} {}", title, url);

    conn.execute(
        "INSERT INTO search_index (entity_type, entity_id, feature_id, title, content)
         VALUES ('link', ?1, ?2, ?3, ?4)",
        params![id, feature_id, title, content],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn index_session(
    conn: &Connection,
    id: &str,
    feature_id: &str,
    title: &str,
    summary: Option<&str>,
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM search_index WHERE entity_type = 'session' AND entity_id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    let content = format!("{} {}", title, summary.unwrap_or(""));

    conn.execute(
        "INSERT INTO search_index (entity_type, entity_id, feature_id, title, content)
         VALUES ('session', ?1, ?2, ?3, ?4)",
        params![id, feature_id, title, content],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn index_note(
    conn: &Connection,
    feature_id: &str,
    content: &str,
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM search_index WHERE entity_type = 'note' AND feature_id = ?1",
        params![feature_id],
    )
    .map_err(|e| e.to_string())?;

    // Use first 100 chars as title
    let title: String = content.chars().take(100).collect();

    conn.execute(
        "INSERT INTO search_index (entity_type, entity_id, feature_id, title, content)
         VALUES ('note', ?1, ?2, ?3, ?4)",
        params![feature_id, feature_id, title, content],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn index_file(
    conn: &Connection,
    id: &str,
    feature_id: &str,
    filename: &str,
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM search_index WHERE entity_type = 'file' AND entity_id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    // Look up feature title to show in snippet
    let feature_title: String = conn
        .query_row(
            "SELECT title FROM features WHERE id = ?1",
            params![feature_id],
            |row| row.get(0),
        )
        .unwrap_or_default();

    conn.execute(
        "INSERT INTO search_index (entity_type, entity_id, feature_id, title, content)
         VALUES ('file', ?1, ?2, ?3, ?4)",
        params![id, feature_id, filename, feature_title],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn remove_file_from_index(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute(
        "DELETE FROM search_index WHERE entity_type = 'file' AND entity_id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn index_knowledge_entry(
    conn: &Connection,
    id: &str,
    title: &str,
    description: &str,
    content: &str,
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM search_index WHERE entity_type = 'knowledge' AND entity_id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    let search_content = format!("{} {} {}", title, description, content);

    conn.execute(
        "INSERT INTO search_index (entity_type, entity_id, feature_id, title, content)
         VALUES ('knowledge', ?1, '', ?2, ?3)",
        params![id, title, search_content],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn delete_knowledge_entry(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute(
        "DELETE FROM search_index WHERE entity_type = 'knowledge' AND entity_id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn rebuild_search_index(conn: &Connection) -> Result<(), String> {
    // Clear the index
    conn.execute("DELETE FROM search_index", [])
        .map_err(|e| e.to_string())?;

    // Re-index features
    let mut stmt = conn
        .prepare("SELECT id, title, ticket_id FROM features")
        .map_err(|e| e.to_string())?;

    let features: Vec<(String, String, Option<String>)> = stmt
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for (id, title, ticket_id) in &features {
        index_feature(conn, id, title, ticket_id.as_deref())?;
    }

    // Re-index links
    let mut stmt = conn
        .prepare("SELECT id, feature_id, title, url FROM links")
        .map_err(|e| e.to_string())?;

    let links: Vec<(String, String, String, String)> = stmt
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for (id, feature_id, title, url) in &links {
        index_link(conn, id, feature_id, title, url)?;
    }

    // Re-index sessions
    let mut stmt = conn
        .prepare("SELECT id, feature_id, title, summary FROM sessions")
        .map_err(|e| e.to_string())?;

    let sessions: Vec<(String, String, Option<String>, Option<String>)> = stmt
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for (id, feature_id, title, summary) in &sessions {
        if let Some(t) = title {
            index_session(conn, id, feature_id, t, summary.as_deref())?;
        }
    }

    // Re-index notes
    let mut stmt = conn
        .prepare("SELECT feature_id, content FROM notes")
        .map_err(|e| e.to_string())?;

    let notes: Vec<(String, String)> = stmt
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for (feature_id, content) in &notes {
        index_note(conn, feature_id, content)?;
    }

    // Re-index files
    let mut stmt = conn
        .prepare("SELECT id, feature_id, filename FROM files")
        .map_err(|e| e.to_string())?;

    let files: Vec<(String, String, String)> = stmt
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for (id, feature_id, filename) in &files {
        index_file(conn, id, feature_id, filename)?;
    }

    // Re-index knowledge entries
    let mut stmt = conn
        .prepare("SELECT id, title, description, content FROM knowledge_entries")
        .map_err(|e| e.to_string())?;

    let entries: Vec<(String, String, String, String)> = stmt
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for (id, title, description, content) in &entries {
        index_knowledge_entry(conn, id, title, description, content)?;
    }

    Ok(())
}
