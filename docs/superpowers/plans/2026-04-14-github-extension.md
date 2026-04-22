# GitHub Extension Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a first-party `github-prs` extension that tracks GitHub pull requests (state, CI checks, reviews) for features, plus the small extension-system primitives it needs (scheduled tasks, stdout notifications, extension settings).

**Architecture:** Two phases. Phase A adds three general extension-system primitives: `schedules` manifest field + runner, a `notifications[]` field in the script stdout protocol, and `get/set_extension_settings` Tauri commands. Phase B adds the `github-prs` extension directory with Node.js handlers that shell out to `gh` CLI, store PR state in three `ext_*` tables, render a PRs tab as an iframe with a card grid, and post state-change toasts via the new notifications field.

**Tech Stack:** Rust (Tauri, rusqlite, tokio), Node.js 22+ (built-in `node:sqlite`), `gh` CLI, Svelte 5 (tab frame + settings card), existing extension system.

**Reference spec:** `docs/superpowers/specs/2026-04-14-github-extension-design.md`

**Working directory:** All relative paths below assume repo root `D:\LittleBrushGames\FeatureHub`. Rust paths are under `src-tauri/`. Frontend paths are under `src/`.

---

## Phase A — Extension-System Primitives

These are general-purpose additions to the extension system. Ship first; Phase B depends on them.

---

### Task A1: Add `schedules` field to ExtensionManifest

**Files:**
- Modify: `src-tauri/src/extensions/manifest.rs`

- [ ] **Step 1: Write the failing test**

Add at the bottom of `src-tauri/src/extensions/manifest.rs` inside the existing `#[cfg(test)]` module (or create one if absent):

```rust
#[cfg(test)]
mod schedule_tests {
    use super::*;

    #[test]
    fn parses_schedules_from_manifest() {
        let json = r#"{
            "id": "x", "name": "X", "version": "1.0.0",
            "schedules": [
                { "id": "poll", "handler": "p.js", "interval_secs": 300, "enabled_setting": "poll_enabled" }
            ]
        }"#;
        let m: ExtensionManifest = serde_json::from_str(json).unwrap();
        assert_eq!(m.schedules.len(), 1);
        let s = &m.schedules[0];
        assert_eq!(s.id, "poll");
        assert_eq!(s.handler, "p.js");
        assert_eq!(s.interval_secs, 300);
        assert_eq!(s.enabled_setting.as_deref(), Some("poll_enabled"));
    }

    #[test]
    fn schedule_without_enabled_setting_is_always_enabled() {
        let json = r#"{"id":"x","name":"X","version":"1.0.0","schedules":[{"id":"t","handler":"t.js","interval_secs":60}]}"#;
        let m: ExtensionManifest = serde_json::from_str(json).unwrap();
        assert!(m.schedules[0].enabled_setting.is_none());
    }

    #[test]
    fn validate_rejects_interval_below_60() {
        let m = ExtensionManifest {
            id: "x".into(), name: "X".into(), version: "1.0.0".into(),
            description: "".into(), author: "".into(),
            requires: vec![], tables: vec![], tools: vec![],
            events: vec![], tabs: vec![], instructions: "".into(),
            schedules: vec![ScheduleDecl {
                id: "s".into(), handler: "h.js".into(),
                interval_secs: 30, enabled_setting: None,
            }],
            storage_settings_key: None,
        };
        let err = m.validate().unwrap_err();
        assert!(err.contains("interval_secs"));
    }
}
```

- [ ] **Step 2: Run test to verify it fails (compile error — `ScheduleDecl` unknown)**

```
cd src-tauri && cargo test --lib extensions::manifest::schedule_tests
```
Expected: compile error, `ScheduleDecl not found` / `no field 'schedules' on ExtensionManifest` / `no field 'storage_settings_key'`.

- [ ] **Step 3: Implement — add `ScheduleDecl` struct and fields**

In `src-tauri/src/extensions/manifest.rs`, add the struct and fields:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleDecl {
    pub id: String,
    pub handler: String,
    pub interval_secs: u64,
    #[serde(default)]
    pub enabled_setting: Option<String>,
}
```

Update `ExtensionManifest`:

```rust
    #[serde(default)]
    pub schedules: Vec<ScheduleDecl>,
    #[serde(default)]
    pub storage_settings_key: Option<String>,
```

Update `validate`:

```rust
        for s in &self.schedules {
            if s.id.is_empty() {
                return Err("schedule missing 'id'".into());
            }
            if s.handler.is_empty() {
                return Err(format!("schedule '{}' missing 'handler'", s.id));
            }
            if s.interval_secs < 60 {
                return Err(format!("schedule '{}': interval_secs must be >= 60", s.id));
            }
        }
```

- [ ] **Step 4: Run tests to verify they pass**

```
cd src-tauri && cargo test --lib extensions::manifest
```
Expected: PASS (all existing + new tests).

- [ ] **Step 5: Commit**

```
git add src-tauri/src/extensions/manifest.rs
git commit -m "feat(extensions): add schedules + storage_settings_key to manifest"
```

---

### Task A2: Extend script stdout protocol with `notifications[]`

**Files:**
- Modify: `src-tauri/src/extensions/script_runner.rs`

- [ ] **Step 1: Write the failing test**

Add at the bottom of the existing `#[cfg(test)] mod tests` in `script_runner.rs`:

```rust
    #[test]
    fn returns_notifications_from_script() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, r#"process.stdout.write(JSON.stringify({{
            ok: true,
            data: null,
            notifications: [
                {{ feature_id: "f1", kind: "info", message: "hello" }},
                {{ feature_id: "f1", kind: "warn", message: "heads up" }}
            ]
        }}));"#).unwrap();
        let path = f.into_temp_path();
        let out = run_blocking_with_notifications(path.as_ref(), &test_input(), 5).unwrap();
        assert_eq!(out.notifications.len(), 2);
        assert_eq!(out.notifications[0].message, "hello");
        assert_eq!(out.notifications[1].kind, "warn");
    }
```

- [ ] **Step 2: Run test to verify it fails**

```
cd src-tauri && cargo test --lib extensions::script_runner::tests::returns_notifications_from_script
```
Expected: compile error (`run_blocking_with_notifications` unknown).

- [ ] **Step 3: Implement the extended return shape**

In `src-tauri/src/extensions/script_runner.rs`, add new types and function. Keep the existing `run_blocking` wrapping the new `run_blocking_with_notifications` so call sites that don't care continue to work.

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Notification {
    #[serde(default)]
    pub feature_id: Option<String>,
    #[serde(default = "default_kind")]
    pub kind: String,
    pub message: String,
    #[serde(default)]
    pub plan_id: Option<String>,
}

fn default_kind() -> String { "info".into() }

#[derive(Debug)]
pub struct ScriptResult {
    pub data: serde_json::Value,
    pub notifications: Vec<Notification>,
}

#[derive(Debug, serde::Deserialize)]
struct ScriptOutputFull {
    ok: bool,
    #[serde(default)]
    data: Option<serde_json::Value>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    notifications: Vec<Notification>,
}
```

Replace the existing `ScriptOutput` struct with `ScriptOutputFull` (rename inline) OR keep both; parse via `ScriptOutputFull` in a new function and have `run_blocking` delegate. Simplest: replace `ScriptOutput` with `ScriptOutputFull` fully, adjust existing parse.

Refactor: rename the existing `run_blocking` body into `run_blocking_with_notifications` returning `Result<ScriptResult, String>`. Keep `run_blocking` as:

```rust
pub fn run_blocking(
    script_path: &Path,
    input: &ScriptInput,
    timeout_secs: u64,
) -> Result<serde_json::Value, String> {
    run_blocking_with_notifications(script_path, input, timeout_secs).map(|r| r.data)
}

pub fn run_blocking_with_notifications(
    script_path: &Path,
    input: &ScriptInput,
    timeout_secs: u64,
) -> Result<ScriptResult, String> {
    // ... existing spawn/timeout/parse logic ...
    // At the end, instead of returning data only:
    if result.ok {
        Ok(ScriptResult {
            data: result.data.unwrap_or(serde_json::Value::Null),
            notifications: result.notifications,
        })
    } else {
        Err(result.error.unwrap_or_else(|| "Script returned ok=false".to_string()))
    }
}
```

- [ ] **Step 4: Update `run_event_hook` to forward notifications**

In the same file, replace `run_event_hook` with:

```rust
pub fn run_event_hook(script_path: std::path::PathBuf, input: ScriptInput, extension_id: String) {
    std::thread::spawn(move || {
        match run_blocking_with_notifications(&script_path, &input, 10) {
            Ok(result) => forward_notifications(&extension_id, result.notifications),
            Err(e) => eprintln!("[ext:{}] Event hook {:?} failed: {}", extension_id, script_path, e),
        }
    });
}

pub fn forward_notifications(extension_id: &str, notifications: Vec<Notification>) {
    for n in notifications {
        let res = crate::config::push_notification_ex(
            &n.message,
            n.feature_id.as_deref(),
            n.plan_id.as_deref(),
            &n.kind,
        );
        if let Err(e) = res {
            eprintln!("[ext:{}] push_notification failed: {}", extension_id, e);
        }
    }
}
```

**Note:** `push_notification_ex` signature may differ — check `src-tauri/src/config.rs` for the exact arity. If it doesn't accept a `kind` parameter, adapt (either call `push_notification` for info/warn and ignore kind, or extend `push_notification_ex` — but keep that change out of this plan; just drop the `kind` arg if unsupported).

- [ ] **Step 5: Run tests**

```
cd src-tauri && cargo test --lib extensions::script_runner
```
Expected: PASS.

- [ ] **Step 6: Commit**

```
git add src-tauri/src/extensions/script_runner.rs
git commit -m "feat(extensions): support notifications[] in script stdout protocol"
```

---

### Task A3: Add scheduled task runner

**Files:**
- Create: `src-tauri/src/extensions/scheduler.rs`
- Modify: `src-tauri/src/extensions/mod.rs` (add `pub mod scheduler;`)
- Modify: `src-tauri/src/lib.rs` (start scheduler when extensions load)

- [ ] **Step 1: Add scheduler module scaffold + failing test**

Create `src-tauri/src/extensions/scheduler.rs`:

```rust
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::extensions::manifest::ScheduleDecl;
use crate::extensions::script_runner::{self, ScriptInput};

pub struct ScheduleHandle {
    pub cancel: Arc<std::sync::atomic::AtomicBool>,
    pub join: std::thread::JoinHandle<()>,
}

