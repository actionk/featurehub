use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub feature_id: String,
    pub title: String,
    pub done: bool,
    pub sort_order: i32,
    pub created_at: String,
    pub source: String,
    pub external_key: Option<String>,
    pub external_url: Option<String>,
    pub external_status: Option<String>,
    pub description: Option<String>,
}

const SELECT_COLS: &str = "id, feature_id, title, done, sort_order, created_at, source, external_key, external_url, external_status, description";

fn row_to_task(row: &rusqlite::Row) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get(0)?,
        feature_id: row.get(1)?,
        title: row.get(2)?,
        done: row.get(3)?,
        sort_order: row.get(4)?,
        created_at: row.get(5)?,
        source: row.get(6)?,
        external_key: row.get(7)?,
        external_url: row.get(8)?,
        external_status: row.get(9)?,
        description: row.get(10)?,
    })
}

pub fn get_tasks(conn: &Connection, feature_id: &str) -> Result<Vec<Task>, String> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {} FROM tasks WHERE feature_id = ?1 ORDER BY sort_order, created_at",
            SELECT_COLS
        ))
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![feature_id], |row| row_to_task(row))
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn create_task(
    conn: &Connection,
    feature_id: &str,
    title: &str,
    source: Option<&str>,
    external_key: Option<&str>,
    external_url: Option<&str>,
    external_status: Option<&str>,
    description: Option<&str>,
) -> Result<Task, String> {
    let source = source.unwrap_or("manual");
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let max_order: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM tasks WHERE feature_id = ?1",
            params![feature_id],
            |row| row.get(0),
        )
        .unwrap_or(-1);

    // Atomic upsert for external tasks — avoids race condition on duplicate external_key
    if external_key.is_some() {
        conn.execute(
            "INSERT INTO tasks (id, feature_id, title, done, sort_order, created_at, source, external_key, external_url, external_status, description)
             VALUES (?1, ?2, ?3, 0, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(feature_id, external_key) DO UPDATE SET
                title = excluded.title,
                external_status = COALESCE(excluded.external_status, tasks.external_status),
                description = COALESCE(excluded.description, tasks.description)",
            params![id, feature_id, title, max_order + 1, now, source, external_key, external_url, external_status, description],
        )
        .map_err(|e| e.to_string())?;
    } else {
        conn.execute(
            "INSERT INTO tasks (id, feature_id, title, done, sort_order, created_at, source, external_key, external_url, external_status, description)
             VALUES (?1, ?2, ?3, 0, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![id, feature_id, title, max_order + 1, now, source, external_key, external_url, external_status, description],
        )
        .map_err(|e| e.to_string())?;
    }

    // Touch feature updated_at
    conn.execute(
        "UPDATE features SET updated_at = ?1 WHERE id = ?2",
        params![now, feature_id],
    )
    .map_err(|e| e.to_string())?;

    // Return the task (may be the upserted one for external tasks)
    if let Some(ext_key) = external_key {
        let task = conn
            .query_row(
                &format!(
                    "SELECT {} FROM tasks WHERE feature_id = ?1 AND external_key = ?2",
                    SELECT_COLS
                ),
                params![feature_id, ext_key],
                |row| row_to_task(row),
            )
            .map_err(|e| e.to_string())?;
        Ok(task)
    } else {
        Ok(Task {
            id,
            feature_id: feature_id.to_string(),
            title: title.to_string(),
            done: false,
            sort_order: max_order + 1,
            created_at: now,
            source: source.to_string(),
            external_key: None,
            external_url: external_url.map(|s| s.to_string()),
            external_status: external_status.map(|s| s.to_string()),
            description: description.map(|s| s.to_string()),
        })
    }
}

pub fn update_task(
    conn: &Connection,
    id: &str,
    title: Option<&str>,
    done: Option<bool>,
    external_status: Option<&str>,
    description: Option<&str>,
) -> Result<Task, String> {
    let now = Utc::now().to_rfc3339();

    // Build a single UPDATE with all provided fields
    let mut set_clauses = Vec::new();
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(t) = title {
        set_clauses.push(format!("title = ?{}", param_values.len() + 1));
        param_values.push(Box::new(t.to_string()));
    }
    if let Some(d) = done {
        set_clauses.push(format!("done = ?{}", param_values.len() + 1));
        param_values.push(Box::new(d));
    }
    if let Some(es) = external_status {
        set_clauses.push(format!("external_status = ?{}", param_values.len() + 1));
        param_values.push(Box::new(es.to_string()));
    }
    if let Some(desc) = description {
        set_clauses.push(format!("description = ?{}", param_values.len() + 1));
        param_values.push(Box::new(desc.to_string()));
    }

    if !set_clauses.is_empty() {
        let id_param = param_values.len() + 1;
        let query = format!(
            "UPDATE tasks SET {} WHERE id = ?{}",
            set_clauses.join(", "),
            id_param
        );
        param_values.push(Box::new(id.to_string()));

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|v| v.as_ref()).collect();
        conn.execute(&query, params_refs.as_slice())
            .map_err(|e| e.to_string())?;
    }

    // Touch parent feature
    conn.execute(
        "UPDATE features SET updated_at = ?1 WHERE id = (SELECT feature_id FROM tasks WHERE id = ?2)",
        params![now, id],
    )
    .map_err(|e| e.to_string())?;

    // Return updated task
    conn.query_row(
        &format!("SELECT {} FROM tasks WHERE id = ?1", SELECT_COLS),
        params![id],
        |row| row_to_task(row),
    )
    .map_err(|e| e.to_string())
}

pub fn delete_task(conn: &Connection, id: &str) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();

    // Touch parent feature
    conn.execute(
        "UPDATE features SET updated_at = ?1 WHERE id = (SELECT feature_id FROM tasks WHERE id = ?2)",
        params![now, id],
    )
    .map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
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
    fn create_task_returns_task_with_defaults() {
        let (conn, feature_id) = setup();
        let task =
            create_task(&conn, &feature_id, "My Task", None, None, None, None, None).unwrap();

        assert_eq!(task.title, "My Task");
        assert_eq!(task.feature_id, feature_id);
        assert!(!task.done);
        assert_eq!(task.source, "manual");
    }

    #[test]
    fn get_tasks_returns_all_for_feature() {
        let (conn, feature_id) = setup();
        create_task(&conn, &feature_id, "Task A", None, None, None, None, None).unwrap();
        create_task(&conn, &feature_id, "Task B", None, None, None, None, None).unwrap();

        let tasks = get_tasks(&conn, &feature_id).unwrap();
        assert_eq!(tasks.len(), 2);
    }

    #[test]
    fn update_task_marks_done() {
        let (conn, feature_id) = setup();
        let task =
            create_task(&conn, &feature_id, "Do This", None, None, None, None, None).unwrap();

        let updated = update_task(&conn, &task.id, None, Some(true), None, None).unwrap();
        assert!(updated.done);
    }

    #[test]
    fn delete_task_removes_it() {
        let (conn, feature_id) = setup();
        let task = create_task(
            &conn,
            &feature_id,
            "Remove Me",
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        delete_task(&conn, &task.id).unwrap();
        let tasks = get_tasks(&conn, &feature_id).unwrap();
        assert!(tasks.is_empty());
    }
}
