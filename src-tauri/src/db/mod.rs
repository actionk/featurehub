pub mod context;
pub mod directories;
pub mod feature_groups;
pub mod features;
pub mod files;
pub mod folders;
pub mod links;
pub mod mcp_servers;
pub mod notes;
pub mod skills;
pub mod plans;
pub mod search;
pub mod sessions;
pub mod tags;
pub mod tasks;
pub mod knowledge;

#[cfg(test)]
pub mod test_utils;

use std::path::Path;
use rusqlite::{Connection, Result};

pub fn initialize(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    conn.execute_batch("PRAGMA busy_timeout=3000;")?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS features (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            ticket_id TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            left_off_text TEXT,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            color TEXT NOT NULL DEFAULT '#6b7280'
        );

        CREATE TABLE IF NOT EXISTS feature_tags (
            feature_id TEXT NOT NULL,
            tag_id TEXT NOT NULL,
            PRIMARY KEY (feature_id, tag_id),
            FOREIGN KEY (feature_id) REFERENCES features(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS links (
            id TEXT PRIMARY KEY,
            feature_id TEXT NOT NULL,
            title TEXT NOT NULL,
            url TEXT NOT NULL,
            link_type TEXT NOT NULL DEFAULT 'other',
            created_at TEXT NOT NULL,
            FOREIGN KEY (feature_id) REFERENCES features(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS directories (
            id TEXT PRIMARY KEY,
            feature_id TEXT NOT NULL,
            path TEXT NOT NULL,
            label TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY (feature_id) REFERENCES features(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            feature_id TEXT NOT NULL,
            filename TEXT NOT NULL,
            original_path TEXT NOT NULL,
            stored_path TEXT NOT NULL,
            size INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            FOREIGN KEY (feature_id) REFERENCES features(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS folders (
            id TEXT PRIMARY KEY,
            feature_id TEXT NOT NULL,
            parent_id TEXT,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (feature_id) REFERENCES features(id) ON DELETE CASCADE,
            FOREIGN KEY (parent_id) REFERENCES folders(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            feature_id TEXT NOT NULL,
            claude_session_id TEXT NOT NULL,
            title TEXT,
            summary TEXT,
            project_path TEXT,
            branch TEXT,
            linked_at TEXT NOT NULL,
            FOREIGN KEY (feature_id) REFERENCES features(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            feature_id TEXT NOT NULL,
            title TEXT NOT NULL,
            done INTEGER NOT NULL DEFAULT 0,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            FOREIGN KEY (feature_id) REFERENCES features(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS notes (
            id TEXT PRIMARY KEY,
            feature_id TEXT NOT NULL UNIQUE,
            content TEXT NOT NULL DEFAULT '',
            updated_at TEXT NOT NULL,
            FOREIGN KEY (feature_id) REFERENCES features(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS context (
            id TEXT PRIMARY KEY,
            feature_id TEXT NOT NULL UNIQUE,
            content TEXT NOT NULL DEFAULT '',
            updated_at TEXT NOT NULL,
            FOREIGN KEY (feature_id) REFERENCES features(id) ON DELETE CASCADE
        );
        ",
    )?;

    // Indexes on foreign keys for query performance
    conn.execute_batch(
        "
        CREATE INDEX IF NOT EXISTS idx_links_feature_id ON links(feature_id);
        CREATE INDEX IF NOT EXISTS idx_directories_feature_id ON directories(feature_id);
        CREATE INDEX IF NOT EXISTS idx_files_feature_id ON files(feature_id);
        CREATE INDEX IF NOT EXISTS idx_folders_feature_id ON folders(feature_id);
        CREATE INDEX IF NOT EXISTS idx_folders_parent_id ON folders(parent_id);
        CREATE INDEX IF NOT EXISTS idx_sessions_feature_id ON sessions(feature_id);
        CREATE INDEX IF NOT EXISTS idx_sessions_claude_session_id ON sessions(claude_session_id);
        CREATE INDEX IF NOT EXISTS idx_tasks_feature_id ON tasks(feature_id);
        CREATE INDEX IF NOT EXISTS idx_feature_tags_tag_id ON feature_tags(tag_id);
        ",
    )?;

    // Create FTS5 virtual table for search
    conn.execute_batch(
        "
        CREATE VIRTUAL TABLE IF NOT EXISTS search_index USING fts5(
            entity_type,
            entity_id,
            feature_id,
            title,
            content,
            tokenize='porter unicode61'
        );
        ",
    )?;

    // Migration: add folder_id column to files
    let has_folder_id = conn.prepare("SELECT folder_id FROM files LIMIT 0").is_ok();
    if !has_folder_id {
        conn.execute_batch("ALTER TABLE files ADD COLUMN folder_id TEXT REFERENCES folders(id) ON DELETE SET NULL;")?;
    }

    // Migration: add Jira/external task columns
    let has_task_source = conn.prepare("SELECT source FROM tasks LIMIT 0").is_ok();
    if !has_task_source {
        conn.execute_batch("ALTER TABLE tasks ADD COLUMN source TEXT NOT NULL DEFAULT 'manual';")?;
        conn.execute_batch("ALTER TABLE tasks ADD COLUMN external_key TEXT;")?;
        conn.execute_batch("ALTER TABLE tasks ADD COLUMN external_url TEXT;")?;
        conn.execute_batch("ALTER TABLE tasks ADD COLUMN external_status TEXT;")?;
        conn.execute_batch("ALTER TABLE tasks ADD COLUMN description TEXT;")?;
    }
    conn.execute_batch("CREATE UNIQUE INDEX IF NOT EXISTS idx_tasks_external_key ON tasks(feature_id, external_key) WHERE external_key IS NOT NULL;")?;

    // Migration: add session timing columns
    // SQLite doesn't support IF NOT EXISTS for ALTER TABLE, so we check first.
    let has_started_at: bool = conn
        .prepare("SELECT started_at FROM sessions LIMIT 0")
        .is_ok();
    if !has_started_at {
        conn.execute_batch("ALTER TABLE sessions ADD COLUMN started_at TEXT;")?;
        conn.execute_batch("ALTER TABLE sessions ADD COLUMN ended_at TEXT;")?;
    }

    // Migration: add description column to features
    let has_description = conn.prepare("SELECT description FROM features LIMIT 0").is_ok();
    if !has_description {
        conn.execute_batch("ALTER TABLE features ADD COLUMN description TEXT;")?;
    }

    // Migration: add description column to links
    let has_link_desc = conn.prepare("SELECT description FROM links LIMIT 0").is_ok();
    if !has_link_desc {
        conn.execute_batch("ALTER TABLE links ADD COLUMN description TEXT;")?;
    }

    // Migration: add metadata column to links (JSON blob for extension data)
    let has_link_metadata = conn.prepare("SELECT metadata FROM links LIMIT 0").is_ok();
    if !has_link_metadata {
        conn.execute_batch("ALTER TABLE links ADD COLUMN metadata TEXT;")?;
    }

    // Migration: add pinned column to features
    let has_pinned = conn.prepare("SELECT pinned FROM features LIMIT 0").is_ok();
    if !has_pinned {
        conn.execute_batch("ALTER TABLE features ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0;")?;
    }

    // Migration: add archived column to features
    let has_archived = conn.prepare("SELECT archived FROM features LIMIT 0").is_ok();
    if !has_archived {
        conn.execute_batch("ALTER TABLE features ADD COLUMN archived INTEGER NOT NULL DEFAULT 0;")?;
    }

    // Migration: add repo clone columns to directories
    let has_repo_url = conn.prepare("SELECT repo_url FROM directories LIMIT 0").is_ok();
    if !has_repo_url {
        conn.execute_batch("ALTER TABLE directories ADD COLUMN repo_url TEXT;")?;
        conn.execute_batch("ALTER TABLE directories ADD COLUMN clone_status TEXT DEFAULT 'ready';")?;
        conn.execute_batch("ALTER TABLE directories ADD COLUMN clone_error TEXT;")?;
    }

    // Migration: add title_manual flag to sessions (tracks user renames vs auto-discovered titles)
    let has_title_manual = conn.prepare("SELECT title_manual FROM sessions LIMIT 0").is_ok();
    if !has_title_manual {
        conn.execute_batch("ALTER TABLE sessions ADD COLUMN title_manual INTEGER NOT NULL DEFAULT 0;")?;
    }

    // Feature branches
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS feature_branches (
            id TEXT PRIMARY KEY,
            feature_id TEXT NOT NULL,
            directory_id TEXT NOT NULL,
            branch_name TEXT NOT NULL,
            created_at TEXT NOT NULL,
            UNIQUE(feature_id, directory_id, branch_name),
            FOREIGN KEY (feature_id) REFERENCES features(id) ON DELETE CASCADE,
            FOREIGN KEY (directory_id) REFERENCES directories(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_feature_branches_feature_id ON feature_branches(feature_id);
        ",
    )?;

    // Plans for Claude Code implementation plan submissions
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS plans (
            id TEXT PRIMARY KEY,
            feature_id TEXT NOT NULL,
            session_id TEXT,
            title TEXT NOT NULL,
            body TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            feedback TEXT,
            created_at TEXT NOT NULL,
            resolved_at TEXT,
            FOREIGN KEY (feature_id) REFERENCES features(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_plans_feature_id ON plans(feature_id);
        CREATE INDEX IF NOT EXISTS idx_plans_status ON plans(status, feature_id);
        ",
    )?;

    // Feature-level MCP server overrides
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS feature_mcp_servers (
            feature_id TEXT NOT NULL,
            server_name TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            PRIMARY KEY (feature_id, server_name),
            FOREIGN KEY (feature_id) REFERENCES features(id) ON DELETE CASCADE
        );
        ",
    )?;

    // Feature-level skill overrides
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS feature_skills (
            feature_id TEXT NOT NULL,
            skill_id TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            PRIMARY KEY (feature_id, skill_id),
            FOREIGN KEY (feature_id) REFERENCES features(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_feature_skills_skill_id ON feature_skills(skill_id);
        ",
    )?;

    // Migration: add parent_id column to features for hierarchy
    let has_parent_id = conn.prepare("SELECT parent_id FROM features LIMIT 0").is_ok();
    if !has_parent_id {
        conn.execute_batch("ALTER TABLE features ADD COLUMN parent_id TEXT;")?;
    }
    conn.execute_batch("CREATE INDEX IF NOT EXISTS idx_features_parent_id ON features(parent_id);")?;

    // Feature groups for organizing features in the sidebar
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS feature_groups (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            color TEXT,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        );
        ",
    )?;

    // Migration: add group_id column to features
    let has_group_id = conn.prepare("SELECT group_id FROM features LIMIT 0").is_ok();
    if !has_group_id {
        conn.execute_batch("ALTER TABLE features ADD COLUMN group_id TEXT REFERENCES feature_groups(id) ON DELETE SET NULL;")?;
    }
    conn.execute_batch("CREATE INDEX IF NOT EXISTS idx_features_group_id ON features(group_id);")?;

    // Knowledge base — storage-scoped folders and entries
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS knowledge_folders (
            id TEXT PRIMARY KEY,
            parent_id TEXT,
            name TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            FOREIGN KEY (parent_id) REFERENCES knowledge_folders(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS knowledge_entries (
            id TEXT PRIMARY KEY,
            folder_id TEXT,
            feature_id TEXT,
            title TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            content TEXT NOT NULL DEFAULT '',
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (folder_id) REFERENCES knowledge_folders(id) ON DELETE SET NULL,
            FOREIGN KEY (feature_id) REFERENCES features(id) ON DELETE SET NULL
        );

        CREATE INDEX IF NOT EXISTS idx_knowledge_folders_parent_id ON knowledge_folders(parent_id);
        CREATE INDEX IF NOT EXISTS idx_knowledge_entries_folder_id ON knowledge_entries(folder_id);
        CREATE INDEX IF NOT EXISTS idx_knowledge_entries_feature_id ON knowledge_entries(feature_id);
        ",
    )?;

    Ok(())
}

/// Migrate existing absolute paths to storage-relative paths.
/// Safe to call multiple times — skips paths that are already relative.
pub fn migrate_to_relative_paths(conn: &Connection, storage_base: &Path) {
    use crate::paths::find_workspaces_relative;

    // Helper: convert absolute path to relative if it contains workspaces/
    let to_relative = |abs_path: &str| -> Option<String> {
        let p = Path::new(abs_path);
        if !p.is_absolute() {
            return None; // Already relative
        }
        // Try strip_prefix first (exact match with current storage)
        if let Ok(rel) = p.strip_prefix(storage_base) {
            return Some(rel.to_string_lossy().replace('\\', "/"));
        }
        // Fallback: extract workspaces/... from any absolute path
        find_workspaces_relative(p)
            .map(|rel| rel.to_string_lossy().replace('\\', "/"))
    };

    // Migrate files.stored_path
    if let Ok(mut stmt) = conn.prepare("SELECT id, stored_path FROM files") {
        let rows: Vec<(String, String)> = match stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        {
            Ok(mapped) => mapped.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        };
        for (id, old_path) in rows {
            if let Some(rel) = to_relative(&old_path) {
                let _ = conn.execute(
                    "UPDATE files SET stored_path = ?1 WHERE id = ?2",
                    rusqlite::params![rel, id],
                );
            }
        }
    }

    // Migrate directories.path — only cloned repos (repo_url IS NOT NULL)
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, path FROM directories WHERE repo_url IS NOT NULL AND repo_url != ''"
    ) {
        let rows: Vec<(String, String)> = match stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        {
            Ok(mapped) => mapped.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        };
        for (id, old_path) in rows {
            if let Some(rel) = to_relative(&old_path) {
                let _ = conn.execute(
                    "UPDATE directories SET path = ?1 WHERE id = ?2",
                    rusqlite::params![rel, id],
                );
            }
        }
    }

    // Migrate sessions.project_path
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, project_path FROM sessions WHERE project_path IS NOT NULL"
    ) {
        let rows: Vec<(String, String)> = match stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        {
            Ok(mapped) => mapped.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        };
        for (id, old_path) in rows {
            if let Some(rel) = to_relative(&old_path) {
                let _ = conn.execute(
                    "UPDATE sessions SET project_path = ?1 WHERE id = ?2",
                    rusqlite::params![rel, id],
                );
            }
        }
    }
}
