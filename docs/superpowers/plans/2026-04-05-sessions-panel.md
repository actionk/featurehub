# Sessions Panel Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a persistent 260px right-side panel to the app shell that lists all Claude sessions across all features globally, with active sessions highlighted, JSONL-derived model/token stats, and click-to-open-terminal behaviour.

**Architecture:** Extend the existing `sessionActivity` Svelte store to also fetch a new `get_sessions_panel_data` Tauri command on each 10s poll. The command reads all sessions from the DB (joined with feature names), determines which are active via process scan, and enriches each with JSONL-parsed stats (model, token count, cost) cached by file mtime. A new `SessionsPanel.svelte` component renders the data and a new icon rail button toggles it.

**Tech Stack:** Rust (rusqlite, serde, serde_json, chrono, dirs), Svelte 5 runes, TypeScript, CSS custom properties in `src/app.css`.

---

## File Map

| File | Action | Responsibility |
|------|--------|---------------|
| `src-tauri/src/claude/session_parser.rs` | Modify | Add `CachedStats` struct + `parse_session_stats()` |
| `src-tauri/src/claude/scanner.rs` | Modify | Add `find_jsonl_for_session()` helper |
| `src-tauri/src/db/sessions.rs` | Modify | Add `PanelSessionRow` struct + `get_all_sessions_for_panel()` |
| `src-tauri/src/lib.rs` | Modify | Add `stats_cache` field to `AppState`, register new command |
| `src-tauri/src/commands/sessions.rs` | Modify | Add `PanelSession`, `SessionsPanelData`, `get_sessions_panel_data` |
| `src/lib/api/types.ts` | Modify | Add `PanelSession` and `SessionsPanelData` interfaces |
| `src/lib/api/sessions.ts` | Modify | Add `getSessionsPanelData()` wrapper |
| `src/lib/stores/sessionActivity.svelte.ts` | Modify | Add panel state, fetch panel data in refresh loop |
| `src/app.css` | Modify | Add sessions panel CSS |
| `src/lib/components/SessionsPanel.svelte` | Create | Panel UI component |
| `src/App.svelte` | Modify | Icon rail button + panel toggle + render SessionsPanel |

---

## Task 1: Add `CachedStats` and `parse_session_stats` to `session_parser.rs`

**Files:**
- Modify: `src-tauri/src/claude/session_parser.rs`

- [ ] **Step 1: Write the failing tests**

Add this `#[cfg(test)]` block at the bottom of `src-tauri/src/claude/session_parser.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp_jsonl(content: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f.flush().unwrap();
        f
    }

    #[test]
    fn test_parse_session_stats_empty_file() {
        let f = write_temp_jsonl("");
        let stats = parse_session_stats(f.path());
        assert_eq!(stats.total_tokens, 0);
        assert!(stats.model.is_none());
        assert!(stats.cost_usd.is_none());
    }

    #[test]
    fn test_parse_session_stats_extracts_model_and_tokens() {
        let line = r#"{"type":"assistant","message":{"model":"claude-opus-4-6","usage":{"input_tokens":100,"output_tokens":50}}}"#;
        let f = write_temp_jsonl(&format!("{}\n", line));
        let stats = parse_session_stats(f.path());
        assert_eq!(stats.model.as_deref(), Some("claude-opus-4-6"));
        assert_eq!(stats.total_tokens, 150);
        assert!(stats.cost_usd.is_none());
    }

    #[test]
    fn test_parse_session_stats_accumulates_tokens_across_messages() {
        let line1 = r#"{"type":"assistant","message":{"model":"claude-opus-4-6","usage":{"input_tokens":100,"output_tokens":50}}}"#;
        let line2 = r#"{"type":"assistant","message":{"model":"claude-opus-4-6","usage":{"input_tokens":200,"output_tokens":80}}}"#;
        let f = write_temp_jsonl(&format!("{}\n{}\n", line1, line2));
        let stats = parse_session_stats(f.path());
        assert_eq!(stats.total_tokens, 430);
    }

    #[test]
    fn test_parse_session_stats_accumulates_cost() {
        let line1 = r#"{"type":"assistant","costUSD":0.05,"message":{"model":"claude-opus-4-6","usage":{"input_tokens":10,"output_tokens":5}}}"#;
        let line2 = r#"{"type":"assistant","costUSD":0.03,"message":{"model":"claude-opus-4-6","usage":{"input_tokens":10,"output_tokens":5}}}"#;
        let f = write_temp_jsonl(&format!("{}\n{}\n", line1, line2));
        let stats = parse_session_stats(f.path());
        assert!((stats.cost_usd.unwrap() - 0.08).abs() < 1e-6);
    }

    #[test]
    fn test_parse_session_stats_skips_non_assistant_lines() {
        let content = r#"{"type":"user","message":{"role":"user","content":"hello"}}
{"type":"assistant","message":{"model":"claude-sonnet-4-6","usage":{"input_tokens":5,"output_tokens":10}}}
{"type":"progress"}
"#;
        let f = write_temp_jsonl(content);
        let stats = parse_session_stats(f.path());
        assert_eq!(stats.total_tokens, 15);
        assert_eq!(stats.model.as_deref(), Some("claude-sonnet-4-6"));
    }
}
```

