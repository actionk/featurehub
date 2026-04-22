# Extension System Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a directory-based extension system that lets third-party code register custom MCP tools, event hooks, database tables, and UI tabs without recompiling Feature Hub.

**Architecture:** Extensions live in `<storage>/extensions/<id>/extension.json` + handler scripts. The Rust library crate (`feature_hub`) exposes an `extensions` module used by both the Tauri app and the `fh-mcp` binary. Scripts are spawned as child `node` processes communicating over stdin/stdout JSON. Frontend tabs render inside iframes served via Tauri's `asset:` protocol.

**Tech Stack:** Rust (serde_json, rusqlite, std::process), TypeScript/Svelte 5, rmcp 1.2 (`ToolRouter`, `ServerHandler`)

---

## File Map

**New Rust files (in `feature_hub` library crate):**
- `src-tauri/src/extensions/mod.rs` — `ExtensionRegistry`, `LoadedExtension`, `RequiresStatus`; `load_extensions()`, `dispatch_event()`
- `src-tauri/src/extensions/manifest.rs` — all manifest struct definitions (serde deserialization + validation)
- `src-tauri/src/extensions/table_provisioner.rs` — `provision_tables(conn, extensions)` — CREATE TABLE IF NOT EXISTS
- `src-tauri/src/extensions/script_runner.rs` — `run_blocking()`, `run_event_hook()`, `ScriptInput`

**New Rust files (Tauri commands):**
- `src-tauri/src/commands/extensions.rs` — `get_extensions`, `get_extension_badge` Tauri commands

**Modified Rust files:**
- `src-tauri/src/lib.rs` — add `pub mod extensions;`, add `extensions: Mutex<ExtensionRegistry>` to `AppState`, load + provision on startup
- `src-tauri/src/commands/mod.rs` — add `mod extensions; pub use extensions::*;`
- `src-tauri/src/commands/storage.rs` — reload + provision extensions in `do_switch_storage`
- `src-tauri/src/commands/links.rs` — call `dispatch_event` after `add_link` succeeds
- `src-tauri/src/bin/fh_mcp.rs` — add `extension_registry` field to `FeatureHubMcp`, replace `#[tool_handler]` with manual `ServerHandler` impl

**New frontend files:**
- `src/lib/api/extensions.ts` — `getExtensions()`, `getExtensionBadge()` invoke wrappers
- `src/lib/modules/extensions/ExtensionTabFrame.svelte` — generic iframe tab with postMessage bridge

**Modified frontend files:**
- `src/lib/api/types.ts` — add `ExtensionInfo`, `ExtensionTabDecl`, `RequiresStatusInfo`, `InstalledExtensionManifest` types
- `src/lib/api/tauri.ts` — add `export * from "./extensions";`
- `src/lib/modules/index.ts` — call `getExtensions()` at boot, register extension tabs
- `src/lib/components/SettingsModal.svelte` — show installed directory-based extensions in the Extensions tab

---

## Task 0: Bootstrap extensions module

**Files:**
- Create: `src-tauri/src/extensions/mod.rs` (stub)
- Modify: `src-tauri/src/lib.rs`

This task makes the module tree valid so Tasks 1–4 can compile their tests.

- [ ] **Step 1: Create stub `src-tauri/src/extensions/mod.rs`**

```rust
pub mod manifest;
pub mod script_runner;
pub mod table_provisioner;
```

- [ ] **Step 2: Add `pub mod extensions;` to `src-tauri/src/lib.rs`**

After the existing `pub mod` declarations in `src-tauri/src/lib.rs`:

```rust
pub mod extensions;
```

- [ ] **Step 3: Compile check**

```
cd src-tauri && cargo check 2>&1 | tail -10
```
Expected: warnings about empty modules but no errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/extensions/mod.rs src-tauri/src/lib.rs
git commit -m "feat(extensions): scaffold extensions module in library crate"
```

---

## Task 1: Extension Manifest Structs

**Files:**
- Create: `src-tauri/src/extensions/manifest.rs`

- [ ] **Step 1: Write the failing test**

Add at the bottom of `src-tauri/src/extensions/manifest.rs` (create the file first with just the test):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn valid_manifest_json() -> &'static str {
        r#"{
            "id": "my-ext",
            "name": "My Extension",
            "version": "1.0.0",
            "tables": [
                {
                    "name": "ext_my_table",
                    "columns": [
                        {"name": "id", "type": "TEXT PRIMARY KEY"},
                        {"name": "feature_id", "type": "TEXT NOT NULL", "fk": "features(id) ON DELETE CASCADE"}
                    ],
                    "indexes": ["feature_id"]
                }
            ],
            "tools": [
                {"name": "do_thing", "description": "Does a thing", "handler": "handlers/do_thing.js", "params": {
                    "feature_id": {"type": "string", "required": true}
                }}
            ],
            "events": [
                {"on": "link_created", "filter": {"link_type": "github-pr"}, "handler": "handlers/on_link.js"}
            ],
            "tabs": [
                {"id": "prs", "label": "PRs", "emoji": "🔀", "sortOrder": 350, "component": "ui/tab.html",
                 "badge_query": "SELECT COUNT(*) FROM ext_my_table WHERE feature_id = ?"}
            ]
        }"#
    }

    #[test]
    fn parses_valid_manifest() {
        let m: ExtensionManifest = serde_json::from_str(valid_manifest_json()).unwrap();
        assert_eq!(m.id, "my-ext");
        assert_eq!(m.name, "My Extension");
        assert_eq!(m.version, "1.0.0");
        assert_eq!(m.tables.len(), 1);
        assert_eq!(m.tables[0].name, "ext_my_table");
        assert_eq!(m.tables[0].columns.len(), 2);
        assert_eq!(m.tables[0].columns[1].fk.as_deref(), Some("features(id) ON DELETE CASCADE"));
        assert_eq!(m.tools.len(), 1);
        assert_eq!(m.tools[0].name, "do_thing");
        assert!(m.tools[0].params["feature_id"].required);
        assert_eq!(m.events.len(), 1);
        assert_eq!(m.events[0].filter["link_type"], "github-pr");
        assert_eq!(m.tabs.len(), 1);
        assert_eq!(m.tabs[0].sort_order, 350);
    }

    #[test]
    fn validates_missing_id() {
        let json = r#"{"name": "x", "version": "1.0.0"}"#;
        let m: ExtensionManifest = serde_json::from_str(json).unwrap();
        assert!(m.validate().is_err());
    }

    #[test]
    fn validates_unprefixed_table() {
        let json = r#"{"id": "x", "name": "x", "version": "1.0.0", "tables": [{"name": "my_table", "columns": [], "indexes": []}]}"#;
        let m: ExtensionManifest = serde_json::from_str(json).unwrap();
        assert!(m.validate().is_err());
    }

    #[test]
    fn ignores_unknown_fields() {
        let json = r#"{"id": "x", "name": "x", "version": "1.0.0", "unknown_future_field": true}"#;
        let result: Result<ExtensionManifest, _> = serde_json::from_str(json);
        assert!(result.is_ok());
    }
}
```

- [ ] **Step 2: Run test to confirm it fails**

```
cd src-tauri && cargo test --lib extensions::manifest 2>&1 | tail -5
```
Expected: compile error — `ExtensionManifest` not defined yet.

- [ ] **Step 3: Write the full manifest module**

