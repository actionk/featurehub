# Knowledge Base Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a storage-scoped, hierarchical knowledge base that Claude sessions can discover via a TOC in instructions and read/write via MCP tools.

**Architecture:** Two new DB tables (`knowledge_folders`, `knowledge_entries`) with FTS5 indexing. New `db/knowledge.rs` and `commands/knowledge.rs` Rust modules. Nine new MCP tools on `fh-mcp`. TOC injected into `build_feature_context()`. Frontend: sidebar footer button + full-width `KnowledgePanel` replacing the feature detail area.

**Tech Stack:** Rust (rusqlite, serde, uuid, chrono), rmcp MCP macros, Svelte 5, TypeScript, Tauri 2 IPC

**Spec:** `docs/superpowers/specs/2026-04-02-knowledge-base-design.md`

---

### Task 1: DB schema + knowledge.rs module

**Files:**
- Modify: `src-tauri/src/db/mod.rs` (add `pub mod knowledge;` + CREATE TABLE statements)
- Create: `src-tauri/src/db/knowledge.rs`

- [ ] **Step 1: Add module declaration to db/mod.rs**

Add `pub mod knowledge;` after the existing module declarations in `src-tauri/src/db/mod.rs`:

```rust
// In src-tauri/src/db/mod.rs, after line 15 (pub mod tasks;)
pub mod knowledge;
```

- [ ] **Step 2: Add CREATE TABLE statements to initialize()**

In `src-tauri/src/db/mod.rs`, add the following inside `initialize()`, just before the final `Ok(())` (after the `group_id` index at line 324):

```rust
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
```

- [ ] **Step 3: Create db/knowledge.rs with structs and full CRUD**

Create `src-tauri/src/db/knowledge.rs`:

```rust
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

pub fn rename_folder(conn: &Connection, id: &str, new_name: &str) -> Result<KnowledgeFolder, String> {
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

const ENTRY_COLUMNS: &str = "id, folder_id, feature_id, title, description, content, sort_order, created_at, updated_at";

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

pub fn list_entries_in_folder(conn: &Connection, folder_id: Option<&str>) -> Result<Vec<KnowledgeEntry>, String> {
    let mut stmt = if folder_id.is_some() {
        conn.prepare(&format!(
            "SELECT {} FROM knowledge_entries WHERE folder_id = ?1 ORDER BY sort_order ASC, title ASC",
            ENTRY_COLUMNS
        ))
    } else {
        conn.prepare(&format!(
            "SELECT {} FROM knowledge_entries WHERE folder_id IS NULL ORDER BY sort_order ASC, title ASC",
            ENTRY_COLUMNS
        ))
    }
    .map_err(|e| e.to_string())?;

    let entries = if let Some(fid) = folder_id {
        stmt.query_map(params![fid], |row| read_entry(row))
    } else {
        stmt.query_map([], |row| read_entry(row))
    }
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(entries)
}

pub fn get_entry(conn: &Connection, id: &str) -> Result<KnowledgeEntry, String> {
    conn.query_row(
        &format!("SELECT {} FROM knowledge_entries WHERE id = ?1", ENTRY_COLUMNS),
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
        params![new_title, new_content, new_desc, new_folder_id, new_feature_id, now, id],
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
```

- [ ] **Step 4: Verify compilation**

Run from `src-tauri/`:
```bash
cargo check
```

Expected: compiles with errors only about missing `search::index_knowledge_entry` and `search::delete_knowledge_entry` — those are added in Task 2.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/db/mod.rs src-tauri/src/db/knowledge.rs
git commit -m "feat: add knowledge_entries and knowledge_folders DB tables + CRUD module"
```

---

### Task 2: FTS5 search integration

**Files:**
- Modify: `src-tauri/src/db/search.rs`

- [ ] **Step 1: Add index_knowledge_entry function**

Append to `src-tauri/src/db/search.rs`, before the `rebuild_search_index` function:

```rust
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
```

- [ ] **Step 2: Add knowledge entries to rebuild_search_index**

In the `rebuild_search_index` function in `src-tauri/src/db/search.rs`, add a new block after the files re-indexing (before the final `Ok(())`):

```rust
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
```

- [ ] **Step 3: Verify compilation**

Run from `src-tauri/`:
```bash
cargo check
```

Expected: clean compilation, no errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/db/search.rs
git commit -m "feat: add FTS5 indexing for knowledge base entries"
```

---

### Task 3: Tauri IPC commands

**Files:**
- Create: `src-tauri/src/commands/knowledge.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create commands/knowledge.rs**

Create `src-tauri/src/commands/knowledge.rs`:

```rust
use tauri::State;
use crate::db;
use crate::AppState;

// ─── Folder commands ────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_knowledge_folders(
    state: State<'_, AppState>,
) -> Result<Vec<db::knowledge::KnowledgeFolder>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::knowledge::list_folders(&conn)
}

#[tauri::command]
pub fn create_knowledge_folder(
    state: State<'_, AppState>,
    name: String,
    parent_id: Option<String>,
) -> Result<db::knowledge::KnowledgeFolder, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::knowledge::create_folder(&conn, &name, parent_id.as_deref())
}

#[tauri::command]
pub fn rename_knowledge_folder(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> Result<db::knowledge::KnowledgeFolder, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::knowledge::rename_folder(&conn, &id, &name)
}

#[tauri::command]
pub fn delete_knowledge_folder(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::knowledge::delete_folder(&conn, &id)
}

// ─── Entry commands ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_knowledge_entries(
    state: State<'_, AppState>,
    folder_id: Option<String>,
) -> Result<Vec<db::knowledge::KnowledgeEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::knowledge::list_entries_in_folder(&conn, folder_id.as_deref())
}