- [ ] **Step 2: Add `tempfile` dev-dependency**

In `src-tauri/Cargo.toml`, add under `[dev-dependencies]`:
```toml
tempfile = "3"
```

- [ ] **Step 3: Run tests to verify they fail**

```bash
cd src-tauri && cargo test --lib claude::session_parser::tests 2>&1 | tail -20
```

Expected: compile error — `parse_session_stats` and `CachedStats` not defined yet.

- [ ] **Step 4: Add `CachedStats` and `parse_session_stats` to `session_parser.rs`**

Add this after the existing imports at the top of `src-tauri/src/claude/session_parser.rs`:

```rust
#[derive(Clone, Debug)]
pub struct CachedStats {
    pub mtime: std::time::SystemTime,
    pub model: Option<String>,
    pub total_tokens: u64,
    pub cost_usd: Option<f64>,
}

/// Parse model, token counts, and cost from a Claude JSONL transcript.
/// Skips files larger than 100 MB. Results should be cached by mtime.
pub fn parse_session_stats(path: &std::path::Path) -> CachedStats {
    use std::io::{BufRead, BufReader};

    let mtime = std::fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

    // Skip very large files to prevent memory pressure
    if let Ok(meta) = std::fs::metadata(path) {
        if meta.len() > 100 * 1024 * 1024 {
            return CachedStats { mtime, model: None, total_tokens: 0, cost_usd: None };
        }
    }

    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return CachedStats { mtime, model: None, total_tokens: 0, cost_usd: None },
    };

    let reader = BufReader::new(file);
    let mut model: Option<String> = None;
    let mut total_tokens: u64 = 0;
    let mut cost_usd: Option<f64> = None;

    for line in reader.lines() {
        let line = match line {
            Ok(l) if !l.is_empty() => l,
            _ => continue,
        };

        // Quick pre-filter before JSON parsing
        if !line.contains("\"type\":\"assistant\"") {
            continue;
        }

        let val: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if val.get("type").and_then(|v| v.as_str()) != Some("assistant") {
            continue;
        }

        let msg = match val.get("message") {
            Some(m) => m,
            None => continue,
        };

        // Extract model from first assistant message
        if model.is_none() {
            if let Some(m) = msg.get("model").and_then(|v| v.as_str()) {
                model = Some(m.to_string());
            }
        }

        // Accumulate token counts
        if let Some(usage) = msg.get("usage") {
            let input = usage.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            let output = usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            total_tokens = total_tokens.saturating_add(input + output);
        }

        // Accumulate cost if present
        if let Some(cost) = val.get("costUSD").and_then(|v| v.as_f64()) {
            *cost_usd.get_or_insert(0.0) += cost;
        }
    }

    CachedStats { mtime, model, total_tokens, cost_usd }
}
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cd src-tauri && cargo test --lib claude::session_parser::tests 2>&1 | tail -20
```