Replace `src-tauri/src/extensions/manifest.rs` with:

```rust
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub tables: Vec<TableDecl>,
    #[serde(default)]
    pub tools: Vec<ToolDecl>,
    #[serde(default)]
    pub events: Vec<EventDecl>,
    #[serde(default)]
    pub tabs: Vec<TabDecl>,
    #[serde(default)]
    pub instructions: String,
}

impl ExtensionManifest {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("missing 'id'".into());
        }
        if self.name.is_empty() {
            return Err("missing 'name'".into());
        }
        if self.version.is_empty() {
            return Err("missing 'version'".into());
        }
        for table in &self.tables {
            if !table.name.starts_with("ext_") {
                return Err(format!(
                    "table '{}' must be prefixed with 'ext_'",
                    table.name
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDecl {
    pub name: String,
    #[serde(default)]
    pub columns: Vec<ColumnDecl>,
    #[serde(default)]
    pub indexes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDecl {
    pub name: String,
    #[serde(rename = "type")]
    pub col_type: String,
    #[serde(default)]
    pub fk: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDecl {
    pub name: String,
    pub description: String,
    pub handler: String,
    #[serde(default)]
    pub params: HashMap<String, ParamDecl>,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDecl {
    #[serde(rename = "type")]
    pub param_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDecl {
    pub on: String,
    #[serde(default)]
    pub filter: HashMap<String, String>,
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabDecl {
    pub id: String,
    pub label: String,
    pub emoji: String,
    #[serde(rename = "sortOrder", default)]
    pub sort_order: u32,
    pub component: String,
    #[serde(default)]
    pub badge_query: Option<String>,
}

// Tests at end of file — paste the test module from Step 1 here
```

- [ ] **Step 4: Run tests to confirm they pass**

```
cd src-tauri && cargo test --lib extensions::manifest 2>&1 | tail -10
```
Expected: `4 tests passed`

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/extensions/manifest.rs
git commit -m "feat(extensions): add manifest structs with validation"
```

---

## Task 2: Script Runner

**Files:**
- Create: `src-tauri/src/extensions/script_runner.rs`

- [ ] **Step 1: Write the failing test**

Create `src-tauri/src/extensions/script_runner.rs` with just the test:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn echo_script() -> tempfile::TempPath {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        // Script that reads stdin JSON and echoes params back as {ok: true, data: <params>}
        writeln!(f, r#"
const chunks = [];
process.stdin.on('data', c => chunks.push(c));
process.stdin.on('end', () => {{
  const input = JSON.parse(Buffer.concat(chunks).toString());
  process.stdout.write(JSON.stringify({{ ok: true, data: input.params }}));
}});
"#).unwrap();
        f.into_temp_path()
    }

    #[test]
    fn runs_script_and_returns_data() {
        let script = echo_script();
        let input = ScriptInput {
            params: serde_json::json!({"key": "value"}).as_object().unwrap().clone(),
            db_path: "/tmp/test.db".into(),
            storage_path: "/tmp".into(),
            feature_id: None,
        };
        let result = run_blocking(script.as_ref(), &input, 5).unwrap();
        assert_eq!(result["key"], "value");
    }

    #[test]
    fn returns_error_for_nonexistent_script() {
        let input = ScriptInput {
            params: Default::default(),
            db_path: "/tmp/test.db".into(),
            storage_path: "/tmp".into(),
            feature_id: None,
        };
        let result = run_blocking(std::path::Path::new("/nonexistent/script.js"), &input, 5);
        assert!(result.is_err());
    }

    #[test]
    fn returns_error_when_script_returns_ok_false() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, r#"process.stdout.write(JSON.stringify({{ok: false, error: "something went wrong"}}));"#).unwrap();
        let path = f.into_temp_path();
        let input = ScriptInput { params: Default::default(), db_path: "/tmp/test.db".into(), storage_path: "/tmp".into(), feature_id: None };
        let result = run_blocking(path.as_ref(), &input, 5);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("something went wrong"));
    }
}
```

- [ ] **Step 2: Add `tempfile` to dev-dependencies in `src-tauri/Cargo.toml`**

```toml
[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 3: Run test to confirm it fails**

```
cd src-tauri && cargo test --lib extensions::script_runner 2>&1 | tail -5
```
Expected: compile error — `ScriptInput` not defined.

- [ ] **Step 4: Implement `script_runner.rs`**

```rust
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ScriptInput {
    pub params: serde_json::Map<String, serde_json::Value>,
    pub db_path: String,
    pub storage_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_id: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ScriptOutput {
    ok: bool,
    #[serde(default)]
    data: Option<serde_json::Value>,
    #[serde(default)]
    error: Option<String>,
}

/// Run a script synchronously, blocking the calling thread.
/// Returns parsed `data` on success, or an error string.
pub fn run_blocking(
    script_path: &Path,
    input: &ScriptInput,
    timeout_secs: u64,
) -> Result<serde_json::Value, String> {
    let input_json = serde_json::to_string(input).map_err(|e| e.to_string())?;

    let mut child = Command::new("node")
        .arg(script_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_clear()
        .env("FH_DB_PATH", &input.db_path)
        .env("FH_STORAGE_PATH", &input.storage_path)
        .spawn()
        .map_err(|e| format!("Failed to spawn script {:?}: {}", script_path, e))?;

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input_json.as_bytes());
    }

    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(child.wait_with_output());
    });

    let output = rx
        .recv_timeout(Duration::from_secs(timeout_secs))
        .map_err(|_| format!("Script timed out after {}s", timeout_secs))?
        .map_err(|e| format!("Script execution failed: {}", e))?;

    const MAX_OUTPUT: usize = 1024 * 1024;
    if output.stdout.len() > MAX_OUTPUT {
        return Err("Script output exceeded 1MB limit".to_string());
    }

    let stdout = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    let result: ScriptOutput = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse script output: {}. Raw: {}", e, &stdout[..stdout.len().min(200)]))?;

    if result.ok {
        Ok(result.data.unwrap_or(serde_json::Value::Null))
    } else {
        Err(result.error.unwrap_or_else(|| "Script returned ok=false".to_string()))
    }
}

/// Spawn an event hook script fire-and-forget. Errors are logged, not propagated.
pub fn run_event_hook(script_path: std::path::PathBuf, input: ScriptInput, extension_id: String) {
    std::thread::spawn(move || match run_blocking(&script_path, &input, 10) {
        Ok(_) => {}
        Err(e) => eprintln!("[ext:{}] Event hook {:?} failed: {}", extension_id, script_path, e),
    });
}

// Paste tests from Step 1 here
```

- [ ] **Step 5: Run tests**

```
cd src-tauri && cargo test --lib extensions::script_runner 2>&1 | tail -10
```
Expected: `3 tests passed` (note: these tests require `node` to be installed on the machine)

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/extensions/script_runner.rs src-tauri/Cargo.toml
git commit -m "feat(extensions): add script runner with stdin/stdout protocol"
```

---

## Task 3: Table Provisioner

**Files:**
- Create: `src-tauri/src/extensions/table_provisioner.rs`

- [ ] **Step 1: Write the failing test**

