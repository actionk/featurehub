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