Expected: all 5 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/claude/session_parser.rs src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "feat: add parse_session_stats to session_parser"
```

---

## Task 2: Add `find_jsonl_for_session` to `scanner.rs`

**Files:**
- Modify: `src-tauri/src/claude/scanner.rs`

- [ ] **Step 1: Add the function**

Add this after the existing `find_project_path_for_session` function in `src-tauri/src/claude/scanner.rs`:

```rust
/// Find the JSONL transcript file for a given session ID.
/// Searches one level deep under `projects_dir` for `<session_id>.jsonl`.
pub fn find_jsonl_for_session(
    projects_dir: &std::path::Path,
    session_id: &str,
) -> Option<std::path::PathBuf> {
    let filename = format!("{}.jsonl", session_id);
    for entry in std::fs::read_dir(projects_dir).ok()?.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let jsonl = dir.join(&filename);
        if jsonl.exists() {
            return Some(jsonl);
        }
    }
    None
}
```

- [ ] **Step 2: Verify it compiles**

```bash
cd src-tauri && cargo check 2>&1 | grep -E "error|warning: unused" | head -20
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/claude/scanner.rs
git commit -m "feat: add find_jsonl_for_session to scanner"
```

---

## Task 3: Add `get_all_sessions_for_panel` to `db/sessions.rs`

**Files:**
- Modify: `src-tauri/src/db/sessions.rs`

- [ ] **Step 1: Write the failing test**

Add this test inside the existing `#[cfg(test)]` block in `src-tauri/src/db/sessions.rs` (or create one if absent):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_utils::test_db;

    #[test]
    fn test_get_all_sessions_for_panel_joins_feature_names() {
        let conn = test_db();

        // Create a feature
        conn.execute(
            "INSERT INTO features (id, title, status, sort_order) VALUES ('f1', 'My Feature', 'active', 0)",
            [],
        ).unwrap();

        // Create a session with a branch
        conn.execute(
            "INSERT INTO sessions (id, feature_id, claude_session_id, branch, linked_at)
             VALUES ('s1', 'f1', 'claude-abc', 'main', '2026-01-01T00:00:00Z')",
            [],
        ).unwrap();

        let rows = get_all_sessions_for_panel(&conn).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "s1");
        assert_eq!(rows[0].feature_name, "My Feature");
        assert_eq!(rows[0].claude_session_id, "claude-abc");
        assert_eq!(rows[0].branch.as_deref(), Some("main"));
    }

    #[test]
    fn test_get_all_sessions_for_panel_excludes_empty_claude_session_id() {
        let conn = test_db();
        conn.execute(
            "INSERT INTO features (id, title, status, sort_order) VALUES ('f1', 'F', 'active', 0)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO sessions (id, feature_id, claude_session_id, linked_at)
             VALUES ('s1', 'f1', '', '2026-01-01T00:00:00Z')",
            [],
        ).unwrap();

        let rows = get_all_sessions_for_panel(&conn).unwrap();
        assert!(rows.is_empty());
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

```bash
cd src-tauri && cargo test --lib db::sessions::tests 2>&1 | tail -20
```

Expected: compile error — `get_all_sessions_for_panel` and `PanelSessionRow` not defined.

- [ ] **Step 3: Add `PanelSessionRow` struct and `get_all_sessions_for_panel`**

Add after the `RecentSession` struct and `get_recent_sessions` function in `src-tauri/src/db/sessions.rs`:

```rust
#[derive(Debug, serde::Serialize)]
pub struct PanelSessionRow {
    pub id: String,
    pub feature_id: String,
    pub feature_name: String,
    pub claude_session_id: String,
    pub branch: Option<String>,
    pub linked_at: String,
}

/// Load all sessions with their feature name for the sessions panel.
/// Excludes sessions with empty `claude_session_id`.
/// Ordered by `linked_at` descending (newest first).
pub fn get_all_sessions_for_panel(conn: &Connection) -> Result<Vec<PanelSessionRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.feature_id, f.title, s.claude_session_id, s.branch, s.linked_at
             FROM sessions s
             JOIN features f ON f.id = s.feature_id
             WHERE s.claude_session_id != ''
             ORDER BY s.linked_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(PanelSessionRow {
                id: row.get(0)?,
                feature_id: row.get(1)?,
                feature_name: row.get(2)?,
                claude_session_id: row.get(3)?,
                branch: row.get(4)?,
                linked_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cd src-tauri && cargo test --lib db::sessions::tests 2>&1 | tail -20
```

Expected: both new tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/db/sessions.rs
git commit -m "feat: add get_all_sessions_for_panel db query"
```

---

## Task 4: Add `stats_cache` to `AppState` in `lib.rs`

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add the field to `AppState`**

Change the `AppState` struct in `src-tauri/src/lib.rs` from:

```rust
pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub storage_path: Mutex<Option<PathBuf>>,
    pub extensions: Mutex<extensions::ExtensionRegistry>,
}
```

to:

```rust
pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub storage_path: Mutex<Option<PathBuf>>,
    pub extensions: Mutex<extensions::ExtensionRegistry>,
    pub stats_cache: Mutex<std::collections::HashMap<String, claude::session_parser::CachedStats>>,
}
```

- [ ] **Step 2: Initialize the field in both `AppState` construction sites**

There are two places where `AppState { ... }` is constructed in `lib.rs` (one for active storage, one for the in-memory fallback). Both are inside the `setup` closure. Find the single `AppState { ... }` block (search for `let state = AppState {`) and add the new field:

```rust
let state = AppState {
    db: Mutex::new(conn),
    storage_path: Mutex::new(storage_path),
    extensions: Mutex::new(extension_registry),
    stats_cache: Mutex::new(std::collections::HashMap::new()),
};
```

- [ ] **Step 3: Verify it compiles**

```bash
cd src-tauri && cargo check 2>&1 | grep "error" | head -20
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: add stats_cache to AppState"
```

---

## Task 5: Add `get_sessions_panel_data` Tauri command

**Files:**
- Modify: `src-tauri/src/commands/sessions.rs`
- Modify: `src-tauri/src/lib.rs` (register command)

- [ ] **Step 1: Add the types and command to `commands/sessions.rs`**

Add this block after the `SessionActivity` struct at the top of `src-tauri/src/commands/sessions.rs`:

```rust
#[derive(serde::Serialize, Clone)]
pub struct PanelSession {
    pub id: String,
    pub feature_id: String,
    pub feature_name: String,
    pub claude_session_id: String,
    pub branch: Option<String>,
    pub last_modified: String,
    pub is_active: bool,
    pub model: Option<String>,
    pub total_tokens: Option<u64>,
    pub cost_usd: Option<f64>,
}

#[derive(serde::Serialize)]
pub struct SessionsPanelData {
    pub sessions: Vec<PanelSession>,
    pub active_count: usize,
}
```

Then add this command function before the closing brace of the file:

```rust
/// Returns all sessions enriched with JSONL stats (model, tokens, cost) for the sessions panel.
/// Stats are cached by JSONL file mtime and only re-parsed when the file changes.
#[tauri::command]
pub async fn get_sessions_panel_data(state: State<'_, AppState>) -> Result<SessionsPanelData, String> {
    // 1. Read all sessions with feature names from DB
    let rows = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::sessions::get_all_sessions_for_panel(&conn)?
    };

    // 2. Determine which sessions are active via process scan
    let claude_ids: Vec<String> = rows.iter().map(|r| r.claude_session_id.clone()).collect();
    let active_ids: std::collections::HashSet<String> = {
        let ids = tauri::async_runtime::spawn_blocking(move || {
            claude::scanner::get_active_session_ids(&claude_ids)
        })
        .await
        .map_err(|e| e.to_string())?;
        ids.into_iter().collect()
    };

    // 3. Locate JSONL files, check mtime cache, collect what needs re-parsing
    let home = dirs::home_dir().ok_or("Cannot find home directory")?;
    let projects_dir = home.join(".claude").join("projects");

    // (session_id, path) pairs that are cache misses
    let mut to_parse: Vec<(String, std::path::PathBuf)> = Vec::new();
    // Pre-populate hits from cache
    let mut all_stats: std::collections::HashMap<String, crate::claude::session_parser::CachedStats> = {
        let cache = state.stats_cache.lock().map_err(|e| e.to_string())?;
        let mut hits = std::collections::HashMap::new();
        for row in &rows {
            let jsonl_path =
                claude::scanner::find_jsonl_for_session(&projects_dir, &row.claude_session_id);
            if let Some(ref path) = jsonl_path {
                let current_mtime = std::fs::metadata(path)
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                if let Some(cached) = cache.get(&row.claude_session_id) {
                    if cached.mtime == current_mtime {
                        hits.insert(row.claude_session_id.clone(), cached.clone());
                        continue;
                    }
                }
                to_parse.push((row.claude_session_id.clone(), path.clone()));
            }
        }
        hits
    }; // release cache lock before blocking I/O

    // 4. Parse uncached JSONL files in a blocking task
    if !to_parse.is_empty() {
        let parsed: Vec<(String, crate::claude::session_parser::CachedStats)> =
            tauri::async_runtime::spawn_blocking(move || {
                to_parse
                    .into_iter()
                    .map(|(id, path)| {
                        let stats = crate::claude::session_parser::parse_session_stats(&path);
                        (id, stats)
                    })
                    .collect()
            })
            .await
            .map_err(|e| e.to_string())?;

        // Update cache and merge into all_stats
        let mut cache = state.stats_cache.lock().map_err(|e| e.to_string())?;
        for (id, stats) in parsed {
            cache.insert(id.clone(), stats.clone());
            all_stats.insert(id, stats);
        }
    }

    // 5. Build PanelSession list
    let mut sessions: Vec<PanelSession> = rows
        .iter()
        .map(|row| {
            let is_active = active_ids.contains(&row.claude_session_id);
            let stats = all_stats.get(&row.claude_session_id);

            let last_modified = stats
                .map(|s| {
                    let dt: chrono::DateTime<chrono::Utc> = s.mtime.into();
                    dt.to_rfc3339()
                })
                .unwrap_or_else(|| row.linked_at.clone());

            PanelSession {
                id: row.id.clone(),
                feature_id: row.feature_id.clone(),
                feature_name: row.feature_name.clone(),
                claude_session_id: row.claude_session_id.clone(),
                branch: row.branch.clone(),
                last_modified,
                is_active,
                model: stats.and_then(|s| s.model.clone()),
                total_tokens: stats
                    .map(|s| s.total_tokens)
                    .filter(|&t| t > 0),
                cost_usd: stats.and_then(|s| s.cost_usd),
            }
        })
        .collect();

    // Sort: active first, then by last_modified descending
    sessions.sort_by(|a, b| {
        b.is_active
            .cmp(&a.is_active)
            .then_with(|| b.last_modified.cmp(&a.last_modified))
    });

    let active_count = sessions.iter().filter(|s| s.is_active).count();

    Ok(SessionsPanelData { sessions, active_count })
}
```

- [ ] **Step 2: Add `dirs` dependency to `Cargo.toml` if missing**

Check `src-tauri/Cargo.toml` for `dirs`. If not present, add:
```toml
dirs = "5"
```

(It is likely already present since `scanner.rs` uses `dirs::home_dir()`.)

```bash
grep "dirs" src-tauri/Cargo.toml
```

- [ ] **Step 3: Register the command in `lib.rs`**

In the `invoke_handler` list in `src-tauri/src/lib.rs`, add after `commands::get_active_session_activity,`:

```rust
commands::get_sessions_panel_data,
```

- [ ] **Step 4: Verify it compiles**

```bash
cd src-tauri && cargo check 2>&1 | grep "error" | head -20
```

Expected: no errors.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/sessions.rs src-tauri/src/lib.rs src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "feat: add get_sessions_panel_data Tauri command"
```