Create `src-tauri/src/extensions/table_provisioner.rs` with just the test module:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_utils::test_db;
    use crate::extensions::manifest::{TableDecl, ColumnDecl};

    fn make_table(name: &str, valid_prefix: bool) -> TableDecl {
        let table_name = if valid_prefix { name.to_string() } else { name.to_string() };
        TableDecl {
            name: table_name,
            columns: vec![
                ColumnDecl { name: "id".into(), col_type: "TEXT PRIMARY KEY".into(), fk: None },
                ColumnDecl { name: "feature_id".into(), col_type: "TEXT NOT NULL".into(), fk: Some("features(id) ON DELETE CASCADE".into()) },
                ColumnDecl { name: "title".into(), col_type: "TEXT NOT NULL".into(), fk: None },
            ],
            indexes: vec!["feature_id".into()],
        }
    }

    #[test]
    fn creates_table_and_index() {
        let conn = test_db();
        let table = make_table("ext_test_prs", true);
        provision_table(&conn, &table).unwrap();

        // Table should exist
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='ext_test_prs'",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(count, 1);

        // Index should exist
        let idx_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='idx_ext_test_prs_feature_id'",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(idx_count, 1);
    }

    #[test]
    fn is_idempotent() {
        let conn = test_db();
        let table = make_table("ext_idempotent", true);
        provision_table(&conn, &table).unwrap();
        // Second call should not fail
        provision_table(&conn, &table).unwrap();
    }

    #[test]
    fn rejects_unprefixed_table_name() {
        let conn = test_db();
        let table = make_table("no_prefix_table", false);
        let result = provision_table(&conn, &table);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ext_"));
    }
}
```

- [ ] **Step 2: Run test to confirm it fails**

```
cd src-tauri && cargo test --lib extensions::table_provisioner 2>&1 | tail -5
```
Expected: compile error.

- [ ] **Step 3: Implement `table_provisioner.rs`**

```rust
use rusqlite::Connection;
use crate::extensions::manifest::{TableDecl, ColumnDecl};

pub fn provision_tables(
    conn: &Connection,
    tables: &[&TableDecl],
) -> Result<(), String> {
    for table in tables {
        provision_table(conn, table)?;
    }
    Ok(())
}

pub fn provision_table(conn: &Connection, table: &TableDecl) -> Result<(), String> {
    if !table.name.starts_with("ext_") {
        return Err(format!(
            "table '{}' must be prefixed with 'ext_' to avoid collisions",
            table.name
        ));
    }

    let mut col_defs: Vec<String> = table
        .columns
        .iter()
        .map(|c| format!("{} {}", c.name, c.col_type))
        .collect();

    for col in &table.columns {
        if let Some(ref fk) = col.fk {
            col_defs.push(format!("FOREIGN KEY ({}) REFERENCES {}", col.name, fk));
        }
    }

    let create_sql = format!(
        "CREATE TABLE IF NOT EXISTS {} ({})",
        table.name,
        col_defs.join(", ")
    );
    conn.execute_batch(&create_sql).map_err(|e| e.to_string())?;

    for col_name in &table.indexes {
        let idx_sql = format!(
            "CREATE INDEX IF NOT EXISTS idx_{}_{} ON {} ({})",
            table.name, col_name, table.name, col_name
        );
        conn.execute_batch(&idx_sql).map_err(|e| e.to_string())?;
    }

    Ok(())
}

// Paste tests from Step 1 here
```

- [ ] **Step 4: Run tests**

```
cd src-tauri && cargo test --lib extensions::table_provisioner 2>&1 | tail -10
```
Expected: `3 tests passed`

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/extensions/table_provisioner.rs
git commit -m "feat(extensions): add table provisioner for extension-defined SQLite tables"
```

---

## Task 4: Extension Registry

**Files:**
- Create: `src-tauri/src/extensions/mod.rs`

- [ ] **Step 1: Write the failing test**

Create `src-tauri/src/extensions/mod.rs` with just the test (after `pub mod` declarations):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_manifest(dir: &TempDir, ext_id: &str, json: &str) {
        let ext_dir = dir.path().join(ext_id);
        fs::create_dir_all(&ext_dir).unwrap();
        fs::write(ext_dir.join("extension.json"), json).unwrap();
    }

    fn valid_json(id: &str) -> String {
        format!(r#"{{"id": "{}", "name": "Ext {}", "version": "1.0.0"}}"#, id, id)
    }

    #[test]
    fn loads_valid_extension() {
        let tmp = TempDir::new().unwrap();
        write_manifest(&tmp, "my-ext", &valid_json("my-ext"));
        let registry = ExtensionRegistry::load_from_dir(tmp.path());
        assert_eq!(registry.extensions.len(), 1);
        assert_eq!(registry.extensions[0].manifest.id, "my-ext");
        assert!(registry.extensions[0].enabled);
    }

    #[test]
    fn skips_missing_required_fields() {
        let tmp = TempDir::new().unwrap();
        write_manifest(&tmp, "bad-ext", r#"{"name": "x", "version": "1.0.0"}"#); // missing id
        let registry = ExtensionRegistry::load_from_dir(tmp.path());
        assert_eq!(registry.extensions.len(), 0);
    }

    #[test]
    fn skips_duplicate_ids() {
        let tmp = TempDir::new().unwrap();
        write_manifest(&tmp, "ext-a", &valid_json("same-id"));
        write_manifest(&tmp, "ext-b", &valid_json("same-id"));
        let registry = ExtensionRegistry::load_from_dir(tmp.path());
        assert_eq!(registry.extensions.len(), 1);
    }

    #[test]
    fn skips_unprefixed_table() {
        let tmp = TempDir::new().unwrap();
        write_manifest(&tmp, "bad-table", r#"{"id": "x", "name": "x", "version": "1.0.0", "tables": [{"name": "no_prefix", "columns": [], "indexes": []}]}"#);
        let registry = ExtensionRegistry::load_from_dir(tmp.path());
        assert_eq!(registry.extensions.len(), 0);
    }

    #[test]
    fn returns_empty_for_nonexistent_dir() {
        let registry = ExtensionRegistry::load_from_dir(std::path::Path::new("/nonexistent/path"));
        assert_eq!(registry.extensions.len(), 0);
    }

    #[test]
    fn find_tool_returns_matching_extension_and_decl() {
        let tmp = TempDir::new().unwrap();
        write_manifest(&tmp, "my-ext", r#"{
            "id": "my-ext", "name": "My Ext", "version": "1.0.0",
            "tools": [{"name": "my_tool", "description": "test", "handler": "h.js", "params": {}}]
        }"#);
        let registry = ExtensionRegistry::load_from_dir(tmp.path());
        let result = registry.find_tool("my_tool");
        assert!(result.is_some());
        let (ext, tool) = result.unwrap();
        assert_eq!(ext.manifest.id, "my-ext");
        assert_eq!(tool.name, "my_tool");
    }

    #[test]
    fn event_filter_matches_correctly() {
        let payload = serde_json::json!({"link_type": "github-pr", "feature_id": "abc"});
        let mut filter = std::collections::HashMap::new();
        filter.insert("link_type".to_string(), "github-pr".to_string());
        assert!(event_filter_matches(&filter, &payload));

        let mut bad_filter = std::collections::HashMap::new();
        bad_filter.insert("link_type".to_string(), "jira".to_string());
        assert!(!event_filter_matches(&bad_filter, &payload));
    }
}
```

- [ ] **Step 2: Run test to confirm it fails**

```
cd src-tauri && cargo test --lib extensions 2>&1 | tail -5
```

- [ ] **Step 3: Create `src-tauri/src/extensions/mod.rs`**

```rust
pub mod manifest;
pub mod script_runner;
pub mod table_provisioner;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use manifest::{EventDecl, ExtensionManifest, ToolDecl};

#[derive(Debug, Clone)]
pub struct RequiresStatus {
    pub name: String,
    pub found: bool,
    pub path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LoadedExtension {
    pub manifest: ExtensionManifest,
    pub enabled: bool,
    pub dir: PathBuf,
    pub requires_status: Vec<RequiresStatus>,
}

#[derive(Debug, Clone, Default)]
pub struct ExtensionRegistry {
    pub extensions: Vec<LoadedExtension>,
}

impl ExtensionRegistry {
    pub fn load_from_dir(extensions_dir: &Path) -> Self {
        let mut registry = Self::default();
        if !extensions_dir.exists() {
            return registry;
        }
        let entries = match std::fs::read_dir(extensions_dir) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("[extensions] Failed to read dir {:?}: {}", extensions_dir, e);
                return registry;
            }
        };
        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            let manifest_path = dir.join("extension.json");
            if !manifest_path.exists() {
                continue;
            }
            let content = match std::fs::read_to_string(&manifest_path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[extensions] Failed to read {:?}: {}", manifest_path, e);
                    continue;
                }
            };
            let manifest: ExtensionManifest = match serde_json::from_str(&content) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("[extensions] Failed to parse {:?}: {}", manifest_path, e);
                    continue;
                }
            };
            if let Err(e) = manifest.validate() {
                eprintln!("[extensions] Invalid manifest in {:?}: {}", dir, e);
                continue;
            }
            if registry.extensions.iter().any(|e| e.manifest.id == manifest.id) {
                eprintln!("[extensions] Duplicate id '{}', skipping {:?}", manifest.id, dir);
                continue;
            }
            let requires_status = check_requires(&manifest.requires);
            registry.extensions.push(LoadedExtension {
                manifest,
                enabled: true,
                dir,
                requires_status,
            });
        }
        registry
    }

    pub fn find_tool<'a>(&'a self, tool_name: &str) -> Option<(&'a LoadedExtension, &'a ToolDecl)> {
        for ext in &self.extensions {
            if !ext.enabled {
                continue;
            }
            for tool in &ext.manifest.tools {
                if tool.name == tool_name {
                    return Some((ext, tool));
                }
            }
        }
        None
    }

    pub fn handlers_for_event(
        &self,
        event_type: &str,
        payload: &serde_json::Value,
    ) -> Vec<(PathBuf, String)> {
        let mut out = Vec::new();
        for ext in &self.extensions {
            if !ext.enabled {
                continue;
            }
            for event_decl in &ext.manifest.events {
                if event_decl.on != event_type {
                    continue;
                }
                if !event_filter_matches(&event_decl.filter, payload) {
                    continue;
                }
                out.push((ext.dir.join(&event_decl.handler), ext.manifest.id.clone()));
            }
        }
        out
    }
}