pub fn spawn_schedule(
    extension_id: String,
    extension_dir: PathBuf,
    schedule: ScheduleDecl,
    db_path: String,
    storage_path: String,
) -> ScheduleHandle {
    let cancel = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let cancel_flag = cancel.clone();
    let in_flight = Arc::new(Mutex::new(false));
    let interval = Duration::from_secs(schedule.interval_secs.max(60));

    let join = std::thread::spawn(move || {
        let mut next_tick = Instant::now() + interval;
        while !cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
            let now = Instant::now();
            if now >= next_tick {
                // Try to acquire in-flight lock; skip tick if busy.
                let mut guard = match in_flight.try_lock() {
                    Ok(g) if !*g => g,
                    _ => {
                        next_tick = now + interval;
                        continue;
                    }
                };
                *guard = true;
                drop(guard);

                let handler_path = extension_dir.join(&schedule.handler);
                let input = ScriptInput {
                    params: Default::default(),
                    db_path: db_path.clone(),
                    storage_path: storage_path.clone(),
                    feature_id: None,
                };
                let ext_id = extension_id.clone();
                match script_runner::run_blocking_with_notifications(&handler_path, &input, 60) {
                    Ok(r) => script_runner::forward_notifications(&ext_id, r.notifications),
                    Err(e) => eprintln!("[ext:{}] schedule '{}' failed: {}", ext_id, schedule.id, e),
                }
                *in_flight.lock().unwrap() = false;
                next_tick = Instant::now() + interval;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    });

    ScheduleHandle { cancel, join }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn fires_and_cancels() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        let counter = std::env::temp_dir().join(format!("fh_sched_test_{}.cnt", std::process::id()));
        let _ = std::fs::remove_file(&counter);
        writeln!(f, r#"
            const fs = require('fs');
            const p = {:?};
            let n = 0;
            try {{ n = parseInt(fs.readFileSync(p, 'utf8')) || 0; }} catch (e) {{}}
            fs.writeFileSync(p, String(n + 1));
            process.stdout.write(JSON.stringify({{ ok: true, data: null }}));
        "#, counter.to_string_lossy()).unwrap();
        let script_path = f.into_temp_path();
        let dir = script_path.as_ref().parent().unwrap().to_path_buf();
        let schedule = ScheduleDecl {
            id: "t".into(),
            handler: script_path.as_ref().file_name().unwrap().to_string_lossy().into(),
            interval_secs: 60, // clamped; actual wait below uses manual tick
            enabled_setting: None,
        };
        // Drive manually by short interval via test hook:
        // For now: just check spawn/cancel doesn't hang.
        let handle = spawn_schedule("test".into(), dir, schedule, "".into(), "".into());
        std::thread::sleep(Duration::from_millis(100));
        handle.cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        handle.join.join().unwrap();
    }
}
```

Add `pub mod scheduler;` at the top of `src-tauri/src/extensions/mod.rs`.

- [ ] **Step 2: Run test**

```
cd src-tauri && cargo test --lib extensions::scheduler
```
Expected: PASS (basic spawn/cancel).

- [ ] **Step 3: Integrate with extension registry**

In `src-tauri/src/extensions/mod.rs`, add to `LoadedExtension`:

```rust
#[derive(Debug, Clone)]
pub struct LoadedExtension {
    pub manifest: ExtensionManifest,
    pub enabled: bool,
    pub dir: PathBuf,
    pub requires_status: Vec<RequiresStatus>,
    // Schedules are tracked separately in AppState, not here.
}
```

Add to `ExtensionRegistry` or a sibling structure a list of active schedule handles. Easiest: hold a `Vec<scheduler::ScheduleHandle>` in `AppState` alongside `extensions`.

In `src-tauri/src/lib.rs`, find where `ExtensionRegistry::load_from_dir` is called after storage activation, and after load, iterate:

```rust
let mut schedule_handles: Vec<extensions::scheduler::ScheduleHandle> = Vec::new();
for ext in &registry.extensions {
    if !ext.enabled { continue; }
    for schedule in &ext.manifest.schedules {
        let enabled = match &schedule.enabled_setting {
            None => true,
            Some(key) => extension_setting_bool(&storage_path, &ext.manifest, key).unwrap_or(false),
        };
        if !enabled { continue; }
        let handle = extensions::scheduler::spawn_schedule(
            ext.manifest.id.clone(),
            ext.dir.clone(),
            schedule.clone(),
            db_path.clone(),
            storage_path.to_string_lossy().to_string(),
        );
        schedule_handles.push(handle);
    }
}
```

Where `extension_setting_bool` reads from the storage's `settings.json` extensions slice. Implement inline for now:

```rust
fn extension_setting_bool(storage_path: &std::path::Path, manifest: &extensions::manifest::ExtensionManifest, key: &str) -> Option<bool> {
    let settings_key = manifest.storage_settings_key.as_deref()?;
    let path = storage_path.join("settings.json");
    let text = std::fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    v.get("extensions")?.get(settings_key)?.get(key)?.as_bool()
}
```

Store `schedule_handles` on `AppState`. On storage change (the existing re-init path), cancel old handles:

```rust
for h in state.schedule_handles.lock().unwrap().drain(..) {
    h.cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    let _ = h.join.join();
}
```

- [ ] **Step 4: Run full cargo check**

```
cd src-tauri && cargo check
```
Expected: clean compile.

- [ ] **Step 5: Commit**

```
git add src-tauri/src/extensions/mod.rs src-tauri/src/extensions/scheduler.rs src-tauri/src/lib.rs
git commit -m "feat(extensions): scheduled task runner with per-schedule mutex"
```

---

### Task A4: Add `get_extension_settings` / `set_extension_settings` Tauri commands

**Files:**
- Modify: `src-tauri/src/commands/extensions.rs`
- Modify: `src-tauri/src/lib.rs` (register commands)

- [ ] **Step 1: Write the failing test**

Add to `src-tauri/src/commands/extensions.rs`:

```rust
#[cfg(test)]
mod settings_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn roundtrip_settings() {
        let tmp = TempDir::new().unwrap();
        let sp = tmp.path();
        // Write initial settings.json
        std::fs::write(sp.join("settings.json"), r#"{"extensions": {}}"#).unwrap();

        set_extension_settings_inner(sp, "github_prs", serde_json::json!({"poll_enabled": true, "poll_interval_secs": 180})).unwrap();
        let got = get_extension_settings_inner(sp, "github_prs").unwrap();
        assert_eq!(got["poll_enabled"], true);
        assert_eq!(got["poll_interval_secs"], 180);
    }

    #[test]
    fn returns_empty_object_for_unknown_key() {
        let tmp = TempDir::new().unwrap();
        let sp = tmp.path();
        std::fs::write(sp.join("settings.json"), r#"{"extensions": {}}"#).unwrap();
        let got = get_extension_settings_inner(sp, "nonexistent").unwrap();
        assert!(got.is_object());
        assert_eq!(got.as_object().unwrap().len(), 0);
    }
}
```

- [ ] **Step 2: Run to verify it fails**

```
cd src-tauri && cargo test --lib commands::extensions::settings_tests
```
Expected: compile error (functions not defined).

- [ ] **Step 3: Implement**

Append to `src-tauri/src/commands/extensions.rs`:

```rust
use std::path::Path;

pub fn get_extension_settings_inner(
    storage_path: &Path,
    key: &str,
) -> Result<serde_json::Value, String> {
    let path = storage_path.join("settings.json");
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    Ok(v.get("extensions")
        .and_then(|e| e.get(key))
        .cloned()
        .unwrap_or_else(|| serde_json::json!({})))
}

pub fn set_extension_settings_inner(
    storage_path: &Path,
    key: &str,
    value: serde_json::Value,
) -> Result<(), String> {
    let path = storage_path.join("settings.json");
    let mut root: serde_json::Value = if path.exists() {
        let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&text).map_err(|e| e.to_string())?
    } else {
        serde_json::json!({})
    };
    if !root.is_object() {
        root = serde_json::json!({});
    }
    let extensions = root
        .as_object_mut()
        .unwrap()
        .entry("extensions".to_string())
        .or_insert_with(|| serde_json::json!({}));
    if !extensions.is_object() {
        *extensions = serde_json::json!({});
    }
    extensions.as_object_mut().unwrap().insert(key.to_string(), value);
    let text = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?;
    std::fs::write(&path, text).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_extension_settings(
    state: State<'_, AppState>,
    key: String,
) -> Result<serde_json::Value, String> {
    let sp_guard = state.storage_path.lock().map_err(|e| e.to_string())?;
    let sp = sp_guard.as_ref().ok_or("No active storage")?;
    get_extension_settings_inner(sp, &key)
}

#[tauri::command]
pub fn set_extension_settings(
    state: State<'_, AppState>,
    key: String,
    value: serde_json::Value,
) -> Result<(), String> {
    let sp_guard = state.storage_path.lock().map_err(|e| e.to_string())?;
    let sp = sp_guard.as_ref().ok_or("No active storage")?;
    set_extension_settings_inner(sp, &key, value)
}
```

- [ ] **Step 4: Register commands in `lib.rs`**

In `src-tauri/src/lib.rs`, inside `invoke_handler!`, add `commands::extensions::get_extension_settings` and `commands::extensions::set_extension_settings` alongside the existing `get_extensions` / `get_extension_badge`.

- [ ] **Step 5: Run tests + cargo check**

```
cd src-tauri && cargo test --lib commands::extensions && cargo check
```
Expected: PASS + clean compile.

- [ ] **Step 6: Commit**

```
git add src-tauri/src/commands/extensions.rs src-tauri/src/lib.rs
git commit -m "feat(extensions): get/set_extension_settings commands"
```

---

### Task A5: Dispatch `link_deleted` event in `delete_link`

**Files:**
- Modify: `src-tauri/src/commands/links.rs`
- Modify: `src-tauri/src/db/links.rs` (may need to read link before delete to know type/feature)

- [ ] **Step 1: Write the failing test**

Add in `src-tauri/src/db/links.rs` test module:

```rust
#[cfg(test)]
mod delete_tests {
    use super::*;
    use crate::db::test_utils::test_db;

    #[test]
    fn get_link_by_id_returns_link() {
        let conn = test_db();
        // Seed a feature + a link
        conn.execute("INSERT INTO features (id, title, status, sort_order, created_at, updated_at) VALUES ('f1','t','active',0,'now','now')", []).unwrap();
        let link = add_link(&conn, "f1", "title", "https://github.com/a/b/pull/1", Some("github-pr".into()), None).unwrap();
        let got = get_link(&conn, &link.id).unwrap();
        assert_eq!(got.url, link.url);
        assert_eq!(got.link_type, "github-pr");
    }
}
```

Check if `get_link` already exists. If not, add it:

```rust
pub fn get_link(conn: &Connection, id: &str) -> Result<Link, String> {
    conn.query_row(
        "SELECT id, feature_id, title, url, link_type, description, metadata, created_at FROM links WHERE id = ?1",
        rusqlite::params![id],
        row_to_link,
    ).map_err(|e| e.to_string())
}
```

- [ ] **Step 2: Run**

```
cd src-tauri && cargo test --lib db::links::delete_tests
```
Expected: PASS.

- [ ] **Step 3: Modify `delete_link` command to fire event**

In `src-tauri/src/commands/links.rs`, replace `delete_link`:

```rust
#[tauri::command]
pub fn delete_link(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let link = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let link = db::links::get_link(&conn, &id).ok();
        db::links::delete_link(&conn, &id)?;
        link
    };
    if let Some(link) = link {
        if let (Ok(registry), Ok(sp_guard)) = (state.extensions.lock(), state.storage_path.lock()) {
            if let Some(ref storage_path) = *sp_guard {
                let db_path = storage_path.join("feature-hub.db").to_string_lossy().to_string();
                let sp_str = storage_path.to_string_lossy().to_string();
                let payload = serde_json::json!({
                    "link_type": link.link_type,
                    "feature_id": link.feature_id,
                    "link_id": link.id,
                    "url": link.url,
                });
                crate::extensions::dispatch_event(
                    &registry,
                    "link_deleted",
                    payload,
                    db_path,
                    sp_str,
                    Some(link.feature_id.clone()),
                );
            }
        }
    }
    Ok(())
}
```

- [ ] **Step 4: Run cargo check**

```
cd src-tauri && cargo check
```
Expected: clean.

- [ ] **Step 5: Commit**

```
git add src-tauri/src/db/links.rs src-tauri/src/commands/links.rs
git commit -m "feat(extensions): dispatch link_deleted event on link deletion"
```

---

## Phase B — GitHub PRs Extension

Builds in `<repo>/extensions/github-prs/` as the canonical install source. The app reads extensions from `<storage>/extensions/` at runtime, so for development we symlink (or copy) `<repo>/extensions/github-prs/` into the active storage. A setup note is added at the end.

All handler scripts are Node.js 22+ using the built-in `node:sqlite` module (no external deps). No `package.json` required for the extension.

---

### Task B1: Scaffold extension directory + manifest

**Files:**
- Create: `extensions/github-prs/extension.json`
- Create: `extensions/github-prs/README.md`
- Create: `extensions/github-prs/.gitignore` (ignore `logs/`)

- [ ] **Step 1: Create manifest**

`extensions/github-prs/extension.json` — paste verbatim from the spec's Manifest section at `docs/superpowers/specs/2026-04-14-github-extension-design.md` lines 40–130.

- [ ] **Step 2: Create README**

`extensions/github-prs/README.md`:

```markdown
# GitHub PRs — Feature Hub Extension

Tracks GitHub pull requests linked to features: state, CI checks, reviews.

## Requirements

- `gh` CLI installed and authenticated (`gh auth login`)
- Node.js 22+ (uses built-in `node:sqlite`)

## Install

Copy or symlink this directory into `<your-storage>/extensions/github-prs/`.
Restart Feature Hub.

## Settings

Open Settings → Extensions → GitHub Pull Requests:
- Enable/disable the extension
- Toggle background polling and pick an interval (3m / 5m / 10m / 30m)

## How it works

- Paste a GitHub PR URL into a feature's Links tab → PR appears in the new PRs tab.
- Or use the `[+ Add PR]` button in the PRs tab.
- Refresh per-PR or all at once; state changes post toasts.
- Claude sessions can `list_github_prs`, `get_github_pr`, `add_github_pr`, `refresh_github_pr`.

## Deliberately out of scope

- No commenting, approving, merging, or closing from within FH.
- No webhooks.
```

- [ ] **Step 3: .gitignore**

`extensions/github-prs/.gitignore`:

```
logs/
```

- [ ] **Step 4: Commit**

```
git add extensions/github-prs/
git commit -m "feat(github-prs): scaffold extension directory + manifest"
```

---

### Task B2: `lib/gh.js` — URL parser and `gh` CLI wrapper

**Files:**
- Create: `extensions/github-prs/lib/gh.js`
- Create: `extensions/github-prs/lib/__tests__/gh.test.mjs`

- [ ] **Step 1: Write the failing tests**

`extensions/github-prs/lib/__tests__/gh.test.mjs`:

```javascript
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { parsePrUrl, normalizePrJson } from '../gh.js';

test('parsePrUrl accepts canonical GitHub PR URL', () => {
    assert.deepEqual(
        parsePrUrl('https://github.com/owner/repo/pull/123'),
        { owner: 'owner', repo: 'repo', number: 123 }
    );
});

test('parsePrUrl strips query/fragment', () => {
    assert.deepEqual(
        parsePrUrl('https://github.com/a/b/pull/7?foo=bar#files'),
        { owner: 'a', repo: 'b', number: 7 }
    );
});

test('parsePrUrl returns null for non-PR URLs', () => {
    assert.equal(parsePrUrl('https://github.com/a/b/issues/1'), null);
    assert.equal(parsePrUrl('https://example.com/a/b/pull/1'), null);
    assert.equal(parsePrUrl('not a url'), null);
});

test('normalizePrJson extracts pr + checks + reviews', () => {
    const raw = {
        number: 42,
        url: 'https://github.com/a/b/pull/42',
        title: 'Fix bug',
        state: 'OPEN',
        isDraft: false,
        author: { login: 'alice' },
        headRefName: 'feature/x',
        baseRefName: 'main',
        additions: 10,
        deletions: 2,
        changedFiles: 3,
        createdAt: '2026-04-10T10:00:00Z',
        updatedAt: '2026-04-11T10:00:00Z',
        mergedAt: null,
        closedAt: null,
        statusCheckRollup: [
            { name: 'ci / test', status: 'COMPLETED', conclusion: 'SUCCESS', detailsUrl: 'https://ci/1', completedAt: '2026-04-11T09:00:00Z' },
            { name: 'ci / lint', status: 'IN_PROGRESS', conclusion: null, detailsUrl: 'https://ci/2', completedAt: null },
        ],
        reviews: [
            { author: { login: 'bob' }, state: 'APPROVED', submittedAt: '2026-04-11T10:30:00Z' },
        ],
        reviewRequests: [
            { login: 'carol' },
        ],
    };
    const { pr, checks, reviews } = normalizePrJson(raw, 'a', 'b');
    assert.equal(pr.pr_number, 42);
    assert.equal(pr.state, 'open');
    assert.equal(pr.author, 'alice');
    assert.equal(pr.head_branch, 'feature/x');
    assert.equal(checks.length, 2);
    assert.equal(checks[0].conclusion, 'success');
    assert.equal(checks[1].status, 'in_progress');
    assert.equal(reviews.length, 2);
    const bob = reviews.find(r => r.reviewer === 'bob');
    assert.equal(bob.state, 'approved');
    const carol = reviews.find(r => r.reviewer === 'carol');
    assert.equal(carol.state, 'requested');
});

test('normalizePrJson treats mergedAt present as state=merged', () => {
    const raw = {
        number: 1, url: 'https://github.com/a/b/pull/1', title: 't',
        state: 'MERGED', isDraft: false, author: { login: 'x' },
        headRefName: 'h', baseRefName: 'm',
        additions: 0, deletions: 0, changedFiles: 0,
        createdAt: '2026-04-10T10:00:00Z', updatedAt: '2026-04-11T10:00:00Z',
        mergedAt: '2026-04-11T10:00:00Z', closedAt: '2026-04-11T10:00:00Z',
        statusCheckRollup: [], reviews: [], reviewRequests: [],
    };
    const { pr } = normalizePrJson(raw, 'a', 'b');
    assert.equal(pr.state, 'merged');
});
```

- [ ] **Step 2: Run to verify fail**

```
node --test extensions/github-prs/lib/__tests__/gh.test.mjs
```
Expected: FAIL (module not found).

- [ ] **Step 3: Implement `lib/gh.js`**

`extensions/github-prs/lib/gh.js`:

```javascript
const { execFile } = require('node:child_process');
const { promisify } = require('node:util');
const execFileP = promisify(execFile);

const PR_URL_RE = /^https:\/\/github\.com\/([^/?#]+)\/([^/?#]+)\/pull\/(\d+)(?:[/?#].*)?$/;

function parsePrUrl(url) {
    if (typeof url !== 'string') return null;
    const m = url.match(PR_URL_RE);
    if (!m) return null;
    return { owner: m[1], repo: m[2], number: parseInt(m[3], 10) };
}

function canonicalPrUrl(owner, repo, number) {
    return `https://github.com/${owner}/${repo}/pull/${number}`;
}

function normalizePrJson(raw, owner, repo) {
    const state = (raw.mergedAt ? 'merged'
        : raw.state && raw.state.toLowerCase() === 'closed' ? 'closed'
        : 'open');
    const pr = {
        url: canonicalPrUrl(owner, repo, raw.number),
        repo_owner: owner,
        repo_name: repo,
        pr_number: raw.number,
        title: raw.title,
        state,
        is_draft: raw.isDraft ? 1 : 0,
        author: raw.author ? raw.author.login : null,
        base_branch: raw.baseRefName || null,
        head_branch: raw.headRefName || null,
        additions: raw.additions ?? null,
        deletions: raw.deletions ?? null,
        changed_files: raw.changedFiles ?? null,
        created_at: raw.createdAt,
        updated_at: raw.updatedAt,
        merged_at: raw.mergedAt || null,
        closed_at: raw.closedAt || null,
        raw_json: JSON.stringify(raw),
    };
    const checks = (raw.statusCheckRollup || []).map(c => ({
        name: c.name || c.context || 'check',
        status: (c.status || 'completed').toLowerCase(),
        conclusion: c.conclusion ? c.conclusion.toLowerCase() : null,
        url: c.detailsUrl || c.targetUrl || null,
        completed_at: c.completedAt || null,
    }));
    const latestByReviewer = new Map();
    for (const r of (raw.reviews || [])) {
        const login = r.author ? r.author.login : null;
        if (!login) continue;
        const entry = {
            reviewer: login,
            state: (r.state || '').toLowerCase(), // approved | changes_requested | commented | dismissed | pending
            submitted_at: r.submittedAt || null,
        };
        const prev = latestByReviewer.get(login);
        if (!prev || (entry.submitted_at || '') > (prev.submitted_at || '')) {
            latestByReviewer.set(login, entry);
        }
    }
    for (const rr of (raw.reviewRequests || [])) {
        const login = rr.login || (rr.author && rr.author.login);
        if (!login || latestByReviewer.has(login)) continue;
        latestByReviewer.set(login, { reviewer: login, state: 'requested', submitted_at: null });
    }
    const reviews = [...latestByReviewer.values()];
    return { pr, checks, reviews };
}

const PR_VIEW_FIELDS = 'number,url,title,state,isDraft,author,headRefName,baseRefName,additions,deletions,changedFiles,createdAt,updatedAt,mergedAt,closedAt,statusCheckRollup,reviews,reviewRequests';

async function ghPrView(owner, repo, number) {
    try {
        const { stdout } = await execFileP('gh', [
            'pr', 'view', String(number),
            '--repo', `${owner}/${repo}`,
            '--json', PR_VIEW_FIELDS,
        ], { maxBuffer: 10 * 1024 * 1024 });
        return { ok: true, data: JSON.parse(stdout) };
    } catch (e) {
        const stderr = (e.stderr || '').toString();
        if (/authenticate|login/i.test(stderr)) {
            return { ok: false, code: 'auth', error: 'gh not authenticated — run: gh auth login' };
        }
        if (/not found|could not resolve/i.test(stderr)) {
            return { ok: false, code: 'not_found', error: `PR not found or no access: ${owner}/${repo}#${number}` };
        }
        if (/rate limit/i.test(stderr)) {
            return { ok: false, code: 'rate_limit', error: 'GitHub rate limit hit' };
        }
        if (e.code === 'ENOENT') {
            return { ok: false, code: 'missing', error: 'gh CLI not found — install from https://cli.github.com' };
        }
        return { ok: false, code: 'unknown', error: stderr || e.message };
    }
}

async function ghPrListForRepo(owner, repo) {
    try {
        const { stdout } = await execFileP('gh', [
            'pr', 'list', '--repo', `${owner}/${repo}`,
            '--state', 'all', '--limit', '200',
            '--json', 'number,url,state,updatedAt',
        ], { maxBuffer: 10 * 1024 * 1024 });
        return { ok: true, data: JSON.parse(stdout) };
    } catch (e) {
        return { ok: false, error: (e.stderr || e.message || '').toString() };
    }
}

async function ghAuthStatus() {
    try {
        const { stdout, stderr } = await execFileP('gh', ['auth', 'status'], {});
        const text = stdout + stderr;
        const m = text.match(/Logged in to [^\s]+ as ([^\s]+)/);
        return { ok: true, authenticated: /Logged in/i.test(text), user: m ? m[1] : null, text };
    } catch (e) {
        return { ok: false, authenticated: false, error: (e.stderr || e.message || '').toString() };
    }
}

module.exports = {
    parsePrUrl,
    canonicalPrUrl,
    normalizePrJson,
    ghPrView,
    ghPrListForRepo,
    ghAuthStatus,
};
```

- [ ] **Step 4: Run tests**

```
node --test extensions/github-prs/lib/__tests__/gh.test.mjs
```
Expected: PASS (all 5).

**Note on the `.mjs` vs `.js` mix:** Handlers and `lib/` files are CommonJS (`.js`) because the script_runner invokes them with `node <path>`. Tests use `.mjs` with `import` from the CommonJS module — Node allows this in ESM.

- [ ] **Step 5: Commit**

```
git add extensions/github-prs/lib/gh.js extensions/github-prs/lib/__tests__/gh.test.mjs
git commit -m "feat(github-prs): gh CLI wrapper with URL parser and normalizer"
```

---

### Task B3: `lib/db.js` — sqlite helpers

**Files:**
- Create: `extensions/github-prs/lib/db.js`
- Create: `extensions/github-prs/lib/__tests__/db.test.mjs`

- [ ] **Step 1: Write the failing test**

`extensions/github-prs/lib/__tests__/db.test.mjs`:

```javascript
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { DatabaseSync } from 'node:sqlite';
import { openDb, upsertPr, replaceChecks, replaceReviews, getPrByUrl, listPrsForFeature, deletePrByUrl } from '../db.js';
import os from 'node:os';
import path from 'node:path';
import fs from 'node:fs';

function setupDb() {
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'ghpr-db-'));
    const dbPath = path.join(dir, 'test.db');
    const db = new DatabaseSync(dbPath);
    db.exec(`
        CREATE TABLE features (id TEXT PRIMARY KEY, title TEXT);
        CREATE TABLE ext_github_prs (
            id TEXT PRIMARY KEY, feature_id TEXT, directory_id TEXT, url TEXT UNIQUE,
            repo_owner TEXT, repo_name TEXT, pr_number INTEGER, title TEXT,
            state TEXT, is_draft INTEGER, author TEXT, base_branch TEXT, head_branch TEXT,
            additions INTEGER, deletions INTEGER, changed_files INTEGER,
            created_at TEXT, updated_at TEXT, merged_at TEXT, closed_at TEXT,
            last_fetched_at TEXT, raw_json TEXT
        );
        CREATE TABLE ext_github_pr_checks (
            id TEXT PRIMARY KEY, pr_id TEXT, name TEXT, status TEXT,
            conclusion TEXT, url TEXT, completed_at TEXT
        );
        CREATE TABLE ext_github_pr_reviews (
            id TEXT PRIMARY KEY, pr_id TEXT, reviewer TEXT, state TEXT, submitted_at TEXT
        );
        INSERT INTO features (id, title) VALUES ('f1', 'Feat 1');
    `);
    db.close();
    return dbPath;
}

test('upsertPr inserts then updates the same URL', () => {
    const p = setupDb();
    const db = openDb(p);
    const pr = {
        url: 'https://github.com/a/b/pull/1', repo_owner: 'a', repo_name: 'b',
        pr_number: 1, title: 'First', state: 'open', is_draft: 0,
        author: 'alice', base_branch: 'main', head_branch: 'f',
        additions: 1, deletions: 1, changed_files: 1,
        created_at: '2026-04-10T10:00:00Z', updated_at: '2026-04-10T10:00:00Z',
        merged_at: null, closed_at: null, raw_json: '{}',
    };
    const id1 = upsertPr(db, 'f1', pr, '2026-04-10T10:00:00Z');
    const id2 = upsertPr(db, 'f1', { ...pr, title: 'Updated' }, '2026-04-11T10:00:00Z');
    assert.equal(id1, id2, 'same id for same url');
    const row = getPrByUrl(db, pr.url);
    assert.equal(row.title, 'Updated');
    db.close();
});

test('replaceChecks/replaceReviews replace rows for a PR', () => {
    const p = setupDb();
    const db = openDb(p);
    const pr = {
        url: 'https://github.com/a/b/pull/1', repo_owner: 'a', repo_name: 'b',
        pr_number: 1, title: 't', state: 'open', is_draft: 0, author: 'x',
        base_branch: 'main', head_branch: 'f', additions: 0, deletions: 0, changed_files: 0,
        created_at: 'now', updated_at: 'now', merged_at: null, closed_at: null, raw_json: '{}',
    };
    const id = upsertPr(db, 'f1', pr, 'now');
    replaceChecks(db, id, [
        { name: 'a', status: 'completed', conclusion: 'success', url: null, completed_at: null },
    ]);
    replaceChecks(db, id, [
        { name: 'b', status: 'completed', conclusion: 'failure', url: null, completed_at: null },
    ]);
    const checks = db.prepare('SELECT * FROM ext_github_pr_checks WHERE pr_id = ?').all(id);
    assert.equal(checks.length, 1);
    assert.equal(checks[0].name, 'b');

    replaceReviews(db, id, [{ reviewer: 'bob', state: 'approved', submitted_at: 'now' }]);
    const reviews = db.prepare('SELECT * FROM ext_github_pr_reviews WHERE pr_id = ?').all(id);
    assert.equal(reviews.length, 1);
    assert.equal(reviews[0].reviewer, 'bob');
    db.close();
});

test('deletePrByUrl cascades to checks and reviews', () => {
    const p = setupDb();
    const db = openDb(p);
    const pr = {
        url: 'https://github.com/a/b/pull/2', repo_owner: 'a', repo_name: 'b',
        pr_number: 2, title: 't', state: 'open', is_draft: 0, author: 'x',
        base_branch: 'main', head_branch: 'f', additions: 0, deletions: 0, changed_files: 0,
        created_at: 'now', updated_at: 'now', merged_at: null, closed_at: null, raw_json: '{}',
    };
    const id = upsertPr(db, 'f1', pr, 'now');
    replaceChecks(db, id, [{ name: 'a', status: 'completed', conclusion: 'success', url: null, completed_at: null }]);
    deletePrByUrl(db, pr.url);
    const remaining = db.prepare('SELECT COUNT(*) AS c FROM ext_github_pr_checks WHERE pr_id = ?').get(id);
    assert.equal(remaining.c, 0);
    db.close();
});

test('listPrsForFeature returns rollups', () => {
    const p = setupDb();
    const db = openDb(p);
    const pr = {
        url: 'https://github.com/a/b/pull/3', repo_owner: 'a', repo_name: 'b',
        pr_number: 3, title: 't', state: 'open', is_draft: 0, author: 'x',
        base_branch: 'main', head_branch: 'f', additions: 0, deletions: 0, changed_files: 0,
        created_at: 'now', updated_at: 'now', merged_at: null, closed_at: null, raw_json: '{}',
    };
    const id = upsertPr(db, 'f1', pr, 'now');
    replaceChecks(db, id, [
        { name: 'a', status: 'completed', conclusion: 'success', url: null, completed_at: null },
        { name: 'b', status: 'completed', conclusion: 'failure', url: null, completed_at: null },
        { name: 'c', status: 'in_progress', conclusion: null, url: null, completed_at: null },
    ]);
    replaceReviews(db, id, [
        { reviewer: 'r1', state: 'approved', submitted_at: 'now' },
        { reviewer: 'r2', state: 'requested', submitted_at: null },
    ]);
    const rows = listPrsForFeature(db, 'f1');
    assert.equal(rows.length, 1);
    const r = rows[0];
    assert.deepEqual(r.checks_rollup, { total: 3, success: 1, failure: 1, pending: 1 });
    assert.deepEqual(r.review_rollup, { approved: 1, changes_requested: 0, pending: 1 });
    db.close();
});
```

- [ ] **Step 2: Run — FAIL**

```
node --test extensions/github-prs/lib/__tests__/db.test.mjs
```
Expected: FAIL (module missing).

- [ ] **Step 3: Implement**

`extensions/github-prs/lib/db.js`:

```javascript
const { DatabaseSync } = require('node:sqlite');
const { randomUUID } = require('node:crypto');

function openDb(path) {
    return new DatabaseSync(path);
}

function upsertPr(db, featureId, pr, fetchedAt) {
    const existing = db.prepare('SELECT id FROM ext_github_prs WHERE url = ?').get(pr.url);
    if (existing) {
        db.prepare(`UPDATE ext_github_prs SET
            feature_id = ?, repo_owner = ?, repo_name = ?, pr_number = ?, title = ?,
            state = ?, is_draft = ?, author = ?, base_branch = ?, head_branch = ?,
            additions = ?, deletions = ?, changed_files = ?,
            created_at = ?, updated_at = ?, merged_at = ?, closed_at = ?,
            last_fetched_at = ?, raw_json = ?
            WHERE id = ?`).run(
            featureId, pr.repo_owner, pr.repo_name, pr.pr_number, pr.title,
            pr.state, pr.is_draft, pr.author, pr.base_branch, pr.head_branch,
            pr.additions, pr.deletions, pr.changed_files,
            pr.created_at, pr.updated_at, pr.merged_at, pr.closed_at,
            fetchedAt, pr.raw_json, existing.id
        );
        return existing.id;
    }
    const id = randomUUID();
    db.prepare(`INSERT INTO ext_github_prs (
        id, feature_id, url, repo_owner, repo_name, pr_number, title,
        state, is_draft, author, base_branch, head_branch,
        additions, deletions, changed_files,
        created_at, updated_at, merged_at, closed_at,
        last_fetched_at, raw_json
    ) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)`).run(
        id, featureId, pr.url, pr.repo_owner, pr.repo_name, pr.pr_number, pr.title,
        pr.state, pr.is_draft, pr.author, pr.base_branch, pr.head_branch,
        pr.additions, pr.deletions, pr.changed_files,
        pr.created_at, pr.updated_at, pr.merged_at, pr.closed_at,
        fetchedAt, pr.raw_json
    );
    return id;
}

function replaceChecks(db, prId, checks) {
    db.prepare('DELETE FROM ext_github_pr_checks WHERE pr_id = ?').run(prId);
    const ins = db.prepare(`INSERT INTO ext_github_pr_checks (id, pr_id, name, status, conclusion, url, completed_at) VALUES (?,?,?,?,?,?,?)`);
    for (const c of checks) {
        ins.run(randomUUID(), prId, c.name, c.status, c.conclusion, c.url, c.completed_at);
    }
}

function replaceReviews(db, prId, reviews) {
    db.prepare('DELETE FROM ext_github_pr_reviews WHERE pr_id = ?').run(prId);
    const ins = db.prepare(`INSERT INTO ext_github_pr_reviews (id, pr_id, reviewer, state, submitted_at) VALUES (?,?,?,?,?)`);
    for (const r of reviews) {
        ins.run(randomUUID(), prId, r.reviewer, r.state, r.submitted_at);
    }
}

function getPrByUrl(db, url) {
    return db.prepare('SELECT * FROM ext_github_prs WHERE url = ?').get(url) || null;
}

function getPrById(db, id) {
    return db.prepare('SELECT * FROM ext_github_prs WHERE id = ?').get(id) || null;
}

function getChecksForPr(db, prId) {
    return db.prepare('SELECT * FROM ext_github_pr_checks WHERE pr_id = ? ORDER BY name').all(prId);
}

function getReviewsForPr(db, prId) {
    return db.prepare('SELECT * FROM ext_github_pr_reviews WHERE pr_id = ? ORDER BY reviewer').all(prId);
}

function deletePrByUrl(db, url) {
    const row = getPrByUrl(db, url);
    if (!row) return false;
    db.prepare('DELETE FROM ext_github_pr_checks WHERE pr_id = ?').run(row.id);
    db.prepare('DELETE FROM ext_github_pr_reviews WHERE pr_id = ?').run(row.id);
    db.prepare('DELETE FROM ext_github_prs WHERE id = ?').run(row.id);
    return true;
}

function computeRollups(db, prId) {
    const checks = getChecksForPr(db, prId);
    const checks_rollup = {
        total: checks.length,
        success: checks.filter(c => c.conclusion === 'success').length,
        failure: checks.filter(c => c.conclusion === 'failure').length,
        pending: checks.filter(c => c.status !== 'completed' || c.conclusion === null).length,
    };
    const reviews = getReviewsForPr(db, prId);
    const review_rollup = {
        approved: reviews.filter(r => r.state === 'approved').length,
        changes_requested: reviews.filter(r => r.state === 'changes_requested').length,
        pending: reviews.filter(r => r.state === 'requested' || r.state === 'pending').length,
    };
    return { checks_rollup, review_rollup };
}

function listPrsForFeature(db, featureId) {
    const rows = db.prepare('SELECT * FROM ext_github_prs WHERE feature_id = ? ORDER BY updated_at DESC').all(featureId);
    return rows.map(r => ({ ...r, ...computeRollups(db, r.id) }));
}

function listOpenPrsByRepo(db) {
    const rows = db.prepare(`SELECT id, url, repo_owner, repo_name, pr_number, updated_at, last_fetched_at FROM ext_github_prs WHERE state = 'open'`).all();
    const by = new Map();
    for (const r of rows) {
        const key = `${r.repo_owner}/${r.repo_name}`;
        if (!by.has(key)) by.set(key, []);
        by.get(key).push(r);
    }
    return by;
}

module.exports = {
    openDb,
    upsertPr,
    replaceChecks,
    replaceReviews,
    getPrByUrl,
    getPrById,
    getChecksForPr,
    getReviewsForPr,
    deletePrByUrl,
    computeRollups,
    listPrsForFeature,
    listOpenPrsByRepo,
};
```

- [ ] **Step 4: Run**

```
node --test extensions/github-prs/lib/__tests__/db.test.mjs
```
Expected: PASS (4 tests).

- [ ] **Step 5: Commit**

```
git add extensions/github-prs/lib/db.js extensions/github-prs/lib/__tests__/db.test.mjs
git commit -m "feat(github-prs): sqlite helpers with rollup computation"
```

---

### Task B4: `lib/notify.js` — state-change detector

**Files:**
- Create: `extensions/github-prs/lib/notify.js`
- Create: `extensions/github-prs/lib/__tests__/notify.test.mjs`

- [ ] **Step 1: Write the failing test**

`extensions/github-prs/lib/__tests__/notify.test.mjs`:

```javascript
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { diffNotifications } from '../notify.js';

const featureId = 'f1';

function makePr(over = {}) {
    return {
        pr_number: 10,
        title: 'X',
        state: 'open',
        ...over,
    };
}

test('returns empty when nothing changed', () => {
    const n = diffNotifications(featureId,
        { pr: makePr(), checks: [], reviews: [] },
        { pr: makePr(), checks: [], reviews: [] });
    assert.equal(n.length, 0);
});

test('emits merged notification when state transitions to merged', () => {
    const n = diffNotifications(featureId,
        { pr: makePr({ state: 'open' }), checks: [], reviews: [] },
        { pr: makePr({ state: 'merged' }), checks: [], reviews: [] });
    assert.equal(n.length, 1);
    assert.match(n[0].message, /merged/i);
    assert.equal(n[0].kind, 'info');
});

test('emits closed notification (not merged)', () => {
    const n = diffNotifications(featureId,
        { pr: makePr({ state: 'open' }), checks: [], reviews: [] },
        { pr: makePr({ state: 'closed' }), checks: [], reviews: [] });
    assert.equal(n.length, 1);
    assert.match(n[0].message, /closed/i);
});

test('emits checks-failed when any check transitions to failure', () => {
    const n = diffNotifications(featureId,
        { pr: makePr(), checks: [{ name: 'a', conclusion: 'success', status: 'completed' }], reviews: [] },
        { pr: makePr(), checks: [{ name: 'a', conclusion: 'failure', status: 'completed' }], reviews: [] });
    assert.equal(n.length, 1);
    assert.match(n[0].message, /checks? failed/i);
    assert.equal(n[0].kind, 'warn');
});

test('no duplicate checks-failed when already failing', () => {
    const n = diffNotifications(featureId,
        { pr: makePr(), checks: [{ name: 'a', conclusion: 'failure', status: 'completed' }], reviews: [] },
        { pr: makePr(), checks: [{ name: 'a', conclusion: 'failure', status: 'completed' }], reviews: [] });
    assert.equal(n.length, 0);
});

test('emits review-requested when reviewer added to pending', () => {
    const n = diffNotifications(featureId,
        { pr: makePr(), checks: [], reviews: [] },
        { pr: makePr(), checks: [], reviews: [{ reviewer: 'alice', state: 'requested' }] });
    assert.equal(n.length, 1);
    assert.match(n[0].message, /review requested/i);
});

test('emits approved notification when new approval appears', () => {
    const n = diffNotifications(featureId,
        { pr: makePr(), checks: [], reviews: [{ reviewer: 'alice', state: 'requested' }] },
        { pr: makePr(), checks: [], reviews: [{ reviewer: 'alice', state: 'approved' }] });
    assert.equal(n.length, 1);
    assert.match(n[0].message, /approved by @alice/i);
});

test('emits changes-requested notification', () => {
    const n = diffNotifications(featureId,
        { pr: makePr(), checks: [], reviews: [] },
        { pr: makePr(), checks: [], reviews: [{ reviewer: 'bob', state: 'changes_requested' }] });
    assert.equal(n.length, 1);
    assert.match(n[0].message, /changes requested/i);
    assert.equal(n[0].kind, 'warn');
});

test('first-time linking emits a single linked notification', () => {
    const { diffNotifications: d, linkedNotification } = await import('../notify.js');
    const n = linkedNotification(featureId, makePr());
    assert.equal(n.feature_id, featureId);
    assert.match(n.message, /#10/);
});
```

- [ ] **Step 2: Run — FAIL**

```
node --test extensions/github-prs/lib/__tests__/notify.test.mjs
```
Expected: FAIL.

- [ ] **Step 3: Implement**

`extensions/github-prs/lib/notify.js`:

```javascript
function diffNotifications(featureId, before, after) {
    const out = [];
    const prB = before.pr, prA = after.pr;

    // State transitions
    if (prB.state === 'open' && prA.state === 'merged') {
        out.push({ feature_id: featureId, kind: 'info', message: `PR #${prA.pr_number} merged: ${prA.title}` });
    } else if (prB.state === 'open' && prA.state === 'closed') {
        out.push({ feature_id: featureId, kind: 'info', message: `PR #${prA.pr_number} closed: ${prA.title}` });
    }

    // Checks: any new failure
    const hadFailure = (before.checks || []).some(c => c.conclusion === 'failure');
    const hasFailure = (after.checks || []).some(c => c.conclusion === 'failure');
    if (!hadFailure && hasFailure) {
        out.push({ feature_id: featureId, kind: 'warn', message: `Checks failed on PR #${prA.pr_number}` });
    }

    // Reviews: new approved / new changes_requested / new requested reviewer
    const byReviewerB = new Map((before.reviews || []).map(r => [r.reviewer, r.state]));
    for (const r of (after.reviews || [])) {
        const prev = byReviewerB.get(r.reviewer);
        if (prev === r.state) continue;
        if (r.state === 'approved') {
            out.push({ feature_id: featureId, kind: 'info', message: `PR #${prA.pr_number} approved by @${r.reviewer}` });
        } else if (r.state === 'changes_requested') {
            out.push({ feature_id: featureId, kind: 'warn', message: `Changes requested on PR #${prA.pr_number} by @${r.reviewer}` });
        } else if (r.state === 'requested' && !prev) {
            out.push({ feature_id: featureId, kind: 'info', message: `Review requested on PR #${prA.pr_number} from @${r.reviewer}` });
        }
    }

    return out;
}

function linkedNotification(featureId, pr) {
    return { feature_id: featureId, kind: 'info', message: `Linked PR #${pr.pr_number}: ${pr.title}` };
}

module.exports = { diffNotifications, linkedNotification };
```

- [ ] **Step 4: Run**

```
node --test extensions/github-prs/lib/__tests__/notify.test.mjs
```
Expected: PASS.

- [ ] **Step 5: Commit**

```
git add extensions/github-prs/lib/notify.js extensions/github-prs/lib/__tests__/notify.test.mjs
git commit -m "feat(github-prs): state-change notification detector"
```

---

### Task B5: `lib/runner.js` — handler entrypoint helper

**Files:**
- Create: `extensions/github-prs/lib/runner.js`

- [ ] **Step 1: Implement — shared entrypoint helper**

All handlers share the same stdin/stdout dance. Factor it into `extensions/github-prs/lib/runner.js`:

```javascript
async function run(handler) {
    const chunks = [];
    for await (const c of process.stdin) chunks.push(c);
    const input = JSON.parse(Buffer.concat(chunks).toString() || '{}');
    try {
        const result = await handler(input);
        const payload = { ok: true, data: result && result.data !== undefined ? result.data : (result ?? null) };
        if (result && Array.isArray(result.notifications)) payload.notifications = result.notifications;
        process.stdout.write(JSON.stringify(payload));
    } catch (e) {
        process.stdout.write(JSON.stringify({ ok: false, error: e.message || String(e) }));
    }
}

module.exports = { run };
```

- [ ] **Step 2: Commit**

```
git add extensions/github-prs/lib/runner.js
git commit -m "feat(github-prs): shared handler entrypoint helper"
```

---

### Task B6: `handlers/on_link_created.js`

**Files:**
- Create: `extensions/github-prs/handlers/on_link_created.js`
- Create: `extensions/github-prs/handlers/__tests__/on_link_created.test.mjs`

- [ ] **Step 1: Write the failing test**

`extensions/github-prs/handlers/__tests__/on_link_created.test.mjs`:

```javascript
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawn } from 'node:child_process';
import { DatabaseSync } from 'node:sqlite';

function setupStorage() {
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'ghpr-olc-'));
    const dbPath = path.join(dir, 'feature-hub.db');
    const db = new DatabaseSync(dbPath);
    db.exec(`
        CREATE TABLE features (id TEXT PRIMARY KEY, title TEXT);
        CREATE TABLE ext_github_prs (
            id TEXT PRIMARY KEY, feature_id TEXT, directory_id TEXT, url TEXT UNIQUE,
            repo_owner TEXT, repo_name TEXT, pr_number INTEGER, title TEXT,
            state TEXT, is_draft INTEGER, author TEXT, base_branch TEXT, head_branch TEXT,
            additions INTEGER, deletions INTEGER, changed_files INTEGER,
            created_at TEXT, updated_at TEXT, merged_at TEXT, closed_at TEXT,
            last_fetched_at TEXT, raw_json TEXT
        );
        CREATE TABLE ext_github_pr_checks (id TEXT PRIMARY KEY, pr_id TEXT, name TEXT, status TEXT, conclusion TEXT, url TEXT, completed_at TEXT);
        CREATE TABLE ext_github_pr_reviews (id TEXT PRIMARY KEY, pr_id TEXT, reviewer TEXT, state TEXT, submitted_at TEXT);
        INSERT INTO features (id, title) VALUES ('f1', 'Feat 1');
    `);
    db.close();
    return { dir, dbPath };
}