---

## Task 6: TypeScript types and API wrapper

**Files:**
- Modify: `src/lib/api/types.ts`
- Modify: `src/lib/api/sessions.ts`

- [ ] **Step 1: Add types to `types.ts`**

Add after the `SessionActivity` interface in `src/lib/api/types.ts`:

```typescript
export interface PanelSession {
  id: string;
  feature_id: string;
  feature_name: string;
  claude_session_id: string;
  branch: string | null;
  last_modified: string;
  is_active: boolean;
  model: string | null;
  total_tokens: number | null;
  cost_usd: number | null;
}

export interface SessionsPanelData {
  sessions: PanelSession[];
  active_count: number;
}
```

- [ ] **Step 2: Add API wrapper to `sessions.ts`**

Add after `getActiveSessionActivity` in `src/lib/api/sessions.ts`:

```typescript
export async function getSessionsPanelData(): Promise<SessionsPanelData> {
  return invoke<SessionsPanelData>("get_sessions_panel_data");
}
```

Also add `SessionsPanelData` to the import at the top of `sessions.ts`:

```typescript
import type { Session, SessionActivity, SessionsPanelData } from "./types";
```

- [ ] **Step 3: Verify TypeScript compiles**

```bash
npm run build 2>&1 | grep -E "error TS|Error" | head -20
```

