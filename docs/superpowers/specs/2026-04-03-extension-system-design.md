# Extension System Design

**Date**: 2026-04-03
**Status**: Approved
**Scope**: General-purpose extension API for Feature Hub. GitHub PRs is the first planned consumer but is out of scope here.

## Overview

A directory-based extension system that allows third-party code to extend Feature Hub with custom MCP tools, event hooks, database tables, and UI tabs — without recompiling the binary. Designed to support a future plugin marketplace.

## Extension Package Format

Each extension is a directory inside `<storage>/extensions/`:

```
extensions/
  github-prs/
    extension.json
    handlers/
      add_pr.js
      on_link_created.js
    ui/
      tab.html
      tab.js
```

### Manifest (`extension.json`)

```json
{
  "id": "github-prs",
  "name": "GitHub Pull Requests",
  "version": "1.0.0",
  "description": "Track PRs linked to features using gh CLI",
  "author": "LittleBrushGames",
  "requires": ["gh"],

  "tables": [
    {
      "name": "ext_github_prs",
      "columns": [
        { "name": "id", "type": "TEXT PRIMARY KEY" },
        { "name": "feature_id", "type": "TEXT NOT NULL", "fk": "features(id) ON DELETE CASCADE" },
        { "name": "directory_id", "type": "TEXT", "fk": "directories(id) ON DELETE SET NULL" },
        { "name": "url", "type": "TEXT NOT NULL" },
        { "name": "title", "type": "TEXT NOT NULL" },
        { "name": "pr_number", "type": "INTEGER NOT NULL" },
        { "name": "repo_name", "type": "TEXT NOT NULL" },
        { "name": "status", "type": "TEXT NOT NULL DEFAULT 'open'" },
        { "name": "author", "type": "TEXT" },
        { "name": "created_at", "type": "TEXT NOT NULL" },
        { "name": "updated_at", "type": "TEXT NOT NULL" }
      ],
      "indexes": ["feature_id"]
    }
  ],

  "tools": [
    {
      "name": "add_github_pr",
      "description": "Add a GitHub PR to a feature",
      "handler": "handlers/add_pr.js",
      "params": {
        "feature_id": { "type": "string", "required": true },
        "url": { "type": "string", "required": true }
      }
    }
  ],

  "events": [
    {
      "on": "link_created",
      "filter": { "link_type": "github-pr" },
      "handler": "handlers/on_link_created.js"
    }
  ],

  "tabs": [
    {
      "id": "prs",
      "label": "Pull Requests",
      "emoji": "🔀",
      "sortOrder": 350,
      "component": "ui/tab.html",
      "badge_query": "SELECT COUNT(*) FROM ext_github_prs WHERE feature_id = ? AND status = 'open'"
    }
  ],

  "instructions": "MCP instructions injected into fh-mcp context when extension is enabled."
}
```

**Manifest rules:**
- `id`, `name`, `version` are required. Missing any → extension skipped.
- All custom table names must be prefixed with `ext_`. Unprefixed names are rejected.
- Duplicate extension IDs → second one skipped with a warning.
- Unknown manifest fields are ignored for forward compatibility.

## Lifecycle & Registration

### Discovery

At app startup and on storage change, the Rust backend scans `<storage>/extensions/*/extension.json`. Each manifest is parsed, validated, and stored in `AppState` as an `ExtensionRegistry`.

### Table Provisioning

Declared tables are created in SQLite using `CREATE TABLE IF NOT EXISTS` with the same migration-guard pattern as built-in tables. Column additions use `ALTER TABLE` guarded by column-existence checks.

### MCP Tool Registration

`fh-mcp` is a separate binary. At startup it loads extensions independently by scanning `<storage>/extensions/*/extension.json` using the same storage path it receives from config. Each declared tool is exposed as a dynamic MCP tool. Invocation spawns the handler script as a child process with params passed as JSON via stdin.

### Event Routing

Tauri commands that produce events (e.g., `add_link` in `commands/links.rs`) call a new `extensions::dispatch_event(registry, event_type, payload)` function after the DB write succeeds. There is no separate event bus — it is a direct function call that iterates registered handlers, matches by `on` type and optional `filter` fields, and spawns matching scripts asynchronously.

### Tab Registration

A new `get_extensions` Tauri command returns all loaded extensions. The frontend calls this at boot and registers extension tabs dynamically alongside built-in tabs. Extension tab UI is rendered in an iframe.

### `requires` Checks

