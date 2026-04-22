# Agents Display Improvements Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix title garbage, stale time, and status desync issues in agents display; add rich status features (waiting detection, cost/tokens, last action).

**Architecture:** Unify frontend data paths through single `get_sessions_panel_data` endpoint. Improve JSONL discovery with path caching. Add `is_bad_title` filter and proper fallback chain. Parse JSONL tail for status hints.

**Tech Stack:** Rust (Tauri backend), TypeScript/Svelte 5 (frontend), SQLite

---

## Task 1: Add Title Filter Function

**Files:**
- Modify: `src-tauri/src/claude/session_parser.rs`

- [ ] **Step 1: Write failing test for XML tag filtering**

Add to the `#[cfg(test)] mod tests` block in `session_parser.rs`:

```rust
#[test]
fn test_is_bad_title_filters_xml_tags() {
    assert!(is_bad_title("<local-command-caveat>Caveat: The messages..."));
    assert!(is_bad_title("<system>Some system text</system>"));
    assert!(is_bad_title("<user-prompt>test</user-prompt>"));
}

#[test]
fn test_is_bad_title_filters_short_and_commands() {
    assert!(is_bad_title("hi"));  // too short
    assert!(is_bad_title("/help"));  // slash command
    assert!(is_bad_title("git status"));  // git command
    assert!(is_bad_title("Caveat: something"));  // system prefix
}

#[test]
fn test_is_bad_title_allows_good_titles() {
    assert!(!is_bad_title("Add authentication to login flow"));
    assert!(!is_bad_title("Fix the bug in session parsing"));
    assert!(!is_bad_title("implement user management feature"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test test_is_bad_title --lib`
Expected: FAIL with "cannot find function `is_bad_title`"

- [ ] **Step 3: Implement is_bad_title function**

Add before `looks_like_path` function in `session_parser.rs`:

```rust
/// Check if a string is unsuitable as a session title.
pub fn is_bad_title(s: &str) -> bool {
    let trimmed = s.trim();
    
    // Too short to be meaningful
    if trimmed.len() < 5 {
        return true;
    }
    
    // Slash commands
    if trimmed.starts_with('/') {
        return true;
    }
    
    // XML/HTML tags (covers <local-command-caveat>, <system>, etc.)
    if trimmed.starts_with('<') && trimmed.contains('>') {
        return true;
    }
    
    // File paths
    if looks_like_path(trimmed) {
        return true;
    }
    
    // Git commands
    if trimmed.starts_with("git ") {
        return true;
    }
    
    // System prefixes that leak through
    if trimmed.starts_with("Caveat:") {
        return true;
    }
    
    false
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test test_is_bad_title --lib`
Expected: All 3 tests PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/claude/session_parser.rs
git commit -m "feat(sessions): add is_bad_title filter for XML tags and system content"
```

---

## Task 2: Update JSONL Title Parser to Use Filter

**Files:**
- Modify: `src-tauri/src/claude/session_parser.rs`

- [ ] **Step 1: Write failing test for filtered JSONL parsing**

Add to the tests module:

```rust
#[test]
fn test_parse_title_from_jsonl_skips_xml_tags() {
    let content = r#"{"type":"user","message":{"content":"<local-command-caveat>Caveat: The messages below were...</local-command-caveat>"}}
{"type":"user","message":{"content":"implement the authentication feature"}}"#;
    let f = write_temp_jsonl(content);
    let title = parse_title_from_jsonl(f.path());
    assert_eq!(title, Some("implement the authentication feature".to_string()));
}

#[test]
fn test_parse_title_from_jsonl_uses_fallback_if_all_bad() {
    let content = r#"{"type":"user","message":{"content":"<system>skip</system>"}}
{"type":"user","message":{"content":"/help"}}"#;
    let f = write_temp_jsonl(content);
    let title = parse_title_from_jsonl(f.path());
    // Falls back to first message since all are "bad"
    assert_eq!(title, Some("<system>skip</system>".to_string()));
}
```

- [ ] **Step 2: Run tests to verify current behavior**

Run: `cd src-tauri && cargo test test_parse_title_from_jsonl_skips --lib`
Expected: First test FAILS (currently returns the XML tag)

- [ ] **Step 3: Update parse_title_from_jsonl to use is_bad_title**

In `parse_title_from_jsonl`, replace the check at line ~196-203:

```rust
        // Skip messages that are too short, look like slash commands, or look like file paths
        let trimmed = title.trim();
        if trimmed.len() < 5 || trimmed.starts_with('/') {
            continue;
        }
        // Skip file paths (e.g. "C:\Windows\system32\cmd.exe", "/usr/bin/bash")
        if looks_like_path(trimmed) {
            continue;
        }
