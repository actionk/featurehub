use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::links::Link;
use super::tags::Tag;
use super::directories::Directory;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Feature {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub ticket_id: Option<String>,
    pub status: String,
    pub sort_order: i64,
    pub pinned: bool,
    pub archived: bool,
    pub parent_id: Option<String>,
    pub group_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub tags: Vec<Tag>,
    pub links: Vec<Link>,
    pub directories: Vec<Directory>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeatureSummary {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub ticket_id: Option<String>,
    pub status: String,
    pub sort_order: i64,
    pub pinned: bool,
    pub archived: bool,
    pub parent_id: Option<String>,
    pub group_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub tags: Vec<Tag>,
    pub task_count_total: i32,
    pub task_count_done: i32,
}

pub fn get_features(
    conn: &Connection,
    filter: Option<String>,
    sort: Option<String>,
) -> Result<Vec<FeatureSummary>, String> {
    let mut where_clause = String::from("1=1");
    let mut param_values: Vec<String> = Vec::new();

    if let Some(ref f) = filter {
        if !f.is_empty() && f != "all" {
            where_clause.push_str(" AND f.status = ?");
            param_values.push(f.clone());
        }
    }

    let base_order = match sort.as_deref() {
        Some("title") => "f.title ASC",
        Some("created") => "f.created_at DESC",
        Some("updated") => "f.updated_at DESC",
        _ => "f.updated_at DESC",
    };

    // Pinned features always sort first
    let order_clause = format!("f.pinned DESC, {}", base_order);

    let query = format!(
        "SELECT f.id, f.title, f.ticket_id, f.status, f.sort_order, f.created_at, f.updated_at, f.description, f.pinned, f.archived, f.parent_id, f.group_id
         FROM features f
         WHERE {}
         ORDER BY {}",
        where_clause, order_clause
    );

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

    let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values
        .iter()
        .map(|v| v as &dyn rusqlite::types::ToSql)
        .collect();

    let features_iter = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(FeatureSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                ticket_id: row.get(2)?,
                status: row.get(3)?,
                sort_order: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                description: row.get(7)?,
                pinned: row.get::<_, i64>(8).map(|v| v != 0).unwrap_or(false),
                archived: row.get::<_, i64>(9).map(|v| v != 0).unwrap_or(false),
                parent_id: row.get(10)?,
                group_id: row.get(11)?,
                tags: Vec::new(),
                task_count_total: 0,
                task_count_done: 0,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut features: Vec<FeatureSummary> = Vec::new();
    for f in features_iter {
        let feature = f.map_err(|e| e.to_string())?;
        features.push(feature);
    }

    // Batch-load all tags in one query instead of N+1
    if !features.is_empty() {
        let mut tag_stmt = conn
            .prepare(
                "SELECT ft.feature_id, t.id, t.name, t.color FROM tags t
                 INNER JOIN feature_tags ft ON ft.tag_id = t.id",
            )
            .map_err(|e| e.to_string())?;

        let tag_rows = tag_stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    Tag {
                        id: row.get(1)?,
                        name: row.get(2)?,
                        color: row.get(3)?,
                    },
                ))
            })
            .map_err(|e| e.to_string())?;

        let mut tag_map: std::collections::HashMap<String, Vec<Tag>> = std::collections::HashMap::new();
        for row in tag_rows {
            let (feature_id, tag) = row.map_err(|e| e.to_string())?;
            tag_map.entry(feature_id).or_default().push(tag);
        }

        for feature in &mut features {
            if let Some(tags) = tag_map.remove(&feature.id) {
                feature.tags = tags;
            }
        }

        // Batch-load task counts
        let mut task_stmt = conn
            .prepare(
                "SELECT feature_id, COUNT(*) as total, SUM(CASE WHEN done THEN 1 ELSE 0 END) as done
                 FROM tasks GROUP BY feature_id",
            )
            .map_err(|e| e.to_string())?;

        let task_rows = task_stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i32>(1)?,
                    row.get::<_, i32>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;

        let mut task_count_map: std::collections::HashMap<String, (i32, i32)> = std::collections::HashMap::new();
        for row in task_rows {
            let (feature_id, total, done) = row.map_err(|e| e.to_string())?;
            task_count_map.insert(feature_id, (total, done));
        }

        for feature in &mut features {
            if let Some((total, done)) = task_count_map.get(&feature.id) {
                feature.task_count_total = *total;
                feature.task_count_done = *done;
            }
        }
    }

    Ok(features)
}