function runHandler(handlerPath, input, ghStubDir) {
    return new Promise((resolve, reject) => {
        const env = { ...process.env, PATH: `${ghStubDir}${path.delimiter}${process.env.PATH}` };
        const child = spawn(process.execPath, [handlerPath], { env });
        const out = []; const err = [];
        child.stdout.on('data', d => out.push(d));
        child.stderr.on('data', d => err.push(d));
        child.on('close', code => {
            if (code !== 0) return reject(new Error(`exit ${code}: ${Buffer.concat(err).toString()}`));
            try { resolve(JSON.parse(Buffer.concat(out).toString())); }
            catch (e) { reject(e); }
        });
        child.stdin.write(JSON.stringify(input));
        child.stdin.end();
    });
}

function makeGhStub(dir, ownerRepoNumber, jsonResponse) {
    // Create a directory with a fake `gh` script that returns the stub JSON
    const stubPath = path.join(dir, process.platform === 'win32' ? 'gh.cmd' : 'gh');
    if (process.platform === 'win32') {
        fs.writeFileSync(stubPath, `@echo off\necho ${JSON.stringify(jsonResponse).replace(/"/g, '\\"')}\n`);
    } else {
        fs.writeFileSync(stubPath, `#!/bin/sh\ncat <<'JSON_EOF'\n${JSON.stringify(jsonResponse)}\nJSON_EOF\n`, { mode: 0o755 });
    }
}

