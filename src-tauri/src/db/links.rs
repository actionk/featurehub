use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Link {
    pub id: String,
    pub feature_id: String,
    pub title: String,
    pub url: String,
    pub link_type: String,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
}

pub fn detect_link_type(url: &str) -> String {
    let url_lower = url.to_lowercase();
    if url_lower.contains("github.com") {
        if url_lower.contains("/pull/") || url_lower.contains("/pulls") {
            "github-pr".to_string()
        } else if url_lower.contains("/issues") {
            "github-issue".to_string()
        } else {
            "github".to_string()
        }
    } else if url_lower.contains("gitlab.com") {
        "gitlab".to_string()
    } else if url_lower.contains("jira") || url_lower.contains("atlassian.net") {
        "jira".to_string()
    } else if url_lower.contains("linear.app") {
        "linear".to_string()
    } else if url_lower.contains("notion.so") || url_lower.contains("notion.site") {
        "notion".to_string()
    } else if url_lower.contains("figma.com") {
        "figma".to_string()
    } else if url_lower.contains("slack.com") {
        "slack".to_string()
    } else if url_lower.contains("discord.com") || url_lower.contains("discord.gg") {
        "discord".to_string()
    } else if url_lower.contains("docs.google.com") {
        "google-doc".to_string()
    } else if url_lower.contains("trello.com") {
        "trello".to_string()
    } else if url_lower.contains("stackoverflow.com") {
        "stackoverflow".to_string()
    } else if url_lower.contains("slite.com") {
        "slite".to_string()
    } else {
        "other".to_string()
    }
}

fn row_to_link(row: &rusqlite::Row) -> rusqlite::Result<Link> {
    let metadata_str: Option<String> = row.get(6)?;
    let metadata = metadata_str.and_then(|s| serde_json::from_str(&s).ok());
    Ok(Link {
        id: row.get(0)?,
        feature_id: row.get(1)?,
        title: row.get(2)?,
        url: row.get(3)?,
        link_type: row.get(4)?,
        description: row.get(5)?,
        metadata,
        created_at: row.get(7)?,
    })
}

pub fn get_link(conn: &Connection, id: &str) -> Result<Link, String> {
    conn.query_row(
        "SELECT id, feature_id, title, url, link_type, description, metadata, created_at FROM links WHERE id = ?1",
        params![id],
        row_to_link,
    ).map_err(|e| e.to_string())
}

pub fn get_links(conn: &Connection, feature_id: &str) -> Result<Vec<Link>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, feature_id, title, url, link_type, description, metadata, created_at
             FROM links WHERE feature_id = ?1 ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let links = stmt
        .query_map(params![feature_id], |row| row_to_link(row))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(links)
}

pub fn add_link(
    conn: &Connection,
    feature_id: &str,
    title: &str,
    url: &str,
    link_type: Option<String>,
    description: Option<String>,
) -> Result<Link, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let resolved_type = link_type.unwrap_or_else(|| detect_link_type(url));

    conn.execute(
        "INSERT INTO links (id, feature_id, title, url, link_type, description, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, feature_id, title, url, resolved_type, description, now],
    )
    .map_err(|e| e.to_string())?;

    // Update search index
    super::search::index_link(conn, &id, feature_id, title, url).ok();

    // Touch feature updated_at
    conn.execute(
        "UPDATE features SET updated_at = ?1 WHERE id = ?2",
        params![now, feature_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(Link {
        id,
        feature_id: feature_id.to_string(),
        title: title.to_string(),
        url: url.to_string(),
        link_type: resolved_type,
        description,
        metadata: None,
        created_at: now,
    })
}

pub fn update_link(
    conn: &Connection,
    id: &str,
    title: Option<String>,
    url: Option<String>,
    link_type: Option<String>,
    description: Option<Option<String>>,
) -> Result<Link, String> {
    let mut stmt = conn
        .prepare("SELECT id, feature_id, title, url, link_type, description, metadata, created_at FROM links WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    let current = stmt
        .query_row(params![id], |row| row_to_link(row))
        .map_err(|e| e.to_string())?;

    let new_title = title.unwrap_or(current.title);
    let new_url = url.clone().unwrap_or(current.url);
    let new_type = if let Some(lt) = link_type {
        lt
    } else if url.is_some() {
        detect_link_type(&new_url)
    } else {
        current.link_type
    };
    let new_description = description.unwrap_or(current.description);

    conn.execute(
        "UPDATE links SET title=?1, url=?2, link_type=?3, description=?4 WHERE id=?5",
        params![new_title, new_url, new_type, new_description, id],
    )
    .map_err(|e| e.to_string())?;

    // Update search index
    super::search::index_link(conn, id, &current.feature_id, &new_title, &new_url).ok();

    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE features SET updated_at = ?1 WHERE id = ?2",
        params![now, current.feature_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(Link {
        id: id.to_string(),
        feature_id: current.feature_id,
        title: new_title,
        url: new_url,
        link_type: new_type,
        description: new_description,
        metadata: current.metadata,
        created_at: current.created_at,
    })
}

/// Update just the metadata JSON blob on a link.
pub fn update_link_metadata(
    conn: &Connection,
    id: &str,
    metadata: &serde_json::Value,
) -> Result<(), String> {
    let json_str = serde_json::to_string(metadata)
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
    conn.execute(
        "UPDATE links SET metadata = ?1 WHERE id = ?2",
        params![json_str, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_link(conn: &Connection, id: &str) -> Result<(), String> {
    // Remove from search index
    conn.execute(
        "DELETE FROM search_index WHERE entity_type = 'link' AND entity_id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM links WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_utils::test_db;

    #[test]
    fn get_link_by_id_returns_link() {
        let conn = test_db();
        // Seed a feature directly via SQL — minimal columns required by the schema.
        conn.execute(
            "INSERT INTO features (id, title, status, sort_order, created_at, updated_at)
             VALUES ('f1', 'Test', 'active', 0, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
        let link = add_link(
            &conn,
            "f1",
            "title",
            "https://github.com/a/b/pull/1",
            Some("github-pr".into()),
            None,
        )
        .unwrap();
        let got = get_link(&conn, &link.id).unwrap();
        assert_eq!(got.url, link.url);
        assert_eq!(got.link_type, "github-pr");
        assert_eq!(got.feature_id, "f1");
    }

    #[test]
    fn get_link_returns_err_for_unknown_id() {
        let conn = test_db();
        let result = get_link(&conn, "nonexistent");
        assert!(result.is_err());
    }
}