Expected: no TypeScript errors (Rust backend is not running so Tauri IPC calls will not execute, but types should be correct).

- [ ] **Step 4: Commit**

```bash
git add src/lib/api/types.ts src/lib/api/sessions.ts
git commit -m "feat: add PanelSession types and getSessionsPanelData API wrapper"
```

---

## Task 7: Extend `sessionActivity` store

**Files:**
- Modify: `src/lib/stores/sessionActivity.svelte.ts`

- [ ] **Step 1: Update the store**

Replace the entire content of `src/lib/stores/sessionActivity.svelte.ts` with:

```typescript
import { getActiveSessionActivity, getSessionsPanelData } from "../api/sessions";
import type { PanelSession } from "../api/types";
import { subscribe } from "./events.svelte";

let counts = $state<Record<string, number>>({});
let activeSessionIds = $state<Set<string>>(new Set());
let panelSessions = $state<PanelSession[]>([]);
let panelActiveCount = $state(0);

async function refresh() {
  try {
    const [activity, panel] = await Promise.all([
      getActiveSessionActivity(),
      getSessionsPanelData(),
    ]);
    counts = activity.counts;
    activeSessionIds = new Set(activity.active_session_ids);
    panelSessions = panel.sessions;
    panelActiveCount = panel.active_count;
  } catch {
    // ignore errors (e.g. no storage set yet)
  }
}

export function getActiveCountForFeature(featureId: string): number {
  return counts[featureId] ?? 0;
}

export function isSessionActive(claudeSessionId: string): boolean {
  return activeSessionIds.has(claudeSessionId);
}

export function getAllActiveSessionCounts(): Record<string, number> {
  return counts;
}

export function getPanelSessions(): PanelSession[] {
  return panelSessions;
}

export function getPanelActiveCount(): number {
  return panelActiveCount;
}

export function refreshSessionActivity(): Promise<void> {
  return refresh();
}

export function startSessionActivityPolling(intervalMs = 10_000): () => void {
  refresh();
  const interval = setInterval(refresh, intervalMs);
  const unsubscribe = subscribe("sessions:changed", () => {
    refresh();
  });
  return () => {
    clearInterval(interval);
    unsubscribe();
  };
}
```

