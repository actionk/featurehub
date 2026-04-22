# Tier 2 Documentation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close documentation gaps in CLAUDE.md and create gotchas.md so that future AI sessions start with accurate, complete context about this codebase's non-obvious patterns.

**Architecture:** Two independent file edits — create `gotchas.md` at the project root, and make six targeted additions to `CLAUDE.md`. No code changes. Each task produces one commit.

**Tech Stack:** Markdown

---

## File Map

| File | Change |
|------|--------|
| `gotchas.md` | Create — lessons learned and known sharp edges |
| `CLAUDE.md` | Modify — 6 targeted additions (tech stack, settings split, type distinctions, notification flow, tab contract, storage model) |

---

## Task 1: Create `gotchas.md`

**Files:**
- Create: `gotchas.md`

- [ ] **Step 1: Create the file with the following exact content**

Create `D:/LittleBrushGames/FeatureHub/gotchas.md` with this content:

```markdown
# FeatureHub Gotchas

Lessons learned. Add entries here when something bites you.

---

## Rust / Backend

### db/ files must be declared in `db/mod.rs`
Creating a file at `src-tauri/src/db/foo.rs` does **not** make it a Rust module. It must also have `pub mod foo;` in `src-tauri/src/db/mod.rs`. Files without a declaration are silently ignored by the compiler — no warning, no error. Always verify both the file exists AND the `pub mod` line exists when checking or adding db modules.

### Migrations use column-existence probes, not version numbers
Migrations in `db::initialize()` guard `ALTER TABLE` statements with:
```rust
let has_col = conn.prepare("SELECT col FROM table LIMIT 0").is_ok();
if !has_col { conn.execute_batch("ALTER TABLE table ADD COLUMN col TYPE;")?; }
```
This is intentional — SQLite doesn't support `ALTER TABLE ... IF NOT EXISTS`. Never use a version counter or reorder migrations. Always follow this exact pattern for new columns.

### `migrate_to_relative_paths` runs at startup on every launch
The function `db::migrate_to_relative_paths()` is called every time a storage is opened. It must be idempotent (it is — it skips already-relative paths). If you add new path columns to any table, add a migration block here too, not just in `initialize()`.

### Two settings files, not one — know which you're editing
See the "Settings system" section in CLAUDE.md. Global settings (`fh_cli_path`, fonts, IDEs) live in the OS app-data dir. Portable settings (MCP servers, skills, extensions, default repos) live in each storage's own folder. Editing the wrong one has no immediate error — it silently writes to a file the other side doesn't read.

---

## TypeScript / Frontend

### TypeScript interfaces are hand-maintained — they can drift from Rust
There is no code generation. `src/lib/api/types.ts` must mirror the Rust serde output manually. After adding or removing a field in a Rust struct, update the corresponding TypeScript interface immediately. The compiler won't catch the drift — only runtime will.

### `sort_order` exists on Feature, Task, FeatureGroup — NOT on Link
The `links` table has no `sort_order` column. Links sort by `created_at` DESC. Do not add `sort_order` to `Link` in TypeScript or the DB without a full migration + Rust struct update + DB column.

### Worktrees are created from HEAD, not the working tree
`git worktree add` checks out from the last **commit**, not from uncommitted changes. If the main working tree has significant unstaged/uncommitted changes, the worktree won't have them. Check `git status` before creating a worktree. If there are uncommitted changes that the task depends on, work in the main directory instead.

---

## Architecture

### Notification IPC is file-based, not event-based
The MCP server cannot call back into Tauri directly. Instead it appends JSON lines to `notifications.jsonl` in the config dir, and the Tauri app polls this file every 2 seconds. There is no push mechanism — 2s is the minimum latency. Do not try to use Tauri events or sockets for MCP→app communication.

### `CombinedSettings` is a merge of two structs — save_settings splits them
`get_settings` returns a `CombinedSettings` that merges `AppSettings` (global) and `StorageSettings` (per-storage). `save_settings` re-splits them and writes to two separate files. Don't add a field to `CombinedSettings` without deciding which underlying struct it belongs to, and updating both the Rust `save_settings` command and the TypeScript `AppSettings` interface.
```

- [ ] **Step 2: Verify the file was created**

```bash
ls -la D:/LittleBrushGames/FeatureHub/gotchas.md
```

Expected: file exists, non-zero size.

- [ ] **Step 3: Commit**

```bash
cd D:/LittleBrushGames/FeatureHub
git add gotchas.md
git commit -m "docs: add gotchas.md with known sharp edges and lessons learned"
```