pub fn check_requires(requires: &[String]) -> Vec<RequiresStatus> {
    requires
        .iter()
        .map(|name| {
            let result = if cfg!(windows) {
                std::process::Command::new("where").arg(name).output()
            } else {
                std::process::Command::new("which").arg(name).output()
            };
            match result {
                Ok(output) if output.status.success() => RequiresStatus {
                    name: name.clone(),
                    found: true,
                    path: String::from_utf8(output.stdout)
                        .ok()
                        .map(|s| s.trim().to_string()),
                },
                _ => RequiresStatus {
                    name: name.clone(),
                    found: false,
                    path: None,
                },
            }
        })
        .collect()
}

pub(crate) fn event_filter_matches(
    filter: &HashMap<String, String>,
    payload: &serde_json::Value,
) -> bool {
    for (key, expected) in filter {
        match payload.get(key).and_then(|v| v.as_str()) {
            Some(val) if val == expected => {}
            _ => return false,
        }
    }
    true
}

pub fn dispatch_event(
    registry: &ExtensionRegistry,
    event_type: &str,
    payload: serde_json::Value,
    db_path: String,
    storage_path: String,
    feature_id: Option<String>,
) {
    let handlers = registry.handlers_for_event(event_type, &payload);
    for (script_path, extension_id) in handlers {
        let input = script_runner::ScriptInput {
            params: payload.as_object().cloned().unwrap_or_default(),
            db_path: db_path.clone(),
            storage_path: storage_path.clone(),
            feature_id: feature_id.clone(),
        };
        script_runner::run_event_hook(script_path, input, extension_id);
    }
}

// Paste tests from Step 1 here
```

- [ ] **Step 4: Run tests**

```
cd src-tauri && cargo test --lib extensions 2>&1 | tail -10
```
Expected: all tests pass. (Skip if `node` not available — script_runner tests are integration-level)

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/extensions/
git commit -m "feat(extensions): add extension registry with discovery and event routing"
```

---

## Task 5: Wire extensions into AppState

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/commands/mod.rs` (add `extensions` module stub for now)

- [ ] **Step 1: Add `pub mod extensions;` to `src-tauri/src/lib.rs`**

In `src-tauri/src/lib.rs`, add after the existing `pub mod` declarations:

```rust
pub mod extensions;
```

- [ ] **Step 2: Add `extensions` field to `AppState`**

In `src-tauri/src/lib.rs`, change:

```rust
pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub storage_path: Mutex<Option<PathBuf>>,
}
```

to:

```rust
pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub storage_path: Mutex<Option<PathBuf>>,
    pub extensions: Mutex<extensions::ExtensionRegistry>,
}
```

- [ ] **Step 3: Load extensions and provision tables on startup**

In `src-tauri/src/lib.rs`, in the `run()` function, after `db::initialize(&conn)` succeeds, add extension loading. Find the block that creates `AppState` and update it:

```rust
// Load extensions for the active storage
let extension_registry = if let Some(ref path) = storage_path {
    let ext_dir = path.join("extensions");
    let registry = extensions::ExtensionRegistry::load_from_dir(&ext_dir);
    // Provision extension tables
    let table_decls: Vec<_> = registry
        .extensions
        .iter()
        .flat_map(|e| &e.manifest.tables)
        .collect();
    for table in &table_decls {
        if let Err(e) = extensions::table_provisioner::provision_table(&conn, table) {
            eprintln!("[extensions] Table provisioning failed: {}", e);
        }
    }
    registry
} else {
    extensions::ExtensionRegistry::default()
};

let state = AppState {
    db: Mutex::new(conn),
    storage_path: Mutex::new(storage_path),
    extensions: Mutex::new(extension_registry),
};
```

This change applies to both the `Ok(Some(entry))` and the in-memory placeholder arm. In the placeholder arm, just use `extensions::ExtensionRegistry::default()` with no provisioning.

- [ ] **Step 4: Check that it compiles**

```
cd src-tauri && cargo check 2>&1 | tail -20
```
Expected: no errors.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat(extensions): load extension registry into AppState on startup"
```

---

## Task 6: Reload extensions on storage switch

**Files:**
- Modify: `src-tauri/src/commands/storage.rs`

- [ ] **Step 1: Find the end of `do_switch_storage` (after DB swap, before returning)**