#[tauri::command]
pub fn get_all_knowledge_entries(
    state: State<'_, AppState>,
) -> Result<Vec<db::knowledge::KnowledgeEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::knowledge::list_entries(&conn)
}

#[tauri::command]
pub fn get_knowledge_entry(
    state: State<'_, AppState>,
    id: String,
) -> Result<db::knowledge::KnowledgeEntry, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::knowledge::get_entry(&conn, &id)
}

#[tauri::command]
pub fn create_knowledge_entry(
    state: State<'_, AppState>,
    title: String,
    content: String,
    description: Option<String>,
    folder_id: Option<String>,
    feature_id: Option<String>,
) -> Result<db::knowledge::KnowledgeEntry, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::knowledge::create_entry(
        &conn,
        &title,
        &content,
        description.as_deref(),
        folder_id.as_deref(),
        feature_id.as_deref(),
    )
}

#[tauri::command]
pub fn update_knowledge_entry(
    state: State<'_, AppState>,
    id: String,
    title: Option<String>,
    content: Option<String>,
    description: Option<String>,
    folder_id: Option<Option<String>>,
    feature_id: Option<Option<String>>,
) -> Result<db::knowledge::KnowledgeEntry, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::knowledge::update_entry(
        &conn,
        &id,
        title.as_deref(),
        content.as_deref(),
        description.as_deref(),
        folder_id.as_ref().map(|o| o.as_deref()),
        feature_id.as_ref().map(|o| o.as_deref()),
    )
}

#[tauri::command]
pub fn delete_knowledge_entry(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::knowledge::delete_entry(&conn, &id)
}
```

- [ ] **Step 2: Register module in commands/mod.rs**

In `src-tauri/src/commands/mod.rs`, add after the existing mod declarations (after `mod timeline;`):

```rust
mod knowledge;
```

And add the re-export after the existing `pub use` lines (after `pub use timeline::*;`):

```rust
pub use knowledge::*;
```

- [ ] **Step 3: Register commands in lib.rs invoke_handler**

In `src-tauri/src/lib.rs`, add the following entries inside the `tauri::generate_handler![]` macro, after the `commands::restore_repo_from_export` line:

```rust
            commands::get_knowledge_folders,
            commands::create_knowledge_folder,
            commands::rename_knowledge_folder,
            commands::delete_knowledge_folder,
            commands::get_knowledge_entries,
            commands::get_all_knowledge_entries,
            commands::get_knowledge_entry,
            commands::create_knowledge_entry,
            commands::update_knowledge_entry,
            commands::delete_knowledge_entry,
```

- [ ] **Step 4: Verify compilation**

Run from `src-tauri/`:
```bash
cargo check
```

Expected: clean compilation.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/knowledge.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add Tauri IPC commands for knowledge base CRUD"
```

---

### Task 4: MCP server — knowledge tools

**Files:**
- Modify: `src-tauri/src/bin/fh_mcp.rs`

- [ ] **Step 1: Add parameter structs**

In `src-tauri/src/bin/fh_mcp.rs`, add the following parameter structs after the existing param structs (before the `// ─── MCP Server` section around line 370):

```rust
// ─── Knowledge Base params ──────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ListKnowledgeParams {
    /// Optional folder UUID to filter entries
    #[serde(default)]
    folder_id: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct KnowledgeEntryIdParam {
    /// The knowledge entry UUID
    id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct CreateKnowledgeEntryParams {
    /// Entry title
    title: String,
    /// Full markdown content
    content: String,
    /// One-line summary (shown in TOC)
    #[serde(default)]
    description: Option<String>,
    /// Folder UUID to place entry in
    #[serde(default)]
    folder_id: Option<String>,
    /// Feature UUID to link entry to
    #[serde(default)]
    feature_id: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct UpdateKnowledgeEntryParams {
    /// The knowledge entry UUID
    id: String,
    /// New title (optional)
    #[serde(default)]
    title: Option<String>,
    /// New content (optional)
    #[serde(default)]
    content: Option<String>,
    /// New description (optional)
    #[serde(default)]
    description: Option<String>,
    /// Move to folder (optional, null for root)
    #[serde(default)]
    folder_id: Option<Option<String>>,
    /// Link to feature (optional, null to unlink)
    #[serde(default)]
    feature_id: Option<Option<String>>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct CreateKnowledgeFolderParams {
    /// Folder name
    name: String,
    /// Parent folder UUID (null for root)
    #[serde(default)]
    parent_id: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct RenameKnowledgeFolderParams {
    /// The folder UUID
    id: String,
    /// New folder name
    name: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct DeleteKnowledgeFolderParams {
    /// The folder UUID
    id: String,
}
```

- [ ] **Step 2: Add tool handlers inside the #[tool_router] impl block**

In the `#[tool_router] impl FeatureHubMcp` block, add the following tools. Place them after the last existing tool (before the closing `}` of the impl block, around line 1119):