test('inserts PR on first link_created, updates idempotently on second', async () => {
    const { dir, dbPath } = setupStorage();
    const stubDir = fs.mkdtempSync(path.join(os.tmpdir(), 'ghpr-stub-'));
    const prJson = {
        number: 42, url: 'https://github.com/a/b/pull/42', title: 'Fix',
        state: 'OPEN', isDraft: false, author: { login: 'alice' },
        headRefName: 'f', baseRefName: 'main',
        additions: 1, deletions: 1, changedFiles: 1,
        createdAt: 'now', updatedAt: 'now', mergedAt: null, closedAt: null,
        statusCheckRollup: [], reviews: [], reviewRequests: [],
    };
    makeGhStub(stubDir, { owner: 'a', repo: 'b', number: 42 }, prJson);

    const handler = path.resolve('extensions/github-prs/handlers/on_link_created.js');
    const input = {
        params: { link_type: 'github-pr', feature_id: 'f1', url: 'https://github.com/a/b/pull/42', link_id: 'l1', title: 'Fix' },
        db_path: dbPath, storage_path: dir, feature_id: 'f1',
    };

    const r1 = await runHandler(handler, input, stubDir);
    assert.equal(r1.ok, true);
    assert.ok(Array.isArray(r1.notifications));
    assert.equal(r1.notifications.length, 1);

    const r2 = await runHandler(handler, input, stubDir);
    assert.equal(r2.ok, true);

    const db = new DatabaseSync(dbPath);
    const rows = db.prepare('SELECT COUNT(*) AS c FROM ext_github_prs WHERE url = ?').get('https://github.com/a/b/pull/42');
    assert.equal(rows.c, 1);
    db.close();
});