In `src-tauri/src/commands/storage.rs`, the `do_switch_storage` function ends at line ~125. After setting `storage_path` and before `storage::save_config`, add:

```rust
// Reload extension registry for the new storage
{
    let ext_dir = path.join("extensions");
    let new_registry = crate::extensions::ExtensionRegistry::load_from_dir(&ext_dir);
    // Provision extension tables into the new DB
    {
        let conn_guard = state.db.lock().map_err(|e| e.to_string())?;
        for ext in &new_registry.extensions {
            for table in &ext.manifest.tables {
                if let Err(e) = crate::extensions::table_provisioner::provision_table(&conn_guard, table) {
                    eprintln!("[extensions] Table provisioning failed on storage switch: {}", e);
                }
            }
        }
    }
    let mut ext_guard = state.extensions.lock().map_err(|e| e.to_string())?;
    *ext_guard = new_registry;
}
```

- [ ] **Step 2: Compile check**

```
cd src-tauri && cargo check 2>&1 | tail -10
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands/storage.rs
git commit -m "feat(extensions): reload extension registry on storage switch"
```

---

## Task 7: Event dispatch in add_link

**Files:**
- Modify: `src-tauri/src/commands/links.rs`

- [ ] **Step 1: Update `add_link` to fire `link_created` event**

In `src-tauri/src/commands/links.rs`, replace the current `add_link` command:

```rust
#[tauri::command]
pub fn add_link(
    state: State<'_, AppState>,
    feature_id: String,
    title: String,
    url: String,
    link_type: Option<String>,
    description: Option<String>,
) -> Result<db::links::Link, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let link = db::links::add_link(&conn, &feature_id, &title, &url, link_type, description)?;

    // Fire extension event hooks for link_created (fire-and-forget)
    if let (Ok(registry), Ok(sp_guard)) = (state.extensions.lock(), state.storage_path.lock()) {
        if let Some(ref storage_path) = *sp_guard {
            let db_path = storage_path.join("feature-hub.db").to_string_lossy().to_string();
            let sp_str = storage_path.to_string_lossy().to_string();
            let payload = serde_json::json!({
                "link_type": link.link_type,
                "feature_id": feature_id,
                "link_id": link.id,
                "url": link.url,
                "title": link.title,
            });
            crate::extensions::dispatch_event(
                &registry,
                "link_created",
                payload,
                db_path,
                sp_str,
                Some(feature_id.clone()),
            );
        }
    }

    Ok(link)
}
```

- [ ] **Step 2: Compile check**

```
cd src-tauri && cargo check 2>&1 | tail -10
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands/links.rs
git commit -m "feat(extensions): fire link_created event to extension hooks after add_link"
```

---

## Task 8: Tauri Commands — get_extensions & get_extension_badge

**Files:**
- Create: `src-tauri/src/commands/extensions.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs` (register commands)

- [ ] **Step 1: Write the failing test**

Add to `src-tauri/src/commands/extensions.rs` (new file, test only):

```rust
#[cfg(test)]
mod tests {
    // These commands require Tauri State so they're tested via compile-check only.
    // Integration tested manually.
}
```

(The commands are best covered by manual integration test — see Task 13.)

- [ ] **Step 2: Create `src-tauri/src/commands/extensions.rs`**

```rust
use serde::Serialize;
use tauri::State;

use crate::extensions::manifest::{ExtensionManifest, TabDecl};
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct RequiresStatusInfo {
    pub name: String,
    pub found: bool,
    pub path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExtensionInfo {
    pub manifest: ExtensionManifest,
    pub enabled: bool,
    pub dir: String,
    pub requires_status: Vec<RequiresStatusInfo>,
}

#[tauri::command]
pub fn get_extensions(state: State<'_, AppState>) -> Result<Vec<ExtensionInfo>, String> {
    let registry = state.extensions.lock().map_err(|e| e.to_string())?;
    Ok(registry
        .extensions
        .iter()
        .map(|ext| ExtensionInfo {
            manifest: ext.manifest.clone(),
            enabled: ext.enabled,
            dir: ext.dir.to_string_lossy().to_string(),
            requires_status: ext
                .requires_status
                .iter()
                .map(|r| RequiresStatusInfo {
                    name: r.name.clone(),
                    found: r.found,
                    path: r.path.clone(),
                })
                .collect(),
        })
        .collect())
}

#[tauri::command]
pub fn get_extension_badge(
    state: State<'_, AppState>,
    extension_id: String,
    tab_id: String,
    feature_id: String,
) -> Result<i64, String> {
    let registry = state.extensions.lock().map_err(|e| e.to_string())?;
    let ext = registry
        .extensions
        .iter()
        .find(|e| e.manifest.id == extension_id && e.enabled)
        .ok_or_else(|| format!("Extension '{}' not found or disabled", extension_id))?;
    let tab = ext
        .manifest
        .tabs
        .iter()
        .find(|t| t.id == tab_id)
        .ok_or_else(|| format!("Tab '{}' not found in extension '{}'", tab_id, extension_id))?;
    let query = match &tab.badge_query {
        Some(q) => q.clone(),
        None => return Ok(0),
    };
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let count: i64 = conn
        .query_row(&query, rusqlite::params![feature_id], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    Ok(count)
}
```

- [ ] **Step 3: Add to `src-tauri/src/commands/mod.rs`**

Add at the end of the module list and pub use list:

```rust
mod extensions;
// ...
pub use extensions::*;
```

- [ ] **Step 4: Register commands in `src-tauri/src/lib.rs`**

In the `invoke_handler!` macro call, add:

```rust
commands::get_extensions,
commands::get_extension_badge,
```

- [ ] **Step 5: Compile check**

```
cd src-tauri && cargo check 2>&1 | tail -10
```

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/commands/extensions.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat(extensions): add get_extensions and get_extension_badge Tauri commands"
```

---

## Task 9: MCP Dynamic Tool Registration

**Files:**
- Modify: `src-tauri/src/bin/fh_mcp.rs`

- [ ] **Step 1: Add `extension_registry` field to `FeatureHubMcp`**

In `src-tauri/src/bin/fh_mcp.rs`, add `use feature_hub::extensions;` to the imports, then add the field to the struct:

```rust
use feature_hub::{config, db, extensions};
```

Find `struct FeatureHubMcp` and add the field:

```rust
#[derive(Clone)]
struct FeatureHubMcp {
    db: std::sync::Arc<Mutex<Connection>>,
    default_feature_id: Option<String>,
    claude_session_id: Option<String>,
    storage_path: std::path::PathBuf,
    feature_context: Option<String>,
    tool_router: ToolRouter<Self>,
    extension_registry: extensions::ExtensionRegistry,  // NEW
}
```

- [ ] **Step 2: Initialize `extension_registry` in `FeatureHubMcp::new`**

Inside the `#[tool_router] impl FeatureHubMcp` block, find `fn new(...)` and add loading:

```rust
fn new(conn: Connection, default_feature_id: Option<String>, claude_session_id: Option<String>, storage_path: std::path::PathBuf) -> Self {
    let feature_context = default_feature_id.as_ref().and_then(|id| {
        build_feature_context(&conn, id, claude_session_id.as_deref(), &storage_path).ok()
    });
    let extension_registry = extensions::ExtensionRegistry::load_from_dir(&storage_path.join("extensions"));
    Self {
        db: std::sync::Arc::new(Mutex::new(conn)),
        default_feature_id,
        claude_session_id,
        storage_path,
        feature_context,
        tool_router: Self::tool_router(),
        extension_registry,
    }
}
```

