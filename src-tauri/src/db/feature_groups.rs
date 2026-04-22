use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeatureGroup {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
}

pub fn get_feature_groups(conn: &Connection) -> Result<Vec<FeatureGroup>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, color, sort_order, created_at FROM feature_groups ORDER BY sort_order ASC")
        .map_err(|e| e.to_string())?;

    let groups = stmt
        .query_map([], |row| {
            Ok(FeatureGroup {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                sort_order: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(groups)
}

pub fn create_feature_group(
    conn: &Connection,
    name: &str,
    color: Option<String>,
) -> Result<FeatureGroup, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let max_sort: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM feature_groups",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO feature_groups (id, name, color, sort_order, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, name, color, max_sort + 1, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(FeatureGroup {
        id,
        name: name.to_string(),
        color,
        sort_order: max_sort + 1,
        created_at: now,
    })
}

pub fn update_feature_group(
    conn: &Connection,
    id: &str,
    name: Option<String>,
    color: Option<String>,
) -> Result<FeatureGroup, String> {
    let current = conn
        .query_row(
            "SELECT id, name, color, sort_order, created_at FROM feature_groups WHERE id = ?1",
            params![id],
            |row| {
                Ok(FeatureGroup {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                    sort_order: row.get(3)?,
                    created_at: row.get(4)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    let new_name = name.unwrap_or(current.name);
    let new_color = if color.is_some() { color } else { current.color };

    conn.execute(
        "UPDATE feature_groups SET name = ?1, color = ?2 WHERE id = ?3",
        params![new_name, new_color, id],
    )
    .map_err(|e| e.to_string())?;

    Ok(FeatureGroup {
        id: id.to_string(),
        name: new_name,
        color: new_color,
        sort_order: current.sort_order,
        created_at: current.created_at,
    })
}

pub fn delete_feature_group(conn: &Connection, id: &str) -> Result<(), String> {
    // Ungroup all features in this group
    conn.execute(
        "UPDATE features SET group_id = NULL WHERE group_id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM feature_groups WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn reorder_feature_groups(conn: &Connection, ids: &[String]) -> Result<(), String> {
    let mut stmt = conn
        .prepare("UPDATE feature_groups SET sort_order = ?1 WHERE id = ?2")
        .map_err(|e| e.to_string())?;

    for (i, id) in ids.iter().enumerate() {
        stmt.execute(params![i as i64, id])
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn set_feature_group(
    conn: &Connection,
    feature_id: &str,
    group_id: Option<String>,
) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE features SET group_id = ?1, updated_at = ?2 WHERE id = ?3",
        params![group_id, now, feature_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