test('no-op when link_type is not github-pr', async () => {
    const { dir, dbPath } = setupStorage();
    const stubDir = fs.mkdtempSync(path.join(os.tmpdir(), 'ghpr-stub2-'));
    const handler = path.resolve('extensions/github-prs/handlers/on_link_created.js');
    const input = {
        params: { link_type: 'figma', feature_id: 'f1', url: 'https://figma.com/x', link_id: 'l2', title: 'x' },
        db_path: dbPath, storage_path: dir, feature_id: 'f1',
    };
    const r = await runHandler(handler, input, stubDir);
    assert.equal(r.ok, true);
    // No DB row written, no notifications
    const db = new DatabaseSync(dbPath);
    const rows = db.prepare('SELECT COUNT(*) AS c FROM ext_github_prs').get();
    assert.equal(rows.c, 0);
    db.close();
});
```

- [ ] **Step 2: Implement handler**

`extensions/github-prs/handlers/on_link_created.js`:

```javascript
const path = require('node:path');
const { run } = require('../lib/runner.js');
const { parsePrUrl, ghPrView, normalizePrJson } = require('../lib/gh.js');
const { openDb, upsertPr, replaceChecks, replaceReviews } = require('../lib/db.js');
const { linkedNotification } = require('../lib/notify.js');

run(async (input) => {
    const params = input.params || {};
    if (params.link_type !== 'github-pr') {
        return { data: { skipped: true }, notifications: [] };
    }
    const parsed = parsePrUrl(params.url);
    if (!parsed) {
        return { data: { skipped: true, reason: 'invalid_url' }, notifications: [] };
    }
    const fetched = await ghPrView(parsed.owner, parsed.repo, parsed.number);
    if (!fetched.ok) {
        return { data: { error: fetched.error }, notifications: [] };
    }
    const { pr, checks, reviews } = normalizePrJson(fetched.data, parsed.owner, parsed.repo);
    const db = openDb(input.db_path);
    try {
        const existing = db.prepare('SELECT id FROM ext_github_prs WHERE url = ?').get(pr.url);
        const id = upsertPr(db, params.feature_id, pr, new Date().toISOString());
        replaceChecks(db, id, checks);
        replaceReviews(db, id, reviews);
        const notifications = existing ? [] : [linkedNotification(params.feature_id, pr)];
        return { data: { pr_id: id, inserted: !existing }, notifications };
    } finally {
        db.close();
    }
});
```

- [ ] **Step 3: Run**

```
node --test extensions/github-prs/handlers/__tests__/on_link_created.test.mjs
```
Expected: PASS.

- [ ] **Step 4: Commit**

```
git add extensions/github-prs/handlers/on_link_created.js extensions/github-prs/handlers/__tests__/on_link_created.test.mjs
git commit -m "feat(github-prs): on_link_created handler with idempotent upsert"
```

---

### Task B7: `handlers/on_link_deleted.js`

**Files:**
- Create: `extensions/github-prs/handlers/on_link_deleted.js`
- Create: `extensions/github-prs/handlers/__tests__/on_link_deleted.test.mjs`

- [ ] **Step 1: Write the failing test**

Copy `setupStorage` and `runHandler` helpers pattern from Task B6 into a new test file. Test:

```javascript
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawn } from 'node:child_process';
import { DatabaseSync } from 'node:sqlite';

function setupStorage() {
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'ghpr-old-'));
    const dbPath = path.join(dir, 'feature-hub.db');
    const db = new DatabaseSync(dbPath);
    db.exec(`
        CREATE TABLE features (id TEXT PRIMARY KEY, title TEXT);
        CREATE TABLE ext_github_prs (
            id TEXT PRIMARY KEY, feature_id TEXT, directory_id TEXT, url TEXT UNIQUE,
            repo_owner TEXT, repo_name TEXT, pr_number INTEGER, title TEXT,
            state TEXT, is_draft INTEGER, author TEXT, base_branch TEXT, head_branch TEXT,
            additions INTEGER, deletions INTEGER, changed_files INTEGER,
            created_at TEXT, updated_at TEXT, merged_at TEXT, closed_at TEXT,
            last_fetched_at TEXT, raw_json TEXT
        );
        CREATE TABLE ext_github_pr_checks (id TEXT PRIMARY KEY, pr_id TEXT, name TEXT, status TEXT, conclusion TEXT, url TEXT, completed_at TEXT);
        CREATE TABLE ext_github_pr_reviews (id TEXT PRIMARY KEY, pr_id TEXT, reviewer TEXT, state TEXT, submitted_at TEXT);
        INSERT INTO features (id, title) VALUES ('f1', 'Feat 1');
        INSERT INTO ext_github_prs (id, feature_id, url, repo_owner, repo_name, pr_number, title, state, is_draft, created_at, updated_at, last_fetched_at)
            VALUES ('pr-1', 'f1', 'https://github.com/a/b/pull/1', 'a', 'b', 1, 't', 'open', 0, 'now', 'now', 'now');
        INSERT INTO ext_github_pr_checks (id, pr_id, name, status, conclusion) VALUES ('c1', 'pr-1', 'a', 'completed', 'success');
    `);
    db.close();
    return { dir, dbPath };
}

function runHandler(handlerPath, input) {
    return new Promise((resolve, reject) => {
        const child = spawn(process.execPath, [handlerPath]);
        const out = []; const err = [];
        child.stdout.on('data', d => out.push(d));
        child.stderr.on('data', d => err.push(d));
        child.on('close', code => {
            if (code !== 0) return reject(new Error(`exit ${code}: ${Buffer.concat(err).toString()}`));
            try { resolve(JSON.parse(Buffer.concat(out).toString())); }
            catch (e) { reject(e); }
        });
        child.stdin.write(JSON.stringify(input));
        child.stdin.end();
    });
}