- [ ] **Step 3: Replace `#[tool_handler]` with manual `ServerHandler` implementation**

Find the block:
```rust
#[tool_handler]
impl ServerHandler for FeatureHubMcp {
    fn get_info(&self) -> ServerInfo {
        // ...
    }
}
```

Remove the `#[tool_handler]` attribute and add `list_tools`, `call_tool`, and `get_tool` implementations after `get_info`:

```rust
impl ServerHandler for FeatureHubMcp {
    fn get_info(&self) -> ServerInfo {
        // ... existing implementation unchanged ...
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        async move {
            let mut tools = self.tool_router.list_all();

            for ext in &self.extension_registry.extensions {
                if !ext.enabled {
                    continue;
                }
                for tool_decl in &ext.manifest.tools {
                    let mut props = serde_json::Map::new();
                    let mut required_params: Vec<String> = Vec::new();
                    for (param_name, param_decl) in &tool_decl.params {
                        let mut prop = serde_json::json!({ "type": param_decl.param_type });
                        if let Some(ref desc) = param_decl.description {
                            prop["description"] = serde_json::Value::String(desc.clone());
                        }
                        props.insert(param_name.clone(), prop);
                        if param_decl.required {
                            required_params.push(param_name.clone());
                        }
                    }
                    let schema = serde_json::json!({
                        "type": "object",
                        "properties": props,
                        "required": required_params
                    });
                    let input_schema = std::sync::Arc::new(
                        schema.as_object().unwrap().clone(),
                    );
                    tools.push(rmcp::model::Tool {
                        name: tool_decl.name.clone().into(),
                        description: Some(tool_decl.description.clone().into()),
                        input_schema,
                        output_schema: None,
                        annotations: None,
                    });
                }
            }

            Ok(ListToolsResult::with_all_items(tools))
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        async move {
            let tool_name = request.name.as_ref();

            // Check extension tools first
            if let Some((ext, tool_decl)) = self.extension_registry.find_tool(tool_name) {
                let script_path = ext.dir.join(&tool_decl.handler);
                let db_path = self
                    .storage_path
                    .join("feature-hub.db")
                    .to_string_lossy()
                    .to_string();
                let sp = self.storage_path.to_string_lossy().to_string();
                let params = request.arguments.unwrap_or_default();
                let input = extensions::script_runner::ScriptInput {
                    params,
                    db_path,
                    storage_path: sp,
                    feature_id: self.default_feature_id.clone(),
                };
                let timeout = tool_decl.timeout_secs.unwrap_or(10);
                return tokio::task::spawn_blocking(move || {
                    extensions::script_runner::run_blocking(&script_path, &input, timeout)
                })
                .await
                .map_err(|e| McpError::internal_error(e.to_string(), None))?
                .map(|data| {
                    let text = serde_json::to_string(&data).unwrap_or_default();
                    CallToolResult::success(vec![Content::text(text)])
                })
                .map_err(|e| McpError::internal_error(e, None));
            }

            // Fall through to static tool router
            let tool_context = rmcp::handler::server::tool::ToolCallContext::new(self, request, context);
            self.tool_router.call(tool_context).await
        }
    }

    fn get_tool(&self, name: &str) -> Option<rmcp::model::Tool> {
        self.tool_router.get(name).cloned()
    }
}
```

- [ ] **Step 4: Compile check**

```
cd src-tauri && cargo check --bin fh-mcp 2>&1 | tail -20
```
Expected: no errors. Fix any lifetime or type issues reported.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/bin/fh_mcp.rs
git commit -m "feat(extensions): add dynamic extension tool registration to fh-mcp"
```

---

## Task 10: Frontend Types and API

**Files:**
- Modify: `src/lib/api/types.ts`
- Create: `src/lib/api/extensions.ts`
- Modify: `src/lib/api/tauri.ts`

- [ ] **Step 1: Write the failing test**

Create `src/lib/api/extensions.test.ts`:

```typescript
import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock the Tauri invoke
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { getExtensions, getExtensionBadge } from "./extensions";

describe("extensions API", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("getExtensions calls get_extensions command", async () => {
    (invoke as any).mockResolvedValue([]);
    const result = await getExtensions();
    expect(invoke).toHaveBeenCalledWith("get_extensions");
    expect(result).toEqual([]);
  });

  it("getExtensionBadge calls get_extension_badge with correct params", async () => {
    (invoke as any).mockResolvedValue(3);
    const count = await getExtensionBadge("my-ext", "prs", "feature-123");
    expect(invoke).toHaveBeenCalledWith("get_extension_badge", {
      extensionId: "my-ext",
      tabId: "prs",
      featureId: "feature-123",
    });
    expect(count).toBe(3);
  });
});
```

- [ ] **Step 2: Run test to confirm it fails**

```
npm run test -- extensions.test 2>&1 | tail -10
```
Expected: module not found or type errors.

- [ ] **Step 3: Add types to `src/lib/api/types.ts`**

Add at the end of the file:

```typescript
export interface InstalledExtensionParamDecl {
  type: string;
  required: boolean;
  description?: string | null;
}

export interface InstalledExtensionToolDecl {
  name: string;
  description: string;
  handler: string;
  params: Record<string, InstalledExtensionParamDecl>;
  timeout_secs?: number | null;
}

export interface InstalledExtensionTabDecl {
  id: string;
  label: string;
  emoji: string;
  sort_order: number;
  component: string;
  badge_query?: string | null;
}

export interface InstalledExtensionManifest {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  requires: string[];
  tools: InstalledExtensionToolDecl[];
  tabs: InstalledExtensionTabDecl[];
  instructions: string;
}

export interface RequiresStatusInfo {
  name: string;
  found: boolean;
  path: string | null;
}

export interface ExtensionInfo {
  manifest: InstalledExtensionManifest;
  enabled: boolean;
  dir: string;
  requires_status: RequiresStatusInfo[];
}
```

- [ ] **Step 4: Create `src/lib/api/extensions.ts`**

```typescript
import { invoke } from "@tauri-apps/api/core";
import type { ExtensionInfo } from "./types";

export async function getExtensions(): Promise<ExtensionInfo[]> {
  return invoke<ExtensionInfo[]>("get_extensions");
}