```rust
    // ── Knowledge Base tools ────────────────────────────────────────────

    #[tool(description = "List knowledge base entries. Optionally filter by folder_id. Returns titles, descriptions, and IDs (not full content). Use get_knowledge_entry to read full content.")]
    async fn list_knowledge(
        &self,
        Parameters(p): Parameters<ListKnowledgeParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let entries = if let Some(ref fid) = p.folder_id {
                db::knowledge::list_entries_in_folder(&conn, Some(fid)).map_err(db_err)?
            } else {
                db::knowledge::list_entries(&conn).map_err(db_err)?
            };
            let folders = db::knowledge::list_folders(&conn).map_err(db_err)?;
            let result = serde_json::json!({ "entries": entries, "folders": folders });
            json_result(&result)
        })
    }

    #[tool(description = "Get full content of a knowledge base entry by ID")]
    async fn get_knowledge_entry(
        &self,
        Parameters(p): Parameters<KnowledgeEntryIdParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let entry = db::knowledge::get_entry(&conn, &p.id).map_err(db_err)?;
            json_result(&entry)
        })
    }

    #[tool(description = "Search knowledge base entries using full-text search")]
    async fn search_knowledge(
        &self,
        Parameters(p): Parameters<SearchParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let results = db::search::global_search(&conn, &p.query).map_err(db_err)?;
            let kb_results: Vec<_> = results.into_iter().filter(|r| r.entity_type == "knowledge").collect();
            json_result(&kb_results)
        })
    }

    #[tool(description = "Create a new knowledge base entry. Use this to save HOW-TOs, findings, research results, and other reusable knowledge.")]
    async fn create_knowledge_entry(
        &self,
        Parameters(p): Parameters<CreateKnowledgeEntryParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let entry = db::knowledge::create_entry(
                &conn,
                &p.title,
                &p.content,
                p.description.as_deref(),
                p.folder_id.as_deref(),
                p.feature_id.as_deref(),
            ).map_err(db_err)?;
            let _ = config::push_notification(&format!("Knowledge entry created: {}", entry.title), None);
            json_result(&entry)
        })
    }

    #[tool(description = "Update an existing knowledge base entry")]
    async fn update_knowledge_entry(
        &self,
        Parameters(p): Parameters<UpdateKnowledgeEntryParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let entry = db::knowledge::update_entry(
                &conn,
                &p.id,
                p.title.as_deref(),
                p.content.as_deref(),
                p.description.as_deref(),
                p.folder_id.as_ref().map(|o| o.as_deref()),
                p.feature_id.as_ref().map(|o| o.as_deref()),
            ).map_err(db_err)?;
            let _ = config::push_notification(&format!("Knowledge entry updated: {}", entry.title), None);
            json_result(&entry)
        })
    }

    #[tool(description = "Delete a knowledge base entry")]
    async fn delete_knowledge_entry(
        &self,
        Parameters(p): Parameters<KnowledgeEntryIdParam>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            db::knowledge::delete_entry(&conn, &p.id).map_err(db_err)?;
            Ok(CallToolResult::success(vec![Content::text("Knowledge entry deleted")]))
        })
    }

    #[tool(description = "Create a knowledge base folder for organizing entries")]
    async fn create_knowledge_folder(
        &self,
        Parameters(p): Parameters<CreateKnowledgeFolderParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let folder = db::knowledge::create_folder(&conn, &p.name, p.parent_id.as_deref()).map_err(db_err)?;
            let _ = config::push_notification(&format!("Knowledge folder created: {}", folder.name), None);
            json_result(&folder)
        })
    }

    #[tool(description = "Rename a knowledge base folder")]
    async fn rename_knowledge_folder(
        &self,
        Parameters(p): Parameters<RenameKnowledgeFolderParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            let folder = db::knowledge::rename_folder(&conn, &p.id, &p.name).map_err(db_err)?;
            json_result(&folder)
        })
    }

    #[tool(description = "Delete a knowledge base folder. Entries inside are moved to the parent folder.")]
    async fn delete_knowledge_folder(
        &self,
        Parameters(p): Parameters<DeleteKnowledgeFolderParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        with_db!(self, conn => {
            db::knowledge::delete_folder(&conn, &p.id).map_err(db_err)?;
            Ok(CallToolResult::success(vec![Content::text("Knowledge folder deleted")]))
        })
    }
```

- [ ] **Step 3: Verify compilation**

Run from `src-tauri/`:
```bash
cargo check
```