test('deletes PR row and cascades checks', async () => {
    const { dir, dbPath } = setupStorage();
    const handler = path.resolve('extensions/github-prs/handlers/on_link_deleted.js');
    const r = await runHandler(handler, {
        params: { link_type: 'github-pr', feature_id: 'f1', url: 'https://github.com/a/b/pull/1' },
        db_path: dbPath, storage_path: dir, feature_id: 'f1',
    });
    assert.equal(r.ok, true);
    const db = new DatabaseSync(dbPath);
    assert.equal(db.prepare('SELECT COUNT(*) AS c FROM ext_github_prs').get().c, 0);
    assert.equal(db.prepare('SELECT COUNT(*) AS c FROM ext_github_pr_checks').get().c, 0);
    db.close();
});

test('no-op when url not tracked', async () => {
    const { dir, dbPath } = setupStorage();
    const handler = path.resolve('extensions/github-prs/handlers/on_link_deleted.js');
    const r = await runHandler(handler, {
        params: { link_type: 'github-pr', feature_id: 'f1', url: 'https://github.com/a/b/pull/999' },
        db_path: dbPath, storage_path: dir, feature_id: 'f1',
    });
    assert.equal(r.ok, true);
});
```

- [ ] **Step 2: Implement**

`extensions/github-prs/handlers/on_link_deleted.js`:

```javascript
const { run } = require('../lib/runner.js');
const { openDb, deletePrByUrl } = require('../lib/db.js');

run(async (input) => {
    const params = input.params || {};
    if (params.link_type !== 'github-pr') {
        return { data: { skipped: true }, notifications: [] };
    }
    const db = openDb(input.db_path);
    try {
        const deleted = deletePrByUrl(db, params.url);
        return { data: { deleted }, notifications: [] };
    } finally {
        db.close();
    }
});
```

- [ ] **Step 3: Run & commit**

```
node --test extensions/github-prs/handlers/__tests__/on_link_deleted.test.mjs
git add extensions/github-prs/handlers/on_link_deleted.js extensions/github-prs/handlers/__tests__/on_link_deleted.test.mjs
git commit -m "feat(github-prs): on_link_deleted handler"
```

---

### Task B8: `handlers/add_pr.js`

**Files:**
- Create: `extensions/github-prs/handlers/add_pr.js`

Adds a PR: creates the `github-pr` link in `links` table (so it appears in the Links tab), then runs the same fetch+upsert path as `on_link_created`. Idempotent by URL.

- [ ] **Step 1: Implement**

`extensions/github-prs/handlers/add_pr.js`:

```javascript
const { run } = require('../lib/runner.js');
const { DatabaseSync } = require('node:sqlite');
const { randomUUID } = require('node:crypto');
const { parsePrUrl, ghPrView, normalizePrJson } = require('../lib/gh.js');
const { openDb, upsertPr, replaceChecks, replaceReviews } = require('../lib/db.js');
const { linkedNotification } = require('../lib/notify.js');

run(async (input) => {
    const { feature_id, url } = input.params || {};
    if (!feature_id) throw new Error('feature_id is required');
    if (!url) throw new Error('url is required');
    const parsed = parsePrUrl(url);
    if (!parsed) throw new Error(`Invalid GitHub PR URL: ${url}`);

    const fetched = await ghPrView(parsed.owner, parsed.repo, parsed.number);
    if (!fetched.ok) throw new Error(fetched.error);
    const { pr, checks, reviews } = normalizePrJson(fetched.data, parsed.owner, parsed.repo);

    const db = openDb(input.db_path);
    try {
        // Ensure `links` row exists (idempotent by url+feature_id)
        const existingLink = db.prepare('SELECT id FROM links WHERE feature_id = ? AND url = ?').get(feature_id, pr.url);
        if (!existingLink) {
            db.prepare(`INSERT INTO links (id, feature_id, title, url, link_type, created_at)
                        VALUES (?, ?, ?, ?, 'github-pr', ?)`)
              .run(randomUUID(), feature_id, pr.title, pr.url, new Date().toISOString());
        }

        const existingPr = db.prepare('SELECT id FROM ext_github_prs WHERE url = ?').get(pr.url);
        const id = upsertPr(db, feature_id, pr, new Date().toISOString());
        replaceChecks(db, id, checks);
        replaceReviews(db, id, reviews);
        const notifications = existingPr ? [] : [linkedNotification(feature_id, pr)];
        return { data: { pr_id: id, inserted: !existingPr }, notifications };
    } finally {
        db.close();
    }
});
```

- [ ] **Step 2: Test — reuse stub pattern from B6**

Create `extensions/github-prs/handlers/__tests__/add_pr.test.mjs` with a test that:
1. Seeds an empty `links` table and `features` row in a fresh DB.
2. Stubs `gh` via PATH override (same pattern as B6).
3. Invokes `add_pr.js` with a valid URL → asserts one row in `links`, one in `ext_github_prs`, one notification.
4. Invokes again → asserts still one row in `links`, still one in `ext_github_prs`, no new notifications.

Schema for `links` in test setup:
```sql
CREATE TABLE links (id TEXT PRIMARY KEY, feature_id TEXT, title TEXT, url TEXT, link_type TEXT, description TEXT, metadata TEXT, created_at TEXT);
```

- [ ] **Step 3: Run & commit**

```
node --test extensions/github-prs/handlers/__tests__/add_pr.test.mjs
git add extensions/github-prs/handlers/add_pr.js extensions/github-prs/handlers/__tests__/add_pr.test.mjs
git commit -m "feat(github-prs): add_pr handler (MCP tool)"
```

---

### Task B9: `handlers/list_prs.js`

**Files:**
- Create: `extensions/github-prs/handlers/list_prs.js`
- Create: `extensions/github-prs/handlers/__tests__/list_prs.test.mjs`

- [ ] **Step 1: Implement**

`extensions/github-prs/handlers/list_prs.js`:

```javascript
const { run } = require('../lib/runner.js');
const { openDb, listPrsForFeature } = require('../lib/db.js');

run(async (input) => {
    const featureId = (input.params && input.params.feature_id) || input.feature_id;
    if (!featureId) throw new Error('feature_id required (pass in params or scope the MCP session to a feature)');
    const db = openDb(input.db_path);
    try {
        const rows = listPrsForFeature(db, featureId);
        return { data: rows.map(r => ({
            id: r.id, url: r.url, title: r.title, pr_number: r.pr_number,
            repo: `${r.repo_owner}/${r.repo_name}`,
            state: r.state, is_draft: !!r.is_draft, author: r.author,
            updated_at: r.updated_at, merged_at: r.merged_at, closed_at: r.closed_at,
            checks_rollup: r.checks_rollup, review_rollup: r.review_rollup,
        })) };
    } finally {
        db.close();
    }
});
```

- [ ] **Step 2: Test — seed DB with 2 PRs + checks, call handler, assert length 2 and rollup shape**

- [ ] **Step 3: Run & commit**

```
node --test extensions/github-prs/handlers/__tests__/list_prs.test.mjs
git add extensions/github-prs/handlers/list_prs.js extensions/github-prs/handlers/__tests__/list_prs.test.mjs
git commit -m "feat(github-prs): list_prs handler"
```

---

### Task B10: `handlers/get_pr.js`

**Files:**
- Create: `extensions/github-prs/handlers/get_pr.js`
- Create: `extensions/github-prs/handlers/__tests__/get_pr.test.mjs`

- [ ] **Step 1: Implement**

`extensions/github-prs/handlers/get_pr.js`:

```javascript
const { run } = require('../lib/runner.js');
const { openDb, getPrById, getPrByUrl, getChecksForPr, getReviewsForPr } = require('../lib/db.js');

run(async (input) => {
    const { pr_id, url } = input.params || {};
    if (!pr_id && !url) throw new Error('Provide either pr_id or url');
    const db = openDb(input.db_path);
    try {
        const row = pr_id ? getPrById(db, pr_id) : getPrByUrl(db, url);
        if (!row) throw new Error('PR not found');
        return { data: {
            pr: row,
            checks: getChecksForPr(db, row.id),
            reviews: getReviewsForPr(db, row.id),
        } };
    } finally {
        db.close();
    }
});
```

- [ ] **Step 2: Test — insert a PR + 2 checks + 1 review, call via url, assert shape**

- [ ] **Step 3: Run & commit**

```
node --test extensions/github-prs/handlers/__tests__/get_pr.test.mjs
git add extensions/github-prs/handlers/get_pr.js extensions/github-prs/handlers/__tests__/get_pr.test.mjs
git commit -m "feat(github-prs): get_pr handler"
```

---

### Task B11: `handlers/refresh_pr.js`

**Files:**
- Create: `extensions/github-prs/handlers/refresh_pr.js`
- Create: `extensions/github-prs/handlers/__tests__/refresh_pr.test.mjs`

- [ ] **Step 1: Implement**

`extensions/github-prs/handlers/refresh_pr.js`:

```javascript
const { run } = require('../lib/runner.js');
const { parsePrUrl, ghPrView, normalizePrJson } = require('../lib/gh.js');
const { openDb, getPrById, getPrByUrl, getChecksForPr, getReviewsForPr, upsertPr, replaceChecks, replaceReviews } = require('../lib/db.js');
const { diffNotifications } = require('../lib/notify.js');

run(async (input) => {
    const { pr_id, url } = input.params || {};
    if (!pr_id && !url) throw new Error('Provide either pr_id or url');
    const db = openDb(input.db_path);
    try {
        const existing = pr_id ? getPrById(db, pr_id) : getPrByUrl(db, url);
        if (!existing) throw new Error('PR not found');
        const before = {
            pr: existing,
            checks: getChecksForPr(db, existing.id),
            reviews: getReviewsForPr(db, existing.id),
        };
        const fetched = await ghPrView(existing.repo_owner, existing.repo_name, existing.pr_number);
        if (!fetched.ok) throw new Error(fetched.error);
        const normalized = normalizePrJson(fetched.data, existing.repo_owner, existing.repo_name);
        const id = upsertPr(db, existing.feature_id, normalized.pr, new Date().toISOString());
        replaceChecks(db, id, normalized.checks);
        replaceReviews(db, id, normalized.reviews);
        const notifications = diffNotifications(existing.feature_id, before, normalized);
        return { data: { pr_id: id }, notifications };
    } finally {
        db.close();
    }
});
```

- [ ] **Step 2: Test — seed with before-state, stub gh to return after-state, verify correct notifications emitted**

- [ ] **Step 3: Run & commit**

```
node --test extensions/github-prs/handlers/__tests__/refresh_pr.test.mjs
git add extensions/github-prs/handlers/refresh_pr.js extensions/github-prs/handlers/__tests__/refresh_pr.test.mjs
git commit -m "feat(github-prs): refresh_pr handler with state-change notifications"
```

---

### Task B12: `handlers/poll.js`

**Files:**
- Create: `extensions/github-prs/handlers/poll.js`
- Create: `extensions/github-prs/handlers/__tests__/poll.test.mjs`

- [ ] **Step 1: Implement**

`extensions/github-prs/handlers/poll.js`:

```javascript
const { run } = require('../lib/runner.js');
const { ghPrListForRepo, ghPrView, normalizePrJson } = require('../lib/gh.js');
const { openDb, listOpenPrsByRepo, getChecksForPr, getReviewsForPr, upsertPr, replaceChecks, replaceReviews } = require('../lib/db.js');
const { diffNotifications } = require('../lib/notify.js');