export async function getExtensionBadge(
  extensionId: string,
  tabId: string,
  featureId: string
): Promise<number> {
  return invoke<number>("get_extension_badge", { extensionId, tabId, featureId });
}
```

- [ ] **Step 5: Add export to `src/lib/api/tauri.ts`**

```typescript
export * from "./extensions";
```

- [ ] **Step 6: Run tests**

```
npm run test -- extensions.test 2>&1 | tail -10
```
Expected: `2 tests passed`

- [ ] **Step 7: Commit**

```bash
git add src/lib/api/types.ts src/lib/api/extensions.ts src/lib/api/tauri.ts
git commit -m "feat(extensions): add frontend types and API for extension system"
```

---

## Task 11: ExtensionTabFrame Component

**Files:**
- Create: `src/lib/modules/extensions/ExtensionTabFrame.svelte`

- [ ] **Step 1: Create `src/lib/modules/extensions/ExtensionTabFrame.svelte`**

```svelte
<script lang="ts">
  import type { TabContext } from "../registry";

  let { ctx, componentPath }: { ctx: TabContext; componentPath: string } = $props();

  let iframeEl = $state<HTMLIFrameElement | null>(null);
  let initialized = $state(false);

  // Build the asset URL for the iframe. On Windows, paths use backslashes which
  // must be converted to forward slashes for asset:// URLs.
  const assetUrl = `asset://${componentPath.replace(/\\/g, "/")}`;

  function handleLoad() {
    if (!iframeEl?.contentWindow) return;
    iframeEl.contentWindow.postMessage(
      {
        type: "fh:init",
        featureId: ctx.featureId,
        feature: ctx.feature,
      },
      "*"
    );
    initialized = true;
  }

  function handleMessage(event: MessageEvent) {
    if (event.data?.type !== "fh:invoke") return;
    const { command, params, requestId } = event.data;
    if (!command || !requestId) return;

    import("@tauri-apps/api/core")
      .then(({ invoke }) => invoke(command, params ?? {}))
      .then((data) => {
        iframeEl?.contentWindow?.postMessage(
          { type: "fh:invoke-result", requestId, ok: true, data },
          "*"
        );
      })
      .catch((err) => {
        iframeEl?.contentWindow?.postMessage(
          {
            type: "fh:invoke-result",
            requestId,
            ok: false,
            error: String(err),
          },
          "*"
        );
      });
  }
</script>

<svelte:window onmessage={handleMessage} />

<iframe
  bind:this={iframeEl}
  src={assetUrl}
  onload={handleLoad}
  title="Extension tab"
  style="width: 100%; height: 100%; border: none; background: transparent;"
  sandbox="allow-scripts allow-same-origin"
></iframe>
```

- [ ] **Step 2: Verify the component builds cleanly**

```
npm run build 2>&1 | tail -20
```
Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/modules/extensions/ExtensionTabFrame.svelte
git commit -m "feat(extensions): add ExtensionTabFrame iframe component with postMessage bridge"
```

---

## Task 12: Dynamic Tab Registration

**Files:**
- Modify: `src/lib/modules/index.ts`
- Create: `src/lib/modules/extensions/index.ts`

- [ ] **Step 1: Write the failing test**

Create `src/lib/modules/extensions/extensions-tabs.test.ts`:

```typescript
import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("../registry", () => {
  const tabs: any[] = [];
  return {
    registerTab: vi.fn((mod: any) => tabs.push(mod)),
    getRegisteredTabs: vi.fn(() => tabs),
  };
});

import { invoke } from "@tauri-apps/api/core";
import { registerTab, getRegisteredTabs } from "../registry";
import { registerExtensionTabs } from "./index";
import type { ExtensionInfo } from "../../api/types";

const mockExt: ExtensionInfo = {
  manifest: {
    id: "my-ext",
    name: "My Extension",
    version: "1.0.0",
    description: "",
    author: "",
    requires: [],
    tools: [],
    tabs: [
      {
        id: "prs",
        label: "PRs",
        emoji: "🔀",
        sort_order: 350,
        component: "/path/to/tab.html",
        badge_query: null,
      },
    ],
    instructions: "",
  },
  enabled: true,
  dir: "/path/to/ext",
  requires_status: [],
};

describe("registerExtensionTabs", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (invoke as any).mockResolvedValue([mockExt]);
  });

  it("registers a tab for each enabled extension tab", async () => {
    await registerExtensionTabs();
    expect(registerTab).toHaveBeenCalledTimes(1);
    const call = (registerTab as any).mock.calls[0][0];
    expect(call.id).toBe("my-ext:prs");
    expect(call.label).toBe("PRs");
    expect(call.emoji).toBe("🔀");
    expect(call.sortOrder).toBe(350);
  });

  it("does not register tabs for disabled extensions", async () => {
    (invoke as any).mockResolvedValue([{ ...mockExt, enabled: false }]);
    await registerExtensionTabs();
    expect(registerTab).not.toHaveBeenCalled();
  });
});
```

- [ ] **Step 2: Run test to confirm it fails**

```
npm run test -- extensions-tabs.test 2>&1 | tail -10
```

- [ ] **Step 3: Create `src/lib/modules/extensions/index.ts`**

```typescript
import { registerTab } from "../registry";
import { getExtensions, getExtensionBadge } from "../../api/extensions";
import type { TabContext } from "../registry";
import ExtensionTabFrame from "./ExtensionTabFrame.svelte";
import type { ExtensionInfo } from "../../api/types";

export async function registerExtensionTabs(): Promise<void> {
  let extensions: ExtensionInfo[];
  try {
    extensions = await getExtensions();
  } catch (e) {
    console.error("[extensions] Failed to load extensions:", e);
    return;
  }

  for (const ext of extensions) {
    if (!ext.enabled) continue;
    for (const tabDecl of ext.manifest.tabs) {
      const componentPath = `${ext.dir}/${tabDecl.component}`.replace(/\\/g, "/");
      registerTab({
        id: `${ext.manifest.id}:${tabDecl.id}`,
        label: tabDecl.label,
        emoji: tabDecl.emoji,
        shortcutKey: "", // no keyboard shortcut for extension tabs
        sortOrder: tabDecl.sort_order,
        component: ExtensionTabFrame as any,
        // Note: getBadges is synchronous but extension badge counts require async
        // evaluation via get_extension_badge. Tab bar badges are not shown for
        // extension tabs in this first pass — the iframe can display its own counts.
        getBadges: () => [],
        panelStyle: "padding: 0;",
      });
    }
  }
}
```

- [ ] **Step 4: Run tests**

```
npm run test -- extensions-tabs.test 2>&1 | tail -10
```
Expected: `2 tests passed`

- [ ] **Step 5: Modify `src/lib/modules/index.ts` to call `registerExtensionTabs`**

Change `src/lib/modules/index.ts` from:

```typescript
// Side-effect imports — each module registers itself on import
import "./ai";
import "./links";
import "./repos";
import "./tasks-notes";
import "./files";
import "./timeline";
```

to:

```typescript
// Side-effect imports — each module registers itself on import
import "./ai";
import "./links";
import "./repos";
import "./tasks-notes";
import "./files";
import "./timeline";

// Dynamically register extension tabs at startup
import { registerExtensionTabs } from "./extensions";
export const extensionTabsReady: Promise<void> = registerExtensionTabs();
```

- [ ] **Step 6: In `src/App.svelte`, await `extensionTabsReady` before rendering tabs**

Find where tabs are initialized in `App.svelte` (search for `getRegisteredTabs` or `modules/index`). Import `extensionTabsReady` and `await` it before first render. If `App.svelte` uses an `onMount`, add:

```typescript
import { extensionTabsReady } from "./lib/modules/index";

onMount(async () => {
  await extensionTabsReady;
  // rest of existing onMount logic
});
```

If `App.svelte` does not have an `onMount` and tabs are rendered statically, wrap the relevant reactive state with a guard. Check how `getRegisteredTabs()` is called and ensure it's called only after the promise resolves.

- [ ] **Step 7: Build check**

```
npm run build 2>&1 | tail -20
```

- [ ] **Step 8: Commit**

```bash
git add src/lib/modules/extensions/ src/lib/modules/index.ts src/lib/components/App.svelte
git commit -m "feat(extensions): register extension tabs dynamically at app startup"
```

---

