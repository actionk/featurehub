use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Folder {
    pub id: String,
    pub feature_id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub created_at: String,
}

pub fn get_folders(conn: &Connection, feature_id: &str) -> Result<Vec<Folder>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, feature_id, parent_id, name, created_at
             FROM folders WHERE feature_id = ?1 ORDER BY name ASC",
        )
        .map_err(|e| e.to_string())?;

    let folders = stmt
        .query_map(params![feature_id], |row| {
            Ok(Folder {
                id: row.get(0)?,
                feature_id: row.get(1)?,
                parent_id: row.get(2)?,
                name: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(folders)
}

fn validate_folder_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Folder name must not be empty".to_string());
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err("Folder name must not contain path separators or '..'".to_string());
    }
    Ok(())
}

pub fn create_folder(
    conn: &Connection,
    feature_id: &str,
    parent_id: Option<&str>,
    name: &str,
) -> Result<Folder, String> {
    validate_folder_name(name)?;

    // Enforce max nesting depth of 5
    if let Some(pid) = parent_id {
        let depth = get_folder_depth(conn, pid)?;
        if depth >= 5 {
            return Err("Maximum folder nesting depth of 5 exceeded".to_string());
        }
    }

    // Check for name collision at same level
    let final_name = resolve_folder_name(conn, feature_id, parent_id, name)?;

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO folders (id, feature_id, parent_id, name, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, feature_id, parent_id, final_name, now],
    )
    .map_err(|e| e.to_string())?;

    // Touch feature updated_at
    conn.execute(
        "UPDATE features SET updated_at = ?1 WHERE id = ?2",
        params![now, feature_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(Folder {
        id,
        feature_id: feature_id.to_string(),
        parent_id: parent_id.map(|s| s.to_string()),
        name: final_name,
        created_at: now,
    })
}

pub fn rename_folder(conn: &Connection, id: &str, new_name: &str) -> Result<Folder, String> {
    validate_folder_name(new_name)?;
    let folder = get_folder(conn, id)?;
    let final_name = resolve_folder_name_excluding(
        conn,
        &folder.feature_id,
        folder.parent_id.as_deref(),
        new_name,
        Some(id),
    )?;

    conn.execute(
        "UPDATE folders SET name = ?1 WHERE id = ?2",
        params![final_name, id],
    )
    .map_err(|e| e.to_string())?;

    Ok(Folder {
        name: final_name,
        ..folder
    })
}

pub fn delete_folder(conn: &Connection, id: &str) -> Result<Vec<String>, String> {
    // Collect stored_paths of all files in this folder and its descendants
    let descendant_ids = get_descendant_folder_ids(conn, id)?;
    let mut all_ids = vec![id.to_string()];
    all_ids.extend(descendant_ids);

    let mut stored_paths = Vec::new();
    for folder_id in &all_ids {
        let mut stmt = conn
            .prepare("SELECT stored_path FROM files WHERE folder_id = ?1")
            .map_err(|e| e.to_string())?;
        let paths: Vec<String> = stmt
            .query_map(params![folder_id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        stored_paths.extend(paths);
    }

    // Delete the folder (CASCADE will handle child folders, files get folder_id set to NULL)
    // But we need to delete files in these folders from DB too since we're cleaning up disk
    for folder_id in &all_ids {
        conn.execute("DELETE FROM files WHERE folder_id = ?1", params![folder_id])
            .map_err(|e| e.to_string())?;
    }

    conn.execute("DELETE FROM folders WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(stored_paths)
}

pub fn move_folder(
    conn: &Connection,
    id: &str,
    new_parent_id: Option<&str>,
) -> Result<Folder, String> {
    // Prevent moving a folder into itself or its descendants
    if let Some(new_pid) = new_parent_id {
        if new_pid == id {
            return Err("Cannot move folder into itself".to_string());
        }
        let descendants = get_descendant_folder_ids(conn, id)?;
        if descendants.contains(&new_pid.to_string()) {
            return Err("Cannot move folder into its own descendant".to_string());
        }
    }

    // Enforce depth limit
    if let Some(new_pid) = new_parent_id {
        let parent_depth = get_folder_depth(conn, new_pid)?;
        let subtree_depth = get_subtree_depth(conn, id)?;
        if parent_depth + subtree_depth >= 5 {
            return Err("Move would exceed maximum folder nesting depth of 5".to_string());
        }
    }

    conn.execute(
        "UPDATE folders SET parent_id = ?1 WHERE id = ?2",
        params![new_parent_id, id],
    )
    .map_err(|e| e.to_string())?;

    get_folder(conn, id)
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

fn get_folder(conn: &Connection, id: &str) -> Result<Folder, String> {
    conn.query_row(
        "SELECT id, feature_id, parent_id, name, created_at FROM folders WHERE id = ?1",
        params![id],
        |row| {
            Ok(Folder {
                id: row.get(0)?,
                feature_id: row.get(1)?,
                parent_id: row.get(2)?,
                name: row.get(3)?,
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

fn get_subtree_depth(conn: &Connection, folder_id: &str) -> Result<usize, String> {
    let mut stmt = conn
        .prepare("SELECT id FROM folders WHERE parent_id = ?1")
        .map_err(|e| e.to_string())?;
    let children: Vec<String> = stmt
        .query_map(params![folder_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    if children.is_empty() {
        return Ok(1);
    }

    let mut max_child_depth = 0;
    for child_id in &children {
        let child_depth = get_subtree_depth(conn, child_id)?;
        if child_depth > max_child_depth {
            max_child_depth = child_depth;
        }
    }

    Ok(1 + max_child_depth)
}

fn get_descendant_folder_ids(conn: &Connection, folder_id: &str) -> Result<Vec<String>, String> {
    let mut result = Vec::new();
    let mut stmt = conn
        .prepare("SELECT id FROM folders WHERE parent_id = ?1")
        .map_err(|e| e.to_string())?;
    let children: Vec<String> = stmt
        .query_map(params![folder_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for child_id in &children {
        result.push(child_id.clone());
        let descendants = get_descendant_folder_ids(conn, child_id)?;
        result.extend(descendants);
    }

    Ok(result)
}

fn resolve_folder_name(
    conn: &Connection,
    feature_id: &str,
    parent_id: Option<&str>,
    name: &str,
) -> Result<String, String> {
    resolve_folder_name_excluding(conn, feature_id, parent_id, name, None)
}

fn resolve_folder_name_excluding(
    conn: &Connection,
    feature_id: &str,
    parent_id: Option<&str>,
    name: &str,
    exclude_id: Option<&str>,
) -> Result<String, String> {
    let exclude = exclude_id.unwrap_or("");

    let exists = if let Some(pid) = parent_id {
        conn.query_row(
            "SELECT COUNT(*) FROM folders WHERE feature_id = ?1 AND parent_id = ?2 AND name = ?3 AND id != ?4",
            params![feature_id, pid, name, exclude],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|e| e.to_string())?
            > 0
    } else {
        conn.query_row(
            "SELECT COUNT(*) FROM folders WHERE feature_id = ?1 AND parent_id IS NULL AND name = ?2 AND id != ?3",
            params![feature_id, name, exclude],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|e| e.to_string())?
            > 0
    };

    if !exists {
        return Ok(name.to_string());
    }

    let mut counter = 1;
    loop {
        let candidate = format!("{}_{}", name, counter);
        let candidate_exists = if let Some(pid) = parent_id {
            conn.query_row(
                "SELECT COUNT(*) FROM folders WHERE feature_id = ?1 AND parent_id = ?2 AND name = ?3 AND id != ?4",
                params![feature_id, pid, candidate, exclude],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|e| e.to_string())?
                > 0
        } else {
            conn.query_row(
                "SELECT COUNT(*) FROM folders WHERE feature_id = ?1 AND parent_id IS NULL AND name = ?2 AND id != ?3",
                params![feature_id, candidate, exclude],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|e| e.to_string())?
                > 0
        };

        if !candidate_exists {
            return Ok(candidate);
        }
        counter += 1;
    }
}