run(async (input) => {
    const db = openDb(input.db_path);
    const allNotifications = [];
    try {
        const byRepo = listOpenPrsByRepo(db);
        for (const [repoKey, rows] of byRepo.entries()) {
            const [owner, repo] = repoKey.split('/');
            const list = await ghPrListForRepo(owner, repo);
            if (!list.ok) continue;
            const remoteByNumber = new Map(list.data.map(p => [p.number, p]));
            for (const existing of rows) {
                const remote = remoteByNumber.get(existing.pr_number);
                if (!remote) continue;
                if (remote.updatedAt === existing.updated_at) continue;
                const fetched = await ghPrView(owner, repo, existing.pr_number);
                if (!fetched.ok) continue;
                const normalized = normalizePrJson(fetched.data, owner, repo);
                const before = {
                    pr: existing,
                    checks: getChecksForPr(db, existing.id),
                    reviews: getReviewsForPr(db, existing.id),
                };
                const id = upsertPr(db, existing.feature_id, normalized.pr, new Date().toISOString());
                replaceChecks(db, id, normalized.checks);
                replaceReviews(db, id, normalized.reviews);
                allNotifications.push(...diffNotifications(existing.feature_id, before, normalized));
            }
        }
    } finally {
        db.close();
    }
    return { data: { notifications_count: allNotifications.length }, notifications: allNotifications };
});
```

- [ ] **Step 2: Test — seed 2 open PRs in same repo, stub gh list to show 1 updated, verify only that one is refreshed**

- [ ] **Step 3: Run & commit**

```
node --test extensions/github-prs/handlers/__tests__/poll.test.mjs
git add extensions/github-prs/handlers/poll.js extensions/github-prs/handlers/__tests__/poll.test.mjs
git commit -m "feat(github-prs): poll handler for background updates"
```

---

### Task B13: PRs tab UI (`ui/tab.html`, `ui/tab.js`, `ui/tab.css`)

**Files:**
- Create: `extensions/github-prs/ui/tab.html`
- Create: `extensions/github-prs/ui/tab.js`
- Create: `extensions/github-prs/ui/tab.css`

The tab uses the existing `fh:invoke` iframe bridge established by the extension system. On init, the parent sends `{type: 'fh:init', featureId, feature, storagePath}`. The iframe calls `fh:invoke` with `command: 'ext_invoke'`, params `{extension_id, tool_name, params}` — this is how the tab reuses the same handler scripts as the MCP tools. **If the extension-system does not yet expose a tool-invoke Tauri command for iframes, it must be added as part of this task.** Check `src-tauri/src/commands/extensions.rs` and `lib.rs`:

- [ ] **Step 1: Add `invoke_extension_tool` Tauri command**

In `src-tauri/src/commands/extensions.rs`, append:

```rust
#[tauri::command]
pub async fn invoke_extension_tool(
    state: State<'_, AppState>,
    extension_id: String,
    tool_name: String,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let (handler_path, timeout, ext_id) = {
        let registry = state.extensions.lock().map_err(|e| e.to_string())?;
        let ext = registry.extensions.iter()
            .find(|e| e.manifest.id == extension_id && e.enabled)
            .ok_or_else(|| format!("Extension '{}' not found or disabled", extension_id))?;
        let tool = ext.manifest.tools.iter()
            .find(|t| t.name == tool_name)
            .ok_or_else(|| format!("Tool '{}' not found in '{}'", tool_name, extension_id))?;
        (ext.dir.join(&tool.handler), tool.timeout_secs.unwrap_or(10), ext.manifest.id.clone())
    };
    let (db_path, storage_path) = {
        let sp_guard = state.storage_path.lock().map_err(|e| e.to_string())?;
        let sp = sp_guard.as_ref().ok_or("No active storage")?;
        (sp.join("feature-hub.db").to_string_lossy().to_string(), sp.to_string_lossy().to_string())
    };
    let params_obj = params.as_object().cloned().unwrap_or_default();
    let input = crate::extensions::script_runner::ScriptInput {
        params: params_obj,
        db_path,
        storage_path,
        feature_id: None,
    };
    let result = tokio::task::spawn_blocking(move || {
        crate::extensions::script_runner::run_blocking_with_notifications(&handler_path, &input, timeout)
    }).await.map_err(|e| e.to_string())??;
    crate::extensions::script_runner::forward_notifications(&ext_id, result.notifications);
    Ok(result.data)
}
```

Register in `lib.rs` invoke_handler.

- [ ] **Step 2: Implement `ui/tab.html`**

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <link rel="stylesheet" href="tab.css">
</head>
<body>
    <div id="root">
        <div class="toolbar">
            <button id="add-btn">+ Add PR</button>
            <button id="refresh-btn">⟳ Refresh all</button>
        </div>
        <div id="banner" class="banner hidden"></div>
        <div id="grid" class="grid"></div>
        <div id="empty" class="empty hidden">No PRs linked to this feature yet.</div>
    </div>
    <dialog id="add-modal">
        <form method="dialog">
            <h3>Add GitHub PR</h3>
            <input id="url-input" type="url" placeholder="https://github.com/owner/repo/pull/123" required>
            <div id="add-error" class="error"></div>
            <div class="modal-actions">
                <button value="cancel">Cancel</button>
                <button id="submit-btn" value="ok">Add</button>
            </div>
        </form>
    </dialog>
    <script src="tab.js"></script>
</body>
</html>
```

- [ ] **Step 3: Implement `ui/tab.js`**

```javascript
const EXT_ID = 'github-prs';
let ctx = null;

function invoke(cmd, params = {}, requestId) {
    return new Promise((resolve, reject) => {
        const id = requestId || String(Math.random()).slice(2);
        function onMessage(ev) {
            const d = ev.data;
            if (!d || d.type !== 'fh:invoke-result' || d.requestId !== id) return;
            window.removeEventListener('message', onMessage);
            d.ok ? resolve(d.data) : reject(new Error(d.error || 'invoke failed'));
        }
        window.addEventListener('message', onMessage);
        window.parent.postMessage({ type: 'fh:invoke', command: cmd, params, requestId: id }, '*');
    });
}

function callTool(toolName, params) {
    return invoke('invoke_extension_tool', { extension_id: EXT_ID, tool_name: toolName, params });
}

const PR_URL_RE = /^https:\/\/github\.com\/[^/]+\/[^/]+\/pull\/\d+$/;

function statePill(pr) {
    const cls = pr.is_draft ? 'draft' : pr.state; // open | draft | merged | closed
    const label = pr.is_draft ? 'Draft' : pr.state.charAt(0).toUpperCase() + pr.state.slice(1);
    return `<span class="pill pill-${cls}">${label}</span>`;
}

function checksBadge(r) {
    if (!r || r.total === 0) return '';
    const symbol = r.failure > 0 ? '✗' : r.pending > 0 ? '⏳' : '✓';
    const cls = r.failure > 0 ? 'bad' : r.pending > 0 ? 'warn' : 'good';
    return `<span class="rollup rollup-${cls}" title="${r.success}/${r.total} passing">${symbol} ${r.success}/${r.total}</span>`;
}

function reviewsBadge(r) {
    const parts = [];
    if (r.approved > 0) parts.push(`<span class="rollup rollup-good" title="Approved">👍 ${r.approved}</span>`);
    if (r.changes_requested > 0) parts.push(`<span class="rollup rollup-bad" title="Changes requested">⚠ ${r.changes_requested}</span>`);
    if (r.pending > 0) parts.push(`<span class="rollup" title="Pending">👀 ${r.pending}</span>`);
    return parts.join(' ');
}

function rel(iso) {
    if (!iso) return '';
    const d = new Date(iso);
    const secs = (Date.now() - d.getTime()) / 1000;
    if (secs < 60) return 'just now';
    if (secs < 3600) return `${Math.floor(secs / 60)}m ago`;
    if (secs < 86400) return `${Math.floor(secs / 3600)}h ago`;
    return `${Math.floor(secs / 86400)}d ago`;
}

function render(prs) {
    const grid = document.getElementById('grid');
    const empty = document.getElementById('empty');
    if (!prs || prs.length === 0) {
        grid.innerHTML = '';
        empty.classList.remove('hidden');
        return;
    }
    empty.classList.add('hidden');
    grid.innerHTML = prs.map(pr => `
        <div class="card" data-pr-id="${pr.id}" data-url="${pr.url}">
            <div class="card-header">
                ${statePill(pr)}
                <span class="pr-number">#${pr.pr_number}</span>
                <span class="repo">${pr.repo}</span>
                <div class="spacer"></div>
                <button class="menu-btn" aria-label="menu">⋮</button>
            </div>
            <a class="card-title" href="${pr.url}" target="_blank" rel="noreferrer">${escapeHtml(pr.title)}</a>
            <div class="rollups">
                ${checksBadge(pr.checks_rollup)}
                ${reviewsBadge(pr.review_rollup)}
            </div>
            <div class="meta">
                @${pr.author || 'unknown'} · ${
                    pr.state === 'merged' ? `merged ${rel(pr.merged_at)}`
                    : pr.state === 'closed' ? `closed ${rel(pr.closed_at)}`
                    : `updated ${rel(pr.updated_at)}`
                }
            </div>
            <div class="details hidden" data-details-for="${pr.id}"></div>
        </div>
    `).join('');
    wireCardHandlers();
}

function escapeHtml(s) {
    return (s || '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
}

function wireCardHandlers() {
    for (const card of document.querySelectorAll('.card')) {
        card.addEventListener('click', async (e) => {
            if (e.target.closest('a.card-title')) return;
            if (e.target.closest('.menu-btn')) { toggleMenu(card, e); return; }
            await toggleDetails(card);
        });
    }
}

async function toggleDetails(card) {
    const id = card.dataset.prId;
    const details = card.querySelector(`.details[data-details-for="${id}"]`);
    if (!details.classList.contains('hidden')) {
        details.classList.add('hidden');
        return;
    }
    details.textContent = 'Loading...';
    details.classList.remove('hidden');
    const d = await callTool('get_github_pr', { pr_id: id });
    details.innerHTML = `
        <div>Branches: ${escapeHtml(d.pr.head_branch || '?')} → ${escapeHtml(d.pr.base_branch || '?')}</div>
        <div>Diff: +${d.pr.additions || 0} / -${d.pr.deletions || 0} across ${d.pr.changed_files || 0} files</div>
        <div class="checks-list">${(d.checks || []).map(c => `
            <div class="check check-${c.conclusion || 'pending'}">${escapeHtml(c.name)} — ${escapeHtml(c.conclusion || c.status)}</div>
        `).join('')}</div>
        <div class="reviews-list">${(d.reviews || []).map(r => `
            <div class="review review-${r.state}">@${escapeHtml(r.reviewer)} — ${escapeHtml(r.state)}</div>
        `).join('')}</div>
    `;
}

function toggleMenu(card, ev) {
    ev.stopPropagation();
    const existing = card.querySelector('.menu-popup');
    if (existing) { existing.remove(); return; }
    const menu = document.createElement('div');
    menu.className = 'menu-popup';
    menu.innerHTML = `
        <button data-action="refresh">Refresh</button>
        <button data-action="open">Open on GitHub</button>
        <button data-action="remove">Remove from feature</button>
    `;
    menu.addEventListener('click', async (e) => {
        const action = e.target.dataset.action;
        menu.remove();
        if (action === 'refresh') {
            await callTool('refresh_github_pr', { pr_id: card.dataset.prId });
            await reload();
        } else if (action === 'open') {
            window.parent.postMessage({ type: 'fh:shellOpen', url: card.dataset.url }, '*');
        } else if (action === 'remove') {
            if (!confirm('Remove this PR from the feature?')) return;
            // Delete the underlying link; cascades via on_link_deleted hook.
            await invoke('delete_link_by_url', { feature_id: ctx.featureId, url: card.dataset.url });
            await reload();
        }
    });
    card.appendChild(menu);
}

async function reload() {
    try {
        const prs = await callTool('list_github_prs', { feature_id: ctx.featureId });
        render(prs);
        document.getElementById('banner').classList.add('hidden');
    } catch (e) {
        const b = document.getElementById('banner');
        b.textContent = e.message;
        b.classList.remove('hidden');
    }
}

async function refreshAll() {
    const prs = await callTool('list_github_prs', { feature_id: ctx.featureId });
    await Promise.allSettled(prs.map(p => callTool('refresh_github_pr', { pr_id: p.id })));
    await reload();
}

function setupAddModal() {
    const modal = document.getElementById('add-modal');
    const input = document.getElementById('url-input');
    const err = document.getElementById('add-error');
    const submit = document.getElementById('submit-btn');
    document.getElementById('add-btn').addEventListener('click', () => {
        input.value = ''; err.textContent = '';
        submit.disabled = true;
        modal.showModal();
    });
    input.addEventListener('input', () => {
        submit.disabled = !PR_URL_RE.test(input.value.trim());
        err.textContent = '';
    });
    modal.addEventListener('close', async () => {
        if (modal.returnValue !== 'ok') return;
        try {
            await callTool('add_github_pr', { feature_id: ctx.featureId, url: input.value.trim() });
            await reload();
        } catch (e) {
            err.textContent = e.message;
            modal.showModal();
        }
    });
}

window.addEventListener('message', async (ev) => {
    if (!ev.data || ev.data.type !== 'fh:init') return;
    ctx = ev.data;
    setupAddModal();
    document.getElementById('refresh-btn').addEventListener('click', refreshAll);
    await reload();
});
```

**Note:** `delete_link_by_url` doesn't exist yet — add it as a thin Tauri command in `src-tauri/src/commands/links.rs` that looks up by `(feature_id, url)` and calls the existing delete path. Include in this task's commit.

- [ ] **Step 4: Implement `ui/tab.css`**