## Task 13: Settings UI — Installed Extensions

**Files:**
- Modify: `src/lib/components/SettingsModal.svelte`

- [ ] **Step 1: Write the failing test**

Create `src/lib/components/ExtensionsSettings.test.ts`:

```typescript
import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/svelte";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([
    {
      manifest: {
        id: "github-prs", name: "GitHub PRs", version: "1.0.0",
        description: "Track PRs", author: "", requires: ["gh"],
        tools: [], tabs: [], instructions: "",
      },
      enabled: true,
      dir: "/storage/extensions/github-prs",
      requires_status: [{ name: "gh", found: true, path: "/usr/bin/gh" }],
    }
  ]),
}));

// Minimal test — just verify the installed extension name renders.
// Full SettingsModal is complex; we test the key rendering path.
import InstalledExtensionsPanel from "./InstalledExtensionsPanel.svelte";

describe("InstalledExtensionsPanel", () => {
  it("renders installed extension name and requires status", async () => {
    render(InstalledExtensionsPanel);
    // Wait for async load
    await vi.waitFor(() => screen.getByText("GitHub PRs"));
    expect(screen.getByText("GitHub PRs")).toBeTruthy();
    expect(screen.getByText(/gh/)).toBeTruthy();
  });
});
```

- [ ] **Step 2: Create `src/lib/components/InstalledExtensionsPanel.svelte`**

Extract the installed-extensions UI into its own component so `SettingsModal.svelte` stays focused:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { getExtensions } from "../api/extensions";
  import type { ExtensionInfo } from "../api/types";

  let installedExtensions = $state<ExtensionInfo[]>([]);
  let loading = $state(true);

  onMount(async () => {
    try {
      installedExtensions = await getExtensions();
    } catch (e) {
      console.error("[extensions] Failed to load installed extensions:", e);
    } finally {
      loading = false;
    }
  });
</script>

{#if !loading && installedExtensions.length > 0}
  <div class="settings-section">
    <div class="settings-section-title">Installed Extensions</div>
    {#each installedExtensions as ext}
      <div class="extension-card" class:extension-card--enabled={ext.enabled}>
        <div class="extension-card__header">
          <div class="extension-card__info">
            <span class="extension-card__name">{ext.manifest.name}</span>
            <span class="extension-card__badge" style="background: var(--bg3); color: var(--fg2); font-size: 10px; padding: 2px 6px; border-radius: 4px;">v{ext.manifest.version}</span>
            <span class="extension-card__badge">Installed</span>
          </div>
        </div>
        {#if ext.manifest.description}
          <div class="extension-card__desc">{ext.manifest.description}</div>
        {/if}
        {#if ext.requires_status.length > 0}
          <div class="extension-card__body">
            <div class="extension-card__section" style="font-size: 12px; color: var(--fg2);">
              <strong>Requires:</strong>
              {#each ext.requires_status as req}
                <span style="margin-left: 8px;">
                  {#if req.found}
                    <span style="color: var(--green);">✓</span>
                  {:else}
                    <span style="color: var(--red);">✗</span>
                  {/if}
                  <code>{req.name}</code>
                </span>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/each}
  </div>
{/if}
```

- [ ] **Step 3: Run test to confirm it fails**

```
npm run test -- ExtensionsSettings.test 2>&1 | tail -10
```
Expected: module not found.

- [ ] **Step 4: Add `InstalledExtensionsPanel` to `SettingsModal.svelte`**

In `src/lib/components/SettingsModal.svelte`, at the top of the `<script>` block, add the import:

```typescript
import InstalledExtensionsPanel from "./InstalledExtensionsPanel.svelte";
```

In the extensions tab section (around line 785, inside `{#if activeTab === "extensions"}`), add the panel after the existing extensions grid:

```svelte
<InstalledExtensionsPanel />
```

- [ ] **Step 5: Run test**

```
npm run test -- ExtensionsSettings.test 2>&1 | tail -10
```
Expected: test passes.

- [ ] **Step 6: Build check**

```
npm run build 2>&1 | tail -10
```

- [ ] **Step 7: Commit**

```bash
git add src/lib/components/InstalledExtensionsPanel.svelte src/lib/components/SettingsModal.svelte
git commit -m "feat(extensions): show installed directory extensions in settings UI"
```

---

## Task 14: Manual Integration Test

> This task verifies the end-to-end flow with a real fixture extension.

- [ ] **Step 1: Create a fixture extension in the active storage**

In your active storage directory (find it in FeatureHub settings), create:

```
<storage>/extensions/fixture-ext/extension.json
<storage>/extensions/fixture-ext/handlers/hello.js
```

`extension.json`:
```json
{
  "id": "fixture-ext",
  "name": "Fixture Extension",
  "version": "1.0.0",
  "description": "Test fixture for integration testing",
  "tools": [
    {
      "name": "fixture_hello",
      "description": "Returns a greeting",
      "handler": "handlers/hello.js",
      "params": {
        "name": { "type": "string", "required": true }
      }
    }
  ],
  "events": [
    {
      "on": "link_created",
      "filter": {},
      "handler": "handlers/hello.js"
    }
  ]
}
```

`handlers/hello.js`:
```javascript
const chunks = [];
process.stdin.on('data', c => chunks.push(c));
process.stdin.on('end', () => {
  const input = JSON.parse(Buffer.concat(chunks).toString());
  process.stdout.write(JSON.stringify({
    ok: true,
    data: { message: `Hello from fixture! params: ${JSON.stringify(input.params)}` }
  }));
});
```

- [ ] **Step 2: Start the app and verify extension appears in settings**

Run `npm run tauri dev`. Open Settings → Extensions. Verify "Fixture Extension" appears under "Installed Extensions" with version "1.0.0".

- [ ] **Step 3: Verify tool appears in MCP via Claude session**

Start a Claude session on any feature via `fh start`. In Claude Code, run `/mcp` and check that `fixture_hello` appears in the fh-mcp tool list.

- [ ] **Step 4: Invoke the tool and verify it runs**

In the Claude session, ask Claude to call `fixture_hello` with `name: "world"`. Verify the response contains `Hello from fixture!`.

- [ ] **Step 5: Verify link_created hook fires**

Add any link to the feature. Check the terminal where the app is running for `[ext:fixture-ext]` log output (or absence of error). Add `console.error` to the script to confirm it fires.

- [ ] **Step 6: Commit integration test fixture**

```bash
# Don't commit the storage fixture itself — it's data, not code
# Just verify the flow works and proceed
```

---

## Self-Review Checklist

- [ ] All spec requirements covered: manifest format, table provisioning, MCP tools, event hooks, tab registration, requires checks, Settings UI, error handling
- [ ] No `#[tool_handler]` attribute remains on the `ServerHandler` impl after Task 9
- [ ] `dispatch_event` is called from `add_link` only (not all link mutations — spec says `link_created` is the event)
- [ ] `ExtensionRegistry` is `Clone` (derived) — required for `Mutex<ExtensionRegistry>` in `AppState`
- [ ] The `event_filter_matches` function is not `pub` — only used internally in `mod.rs`
- [ ] `provision_table` (singular) is `pub` — used from `commands/storage.rs`
- [ ] Extension tab IDs use `"<ext_id>:<tab_id>"` format to avoid collisions with built-in tab IDs
- [ ] `node` binary required on path for script execution — the `requires` field handles this check