- [ ] **Step 2: Verify TypeScript compiles**

```bash
npm run build 2>&1 | grep -E "error TS|Error" | head -20
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/stores/sessionActivity.svelte.ts
git commit -m "feat: extend sessionActivity store with panel data"
```

---

## Task 8: Sessions panel CSS

**Files:**
- Modify: `src/app.css`

- [ ] **Step 1: Add CSS at the end of `src/app.css`**

Append this block to `src/app.css`:

```css
/* ===== SESSIONS PANEL ===== */

.sessions-panel {
  width: 260px;
  min-width: 260px;
  height: 100%;
  background: var(--bg-secondary);
  border-left: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  flex-shrink: 0;
}

.sessions-panel-header {
  display: flex;
  align-items: center;
  padding: 12px 12px 10px;
  border-bottom: 1px solid var(--border);
  gap: 8px;
  flex-shrink: 0;
}

.sessions-panel-title {
  font-size: 10px;
  font-weight: 600;
  color: var(--text-muted);
  letter-spacing: 0.08em;
  flex: 1;
}

.sessions-panel-active-badge {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 10px;
  color: var(--color-success, #4caf50);
  font-weight: 600;
}

.sessions-panel-active-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--color-success, #4caf50);
}

.sessions-panel-section-label {
  padding: 10px 12px 4px;
  font-size: 9px;
  font-weight: 600;
  color: var(--text-muted);
  letter-spacing: 0.07em;
  flex-shrink: 0;
}

.sessions-panel-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 0 8px;
}

.sessions-panel-scroll {
  flex: 1;
  overflow-y: auto;
  padding-bottom: 8px;
}

.sessions-panel-empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  color: var(--text-muted);
}

.session-row {
  width: 100%;
  text-align: left;
  border: none;
  border-radius: var(--radius-sm);
  padding: 8px 10px;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 3px;
  transition: background var(--transition-fast);
  border-left: 3px solid transparent;
  background: none;
}

.session-row--active {
  background: var(--bg-hover);
  border-left-color: var(--color-success, #4caf50);
}

.session-row--active:hover {
  background: var(--bg-raised);
}

.session-row--idle {
  background: transparent;
  border-left-color: var(--border-strong);
  opacity: 0.75;
}

.session-row--idle:hover {
  background: var(--bg-hover);
  opacity: 1;
}

.session-row-top {
  display: flex;
  align-items: center;
  gap: 6px;
}

.session-row-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-left: 14px;
}

.session-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
}

.session-dot--active {
  background: var(--color-success, #4caf50);
}

.session-dot--idle {
  background: var(--text-muted);
}

.session-feature-name {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.session-row--idle .session-feature-name {
  color: var(--text-secondary);
}

.session-branch-pill {
  font-size: 9px;
  background: var(--bg-raised);
  color: var(--text-secondary);
  padding: 1px 5px;
  border-radius: var(--radius-xs, 3px);
  flex-shrink: 0;
  max-width: 70px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.session-status {
  font-size: 9px;
}

.session-status--active {
  color: var(--color-success, #4caf50);
}

.session-status--idle {
  color: var(--text-muted);
}

.session-time {
  font-size: 9px;
  color: var(--text-muted);
}

.session-model {
  font-size: 9px;
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.session-tokens {
  font-size: 9px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

/* Icon rail badge for active session count */
.icon-rail-btn-badge {
  position: absolute;
  top: -3px;
  right: -3px;
  min-width: 14px;
  height: 14px;
  background: var(--color-success, #4caf50);
  border-radius: 7px;
  border: 2px solid var(--bg-secondary);
  font-size: 8px;
  font-weight: 700;
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
  padding: 0 2px;
  pointer-events: none;
}
```