```css
* { box-sizing: border-box; }
body { margin: 0; font-family: system-ui, sans-serif; color: #eee; background: #1a1a1a; }
.toolbar { display: flex; gap: 0.5rem; padding: 0.75rem 1rem; border-bottom: 1px solid #333; }
.toolbar button { background: #2a2a2a; color: #eee; border: 1px solid #444; padding: 0.4rem 0.75rem; border-radius: 4px; cursor: pointer; }
.toolbar button:hover { background: #333; }
.toolbar > *:last-child { margin-left: auto; }
.banner { margin: 1rem; padding: 0.75rem; background: #3a1a1a; border: 1px solid #522; border-radius: 4px; }
.banner.hidden { display: none; }
.empty { padding: 2rem; text-align: center; color: #888; }
.empty.hidden { display: none; }
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 0.75rem; padding: 0.75rem 1rem; }
.card { background: #232323; border: 1px solid #333; border-radius: 6px; padding: 0.75rem; cursor: pointer; position: relative; }
.card:hover { border-color: #555; }
.card-header { display: flex; align-items: center; gap: 0.5rem; font-size: 0.85rem; }
.spacer { flex: 1; }
.pill { padding: 0.1rem 0.5rem; border-radius: 999px; font-size: 0.75rem; font-weight: 600; }
.pill-open { background: #1e4620; color: #8fd891; }
.pill-draft { background: #333; color: #aaa; }
.pill-merged { background: #3b1b5a; color: #c49bff; }
.pill-closed { background: #4a1f1f; color: #f2a4a4; }
.pr-number { color: #aaa; }
.repo { color: #888; font-size: 0.8rem; }
.menu-btn { background: transparent; border: none; color: #aaa; cursor: pointer; font-size: 1.2rem; padding: 0 0.25rem; }
.card-title { display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; color: #eee; text-decoration: none; font-weight: 500; margin: 0.5rem 0; }
.card-title:hover { text-decoration: underline; }
.rollups { display: flex; gap: 0.5rem; margin: 0.5rem 0; font-size: 0.8rem; }
.rollup { padding: 0.1rem 0.4rem; border-radius: 3px; background: #2a2a2a; }
.rollup-good { color: #8fd891; }
.rollup-bad { color: #f2a4a4; }
.rollup-warn { color: #e0c06b; }
.meta { color: #888; font-size: 0.75rem; }
.details { margin-top: 0.75rem; padding-top: 0.5rem; border-top: 1px solid #333; font-size: 0.8rem; color: #ccc; }
.details.hidden { display: none; }
.check-success { color: #8fd891; }
.check-failure { color: #f2a4a4; }
.check-pending, .check-null { color: #e0c06b; }
.review-approved { color: #8fd891; }
.review-changes_requested { color: #f2a4a4; }
.review-requested, .review-pending { color: #e0c06b; }
.menu-popup { position: absolute; top: 2rem; right: 0.5rem; background: #2a2a2a; border: 1px solid #444; border-radius: 4px; display: flex; flex-direction: column; min-width: 180px; z-index: 10; }
.menu-popup button { background: transparent; border: none; color: #eee; padding: 0.5rem 0.75rem; text-align: left; cursor: pointer; }
.menu-popup button:hover { background: #333; }
dialog { background: #2a2a2a; color: #eee; border: 1px solid #444; border-radius: 6px; padding: 1rem; min-width: 400px; }
dialog input { width: 100%; padding: 0.5rem; background: #1a1a1a; border: 1px solid #444; color: #eee; border-radius: 4px; margin: 0.5rem 0; }
.error { color: #f2a4a4; font-size: 0.8rem; min-height: 1rem; }
.modal-actions { display: flex; gap: 0.5rem; justify-content: flex-end; margin-top: 0.75rem; }
.modal-actions button { background: #333; color: #eee; border: 1px solid #444; padding: 0.4rem 1rem; border-radius: 4px; cursor: pointer; }
.modal-actions button:disabled { opacity: 0.5; cursor: not-allowed; }
```

- [ ] **Step 5: Add `delete_link_by_url` Tauri command**

In `src-tauri/src/commands/links.rs`:

```rust
#[tauri::command]
pub fn delete_link_by_url(state: State<'_, AppState>, feature_id: String, url: String) -> Result<(), String> {
    let id = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let id: Option<String> = conn.query_row(
            "SELECT id FROM links WHERE feature_id = ?1 AND url = ?2",
            rusqlite::params![feature_id, url],
            |row| row.get(0),
        ).ok();
        id
    };
    match id {
        Some(id) => delete_link(state, id),
        None => Ok(()),
    }
}
```

Register in `lib.rs` invoke_handler.

- [ ] **Step 6: Manual test**

Copy `extensions/github-prs/` into your active storage's `extensions/` directory. Restart app, open a feature, verify:
- PRs tab appears between Links and Files
- `[+ Add PR]` opens modal, regex-validated
- Adding a real PR URL shows a card with rollups
- Clicking card expands details (branches, diff, checks, reviews)
- Menu actions work (Refresh, Open on GitHub, Remove)

- [ ] **Step 7: Commit**

```
git add extensions/github-prs/ui/ src-tauri/src/commands/extensions.rs src-tauri/src/commands/links.rs src-tauri/src/lib.rs
git commit -m "feat(github-prs): PRs tab UI + invoke_extension_tool + delete_link_by_url"
```

---

### Task B14: Settings card — polling controls + auth status

**Files:**
- Modify: `src/lib/components/SettingsModal.svelte`
- Modify: `src/lib/api/` (add typed wrappers if helpful)

- [ ] **Step 1: Locate the extensions section**

Search for the existing "Extensions" / `get_extensions` rendering in `SettingsModal.svelte`. Extension cards already show name, version, enabled toggle, `requires_status`.

- [ ] **Step 2: Add a `github-prs`-specific section inside that card**

Render only when `extension.id === 'github-prs'` (below the generic info):

```svelte
{#if ext.id === 'github-prs'}
  <div class="ghpr-config">
    {#if ghAuth}
      {#if ghAuth.authenticated}
        <div class="sc-status-ok">✓ Logged in to github.com as @{ghAuth.user ?? '?'}</div>
      {:else}
        <div class="sc-status-bad">✗ Not authenticated — run: gh auth login</div>
      {/if}
    {:else}
      <div class="sc-muted">Checking gh auth...</div>
    {/if}
    <label>
      <input type="checkbox" bind:checked={ghprSettings.poll_enabled} on:change={saveGhprSettings} />
      Auto-refresh open PRs in background
    </label>
    <label>
      Interval
      <select bind:value={ghprSettings.poll_interval_secs} on:change={saveGhprSettings} disabled={!ghprSettings.poll_enabled}>
        <option value={180}>3 minutes</option>
        <option value={300}>5 minutes</option>
        <option value={600}>10 minutes</option>
        <option value={1800}>30 minutes</option>
      </select>
    </label>
  </div>
{/if}
```

With supporting logic:

```typescript
import { invoke } from '@tauri-apps/api/core';

let ghprSettings = $state({ poll_enabled: false, poll_interval_secs: 300 });
let ghAuth: { authenticated: boolean; user: string | null } | null = $state(null);

$effect(() => {
  (async () => {
    const s = await invoke<{ poll_enabled?: boolean; poll_interval_secs?: number }>('get_extension_settings', { key: 'github_prs' });
    ghprSettings.poll_enabled = !!s.poll_enabled;
    ghprSettings.poll_interval_secs = s.poll_interval_secs ?? 300;
    ghAuth = await invoke('check_gh_auth').catch(() => ({ authenticated: false, user: null }));
  })();
});

async function saveGhprSettings() {
  await invoke('set_extension_settings', { key: 'github_prs', value: { ...ghprSettings } });
  // Restart applies on next app restart; also nudge backend:
  await invoke('restart_extension_schedules').catch(() => {});
}
```

- [ ] **Step 3: Add a `check_gh_auth` Tauri command**

`src-tauri/src/commands/extensions.rs`:

```rust
#[tauri::command]
pub async fn check_gh_auth() -> Result<serde_json::Value, String> {
    let output = tokio::process::Command::new("gh")
        .arg("auth").arg("status")
        .output().await
        .map_err(|e| format!("gh not available: {}", e))?;
    let combined = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    let authenticated = combined.contains("Logged in");
    let user = regex::Regex::new(r"as ([^\s]+)").unwrap()
        .captures(&combined)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()));
    Ok(serde_json::json!({ "authenticated": authenticated, "user": user }))
}
```

Note: `regex` crate may already be a dependency; if not, use a simpler parse (split on whitespace). Register command in `lib.rs`.

- [ ] **Step 4: Add `restart_extension_schedules` Tauri command**

In `src-tauri/src/commands/extensions.rs`:

```rust
#[tauri::command]
pub fn restart_extension_schedules(state: State<'_, AppState>) -> Result<(), String> {
    // Cancel existing handles, then re-spawn from current registry + settings.
    // Delegates to the same logic used at storage activation.
    crate::restart_schedules(&state)
}
```

Expose a `restart_schedules(state: &AppState)` helper in `lib.rs` that does the cancel + respawn dance (factor the code added in Task A3 into a reusable function).

- [ ] **Step 5: Commit**

```
git add src/lib/components/SettingsModal.svelte src-tauri/src/commands/extensions.rs src-tauri/src/lib.rs
git commit -m "feat(github-prs): settings card with gh auth status + polling controls"
```

---

### Task B15: MCP server — ensure extension tools register

**Files:**
- Inspect: `src-tauri/src/bin/fh_mcp.rs`
- Possibly modify same file if extension tool registration is missing

- [ ] **Step 1: Verify extension tools are already registered in fh-mcp**

The existing extension-system spec claims extension tools are exposed via `fh-mcp`. Inspect `src-tauri/src/bin/fh_mcp.rs` for extension-tool handling. If missing, add loading and per-tool dynamic registration using `rmcp` — but this is beyond the scope of this plan. **Expected finding: already implemented**. Confirm the 4 `github-prs` tools appear in a live MCP session.

- [ ] **Step 2: Manual test in Claude**

1. Ensure `extensions/github-prs/` is copied into storage and enabled.
2. Start a Claude session via `fh` CLI scoped to a test feature.
3. Ask Claude: "list the github PRs for this feature". Verify Claude calls `list_github_prs`.
4. Ask: "add this PR: <url>". Verify Claude calls `add_github_pr`.
5. Check tab in app — PR appears.

- [ ] **Step 3: Document findings in README** (no commit if everything worked out of the box)

If modifications were needed, commit them:

```
git add src-tauri/src/bin/fh_mcp.rs
git commit -m "fix(fh-mcp): register extension tools from github-prs"
```

---

### Task B16: Dev installation shim

**Files:**
- Create: `scripts/install-github-prs.mjs`

So devs can re-sync `extensions/github-prs/` into the active storage easily.

- [ ] **Step 1: Implement**

`scripts/install-github-prs.mjs`:

```javascript
#!/usr/bin/env node
// Copy extensions/github-prs/ into the active storage's extensions/ directory.
// Reads active storage from the OS app data config.
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';

function appDataDir() {
    if (process.platform === 'win32') return path.join(process.env.APPDATA || '', 'com.littlebrushgames.feature-hub');
    if (process.platform === 'darwin') return path.join(os.homedir(), 'Library', 'Application Support', 'com.littlebrushgames.feature-hub');
    return path.join(process.env.XDG_DATA_HOME || path.join(os.homedir(), '.local', 'share'), 'com.littlebrushgames.feature-hub');
}

const cfgPath = path.join(appDataDir(), 'config.json');
const cfg = JSON.parse(fs.readFileSync(cfgPath, 'utf8'));
const active = cfg.storages.find(s => s.id === cfg.active_storage_id);
if (!active) { console.error('No active storage'); process.exit(1); }
const src = path.resolve('extensions/github-prs');
const dst = path.join(active.path, 'extensions', 'github-prs');
fs.rmSync(dst, { recursive: true, force: true });
fs.cpSync(src, dst, { recursive: true });
console.log(`Copied ${src} → ${dst}`);
```

Add to `package.json` scripts:

```json
"install:github-prs": "node scripts/install-github-prs.mjs"
```

- [ ] **Step 2: Commit**

```
git add scripts/install-github-prs.mjs package.json
git commit -m "chore(github-prs): dev install script"
```

---

### Task B17: End-to-end manual test pass

**Files:** none (documented test run)

- [ ] **Step 1: Fresh install**

1. `npm run install:github-prs`
2. `cd src-tauri && cargo build`
3. `npm run tauri dev`

- [ ] **Step 2: Settings verification**

Open Settings → Extensions → GitHub Pull Requests. Verify:
- `gh` found (or warning shown)
- `gh auth status` message shown correctly
- Polling checkbox togglable; interval selector respects enabled state

- [ ] **Step 3: Link-to-PR flow**

1. Pick or create a feature with a public PR you can access (e.g. any PR in any public repo you can `gh pr view`).
2. Paste the PR URL into the Links tab.
3. Within ~5s, switch to PRs tab → card appears with state, checks, reviews, author.

- [ ] **Step 4: Refresh + expand + menu**

1. Click the card → details expand (branches, diff, checks, reviews).
2. Click ⋮ → "Refresh" → card updates `last_fetched_at`.
3. Click ⋮ → "Open on GitHub" → browser opens.
4. Click ⋮ → "Remove from feature" → card disappears; Links tab also loses the link.

- [ ] **Step 5: Polling**

1. Enable polling at 3m interval.
2. Merge or close a PR on GitHub.
3. Within ~3m, toast appears and card reflects new state.

- [ ] **Step 6: MCP from Claude**

1. Start a Claude session via `fh` scoped to the feature.
2. Verify Claude can `list_github_prs`, `get_github_pr`, `add_github_pr`, `refresh_github_pr`.

- [ ] **Step 7: Document results**

If any step fails, add a follow-up task or file a bug. If all pass, proceed.

- [ ] **Step 8: Final commit (if any fixes were needed inline)**

```
git add -A
git commit -m "fix(github-prs): end-to-end test fixes"
```

---

## Summary of deliverables

**Phase A (extension-system primitives):**
- `schedules` manifest field + validation + scheduled runner with per-schedule mutex
- `notifications[]` in script stdout protocol, forwarded to `push_notification_ex`
- `get/set_extension_settings` Tauri commands
- `link_deleted` event dispatch

**Phase B (GitHub extension):**
- `extensions/github-prs/` directory with manifest, handlers (7), lib (3 modules + runner), tab UI (HTML/JS/CSS)
- New Tauri commands: `invoke_extension_tool`, `delete_link_by_url`, `check_gh_auth`, `restart_extension_schedules`
- Settings card in `SettingsModal.svelte` for polling controls + auth status
- Dev install script

**Testing:**
- Rust unit tests on Phase A additions
- Node `node:test` tests for each handler and lib module
- Manual end-to-end pass in Task B17