Expected: clean compilation.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/bin/fh_mcp.rs
git commit -m "feat: add 9 MCP tools for knowledge base read/write"
```

---

### Task 5: TOC injection into build_feature_context()

**Files:**
- Modify: `src-tauri/src/bin/fh_mcp.rs`

- [ ] **Step 1: Add knowledge TOC to build_feature_context()**

In `src-tauri/src/bin/fh_mcp.rs`, inside `build_feature_context()`, add the following block just before the extension instructions section (before the line `let storage_settings = config::load_storage_settings(storage_path).unwrap_or_default();`, around line 1394):

```rust
    // Knowledge Base TOC — storage-scoped, always included if entries exist
    let kb_entries = db::knowledge::list_entries(conn).unwrap_or_default();
    if !kb_entries.is_empty() {
        let kb_folders = db::knowledge::list_folders(conn).unwrap_or_default();
        ctx.push_str(&format!("\n## Knowledge Base ({} entries)\n", kb_entries.len()));
        ctx.push_str("Use get_knowledge_entry(id) to read full content.\n\n");

        for entry in &kb_entries {
            let folder_prefix = if let Some(ref fid) = entry.folder_id {
                if let Ok(path) = db::knowledge::get_folder_path(conn, fid) {
                    format!("[{}/] ", path)
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            if entry.description.is_empty() {
                ctx.push_str(&format!("- {}{} (id: {})\n", folder_prefix, entry.title, entry.id));
            } else {
                ctx.push_str(&format!("- {}{} — {} (id: {})\n", folder_prefix, entry.title, entry.description, entry.id));
            }
        }
    }
```

Note: the `kb_folders` variable is loaded for future use (e.g., listing empty folders) but currently only entries are listed. This is intentional — empty folders don't need TOC presence.

- [ ] **Step 2: Also inject TOC into unscoped server instructions**

In the `get_info()` method (inside the `#[tool_handler] impl ServerHandler for FeatureHubMcp` block), modify the `None` branch to also include KB TOC for unscoped sessions. Replace the existing `None` branch:

Find this block:
```rust
        let instructions = match &self.feature_context {
            Some(ctx) => ctx.clone(),
            None => "FeatureHub MCP server. Use list_features to discover features, \
```

Replace the `None` arm with:

```rust
            None => {
                let mut text = "FeatureHub MCP server. Use list_features to discover features, \
                    then pass feature_id to other tools.\n\n\
                    When asked to \"initialize\" a feature with inputs (links, files, descriptions), \
                    set description via update_feature, add all URLs via add_link (including child stories/tickets as individual links), \
                    save context via save_context, and set status via update_feature.\n\n\
                    Important: Tasks and Notes are user-owned. Do NOT create tasks or notes unless the user explicitly asks."
                    .to_string();

                // Append KB TOC for unscoped sessions too
                if let Ok(conn) = self.db.lock() {
                    let kb_entries = db::knowledge::list_entries(&conn).unwrap_or_default();
                    if !kb_entries.is_empty() {
                        text.push_str(&format!("\n\n## Knowledge Base ({} entries)\n", kb_entries.len()));
                        text.push_str("Use get_knowledge_entry(id) to read full content.\n\n");
                        for entry in &kb_entries {
                            let folder_prefix = if let Some(ref fid) = entry.folder_id {
                                db::knowledge::get_folder_path(&conn, fid)
                                    .map(|p| format!("[{}/] ", p))
                                    .unwrap_or_default()
                            } else {
                                String::new()
                            };
                            if entry.description.is_empty() {
                                text.push_str(&format!("- {}{} (id: {})\n", folder_prefix, entry.title, entry.id));
                            } else {
                                text.push_str(&format!("- {}{} — {} (id: {})\n", folder_prefix, entry.title, entry.description, entry.id));
                            }
                        }
                    }
                }

                text
            },
```

- [ ] **Step 3: Verify compilation**

Run from `src-tauri/`:
```bash
cargo check
```

Expected: clean compilation.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/bin/fh_mcp.rs
git commit -m "feat: inject knowledge base TOC into MCP session instructions"
```

---

### Task 6: TypeScript types + API wrappers

**Files:**
- Modify: `src/lib/api/types.ts`
- Create: `src/lib/api/knowledge.ts`

- [ ] **Step 1: Add TypeScript types**

In `src/lib/api/types.ts`, add at the end of the file:

```typescript
export interface KnowledgeFolder {
  id: string;
  parent_id: string | null;
  name: string;
  sort_order: number;
  created_at: string;
}

export interface KnowledgeEntry {
  id: string;
  folder_id: string | null;
  feature_id: string | null;
  title: string;
  description: string;
  content: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}
```

- [ ] **Step 2: Create API wrapper file**

Create `src/lib/api/knowledge.ts`:

```typescript
import { invoke } from "@tauri-apps/api/core";
import type { KnowledgeEntry, KnowledgeFolder } from "./types";

// ─── Folders ────────────────────────────────────────────────────────────────

export async function getKnowledgeFolders(): Promise<KnowledgeFolder[]> {
  return invoke<KnowledgeFolder[]>("get_knowledge_folders");
}

export async function createKnowledgeFolder(
  name: string,
  parentId?: string | null,
): Promise<KnowledgeFolder> {
  return invoke<KnowledgeFolder>("create_knowledge_folder", {
    name,
    parentId: parentId ?? null,
  });
}

export async function renameKnowledgeFolder(
  id: string,
  name: string,
): Promise<KnowledgeFolder> {
  return invoke<KnowledgeFolder>("rename_knowledge_folder", { id, name });
}

export async function deleteKnowledgeFolder(id: string): Promise<void> {
  return invoke<void>("delete_knowledge_folder", { id });
}

// ─── Entries ────────────────────────────────────────────────────────────────

export async function getKnowledgeEntries(
  folderId?: string | null,
): Promise<KnowledgeEntry[]> {
  return invoke<KnowledgeEntry[]>("get_knowledge_entries", {
    folderId: folderId ?? null,
  });
}

export async function getAllKnowledgeEntries(): Promise<KnowledgeEntry[]> {
  return invoke<KnowledgeEntry[]>("get_all_knowledge_entries");
}

export async function getKnowledgeEntry(id: string): Promise<KnowledgeEntry> {
  return invoke<KnowledgeEntry>("get_knowledge_entry", { id });
}

export async function createKnowledgeEntry(params: {
  title: string;
  content: string;
  description?: string;
  folderId?: string | null;
  featureId?: string | null;
}): Promise<KnowledgeEntry> {
  return invoke<KnowledgeEntry>("create_knowledge_entry", {
    title: params.title,
    content: params.content,
    description: params.description ?? null,
    folderId: params.folderId ?? null,
    featureId: params.featureId ?? null,
  });
}

export async function updateKnowledgeEntry(params: {
  id: string;
  title?: string;
  content?: string;
  description?: string;
  folderId?: string | null;
  featureId?: string | null;
}): Promise<KnowledgeEntry> {
  return invoke<KnowledgeEntry>("update_knowledge_entry", {
    id: params.id,
    title: params.title ?? null,
    content: params.content ?? null,
    description: params.description ?? null,
    folderId: params.folderId !== undefined ? params.folderId : null,
    featureId: params.featureId !== undefined ? params.featureId : null,
  });
}

export async function deleteKnowledgeEntry(id: string): Promise<void> {
  return invoke<void>("delete_knowledge_entry", { id });
}
```

- [ ] **Step 3: Add re-export to barrel file**

Check if `src/lib/api/tauri.ts` is a barrel re-export. If it is, add:

```typescript
export * from "./knowledge";
```

If not, the individual import from `./knowledge` is fine.

- [ ] **Step 4: Commit**

```bash
git add src/lib/api/types.ts src/lib/api/knowledge.ts
git commit -m "feat: add TypeScript types and API wrappers for knowledge base"
```

---

### Task 7: Frontend — KnowledgePanel component

**Files:**
- Create: `src/lib/modules/knowledge/KnowledgePanel.svelte`
- Create: `src/lib/modules/knowledge/KnowledgeFolderTree.svelte`
- Create: `src/lib/modules/knowledge/KnowledgeEntryEditor.svelte`

- [ ] **Step 1: Create KnowledgeFolderTree.svelte**

Create `src/lib/modules/knowledge/KnowledgeFolderTree.svelte`:

```svelte
<script lang="ts">
  import type { KnowledgeFolder, KnowledgeEntry } from "../../api/types";

  let {
    folders,
    entries,
    selectedEntryId,
    selectedFolderId,
    onSelectEntry,
    onSelectFolder,
    onCreateEntry,
    onCreateFolder,
    onDeleteFolder,
    onRenameFolder,
  }: {
    folders: KnowledgeFolder[];
    entries: KnowledgeEntry[];
    selectedEntryId: string | null;
    selectedFolderId: string | null;
    onSelectEntry: (id: string) => void;
    onSelectFolder: (id: string | null) => void;
    onCreateEntry: (folderId: string | null) => void;
    onCreateFolder: (parentId: string | null) => void;
    onDeleteFolder: (id: string) => void;
    onRenameFolder: (id: string, name: string) => void;
  } = $props();

  let expandedFolders = $state<Set<string>>(new Set());
  let renamingFolderId = $state<string | null>(null);
  let renameValue = $state("");

  function toggleExpand(id: string) {
    const next = new Set(expandedFolders);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    expandedFolders = next;
  }

  function childFolders(parentId: string | null): KnowledgeFolder[] {
    return folders.filter(f => f.parent_id === parentId).sort((a, b) => a.sort_order - b.sort_order || a.name.localeCompare(b.name));
  }

  function entriesInFolder(folderId: string | null): KnowledgeEntry[] {
    return entries.filter(e => e.folder_id === folderId).sort((a, b) => a.sort_order - b.sort_order || a.title.localeCompare(b.title));
  }

  function startRename(id: string, currentName: string) {
    renamingFolderId = id;
    renameValue = currentName;
  }

  function commitRename() {
    if (renamingFolderId && renameValue.trim()) {
      onRenameFolder(renamingFolderId, renameValue.trim());
    }
    renamingFolderId = null;
  }
</script>

<div class="kb-tree">
  <div class="kb-tree-header">
    <span class="kb-tree-title">Knowledge Base</span>
    <div class="kb-tree-actions">
      <button class="btn-ghost btn-sm" onclick={() => onCreateEntry(null)} title="New entry">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M8 2a.75.75 0 01.75.75v4.5h4.5a.75.75 0 010 1.5h-4.5v4.5a.75.75 0 01-1.5 0v-4.5h-4.5a.75.75 0 010-1.5h4.5v-4.5A.75.75 0 018 2z"/></svg>
      </button>
      <button class="btn-ghost btn-sm" onclick={() => onCreateFolder(null)} title="New folder">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
          <path d="M1 3.5A1.5 1.5 0 012.5 2h3.879a1.5 1.5 0 011.06.44l1.122 1.12A.5.5 0 008.914 4H13.5A1.5 1.5 0 0115 5.5v7a1.5 1.5 0 01-1.5 1.5h-11A1.5 1.5 0 011 12.5v-9z"/>
        </svg>
      </button>
    </div>
  </div>

  <div class="kb-tree-list">
    {#snippet folderNode(parentId: string | null, depth: number)}
      {#each childFolders(parentId) as folder (folder.id)}
        {@const isExpanded = expandedFolders.has(folder.id)}
        {@const isSelected = selectedFolderId === folder.id}
        <div class="kb-tree-item kb-tree-folder" class:selected={isSelected} style="padding-left: {8 + depth * 16}px">
          <button class="kb-tree-expand" onclick={() => toggleExpand(folder.id)}>
            <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor" style:transform={isExpanded ? "rotate(90deg)" : ""}>
              <path d="M6 4l4 4-4 4"/>
            </svg>
          </button>
          {#if renamingFolderId === folder.id}
            <input
              class="kb-rename-input"
              bind:value={renameValue}
              onblur={commitRename}
              onkeydown={(e) => { if (e.key === 'Enter') commitRename(); if (e.key === 'Escape') { renamingFolderId = null; } }}
              autofocus
            />
          {:else}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <span class="kb-tree-label" onclick={() => onSelectFolder(folder.id)} ondblclick={() => startRename(folder.id, folder.name)}>
              {folder.name}
            </span>
          {/if}
          <button class="btn-ghost btn-xs kb-tree-action" onclick={() => onCreateEntry(folder.id)} title="New entry here">+</button>
          <button class="btn-ghost btn-xs kb-tree-action" onclick={() => onDeleteFolder(folder.id)} title="Delete folder">&times;</button>
        </div>
        {#if isExpanded}
          {#each entriesInFolder(folder.id) as entry (entry.id)}
            <div class="kb-tree-item kb-tree-entry" class:selected={selectedEntryId === entry.id} style="padding-left: {24 + depth * 16}px" onclick={() => onSelectEntry(entry.id)}>
              <span class="kb-entry-icon">📄</span>
              <span class="kb-tree-label">{entry.title || "Untitled"}</span>
            </div>
          {/each}
          {@render folderNode(folder.id, depth + 1)}
        {/if}
      {/each}
    {/snippet}

    <!-- Root entries (no folder) -->
    {#each entriesInFolder(null) as entry (entry.id)}
      <div class="kb-tree-item kb-tree-entry" class:selected={selectedEntryId === entry.id} style="padding-left: 8px" onclick={() => onSelectEntry(entry.id)}>
        <span class="kb-entry-icon">📄</span>
        <span class="kb-tree-label">{entry.title || "Untitled"}</span>
      </div>
    {/each}

    <!-- Folders at root level -->
    {@render folderNode(null, 0)}
  </div>
</div>

<style>
  .kb-tree {
    display: flex;
    flex-direction: column;
    height: 100%;
    border-right: 1px solid var(--border);
    min-width: 220px;
    max-width: 320px;
    overflow-y: auto;
  }
  .kb-tree-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 8px 4px;
    border-bottom: 1px solid var(--border);
  }
  .kb-tree-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .kb-tree-actions {
    display: flex;
    gap: 2px;
  }
  .kb-tree-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }
  .kb-tree-item {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 3px 8px;
    cursor: pointer;
    font-size: 13px;
    color: var(--text-primary);
    border-radius: 4px;
    margin: 0 4px;
  }
  .kb-tree-item:hover {
    background: var(--bg-hover);
  }
  .kb-tree-item.selected {
    background: var(--bg-selected);
  }
  .kb-tree-expand {
    background: none;
    border: none;
    padding: 2px;
    cursor: pointer;
    color: var(--text-muted);
    display: flex;
    align-items: center;
  }
  .kb-tree-label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .kb-entry-icon {
    font-size: 11px;
    flex-shrink: 0;
  }
  .kb-tree-action {
    opacity: 0;
    font-size: 14px;
    padding: 0 4px;
  }
  .kb-tree-item:hover .kb-tree-action {
    opacity: 0.6;
  }
  .kb-tree-action:hover {
    opacity: 1 !important;
  }
  .kb-rename-input {
    flex: 1;
    font-size: 13px;
    padding: 1px 4px;
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: 3px;
    color: var(--text-primary);
    outline: none;
  }
</style>
```

- [ ] **Step 2: Create KnowledgeEntryEditor.svelte**

Create `src/lib/modules/knowledge/KnowledgeEntryEditor.svelte`:

```svelte
<script lang="ts">
  import type { KnowledgeEntry, Feature } from "../../api/types";
  import { updateKnowledgeEntry, deleteKnowledgeEntry } from "../../api/knowledge";
  import MarkdownPreview from "../../components/MarkdownPreview.svelte";

  let {
    entry,
    features = [],
    onSaved,
    onDeleted,
  }: {
    entry: KnowledgeEntry | null;
    features?: Feature[];
    onSaved: () => void;
    onDeleted: () => void;
  } = $props();

  let editTitle = $state("");
  let editDescription = $state("");
  let editContent = $state("");
  let previewMode = $state(false);
  let saveTimer: ReturnType<typeof setTimeout> | null = null;

  // Sync local state when entry changes
  $effect(() => {
    if (entry) {
      editTitle = entry.title;
      editDescription = entry.description;
      editContent = entry.content;
      previewMode = false;
    }
  });

  function scheduleSave() {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(doSave, 800);
  }

  async function doSave() {
    if (!entry) return;
    try {
      await updateKnowledgeEntry({
        id: entry.id,
        title: editTitle,
        description: editDescription,
        content: editContent,
      });
      onSaved();
    } catch (e) {
      console.error("Failed to save knowledge entry:", e);
    }
  }

  async function handleDelete() {
    if (!entry) return;
    if (!confirm(`Delete "${entry.title}"?`)) return;
    try {
      await deleteKnowledgeEntry(entry.id);
      onDeleted();
    } catch (e) {
      console.error("Failed to delete knowledge entry:", e);
    }
  }
</script>

{#if entry}
  <div class="kb-editor">
    <div class="kb-editor-header">
      <input
        class="kb-editor-title"
        bind:value={editTitle}
        oninput={scheduleSave}
        placeholder="Entry title"
      />
      <div class="kb-editor-toolbar">
        <button class="btn-ghost btn-sm" class:active={previewMode} onclick={() => previewMode = !previewMode}>
          {previewMode ? "Edit" : "Preview"}
        </button>
        <button class="btn-ghost btn-sm btn-danger" onclick={handleDelete} title="Delete entry">
          <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M5.5 5.5A.5.5 0 016 6v6a.5.5 0 01-1 0V6a.5.5 0 01.5-.5zm2.5 0a.5.5 0 01.5.5v6a.5.5 0 01-1 0V6a.5.5 0 01.5-.5zm3 .5a.5.5 0 00-1 0v6a.5.5 0 001 0V6z"/><path fill-rule="evenodd" d="M14.5 3a1 1 0 01-1 1H13v9a2 2 0 01-2 2H5a2 2 0 01-2-2V4h-.5a1 1 0 01-1-1V2a1 1 0 011-1H6a1 1 0 011-1h2a1 1 0 011 1h3.5a1 1 0 011 1v1zM4.118 4L4 4.059V13a1 1 0 001 1h6a1 1 0 001-1V4.059L11.882 4H4.118zM2.5 3V2h11v1h-11z"/></svg>
        </button>
      </div>
    </div>
    <input
      class="kb-editor-description"
      bind:value={editDescription}
      oninput={scheduleSave}
      placeholder="One-line description (shown in Claude's TOC)"
    />
    {#if previewMode}
      <div class="kb-editor-preview">
        <MarkdownPreview content={editContent} />
      </div>
    {:else}
      <textarea
        class="kb-editor-content"
        bind:value={editContent}
        oninput={scheduleSave}
        placeholder="Write markdown content..."
      ></textarea>
    {/if}
  </div>
{:else}
  <div class="kb-editor-empty">
    <p>Select an entry or create a new one</p>
  </div>
{/if}

<style>
  .kb-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 12px 16px;
    gap: 8px;
  }
  .kb-editor-header {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .kb-editor-title {
    flex: 1;
    font-size: 18px;
    font-weight: 600;
    background: none;
    border: none;
    color: var(--text-primary);
    padding: 4px 0;
    outline: none;
  }
  .kb-editor-title::placeholder {
    color: var(--text-muted);
  }
  .kb-editor-toolbar {
    display: flex;
    gap: 4px;
  }
  .kb-editor-description {
    font-size: 13px;
    background: none;
    border: none;
    border-bottom: 1px solid var(--border);
    color: var(--text-secondary);
    padding: 4px 0 8px;
    outline: none;
  }
  .kb-editor-description::placeholder {
    color: var(--text-muted);
  }
  .kb-editor-content {
    flex: 1;
    font-family: var(--mono-font, monospace);
    font-size: 13px;
    line-height: 1.6;
    background: none;
    border: none;
    color: var(--text-primary);
    resize: none;
    outline: none;
    padding: 0;
  }
  .kb-editor-content::placeholder {
    color: var(--text-muted);
  }
  .kb-editor-preview {
    flex: 1;
    overflow-y: auto;
  }
  .kb-editor-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted);
    font-size: 14px;
  }
  .btn-danger:hover {
    color: var(--status-blocked, #e55);
  }
  .active {
    color: var(--accent);
  }
</style>
```

- [ ] **Step 3: Create KnowledgePanel.svelte**

Create `src/lib/modules/knowledge/KnowledgePanel.svelte`:

```svelte
<script lang="ts">
  import type { KnowledgeEntry, KnowledgeFolder } from "../../api/types";
  import {
    getAllKnowledgeEntries,
    getKnowledgeFolders,
    createKnowledgeEntry,
    createKnowledgeFolder,
    deleteKnowledgeFolder,
    renameKnowledgeFolder,
  } from "../../api/knowledge";
  import KnowledgeFolderTree from "./KnowledgeFolderTree.svelte";
  import KnowledgeEntryEditor from "./KnowledgeEntryEditor.svelte";

  let {
    onClose,
  }: {
    onClose: () => void;
  } = $props();

  let entries = $state<KnowledgeEntry[]>([]);
  let folders = $state<KnowledgeFolder[]>([]);
  let selectedEntryId = $state<string | null>(null);
  let selectedFolderId = $state<string | null>(null);
  let loading = $state(true);

  let selectedEntry = $derived(
    selectedEntryId ? entries.find(e => e.id === selectedEntryId) ?? null : null,
  );

  $effect(() => {
    loadData();
  });

  async function loadData() {
    loading = true;
    try {
      const [e, f] = await Promise.all([
        getAllKnowledgeEntries(),
        getKnowledgeFolders(),
      ]);
      entries = e;
      folders = f;
    } catch (e) {
      console.error("Failed to load knowledge base:", e);
    } finally {
      loading = false;
    }
  }

  async function handleCreateEntry(folderId: string | null) {
    try {
      const entry = await createKnowledgeEntry({
        title: "New Entry",
        content: "",
        folderId,
      });
      await loadData();
      selectedEntryId = entry.id;
    } catch (e) {
      console.error("Failed to create entry:", e);
    }
  }

  async function handleCreateFolder(parentId: string | null) {
    try {
      await createKnowledgeFolder("New Folder", parentId);
      await loadData();
    } catch (e) {
      console.error("Failed to create folder:", e);
    }
  }

  async function handleDeleteFolder(id: string) {
    if (!confirm("Delete this folder? Entries will be moved to the parent folder.")) return;
    try {
      await deleteKnowledgeFolder(id);
      if (selectedFolderId === id) selectedFolderId = null;
      await loadData();
    } catch (e) {
      console.error("Failed to delete folder:", e);
    }
  }

  async function handleRenameFolder(id: string, name: string) {
    try {
      await renameKnowledgeFolder(id, name);
      await loadData();
    } catch (e) {
      console.error("Failed to rename folder:", e);
    }
  }

  function handleEntryDeleted() {
    selectedEntryId = null;
    loadData();
  }
</script>

<div class="kb-panel">
  <div class="kb-panel-header">
    <h2 class="kb-panel-title">Knowledge Base</h2>
    <button class="btn-ghost btn-sm" onclick={onClose} title="Close">
      <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M3.72 3.72a.75.75 0 011.06 0L8 6.94l3.22-3.22a.75.75 0 111.06 1.06L9.06 8l3.22 3.22a.75.75 0 11-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 01-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 010-1.06z"/></svg>
    </button>
  </div>
  <div class="kb-panel-body">
    <KnowledgeFolderTree
      {folders}
      {entries}
      {selectedEntryId}
      {selectedFolderId}
      onSelectEntry={(id) => selectedEntryId = id}
      onSelectFolder={(id) => selectedFolderId = id}
      onCreateEntry={handleCreateEntry}
      onCreateFolder={handleCreateFolder}
      onDeleteFolder={handleDeleteFolder}
      onRenameFolder={handleRenameFolder}
    />
    <div class="kb-panel-editor">
      <KnowledgeEntryEditor
        entry={selectedEntry}
        onSaved={loadData}
        onDeleted={handleEntryDeleted}
      />
    </div>
  </div>
</div>

<style>
  .kb-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .kb-panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 16px;
    border-bottom: 1px solid var(--border);
  }
  .kb-panel-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }
  .kb-panel-body {
    display: flex;
    flex: 1;
    overflow: hidden;
  }
  .kb-panel-editor {
    flex: 1;
    overflow-y: auto;
  }
</style>
```

- [ ] **Step 4: Commit**

```bash
git add src/lib/modules/knowledge/
git commit -m "feat: add KnowledgePanel, FolderTree, and EntryEditor components"
```

---

### Task 8: App.svelte integration — sidebar button + panel mount

**Files:**
- Modify: `src/App.svelte`
- Modify: `src/lib/components/Sidebar.svelte`

- [ ] **Step 1: Add knowledgeMode state and KB panel to App.svelte**

In `src/App.svelte`, add the import at the top of the `<script>` block (after the existing imports):

```typescript
  import KnowledgePanel from "./lib/modules/knowledge/KnowledgePanel.svelte";
```

Add a state variable near the other state declarations (after `let storageSwitching`):

```typescript
  let showKnowledge = $state(false);
```

- [ ] **Step 2: Add the KB panel alongside the feature detail area**

In `src/App.svelte`, modify the main content area. Find the `<div class="main-content">` section (around line 425) and wrap the existing content with a knowledge mode toggle:

Replace:
```svelte
    <div class="main-content">
      {#if workspaceTabs.length > 0}
```

With:
```svelte
    <div class="main-content">
      {#if showKnowledge}
        <KnowledgePanel onClose={() => showKnowledge = false} />
      {:else if workspaceTabs.length > 0}
```

- [ ] **Step 3: Add KB button callback to Sidebar**

In `src/App.svelte`, add the `onOpenKnowledge` prop to the `<Sidebar>` component:

```svelte
      onOpenKnowledge={() => showKnowledge = true}
```

Add it alongside the other `on*` props on the `<Sidebar>` component.

- [ ] **Step 4: Add knowledge button to Sidebar footer**

In `src/lib/components/Sidebar.svelte`, add the `onOpenKnowledge` prop. In the props interface, add:

```typescript
    onOpenKnowledge?: () => void;
```

And in the destructuring:

```typescript
    onOpenKnowledge,
```

Then in the sidebar footer (after the settings button, around line 1049), add a KB button:

```svelte
    <button class="btn-ghost sidebar-footer-btn" onclick={() => onOpenKnowledge?.()} title="Knowledge Base">
      <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M1 2.828c.885-.37 2.154-.769 3.388-.893 1.33-.134 2.458.063 3.112.752v9.746c-.935-.53-2.12-.603-3.213-.493-1.18.12-2.37.461-3.287.811V2.828zm7.5-.141c.654-.689 1.782-.886 3.112-.752 1.234.124 2.503.523 3.388.893v9.923c-.918-.35-2.107-.692-3.287-.81-1.094-.111-2.278-.039-3.213.492V2.687zM8 1.783C7.015.936 5.587.81 4.287.94c-1.514.153-3.042.672-3.994 1.105A.5.5 0 000 2.5v10a.5.5 0 00.707.455c.882-.4 2.303-.881 3.68-1.02 1.409-.142 2.59.087 3.223.877a.5.5 0 00.78 0c.633-.79 1.814-1.019 3.222-.877 1.378.139 2.8.62 3.681 1.02A.5.5 0 0016 12.5v-10a.5.5 0 00-.293-.455c-.952-.433-2.48-.952-3.994-1.105C10.413.81 8.985.936 8 1.783z"/></svg>
    </button>
```

- [ ] **Step 5: Verify the app compiles and runs**

Run from the project root:
```bash
npm run tauri dev
```

Expected: App launches. KB button appears in sidebar footer. Clicking it shows the KnowledgePanel. Creating entries, folders, renaming, deleting all work. Closing returns to the normal feature view.

- [ ] **Step 6: Commit**

```bash
git add src/App.svelte src/lib/components/Sidebar.svelte
git commit -m "feat: integrate Knowledge Base panel into sidebar and main layout"
```

---

### Task 9: Update CLAUDE.md

**Files:**
- Modify: `CLAUDE.md`

- [ ] **Step 1: Add knowledge base to the database tables section**

In `CLAUDE.md`, add after the `search_index` entry in the "Database tables" section:

```markdown
- `knowledge_folders` — hierarchical folders for organizing KB entries (storage-scoped, no feature_id)
- `knowledge_entries` — markdown documents with title, description, content, optional folder_id and feature_id link
```

- [ ] **Step 2: Add KB section to Architecture**

In `CLAUDE.md`, add a new section after "### Plans system":

```markdown
### Knowledge Base

Storage-scoped knowledge repository for HOW-TOs, findings, and research results. Primary consumer is Claude — a compact TOC (titles + IDs) is injected into every MCP session's instructions so Claude knows what's available. Full content is loaded on demand via `get_knowledge_entry(id)`. Entries are organized in hierarchical folders (max depth 5). Entries optionally link to features via nullable `feature_id`. CRUD via both Tauri commands and MCP tools. FTS5 indexed with `entity_type = 'knowledge'`.
```

- [ ] **Step 3: Update MCP server features**

In the "### MCP server features" section, add `knowledge` to both Read and Write tool lists:

Add to Read: `list_knowledge, get_knowledge_entry, search_knowledge`
Add to Write: `create_knowledge_entry, update_knowledge_entry, delete_knowledge_entry, create_knowledge_folder, rename_knowledge_folder, delete_knowledge_folder`

- [ ] **Step 4: Update Frontend components list**

Add to the frontend components list:

```markdown
- `src/lib/modules/knowledge/` — KnowledgePanel (root), KnowledgeFolderTree, KnowledgeEntryEditor
```

- [ ] **Step 5: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: add knowledge base to CLAUDE.md architecture docs"
```