---

## Task 2: Update CLAUDE.md with six targeted additions

**Files:**
- Modify: `CLAUDE.md`

This task makes six independent additions to `CLAUDE.md`. Apply them in order, verifying each edit lands in the right place before moving on.

---

### Edit A — Tech Stack: add Mermaid

- [ ] **Step 1: Add Mermaid to the Tech Stack section**

Find this line in `CLAUDE.md`:
```
- **Markdown**: `marked` library for notes/context preview
```

Replace with:
```
- **Markdown**: `marked` library for notes/context preview
- **Diagrams**: `mermaid` library for diagram rendering in notes/context (toggled via `mermaid_diagrams` setting)
```

---

### Edit B — Storage model: mention per-storage settings

- [ ] **Step 2: Update the Storage model paragraph**

Find this paragraph:
```
The app supports multiple "storages" (directories on disk), each containing its own `feature-hub.db`, `files/` directory, and `workspaces/` directory. Storage config lives in the OS app data dir (`com.littlebrushgames.feature-hub/config.json`). The user picks an active storage; the Tauri app swaps the DB connection at runtime via `AppState`. Settings live in a separate `settings.json` in the same config dir.
```

Replace with:
```
The app supports multiple "storages" (directories on disk), each containing its own `feature-hub.db`, `files/` directory, `workspaces/` directory, and `settings.json`. Storage config lives in the OS app data dir (`com.littlebrushgames.feature-hub/config.json`). The user picks an active storage; the Tauri app swaps the DB connection at runtime via `AppState`. Machine-specific settings (fonts, CLI path, preferred IDEs) live in a separate `settings.json` in the OS app data dir; portable settings (MCP servers, skills, extensions, default repos) live in each storage's own `settings.json`.
```

---

### Edit C — Settings system: rewrite to explain the split

- [ ] **Step 3: Replace the Settings system section**

Find this entire section:
```
### Settings system

`config.rs` manages `settings.json` with:
- `fh_cli_path` — custom path to the `fh` CLI binary
- `mcp_servers` — list of MCP servers (name, command, args, env, url, default_enabled) injected into Claude sessions
- `default_repositories` — predefined repository URLs (with descriptions) available when initializing features
- `jira_integration` — enables Jira-related MCP instructions
- `mermaid_diagrams` — enables Mermaid diagram rendering in the UI
- `openfga_highlighting` — enables OpenFGA syntax highlighting
- `show_tab_emojis` — show emoji icons on tabs
- `ui_font` / `mono_font` / `ui_font_size` — font customization
- `preferred_ides` — list of preferred IDEs for "open in editor" actions
- `skills` — global skill definitions
- `extensions` — extension configuration
```

Replace with:
```
### Settings system

`config.rs` manages two separate settings files:

**Global / machine settings** — `<OS app data>/com.littlebrushgames.feature-hub/settings.json` (`AppSettings` struct):
- `fh_cli_path` — custom path to the `fh` CLI binary
- `jira_integration` — legacy flag (prefer extension system)
- `mermaid_diagrams` — enables Mermaid diagram rendering in the UI
- `openfga_highlighting` — enables OpenFGA syntax highlighting
- `show_tab_emojis` — show emoji icons on tabs
- `ui_font` / `mono_font` / `ui_font_size` — font customization
- `preferred_ides` — list of preferred IDEs for "open in editor" actions

**Per-storage settings** — `<storage-path>/settings.json` (`StorageSettings` struct, travels with the storage):
- `mcp_servers` — list of MCP servers injected into Claude sessions
- `default_repositories` — predefined repository URLs for feature init
- `skills` — skill definitions
- `extensions` — extension configuration (Jira, Slite, etc.)

The Tauri command `get_settings` merges both into a flat `CombinedSettings` struct. `save_settings` accepts optional fields for both and writes each to the correct file. When editing settings code, always know which struct a field belongs to.
```

---

### Edit D — Add "Key type distinctions" section after Settings system

- [ ] **Step 4: Insert a new section after the Settings system section**

Find the line:
```
### MCP server features
```

Insert the following **before** that line (i.e., between Settings system and MCP server features):