```

With:

```rust
        // Skip messages that don't make good titles
        if is_bad_title(&title) {
            continue;
        }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test test_parse_title_from_jsonl --lib`
Expected: All PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/claude/session_parser.rs
git commit -m "feat(sessions): use is_bad_title filter in JSONL parser"
```

---

## Task 3: Add New Enums and Enhanced Structs

**Files:**
- Modify: `src-tauri/src/claude/session_parser.rs`
- Modify: `src-tauri/src/commands/sessions.rs`

- [ ] **Step 1: Add enums to session_parser.rs**

Add after the `CachedStats` struct:

```rust
#[derive(Clone, Debug, serde::Serialize, PartialEq)]
pub enum SessionStatus {
    Active,
    WaitingForInput,
    Idle,
    Finished,
    Lost,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq)]
pub enum TitleSource {
    SessionsIndex,
    SessionMemory,
    FirstPrompt,
    FeatureName,
    Default,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq)]
pub enum StatusHint {
    ClaudeResponded,
    UserPrompted,
    ToolRunning,
}
```

- [ ] **Step 2: Run cargo check to verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles with no errors

- [ ] **Step 3: Update PanelSession in commands/sessions.rs**

Replace the existing `PanelSession` struct:

```rust
#[derive(serde::Serialize, Clone)]
pub struct PanelSession {
    // Identity
    pub id: String,
    pub feature_id: String,
    pub feature_name: String,
    pub claude_session_id: String,
    
    // Display
    pub title: String,
    pub title_source: crate::claude::session_parser::TitleSource,
    pub branch: Option<String>,
    
    // Time
    pub last_activity: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    
    // Status
    pub is_active: bool,
    pub status: crate::claude::session_parser::SessionStatus,
    pub last_action: Option<String>,
    
    // Stats
    pub model: Option<String>,
    pub total_tokens: Option<u64>,
    pub context_tokens: Option<u64>,
    pub cost_usd: Option<f64>,
}
```

- [ ] **Step 4: Run cargo check to verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Errors about missing fields in struct construction (expected, will fix in later task)

- [ ] **Step 5: Commit enums only (partial commit)**

```bash
git add src-tauri/src/claude/session_parser.rs
git commit -m "feat(sessions): add SessionStatus, TitleSource, StatusHint enums"
```

---

## Task 4: Add JSONL Path Cache to AppState

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add jsonl_path_cache field to AppState**

Update the `AppState` struct:

```rust
pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub storage_path: Mutex<Option<PathBuf>>,
    pub extensions: Mutex<extensions::ExtensionRegistry>,
    pub stats_cache: Mutex<std::collections::HashMap<String, claude::session_parser::CachedStats>>,
    pub jsonl_path_cache: Mutex<std::collections::HashMap<String, PathBuf>>,
}
```

- [ ] **Step 2: Initialize the cache in setup**

In the `app.manage(state)` section, update the `AppState` initialization:

```rust
            let state = AppState {
                db: Mutex::new(conn),
                storage_path: Mutex::new(storage_path),
                extensions: Mutex::new(extension_registry),
                stats_cache: Mutex::new(std::collections::HashMap::new()),
                jsonl_path_cache: Mutex::new(std::collections::HashMap::new()),
            };
```

- [ ] **Step 3: Run cargo check**