Each entry in `requires` is verified via `which <tool>` (Windows: `where`) at registration time. Missing requirements do not block loading — the extension registers but shows a warning in Settings. MCP tools from that extension return a descriptive error when invoked.

## Script Execution Protocol

Handler scripts are executed as child processes. The app writes a JSON payload to stdin and reads the result from stdout.

**stdin:**
```json
{
  "params": { "feature_id": "...", "url": "..." },
  "db_path": "/path/to/feature-hub.db",
  "storage_path": "/path/to/storage",
  "feature_id": "..."
}
```

**stdout (success):**
```json
{ "ok": true, "data": { ... } }
```

**stdout (failure):**
```json
{ "ok": false, "error": "Descriptive error message" }
```

**Constraints:**
- Default timeout: 10 seconds (overridable per-tool in manifest via `"timeout_secs"`)
- stdout capped at 1MB
- Scripts inherit no env vars except `FH_DB_PATH`, `FH_STORAGE_PATH`, `FH_FEATURE_ID`
- Failed scripts log to `<storage>/extensions/<id>/logs/` (rotating, last 100 lines)

Scripts receive `db_path` for direct SQLite access. No query sandboxing in this pass — extensions are user-installed and trusted. A permissions model is deferred to the marketplace phase.

## Frontend Integration

### Dynamic Tab Registration

`src/lib/modules/index.ts` calls `get_extensions` at boot. Enabled extension tabs are registered in the tab registry via the same `registerTab()` call used by built-in modules. The tab `component` is a generic `ExtensionTabFrame.svelte` that renders the extension's iframe.

Extension HTML files are served via Tauri's `asset:` protocol (already available in Tauri 2), which allows the app to load local files from arbitrary paths. The iframe `src` is set to `asset://<absolute-path-to-tab.html>`.

### iframe Communication

The parent app passes context to the iframe via `postMessage` on mount:

```json
{
  "type": "fh:init",
  "featureId": "...",
  "feature": { ... },
  "storagePath": "..."
}
```

The extension iframe calls back via `postMessage` to trigger Tauri commands (proxied through the parent):

```json
{ "type": "fh:invoke", "command": "...", "params": { ... }, "requestId": "123" }
```

The parent replies:

```json
{ "type": "fh:invoke-result", "requestId": "123", "ok": true, "data": { ... } }
```

### Badge Counts

Extension tabs that declare a `badge_query` in the manifest get their badge evaluated via a new `get_extension_badge(extension_id, tab_id, feature_id)` Tauri command. The query runs against the storage DB with `feature_id` bound to the `?` parameter.

### Settings UI

The existing `SettingsModal.svelte` extensions section is extended to show directory-based extensions alongside built-in ones. Each extension card shows:
- Name, version, description
- Enabled toggle
- `requires` check results (e.g., "✓ gh found" / "✗ gh not found — install from brew install gh")
- Source: "Built-in" or "Installed"

## Error Handling

- **Event hook failures**: Fire-and-forget. Hook failure is logged but does not roll back the triggering action.
- **Tool failures**: Returned as MCP error responses with the script's error message.
- **Malformed manifest**: Extension skipped, warning written to app log.
- **Script timeout**: Process killed, error returned to caller.
- **Table collision**: Extension with a table that conflicts with an existing table name is rejected at registration.

## Testing

### Rust unit tests

- `ExtensionRegistry::load_from_dir()` with fixture directories: valid extension, missing required fields, duplicate IDs, unprefixed table name
- Table provisioning: verify `ext_*` tables created in `test_db()`
- Script execution: fixture script that echoes params back, verify stdin/stdout protocol
- Event routing: `link_created` event dispatches to correct handler(s), filtered correctly

### Frontend tests

- `get_extensions` mock: verify extension tabs appear in the tab registry
- Settings panel: enabled/disabled toggle, `requires` check display
- Badge query: verify count renders correctly from mock data

### Manual integration test (documented)

1. Install a minimal fixture extension with one MCP tool
2. Verify tool appears in the MCP tool list (connect a Claude session and check available tools)
3. Invoke the tool via MCP, verify script runs and returns expected data
4. Add a `github-pr` link to a feature, verify `on_link_created` hook fires and logs output
5. Verify extension tab renders in feature detail view

## Out of Scope

- Sandboxed DB query permissions (deferred to marketplace phase)
- Extension dependency management (npm-style)
- Extension auto-update / marketplace fetch
- GitHub PRs extension implementation (separate spec)
- Hot-reload of extensions without app restart