```
### Key type distinctions

Three overlapping types represent features — know which to use:

- **`FeatureSummary`** (`db/features.rs`) — used in the sidebar list. Has task counts (`task_count_total`, `task_count_done`) and tags, but no `directories` or `links`. Returned by `get_features`.
- **`Feature`** (`db/features.rs`) — used in the detail view. Has full `directories`, `links`, and `tags`. Returned by `get_feature` and mutation commands.
- **`FeatureData`** (`commands/mod.rs`) — IPC bundle returned by `get_feature_data`. Wraps a `Feature` together with `all_tags`, `tasks`, `plans`, and `note` in a single round-trip. Used by `FeatureDetail.svelte` on initial load to avoid 5 separate IPC calls.

On the TypeScript side, `src/lib/api/types.ts` has a single `Feature` interface with optional fields (`task_count_total?`, `tags?`, etc.) that covers both shapes — be careful not to assume a field exists when you only have a summary.

```

---

### Edit E — Notification system: add explicit data flow

- [ ] **Step 5: Update the Notification system section**

Find:
```
### Notification system

MCP server → writes JSONL to `notifications.jsonl` in config dir → Tauri app polls every 2s → shows toasts in UI. This is how `fh-mcp` tool calls (create task, update feature, etc.) trigger live UI updates.
```

Replace with:
```
### Notification system

MCP tool calls trigger live UI updates via a file-based IPC chain:

1. `fh-mcp` calls `config::push_notification()` (or `push_notification_ex()` for plan notifications) which **appends** a JSON line to `<config-dir>/notifications.jsonl`
2. The Tauri app polls `poll_notifications` every **2 seconds** — reads and truncates the file
3. `App.svelte` processes each notification: shows a toast, refreshes the sidebar, and if the affected feature is open in a tab, refreshes its data and optionally navigates to the AI tab (for plan submissions)

There is no push mechanism — 2s is the minimum notification latency. The `plan_id` field on a notification causes the UI to auto-navigate to the pending plan in the AI tab.
```

---

### Edit F — Tab modules: add TabContext guarantees and preload flag

- [ ] **Step 6: Update the Tab modules paragraph**

Find:
```
Tab modules receive a `TabContext` interface with featureId, feature, sessions, tasks, plans, note, allTags, activeSessionCount, onRefresh, onSessionsChanged. New tabs can be added by creating a module folder and calling `registerTab()` — no changes to FeatureDetail needed.
```

Replace with:
```
Tab modules receive a `TabContext` interface — all fields are always populated when a tab renders:
- `featureId`, `feature` — the current feature (full `Feature` type with directories/links/tags)
- `sessions` — loaded separately from `get_sessions` (involves disk I/O for title scanning)
- `tasks`, `plans`, `note`, `allTags` — from the initial `get_feature_data` bundle
- `activeSessionCount` — from the 10s session-activity polling store
- `pendingPlanId` / `onPendingPlanHandled` — set when a plan notification arrives; use to auto-focus a plan
- `onRefresh()` — call to reload all feature data from the backend
- `onSessionsChanged()` — call after mutating sessions to trigger activity refresh

The `preload: true` flag on a `TabModule` causes the tab's component to mount immediately on feature load (not lazily on first click) — use for tabs that need to initialize state in the background. The `panelStyle` field sets inline CSS on the tab's content wrapper.

New tabs can be added by creating a module folder and calling `registerTab()` in an `index.ts` — no changes to `FeatureDetail.svelte` needed.
```

---

### Commit

- [ ] **Step 7: Verify all six edits landed correctly**

```bash
cd D:/LittleBrushGames/FeatureHub
grep -n "mermaid library" CLAUDE.md
grep -n "per-storage settings" CLAUDE.md
grep -n "Per-storage settings" CLAUDE.md
grep -n "Key type distinctions" CLAUDE.md
grep -n "push_notification" CLAUDE.md
grep -n "preload" CLAUDE.md
```

Expected: each grep returns at least one line.

- [ ] **Step 8: Commit**

```bash
cd D:/LittleBrushGames/FeatureHub
git add CLAUDE.md
git commit -m "docs: fill CLAUDE.md gaps — settings split, type distinctions, notification flow, tab contract"
```

---

## Self-Review

**Spec coverage:**
- ✅ gotchas.md created with: worktrees/HEAD gotcha, db module declaration gotcha, TypeScript drift gotcha, sort_order gotcha, settings split gotcha, notification IPC gotcha, CombinedSettings gotcha
- ✅ Tech stack: mermaid added
- ✅ Storage model: per-storage settings mentioned
- ✅ Settings system: two-file split documented with field lists for each
- ✅ Key type distinctions: FeatureSummary vs Feature vs FeatureData explained
- ✅ Notification system: explicit step-by-step data flow
- ✅ Tab modules: TabContext fields documented, preload explained

**Placeholder scan:** No TBDs, no "fill in later", all content is concrete.

**Type consistency:** No code types introduced — documentation only.