- [ ] **Step 2: Commit**

```bash
git add src/app.css
git commit -m "feat: add sessions panel CSS"
```

---

## Task 9: Create `SessionsPanel.svelte`

**Files:**
- Create: `src/lib/components/SessionsPanel.svelte`

- [ ] **Step 1: Create the component**

Create `src/lib/components/SessionsPanel.svelte`:

```svelte
<script lang="ts">
  import { getPanelSessions, getPanelActiveCount } from "../stores/sessionActivity.svelte";

  interface Props {
    onSessionClick: (featureId: string, sessionDbId: string) => void;
  }

  let { onSessionClick }: Props = $props();

  function formatRelativeTime(iso: string): string {
    const diff = Date.now() - new Date(iso).getTime();
    const mins = Math.floor(diff / 60_000);
    if (mins < 1) return "just now";
    if (mins < 60) return `${mins}m ago`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `${hours}h ago`;
    return `${Math.floor(hours / 24)}d ago`;
  }

  function formatTokens(n: number | null): string | null {
    if (!n) return null;
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M tok`;
    if (n >= 1_000) return `${Math.round(n / 1_000)}k tok`;
    return `${n} tok`;
  }

  function shortModel(model: string | null): string | null {
    if (!model) return null;
    return model.replace(/^claude-/, "");
  }
</script>