fn get_feature_tags(conn: &Connection, feature_id: &str) -> Result<Vec<Tag>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.name, t.color FROM tags t
             INNER JOIN feature_tags ft ON ft.tag_id = t.id
             WHERE ft.feature_id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let tags = stmt
        .query_map(params![feature_id], |row| {
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

/// Get direct children of a feature (lightweight — returns id and title only).
pub fn get_feature_children(conn: &Connection, parent_id: &str) -> Result<Vec<(String, String)>, String> {
    let mut stmt = conn
        .prepare("SELECT id, title FROM features WHERE parent_id = ?1 ORDER BY sort_order")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![parent_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn get_feature(conn: &Connection, id: &str) -> Result<Feature, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, ticket_id, status, sort_order, created_at, updated_at, description, pinned, archived, parent_id, group_id
             FROM features WHERE id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let feature = stmt
        .query_row(params![id], |row| {
            Ok(Feature {
                id: row.get(0)?,
                title: row.get(1)?,
                ticket_id: row.get(2)?,
                status: row.get(3)?,
                sort_order: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                description: row.get(7)?,
                pinned: row.get::<_, i64>(8).map(|v| v != 0).unwrap_or(false),
                archived: row.get::<_, i64>(9).map(|v| v != 0).unwrap_or(false),
                parent_id: row.get(10)?,
                group_id: row.get(11)?,
                tags: Vec::new(),
                links: Vec::new(),
                directories: Vec::new(),
            })
        })
        .map_err(|e| e.to_string())?;

    let tags = get_feature_tags(conn, &feature.id)?;
    let links = super::links::get_links(conn, &feature.id)?;
    let directories = super::directories::get_directories(conn, &feature.id)?;

    Ok(Feature {
        tags,
        links,
        directories,
        ..feature
    })
}

pub fn create_feature(
    conn: &Connection,
    title: &str,
    ticket_id: Option<String>,
    status: Option<String>,
    description: Option<String>,
    parent_id: Option<String>,
) -> Result<Feature, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let status = status.unwrap_or_else(|| "active".to_string());

    // Shift existing siblings down to make room at the top
    if parent_id.is_some() {
        conn.execute(
            "UPDATE features SET sort_order = sort_order + 1 WHERE parent_id = ?1",
            params![parent_id],
        )
        .map_err(|e| e.to_string())?;
    } else {
        conn.execute(
            "UPDATE features SET sort_order = sort_order + 1 WHERE parent_id IS NULL",
            [],
        )
        .map_err(|e| e.to_string())?;
    }

    conn.execute(
        "INSERT INTO features (id, title, ticket_id, status, description, sort_order, parent_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![id, title, ticket_id, status, description, 0, parent_id, now, now],
    )
    .map_err(|e| e.to_string())?;

    // Update search index
    super::search::index_feature(conn, &id, title, ticket_id.as_deref()).ok();

    get_feature(conn, &id)
}

pub fn update_feature(
    conn: &Connection,
    id: &str,
    title: Option<String>,
    ticket_id: Option<String>,
    status: Option<String>,
    sort_order: Option<i64>,
    description: Option<String>,
) -> Result<Feature, String> {
    let now = Utc::now().to_rfc3339();

    // Fetch current feature to merge fields
    let current = get_feature(conn, id)?;

    let new_title = title.unwrap_or(current.title);
    let new_ticket_id = if ticket_id.is_some() {
        ticket_id
    } else {
        current.ticket_id
    };
    let old_status = current.status.clone();
    let new_status = status.unwrap_or(current.status);
    let new_sort = sort_order.unwrap_or(current.sort_order);
    let new_description = if description.is_some() {
        description
    } else {
        current.description
    };

    // Auto-archive when status changes to "done"
    let auto_archive = new_status == "done" && old_status != "done";

    conn.execute(
        "UPDATE features SET title=?1, ticket_id=?2, status=?3, sort_order=?4, updated_at=?5, description=?6
         WHERE id=?7",
        params![new_title, new_ticket_id, new_status, new_sort, now, new_description, id],
    )
    .map_err(|e| e.to_string())?;

    if auto_archive {
        conn.execute(
            "UPDATE features SET archived = 1 WHERE id = ?1",
            params![id],
        )
        .map_err(|e| e.to_string())?;
    }

    // Update search index
    super::search::index_feature(conn, id, &new_title, new_ticket_id.as_deref()).ok();

    // Re-index files for this feature (they store the feature title in their content)
    if let Ok(files) = super::files::get_files(conn, id) {
        for f in &files {
            super::search::index_file(conn, &f.id, id, &f.filename).ok();
        }
    }

    get_feature(conn, id)
}

pub fn delete_feature(
    conn: &Connection,
    id: &str,
    storage_base: Option<&std::path::Path>,
) -> Result<(), String> {
    // Delete attached files from disk
    let files = super::files::get_files(conn, id)?;
    for file in files {
        let _ = std::fs::remove_file(&file.stored_path);
    }

    // Remove from search index
    conn.execute(
        "DELETE FROM search_index WHERE feature_id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    // Orphan children: move them to root level
    conn.execute(
        "UPDATE features SET parent_id = NULL WHERE parent_id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM features WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    // Clean up storage directory
    if let Some(base) = storage_base {
        let storage_dir = crate::files::manager::get_storage_path(base, id);
        let _ = std::fs::remove_dir_all(&storage_dir);
    }

    Ok(())
}

pub fn reorder_features(conn: &Connection, ids: &[String]) -> Result<(), String> {
    if ids.is_empty() {
        return Ok(());
    }

    let now = Utc::now().to_rfc3339();

    // Use a prepared statement to avoid re-parsing the SQL for each iteration
    let mut stmt = conn
        .prepare("UPDATE features SET sort_order = ?1, updated_at = ?2 WHERE id = ?3")
        .map_err(|e| e.to_string())?;

    for (i, id) in ids.iter().enumerate() {
        stmt.execute(params![i as i64, now, id])
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn toggle_pin_feature(conn: &Connection, id: &str) -> Result<Feature, String> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE features SET pinned = CASE WHEN pinned = 0 THEN 1 ELSE 0 END, updated_at = ?1 WHERE id = ?2",
        params![now, id],
    )
    .map_err(|e| e.to_string())?;
    get_feature(conn, id)
}

pub fn set_archived(conn: &Connection, id: &str, archived: bool) -> Result<Feature, String> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE features SET archived = ?1, updated_at = ?2 WHERE id = ?3",
        params![archived as i64, now, id],
    )
    .map_err(|e| e.to_string())?;
    get_feature(conn, id)
}

pub fn duplicate_feature(conn: &Connection, id: &str, storage_base: Option<&std::path::Path>) -> Result<Feature, String> {
    // Wrap entire duplication in a transaction to prevent partial copies on failure
    conn.execute_batch("BEGIN TRANSACTION").map_err(|e| e.to_string())?;

    match duplicate_feature_inner(conn, id, storage_base) {
        Ok(feature) => {
            conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
            Ok(feature)
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

fn duplicate_feature_inner(conn: &Connection, id: &str, storage_base: Option<&std::path::Path>) -> Result<Feature, String> {
    let source = get_feature(conn, id)?;
    let new_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let max_sort: i64 = conn
        .query_row("SELECT COALESCE(MAX(sort_order), -1) FROM features", [], |row| {
            row.get(0)
        })
        .map_err(|e| e.to_string())?;

    let new_title = format!("{} (copy)", source.title);

    conn.execute(
        "INSERT INTO features (id, title, ticket_id, status, description, sort_order, pinned, archived, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, 0, ?7, ?8)",
        params![
            new_id, new_title, source.ticket_id, source.status, source.description,
            max_sort + 1, now, now
        ],
    )
    .map_err(|e| e.to_string())?;

    // Copy tags
    for tag in &source.tags {
        conn.execute(
            "INSERT INTO feature_tags (feature_id, tag_id) VALUES (?1, ?2)",
            params![new_id, tag.id],
        )
        .map_err(|e| e.to_string())?;
    }

    // Copy links
    for link in &source.links {
        let link_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO links (id, feature_id, title, url, link_type, description, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![link_id, new_id, link.title, link.url, link.link_type, link.description, now],
        )
        .map_err(|e| e.to_string())?;
    }

    // Copy directories
    for dir in &source.directories {
        let dir_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO directories (id, feature_id, path, label, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![dir_id, new_id, dir.path, dir.label, now],
        )
        .map_err(|e| e.to_string())?;
    }

    // Copy tasks
    let tasks = super::tasks::get_tasks(conn, id)?;
    for task in &tasks {
        let task_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO tasks (id, feature_id, title, done, sort_order, source, external_key, external_url, external_status, description, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                task_id, new_id, task.title, task.done as i64, task.sort_order,
                task.source, task.external_key, task.external_url, task.external_status,
                task.description, now
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    // Copy note
    if let Ok(Some(note)) = super::notes::get_note(conn, id) {
        let note_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO notes (id, feature_id, content, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![note_id, new_id, note.content, now],
        )
        .map_err(|e| e.to_string())?;
    }

    // Copy context
    if let Ok(Some(ctx)) = super::context::get_context(conn, id) {
        let ctx_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO context (id, feature_id, content, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![ctx_id, new_id, ctx.content, now],
        )
        .map_err(|e| e.to_string())?;
    }

    // Copy folders (preserving hierarchy via old->new ID mapping)
    let source_folders = super::folders::get_folders(conn, id)?;
    let mut folder_id_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    // Sort folders so parents are created before children
    // Process folders with no parent first, then children
    let mut remaining = source_folders.clone();
    let mut ordered_folders = Vec::new();
    while !remaining.is_empty() {
        let (ready, not_ready): (Vec<_>, Vec<_>) = remaining.into_iter().partition(|f| {
            match &f.parent_id {
                None => true,
                Some(pid) => folder_id_map.contains_key(pid),
            }
        });
        if ready.is_empty() {
            break; // prevent infinite loop on broken data
        }
        for folder in &ready {
            let new_folder_id = Uuid::new_v4().to_string();
            let new_parent_id = folder.parent_id.as_ref().and_then(|pid| folder_id_map.get(pid));
            conn.execute(
                "INSERT INTO folders (id, feature_id, parent_id, name, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![new_folder_id, new_id, new_parent_id, folder.name, now],
            )
            .map_err(|e| e.to_string())?;
            folder_id_map.insert(folder.id.clone(), new_folder_id);
        }
        ordered_folders.extend(ready);
        remaining = not_ready;
    }

    // Copy files (DB records + disk files)
    if let Some(base_path) = storage_base {
        let source_files = super::files::get_files(conn, id)?;
        for file in &source_files {
            let new_folder_id = file.folder_id.as_ref().and_then(|fid| folder_id_map.get(fid));

            // Build subfolder path for the new file
            let subfolder = new_folder_id.and_then(|nfid| {
                super::folders::get_folder_path(conn, nfid).ok()
            });

            // Copy file on disk
            let source_stored = std::path::PathBuf::from(&file.stored_path);
            if source_stored.exists() {
                let new_storage_dir = crate::files::manager::ensure_storage_dir(base_path, &new_id)?;
                let target_dir = if let Some(ref sub) = subfolder {
                    let dir = new_storage_dir.join(sub);
                    std::fs::create_dir_all(&dir)
                        .map_err(|e| format!("Failed to create subfolder: {}", e))?;
                    dir
                } else {
                    new_storage_dir
                };

                let mut dest_path = target_dir.join(&file.filename);
                // Handle filename collisions
                if dest_path.exists() {
                    let stem = std::path::Path::new(&file.filename)
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let ext = std::path::Path::new(&file.filename)
                        .extension()
                        .map(|e| format!(".{}", e.to_string_lossy()))
                        .unwrap_or_default();
                    let mut counter = 1;
                    loop {
                        let new_name = format!("{}_{}{}", stem, counter, ext);
                        dest_path = target_dir.join(&new_name);
                        if !dest_path.exists() {
                            break;
                        }
                        counter += 1;
                    }
                }

                std::fs::copy(&source_stored, &dest_path)
                    .map_err(|e| format!("Failed to copy file: {}", e))?;

                let new_stored_path = dest_path.to_string_lossy().to_string();
                let new_filename = dest_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let file_id = Uuid::new_v4().to_string();
                conn.execute(
                    "INSERT INTO files (id, feature_id, filename, original_path, stored_path, size, folder_id, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![file_id, new_id, new_filename, file.original_path, new_stored_path, file.size, new_folder_id, now],
                )
                .map_err(|e| e.to_string())?;

                super::search::index_file(conn, &file_id, &new_id, &new_filename).ok();
            }
        }
    }

    // Copy disk folders that have no files (ensure empty folders are created on disk too)
    if let Some(base_path) = storage_base {
        for (_old_id, new_folder_id) in &folder_id_map {
            if let Ok(rel_path) = super::folders::get_folder_path(conn, new_folder_id) {
                crate::files::manager::create_folder_on_disk(base_path, &new_id, &rel_path).ok();
            }
        }
    }

    // Update search index
    super::search::index_feature(conn, &new_id, &new_title, source.ticket_id.as_deref()).ok();

    get_feature(conn, &new_id)
}

/// Check if setting `proposed_parent_id` as parent of `feature_id` would create a cycle.
fn would_create_cycle(conn: &Connection, feature_id: &str, proposed_parent_id: &str) -> Result<bool, String> {
    if feature_id == proposed_parent_id {
        return Ok(true);
    }
    // Walk up from proposed_parent_id
    let mut current = Some(proposed_parent_id.to_string());
    while let Some(ref cid) = current {
        let parent: Option<String> = conn
            .query_row(
                "SELECT parent_id FROM features WHERE id = ?1",
                params![cid],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        match parent {
            Some(ref pid) if pid == feature_id => return Ok(true),
            Some(pid) => current = Some(pid),
            None => break,
        }
    }
    Ok(false)
}

pub fn set_feature_parent(
    conn: &Connection,
    id: &str,
    parent_id: Option<String>,
) -> Result<Feature, String> {
    if let Some(ref pid) = parent_id {
        if would_create_cycle(conn, id, pid)? {
            return Err("Cannot set parent: would create a cycle".to_string());
        }
    }

    let now = Utc::now().to_rfc3339();

    // Place at end of new sibling group
    let max_sort: i64 = if parent_id.is_some() {
        conn.query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM features WHERE parent_id = ?1",
            params![parent_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?
    } else {
        conn.query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM features WHERE parent_id IS NULL",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?
    };

    conn.execute(
        "UPDATE features SET parent_id = ?1, sort_order = ?2, updated_at = ?3 WHERE id = ?4",
        params![parent_id, max_sort + 1, now, id],
    )
    .map_err(|e| e.to_string())?;

    get_feature(conn, id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_utils::test_db;

    #[test]
    fn create_feature_returns_feature_with_defaults() {
        let conn = test_db();
        let feature = create_feature(&conn, "My Feature", None, None, None, None).unwrap();

        assert_eq!(feature.title, "My Feature");
        assert_eq!(feature.status, "active");
        assert!(feature.ticket_id.is_none());
        assert!(feature.description.is_none());
        assert!(!feature.pinned);
        assert!(!feature.archived);
        assert!(feature.parent_id.is_none());
    }

    #[test]
    fn get_feature_returns_created_feature() {
        let conn = test_db();
        let created = create_feature(&conn, "Test Feature", None, None, None, None).unwrap();
        let fetched = get_feature(&conn, &created.id).unwrap();

        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.title, "Test Feature");
    }

    #[test]
    fn get_feature_not_found_returns_error() {
        let conn = test_db();
        let result = get_feature(&conn, "nonexistent-id");

        assert!(result.is_err());
    }

    #[test]
    fn update_feature_changes_title() {
        let conn = test_db();
        let created = create_feature(&conn, "Original", None, None, None, None).unwrap();
        let updated = update_feature(
            &conn, &created.id,
            Some("Renamed".to_string()), None, None, None, None,
        ).unwrap();

        assert_eq!(updated.title, "Renamed");
    }

    #[test]
    fn delete_feature_removes_it() {
        let conn = test_db();
        let created = create_feature(&conn, "To Delete", None, None, None, None).unwrap();
        delete_feature(&conn, &created.id, None).unwrap();

        let result = get_feature(&conn, &created.id);
        assert!(result.is_err());
    }

    #[test]
    fn get_features_returns_all_created() {
        let conn = test_db();
        create_feature(&conn, "Feature A", None, None, None, None).unwrap();
        create_feature(&conn, "Feature B", None, None, None, None).unwrap();

        let features = get_features(&conn, None, None).unwrap();
        assert_eq!(features.len(), 2);
    }

    #[test]
    fn toggle_pin_flips_pinned_state() {
        let conn = test_db();
        let created = create_feature(&conn, "Pin Me", None, None, None, None).unwrap();
        assert!(!created.pinned);

        let pinned = toggle_pin_feature(&conn, &created.id).unwrap();
        assert!(pinned.pinned);

        let unpinned = toggle_pin_feature(&conn, &created.id).unwrap();
        assert!(!unpinned.pinned);
    }
}