Run: `cd src-tauri && cargo check`
Expected: Compiles (with warnings about unused field, that's fine)

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat(sessions): add jsonl_path_cache to AppState"
```

---

## Task 5: Improve JSONL File Discovery

**Files:**
- Modify: `src-tauri/src/claude/scanner.rs`

- [ ] **Step 1: Write test for improved discovery**

Add to the tests section (create if needed):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::TempDir;

    #[test]
    fn test_find_jsonl_uses_cache() {
        let cache = Mutex::new(std::collections::HashMap::new());
        let tmp = TempDir::new().unwrap();
        let jsonl_path = tmp.path().join("test-session.jsonl");
        std::fs::write(&jsonl_path, "{}").unwrap();
        
        // Prime the cache
        cache.lock().unwrap().insert("test-session".to_string(), jsonl_path.clone());
        
        // Should find from cache
        let result = find_jsonl_with_cache(&cache, tmp.path(), "test-session");
        assert_eq!(result, Some(jsonl_path));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test test_find_jsonl_uses_cache --lib`
Expected: FAIL with "cannot find function `find_jsonl_with_cache`"

- [ ] **Step 3: Implement find_jsonl_with_cache function**

Add new function in `scanner.rs`:

```rust
/// Find JSONL file with caching. Searches cache first, then filesystem.
pub fn find_jsonl_with_cache(
    cache: &Mutex<std::collections::HashMap<String, std::path::PathBuf>>,
    projects_dir: &std::path::Path,
    session_id: &str,
) -> Option<std::path::PathBuf> {
    // 1. Check cache first
    if let Ok(c) = cache.lock() {
        if let Some(path) = c.get(session_id) {
            if path.exists() {
                return Some(path.clone());
            }
        }
    }
    
    // 2. Search filesystem
    let filename = format!("{}.jsonl", session_id);
    
    for entry in std::fs::read_dir(projects_dir).ok()?.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        
        // Direct child
        let jsonl = dir.join(&filename);
        if jsonl.exists() {
            if let Ok(mut c) = cache.lock() {
                c.insert(session_id.to_string(), jsonl.clone());
            }
            return Some(jsonl);
        }
        
        // Subdirectories (one level deeper)
        for subentry in std::fs::read_dir(&dir).ok()?.flatten() {
            let subdir = subentry.path();
            if subdir.is_dir() && subdir.file_name().map_or(false, |n| n != "session-memory") {
                let jsonl = subdir.join(&filename);
                if jsonl.exists() {
                    if let Ok(mut c) = cache.lock() {
                        c.insert(session_id.to_string(), jsonl.clone());
                    }
                    return Some(jsonl);
                }
            }
        }
    }
    
    None
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test test_find_jsonl_uses_cache --lib`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/claude/scanner.rs
git commit -m "feat(sessions): add cached JSONL path discovery"
```

---

## Task 6: Add Status Hint Parsing

**Files:**
- Modify: `src-tauri/src/claude/session_parser.rs`

- [ ] **Step 1: Write test for status hint detection**

```rust
#[test]
fn test_parse_status_hint_detects_waiting() {
    let content = r#"{"type":"assistant","message":{"content":"Would you like me to proceed with the implementation?"}}"#;
    let f = write_temp_jsonl(content);
    let hint = parse_status_hint(f.path());
    assert_eq!(hint.status_hint, StatusHint::ClaudeResponded);
    assert!(hint.looks_like_waiting);
}

#[test]
fn test_parse_status_hint_detects_user_prompted() {
    let content = r#"{"type":"assistant","message":{"content":"Done!"}}
{"type":"user","message":{"content":"now do the next thing"}}"#;
    let f = write_temp_jsonl(content);
    let hint = parse_status_hint(f.path());
    assert_eq!(hint.status_hint, StatusHint::UserPrompted);
}

#[test]
fn test_parse_last_action_from_tool() {
    let content = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Read"}]}}"#;
    let f = write_temp_jsonl(content);
    let hint = parse_status_hint(f.path());
    assert_eq!(hint.last_action, Some("Reading files".to_string()));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test test_parse_status_hint --lib`
Expected: FAIL with "cannot find function `parse_status_hint`"

- [ ] **Step 3: Add ParsedStatusHint struct and parse_status_hint function**

```rust
#[derive(Clone, Debug, Default)]
pub struct ParsedStatusHint {
    pub status_hint: StatusHint,
    pub looks_like_waiting: bool,
    pub last_action: Option<String>,
}

impl Default for StatusHint {
    fn default() -> Self {
        StatusHint::ClaudeResponded
    }
}

/// Parse the last ~20 lines of a JSONL file to determine status hints.
pub fn parse_status_hint(path: &std::path::Path) -> ParsedStatusHint {
    use std::io::{BufRead, BufReader, Seek, SeekFrom};
    
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return ParsedStatusHint::default(),
    };
    
    // Read last ~8KB to get recent entries
    let file_len = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mut reader = BufReader::new(file);
    if file_len > 8192 {
        reader.seek(SeekFrom::End(-8192)).ok();
        // Skip partial line
        let mut skip = String::new();
        reader.read_line(&mut skip).ok();
    }
    
    let mut last_type: Option<&'static str> = None;
    let mut last_assistant_content: Option<String> = None;
    let mut last_tool: Option<String> = None;
    
    for line in reader.lines().flatten() {
        if line.is_empty() {
            continue;
        }
        
        let val: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        
        let msg_type = val.get("type").and_then(|v| v.as_str());
        
        match msg_type {
            Some("user") => {
                last_type = Some("user");
            }
            Some("assistant") => {
                last_type = Some("assistant");
                // Extract content for waiting detection
                if let Some(msg) = val.get("message") {
                    if let Some(content) = msg.get("content") {
                        if let Some(s) = content.as_str() {
                            last_assistant_content = Some(s.to_string());
                        } else if let Some(arr) = content.as_array() {
                            // Check for tool use
                            for item in arr {
                                if item.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                                    last_tool = item.get("name").and_then(|n| n.as_str()).map(|s| s.to_string());
                                }
                                if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                                    if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                                        last_assistant_content = Some(text.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    
    let status_hint = match last_type {
        Some("user") => StatusHint::UserPrompted,
        Some("assistant") if last_tool.is_some() => StatusHint::ToolRunning,
        _ => StatusHint::ClaudeResponded,
    };
    
    let looks_like_waiting = last_assistant_content.as_ref().map_or(false, |c| {
        c.ends_with('?')
            || c.contains("Would you like")
            || c.contains("Should I")
            || c.contains("Do you want")
            || c.contains("Let me know")
            || c.contains("What would you prefer")
    });
    
    let last_action = last_tool.map(|tool| match tool.as_str() {
        "Read" => "Reading files".to_string(),
        "Edit" | "Write" => "Writing code".to_string(),
        "Bash" => "Running command".to_string(),
        "Grep" | "Glob" => "Searching".to_string(),
        "Agent" => "Running subagent".to_string(),
        other => format!("Using {}", other),
    });
    
    ParsedStatusHint {
        status_hint,
        looks_like_waiting,
        last_action,
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test test_parse_status_hint --lib && cargo test test_parse_last_action --lib`
Expected: All PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/claude/session_parser.rs
git commit -m "feat(sessions): add status hint and last action parsing"
```

---

## Task 7: Update get_sessions_panel_data with New Fields

**Files:**
- Modify: `src-tauri/src/commands/sessions.rs`

- [ ] **Step 1: Update PanelSession construction in get_sessions_panel_data**

Replace the session mapping logic (around line 393-423) with:

```rust
    // 5. Build PanelSession list with title resolution
    let mut sessions: Vec<PanelSession> = rows
        .iter()
        .map(|row| {
            let is_active = active_ids.contains(&row.claude_session_id);
            let stats = all_stats.get(&row.claude_session_id);
            let jsonl_path = path_map.iter()
                .find(|(id, _)| id == &row.claude_session_id)
                .and_then(|(_, info)| info.as_ref().map(|(p, _)| p.clone()));

            // Resolve title with fallback chain
            let (title, title_source) = resolve_session_title(
                &row.claude_session_id,
                row.title.as_deref(),
                &row.feature_name,
            );

            // Parse status hint from JSONL if available
            let status_hint = jsonl_path.as_ref()
                .map(|p| crate::claude::session_parser::parse_status_hint(p))
                .unwrap_or_default();

            // Determine session status
            let status = resolve_session_status(
                is_active,
                &status_hint,
                row.claude_session_id.is_empty(),
                false, // ended_at not in PanelSessionRow yet
            );

            let last_activity = stats
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
                title,
                title_source,
                branch: row.branch.clone(),
                last_activity,
                started_at: None, // TODO: add to PanelSessionRow
                ended_at: None,   // TODO: add to PanelSessionRow
                is_active,
                status,
                last_action: status_hint.last_action,
                model: stats.and_then(|s| s.model.clone()),
                total_tokens: stats.map(|s| s.total_tokens).filter(|&t| t > 0),
                context_tokens: stats.and_then(|s| s.context_tokens),
                cost_usd: stats.and_then(|s| s.cost_usd),
            }
        })
        .collect();
```

- [ ] **Step 2: Add helper functions for title and status resolution**

Add before `get_sessions_panel_data`:

```rust
/// Resolve session title using fallback chain.
fn resolve_session_title(
    claude_session_id: &str,
    db_title: Option<&str>,
    feature_name: &str,
) -> (String, crate::claude::session_parser::TitleSource) {
    use crate::claude::session_parser::TitleSource;
    
    // If DB has a title and it's not garbage, use it
    if let Some(t) = db_title {
        if !t.is_empty() && !crate::claude::session_parser::is_bad_title(t) {
            return (t.to_string(), TitleSource::FirstPrompt);
        }
    }
    
    // Try sessions-index.json
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return (format!("{} Session", feature_name), TitleSource::FeatureName),
    };
    let projects_dir = home.join(".claude").join("projects");
    
    for entry in std::fs::read_dir(&projects_dir).ok().into_iter().flatten().flatten() {
        let project_dir = entry.path();
        if !project_dir.is_dir() {
            continue;
        }
        
        let (index_title, first_prompt) = crate::claude::session_parser::find_title_in_sessions_index(&project_dir, claude_session_id);
        
        if let Some(title) = index_title {
            if !crate::claude::session_parser::is_bad_title(&title) {
                return (title, TitleSource::SessionsIndex);
            }
        }
        
        if let Some(prompt) = first_prompt {
            if !crate::claude::session_parser::is_bad_title(&prompt) {
                return (prompt, TitleSource::FirstPrompt);
            }
        }
    }
    
    // Fallback to feature name
    (format!("{} Session", feature_name), TitleSource::FeatureName)
}

/// Resolve session status from multiple signals.
fn resolve_session_status(
    is_active: bool,
    status_hint: &crate::claude::session_parser::ParsedStatusHint,
    no_session_id: bool,
    has_ended: bool,
) -> crate::claude::session_parser::SessionStatus {
    use crate::claude::session_parser::{SessionStatus, StatusHint};
    
    if no_session_id {
        return SessionStatus::Lost;
    }
    
    if is_active {
        if status_hint.looks_like_waiting {
            return SessionStatus::WaitingForInput;
        }
        return SessionStatus::Active;
    }
    
    if has_ended {
        return SessionStatus::Finished;
    }
    
    SessionStatus::Idle
}
```

- [ ] **Step 3: Run cargo check**

Run: `cd src-tauri && cargo check`
Expected: Compiles (may have warnings)

- [ ] **Step 4: Run existing tests**

Run: `cd src-tauri && cargo test --lib`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/sessions.rs
git commit -m "feat(sessions): update PanelSession with title resolution and status"
```

---

## Task 8: Update Frontend Types

**Files:**
- Modify: `src/lib/api/types.ts`

- [ ] **Step 1: Add new enum types**

Add after the `SessionsPanelData` interface:

```typescript
export type SessionStatus = 'Active' | 'WaitingForInput' | 'Idle' | 'Finished' | 'Lost';

export type TitleSource = 'SessionsIndex' | 'SessionMemory' | 'FirstPrompt' | 'FeatureName' | 'Default';
```

- [ ] **Step 2: Update PanelSession interface**

Replace the existing `PanelSession` interface:

```typescript
export interface PanelSession {
  // Identity
  id: string;
  feature_id: string;
  feature_name: string;
  claude_session_id: string;
  
  // Display
  title: string;
  title_source: TitleSource;
  branch: string | null;
  
  // Time
  last_activity: string;
  started_at: string | null;
  ended_at: string | null;
  
  // Status
  is_active: boolean;
  status: SessionStatus;
  last_action: string | null;
  
  // Stats
  model: string | null;
  total_tokens: number | null;
  context_tokens: number | null;
  cost_usd: number | null;
}
```

- [ ] **Step 3: Run TypeScript check**

Run: `npm run build`
Expected: May have errors in components using old field names (expected, will fix next)

- [ ] **Step 4: Commit**

```bash
git add src/lib/api/types.ts
git commit -m "feat(sessions): update PanelSession TypeScript types"
```

---

## Task 9: Update SessionsPanel Component

**Files:**
- Modify: `src/lib/components/SessionsPanel.svelte`

- [ ] **Step 1: Update time display to use last_activity**

Replace `session.last_modified` with `session.last_activity` (line ~89, ~129):

```svelte
<span class="session-time">{formatRelativeTime(session.last_activity)}</span>
```

- [ ] **Step 2: Add status badge display**

After the time display, add status indicator:

```svelte
{#if session.status === 'WaitingForInput'}
  <span class="session-status session-status--waiting">Waiting</span>
{:else if session.status === 'Finished'}
  <span class="session-status session-status--finished">Finished</span>
{/if}
```

- [ ] **Step 3: Add last_action display**

After the model display:

```svelte
{#if session.last_action}
  <span class="session-last-action">{session.last_action}</span>
{/if}
```

- [ ] **Step 4: Add CSS for new status classes**

Add to the `<style>` section:

```css
.session-status {
  font-size: 9px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.02em;
  padding: 1px 4px;
  border-radius: 3px;
}

.session-status--waiting {
  color: var(--amber);
  background: color-mix(in srgb, var(--amber) 15%, transparent);
}

.session-status--finished {
  color: var(--text-muted);
  background: var(--bg-hover);
}

.session-last-action {
  font-size: 9px;
  color: var(--text-muted);
  font-style: italic;
}
```

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/SessionsPanel.svelte
git commit -m "feat(sessions): update SessionsPanel with status and last_action display"
```

---

## Task 10: Unify Frontend Store

**Files:**
- Modify: `src/lib/stores/sessionActivity.svelte.ts`

- [ ] **Step 1: Update refresh to derive all state from panel data**

Replace the `refresh` function:

```typescript
async function refresh() {
  try {
    const panel = await getSessionsPanelData();
    
    panelSessions = panel.sessions;
    panelActiveCount = panel.active_count;
    
    // Derive counts and active IDs from panel data (single source of truth)
    const newCounts: Record<string, number> = {};
    const newActiveIds = new Set<string>();
    
    for (const s of panel.sessions) {
      if (s.is_active || s.status === 'Active' || s.status === 'WaitingForInput') {
        newCounts[s.feature_id] = (newCounts[s.feature_id] ?? 0) + 1;
        newActiveIds.add(s.claude_session_id);
      }
    }
    
    counts = newCounts;
    activeSessionIds = newActiveIds;
  } catch {
    // ignore errors (e.g. no storage set yet)
  }
}
```

- [ ] **Step 2: Remove getActiveSessionActivity call**

The function now only uses `getSessionsPanelData`. Update the import if needed:

```typescript
import { getSessionsPanelData } from "../api/sessions";
```

Remove the unused import of `getActiveSessionActivity` if present.

- [ ] **Step 3: Run TypeScript check**

Run: `npm run build`
Expected: Compiles successfully

- [ ] **Step 4: Commit**

```bash
git add src/lib/stores/sessionActivity.svelte.ts
git commit -m "feat(sessions): unify store to use single data source"
```

---

## Task 11: Update SessionCard Component

**Files:**
- Modify: `src/lib/modules/ai/SessionCard.svelte`

- [ ] **Step 1: Add status-based styling**

Update the status badge section (around line 189-192) to handle new statuses:

```svelte
{#if hasSessionId}
  <!-- existing button content -->
{:else}
  <div class="sc__main sc__main--static">
    <span class="sc__dot" class:sc__dot--live={isInProgress}></span>
    <!-- ... existing content ... -->
    <span class="sc__badge" class:sc__badge--amber={isInProgress} class:sc__badge--red={!isInProgress}>
      {isInProgress ? "In progress" : "Session lost"}
    </span>
  </div>
{/if}
```

The existing logic already handles most cases through `isActive` derived state. The new `status` field from backend provides more granularity but the visual display can remain similar.

- [ ] **Step 2: Run dev server and test**

Run: `npm run tauri dev`
Expected: Sessions display with correct active status in both panels

- [ ] **Step 3: Commit**

```bash
git add src/lib/modules/ai/SessionCard.svelte
git commit -m "feat(sessions): ensure SessionCard uses unified active status"
```

---

## Task 12: Integration Test and Cleanup

**Files:**
- Multiple files for verification

- [ ] **Step 1: Run all Rust tests**

Run: `cd src-tauri && cargo test --lib`
Expected: All tests pass

- [ ] **Step 2: Run frontend build**

Run: `npm run build`
Expected: Builds successfully with no errors

- [ ] **Step 3: Manual testing checklist**

Start dev server: `npm run tauri dev`

Test each item:
- [ ] Active session shows green dot in BOTH panels simultaneously
- [ ] Title shows Claude's summary (not XML tags like `<local-command-caveat>`)
- [ ] Time updates reflect actual last activity (not stale creation time)
- [ ] "Waiting" status appears when Claude asks a question
- [ ] Tokens/cost display correctly for sessions with activity

- [ ] **Step 4: Final commit if any fixes needed**

```bash
git add -A
git commit -m "fix(sessions): integration fixes from testing"
```

- [ ] **Step 5: Create summary commit**

If all working:
```bash
git log --oneline -10
```

Verify commits are clean and descriptive.

---

## Summary

This plan implements:
1. **Title filtering** — `is_bad_title` function filters XML tags, commands, paths
2. **JSONL discovery** — Cached path lookup with deeper directory search
3. **Rich status** — `SessionStatus` enum with waiting detection and last action
4. **Unified data flow** — Single `get_sessions_panel_data` endpoint, derived store state
5. **UI updates** — Status badges, last action display in both panels