<div class="sessions-panel">
  <div class="sessions-panel-header">
    <span class="sessions-panel-title">SESSIONS</span>
    {#if getPanelActiveCount() > 0}
      <span class="sessions-panel-active-badge">
        <span class="sessions-panel-active-dot"></span>
        {getPanelActiveCount()} active
      </span>
    {/if}
  </div>

  <div class="sessions-panel-scroll">
    {@const active = getPanelSessions().filter(s => s.is_active)}
    {@const recent = getPanelSessions().filter(s => !s.is_active)}

    {#if active.length > 0}
      <div class="sessions-panel-section-label">ACTIVE</div>
      <div class="sessions-panel-list">
        {#each active as session (session.id)}
          <button
            class="session-row session-row--active"
            onclick={() => onSessionClick(session.feature_id, session.id)}
          >
            <div class="session-row-top">
              <span class="session-dot session-dot--active"></span>
              <span class="session-feature-name">{session.feature_name}</span>
              {#if session.branch}
                <span class="session-branch-pill">{session.branch}</span>
              {/if}
            </div>
            <div class="session-row-meta">
              <span class="session-status session-status--active">● Active</span>
              <span class="session-time">{formatRelativeTime(session.last_modified)}</span>
            </div>
            <div class="session-row-meta">
              <span class="session-model">{shortModel(session.model) ?? ''}</span>
              {#if formatTokens(session.total_tokens)}
                <span class="session-tokens">{formatTokens(session.total_tokens)}</span>
              {/if}
            </div>
          </button>
        {/each}
      </div>
    {/if}

    {#if recent.length > 0}
      <div class="sessions-panel-section-label">RECENT</div>
      <div class="sessions-panel-list">
        {#each recent as session (session.id)}
          <button
            class="session-row session-row--idle"
            onclick={() => onSessionClick(session.feature_id, session.id)}
          >
            <div class="session-row-top">
              <span class="session-dot session-dot--idle"></span>
              <span class="session-feature-name">{session.feature_name}</span>
              {#if session.branch}
                <span class="session-branch-pill">{session.branch}</span>
              {/if}
            </div>
            <div class="session-row-meta">
              <span class="session-status session-status--idle">⊘ Idle</span>
              <span class="session-time">{formatRelativeTime(session.last_modified)}</span>
            </div>
            <div class="session-row-meta">
              <span class="session-model">{shortModel(session.model) ?? ''}</span>
              {#if formatTokens(session.total_tokens)}
                <span class="session-tokens">{formatTokens(session.total_tokens)}</span>
              {/if}
            </div>
          </button>
        {/each}
      </div>
    {/if}

    {#if getPanelSessions().length === 0}
      <div class="sessions-panel-empty">No sessions yet</div>
    {/if}
  </div>
</div>
```

- [ ] **Step 2: Verify TypeScript compiles**

```bash
npm run build 2>&1 | grep -E "error TS|Error" | head -20
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/SessionsPanel.svelte
git commit -m "feat: add SessionsPanel component"
```

---

## Task 10: Integrate into `App.svelte`

**Files:**
- Modify: `src/App.svelte`

- [ ] **Step 1: Add import for `SessionsPanel` and `getActiveTerminals`**

At the top of the `<script>` section in `src/App.svelte`, add these imports (after existing component imports):

```typescript
import SessionsPanel from "./lib/components/SessionsPanel.svelte";
import { getActiveTerminals, requestViewTerminal } from "./lib/stores/terminals.svelte";
import { getPanelActiveCount } from "./lib/stores/sessionActivity.svelte";
```

(`requestViewTerminal` may already be imported — check first and skip if so.)

- [ ] **Step 2: Add `sessionsPanelOpen` state**

Add near the other top-level state declarations in the `<script>` section:

```typescript
let sessionsPanelOpen = $state(
  localStorage.getItem("featurehub:sessionsPanel") === "true"
);

$effect(() => {
  localStorage.setItem("featurehub:sessionsPanel", String(sessionsPanelOpen));
});
```

- [ ] **Step 3: Add `handleSessionPanelClick` handler**

Add this function in the `<script>` section (after the existing session-related handlers):

```typescript
function handleSessionPanelClick(featureId: string, sessionDbId: string) {
  openTab(featureId);
  initialTab = "ai";
  initialTabTargetId = featureId;
  // If there's an active terminal for this session, switch to it
  const terminal = getActiveTerminals().find(t => t.sessionDbId === sessionDbId);
  if (terminal) {
    requestViewTerminal(terminal.terminalId);
  }
}
```

- [ ] **Step 4: Add the sessions panel icon rail button**

In the template, find the icon rail `<div class="icon-rail-sep"></div>` / `<div class="icon-rail-spacer"></div>` separator block. Add this button immediately before the separator (i.e., after the Timeline button, before the sep):

```svelte
<button
  class="icon-rail-btn"
  class:icon-rail-btn--on={sessionsPanelOpen}
  data-tip="Sessions"
  aria-label="Sessions panel"
  onclick={() => (sessionsPanelOpen = !sessionsPanelOpen)}
>
  <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
    <path d="M2 2a2 2 0 012-2h8a2 2 0 012 2v12a2 2 0 01-2 2H4a2 2 0 01-2-2V2zm2 0v12h8V2H4zm1 2h6v1H5V4zm0 2h6v1H5V6zm0 2h4v1H5V8z"/>
  </svg>
  {#if getPanelActiveCount() > 0}
    <span class="icon-rail-btn-badge">{getPanelActiveCount()}</span>
  {/if}
</button>
```

- [ ] **Step 5: Render `SessionsPanel` in the shell**

In the template, find `</div><!-- .app-shell -->`. Immediately before that closing tag, and after the `</div>` that closes `.main-content`, add:

```svelte
{#if sessionsPanelOpen}
  <SessionsPanel onSessionClick={handleSessionPanelClick} />
{/if}
```

- [ ] **Step 6: Verify the app runs in dev mode**

```bash
npm run tauri dev
```

Expected:
- App opens without errors
- Lightning/sessions icon appears in icon rail (before the separator)
- Clicking it toggles a right panel
- Panel shows ACTIVE / RECENT sections based on real session data
- Clicking a session row navigates to that feature + AI tab

- [ ] **Step 7: Commit**

```bash
git add src/App.svelte
git commit -m "feat: integrate sessions panel into app shell"
```

---

## Self-Review

**Spec coverage check:**
- ✅ Persistent right-side panel, 260px, toggleable via icon rail button
- ✅ Icon rail button has green badge showing active count
- ✅ Active sessions grouped first (ACTIVE), idle below (RECENT)
- ✅ Row: feature name, branch pill, status (Active/Idle), relative time, model, token count
- ✅ Click → navigate to feature + AI tab + open terminal if active
- ✅ JSONL stats parsed: model from first assistant message, tokens accumulated, cost summed
- ✅ mtime cache — re-parse only when JSONL file changes
- ✅ 100MB skip guard on JSONL parsing
- ✅ Panel open/closed state persisted in localStorage
- ✅ Out of scope: resize, filter, kill, context menu, cost from pricing tables

**Placeholder scan:** None found.

**Type consistency check:**
- `PanelSession.id` (Rust `id: String`) → used as `sessionDbId` in click handler → matched to `ActiveTerminal.sessionDbId` ✅
- `PanelSession.feature_id` → passed to `openTab(featureId)` ✅
- `getPanelSessions()` returns `PanelSession[]` → used in template ✅
- `onSessionClick` prop type matches `handleSessionPanelClick` signature ✅
- `CachedStats` defined in `session_parser.rs`, referenced in `AppState` via full path `claude::session_parser::CachedStats` ✅
