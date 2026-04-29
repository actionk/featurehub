# AGENTS.md

This file provides guidance to Codex (Codex.ai/code) when working with code in this repository.

## What Is This

Feature Hub — a desktop app for managing work features/epics with linked Codex sessions, external links, files, tasks, and notes. It also ships a CLI (`fh`) and an MCP server (`fh-mcp`) for interacting with features from the terminal or from Codex sessions.

## Tech Stack

- **Shell**: Tauri 2 (Rust backend)
- **UI**: Svelte 5 + TypeScript + Vite
- **Storage**: SQLite via `rusqlite` (bundled), stored in a user-chosen "storage" directory as `feature-hub.db`
- **Styling**: CSS custom properties (dark theme) in `src/app.css` + minimal Tailwind v4
- **Markdown**: `marked` library for notes/context preview
- **Diagrams**: `mermaid` library for diagram rendering in notes/context (toggled via `mermaid_diagrams` setting)
- **Search**: SQLite FTS5 full-text search
- **Testing (Rust)**: `#[cfg(test)]` modules with in-memory SQLite via `db::test_utils::test_db()`
- **Testing (Frontend)**: Vitest + @testing-library/svelte + jsdom
- **MCP**: `rmcp` crate for the MCP server binary
- **Window State**: `tauri-plugin-window-state` for persisting window position/size across sessions

## Dev Commands

- `npm run tauri dev` — Start dev mode (Vite + Tauri)
- `npm run tauri build` — Production build with installers
- `npm run build:cli` — Build CLI binaries and install to local Programs directory
- `cargo check` — Check Rust compilation (run from `src-tauri/`)
- `cargo build` — Build all binaries including `fh` and `fh-mcp` (run from `src-tauri/`)
- `cargo test --lib` — Run Rust unit tests (run from `src-tauri/`)
- `npm run test` — Run frontend tests (Vitest)
- `npm run test:watch` — Run frontend tests in watch mode

## Architecture

### Three entry points

1. **Tauri app** (`src-tauri/src/main.rs` → `lib.rs`) — the GUI. Tauri IPC commands live in `src-tauri/src/commands/` (split into domain modules), which delegate to `db::*` and `files::*` modules.
2. **`fh` CLI** (`src-tauri/src/bin/fh.rs`) — terminal tool to `start`/`resume`/`list` Codex sessions linked to features. Spawns `Codex` as a child process with `--session-id`, `--mcp-config`, and `--add-dir` flags. Uses per-feature workspace directories under the storage's `workspaces/` folder.
3. **`fh-mcp` MCP server** (`src-tauri/src/bin/fh_mcp.rs`) — stdio-based MCP server that Codex connects to during sessions. Exposes tools for reading/writing features, tasks, notes, context, links, repositories, tags, plans, skills, and settings. Uses `rmcp` crate with `#[tool]` / `#[tool_router]` / `#[tool_handler]` macros. When scoped to a feature (`--feature` flag), pre-loads a rich context snapshot into `ServerInfo.instructions`.

All three share the same `db`, `config`, and `storage` modules from the `feature_hub` library crate.

### Storage model

The app supports multiple "storages" (directories on disk), each containing its own `feature-hub.db`, `files/` directory, `workspaces/` directory, and `settings.json`. Storage config lives in the OS app data dir (`com.littlebrushgames.feature-hub/config.json`). The user picks an active storage; the Tauri app swaps the DB connection at runtime via `AppState`. Machine-specific settings (fonts, CLI path, preferred IDEs) live in a separate `settings.json` in the OS app data dir; portable settings (MCP servers, skills, extensions, default repos) live in each storage's own `settings.json`.

### Frontend ↔ Backend

- All Tauri commands are registered in `lib.rs` `invoke_handler` and implemented in `src-tauri/src/commands/` — modules: context, export_import, features, files, folders, git, groups, ide, links, mcp, notes, notifications, plans, repos, search, sessions, settings, skills, storage, tags, tasks, terminal, timeline
- TypeScript wrappers live in `src/lib/api/` — domain files (features.ts, sessions.ts, tasks.ts, plans.ts, etc.) with shared types in `types.ts`. `tauri.ts` is a barrel re-export for backward compatibility.
- Tauri IPC uses camelCase args on the TS side, snake_case on Rust side (Tauri auto-converts)
- Svelte 5 rune-based stores live in `src/lib/stores/` (settings, events, sessionActivity, tabToolbar, terminals)

### Database

- Schema defined in `src-tauri/src/db/mod.rs` `initialize()` — creates tables + runs migrations
- Migrations are inline ALTER TABLE statements guarded by column-existence checks
- Each entity has its own module in `db/`: features, links, tags, directories, files, folders, sessions, tasks, notes, context, search, mcp_servers, plans, skills, feature_groups
- FTS5 `search_index` table for global search across entities

### Database tables

- `features` — core entity with title, ticket_id, status, left_off_text, sort_order, description, pinned, archived, parent_id, group_id
- `feature_groups` — groups for organizing features in the sidebar
- `tags` / `feature_tags` — tagging system with colors, many-to-many join
- `links` — URLs attached to features with type auto-detection, description, and metadata
- `directories` — project directory paths linked to features with optional labels, repo_url, clone_status
- `files` — uploaded files stored in per-feature directories, with folder_id for organization
- `folders` — hierarchical folder tree for organizing files (parent_id self-reference)
- `sessions` — Codex sessions with claude_session_id, timing (started_at/ended_at), project_path, branch
- `tasks` — feature tasks with done status, supports external sources (Jira) via source, external_key, external_url, external_status, description
- `notes` — one note per feature (markdown content)
- `context` — one persistent context per feature (instructions/requirements that persist across Codex sessions)
- `plans` — Codex implementation plan submissions with status (pending/approved/rejected)
- `feature_mcp_servers` — per-feature MCP server enable/disable overrides
- `feature_skills` — per-feature skill overrides
- `feature_branches` — git branch tracking per feature/directory
- `search_index` — FTS5 virtual table for full-text search
- `knowledge_folders` — hierarchical folders for organizing KB entries (storage-scoped, no feature_id)
- `knowledge_entries` — markdown documents with title, description, content, optional folder_id and feature_id link

### Files subsystem

- `files::manager` — copies files into per-feature storage directories, handles folder creation/rename/move/delete on disk
- `files::preview` — generates previews for text files (syntax-highlighted), images (base64-encoded), and binary files. Supports sniffing unknown extensions via null-byte detection. Size limits: 2MB text, 10MB images.

### Codex integration

- `Codex::launcher` — spawns `Codex` CLI with session/MCP/directory args for start and resume
- `Codex::scanner` — scans local `.Codex/projects/` directories for Codex sessions, checks if sessions are active
- `Codex::session_parser` — parses session JSONL transcripts and summary `.md` files for titles/summaries

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
- `mcp_servers` — list of MCP servers injected into Codex sessions
- `default_repositories` — predefined repository URLs for feature init
- `skills` — skill definitions
- `extensions` — extension configuration (Jira, Slite, etc.)

The Tauri command `get_settings` merges both into a flat `CombinedSettings` struct. `save_settings` accepts optional fields for both and writes each to the correct file. When editing settings code, always know which struct a field belongs to.

### Key type distinctions

Three overlapping types represent features — know which to use:

- **`FeatureSummary`** (`db/features.rs`) — used in the sidebar list. Has task counts (`task_count_total`, `task_count_done`) and tags, but no `directories` or `links`. Returned by `get_features`.
- **`Feature`** (`db/features.rs`) — used in the detail view. Has full `directories`, `links`, and `tags`. Returned by `get_feature` and mutation commands.
- **`FeatureData`** (`commands/mod.rs`) — IPC bundle returned by `get_feature_data`. Wraps a `Feature` together with `all_tags`, `tasks`, `plans`, and `note` in a single round-trip. Used by `FeatureDetail.svelte` on initial load to avoid 5 separate IPC calls.

On the TypeScript side, `src/lib/api/types.ts` has a single `Feature` interface with optional fields (`task_count_total?`, `tags?`, etc.) that covers both shapes — be careful not to assume a field exists when you only have a summary.

### MCP server features

The MCP server (`fh-mcp`) exposes these tool categories:
- **Read**: list_features, get_feature, get_tasks, get_links, get_note, get_sessions, get_files, get_repositories, get_default_repositories, get_tags, get_context, get_skills, get_plan_status, search, get_current_feature, get_settings, list_knowledge, get_knowledge_entry, search_knowledge
- **Write**: create_feature, update_feature, delete_feature, set_feature_parent, toggle_pin, set_archived, create_task, update_task, delete_task, save_note, save_context, add_link, update_link, delete_link, clone_repository, create_tag, toggle_tag, submit_plan, update_plan, link_session, create_skill, update_skill, delete_skill, save_settings, create_knowledge_entry, update_knowledge_entry, delete_knowledge_entry, create_knowledge_folder, rename_knowledge_folder, delete_knowledge_folder

When scoped to a feature, the server injects a rich context into `ServerInfo.instructions` including feature summary, directories, tasks, links, context, and notes — plus behavioral instructions for feature initialization, context maintenance, and custom commands like "refresh feature".

### Plans system

Codex sessions can submit implementation plans via `submit_plan`. Plans have a status lifecycle: `pending` → `approved`/`rejected`. The UI shows plan cards in the AI tab where the user can review and approve/reject them. Codex sessions can poll `get_plan_status` to wait for approval before proceeding.

### Knowledge Base

Storage-scoped knowledge repository for HOW-TOs, findings, and research results. Primary consumer is Codex — a compact TOC (titles + IDs) is injected into every MCP session's instructions so Codex knows what's available. Full content is loaded on demand via `get_knowledge_entry(id)`. Entries are organized in hierarchical folders (max depth 5). Entries optionally link to features via nullable `feature_id`. CRUD via both Tauri commands and MCP tools. FTS5 indexed with `entity_type = 'knowledge'`.

### Skills system

Skills are reusable instruction templates that can be attached to features. Managed via `create_skill`/`update_skill`/`delete_skill` MCP tools. Per-feature skill overrides stored in `feature_skills` table. The AI tab includes a SkillsPanel for management.

### Notification system

MCP tool calls trigger live UI updates via a file-based IPC chain:

1. `fh-mcp` calls `config::push_notification()` (or `push_notification_ex()` for plan notifications) which **appends** a JSON line to `<config-dir>/notifications.jsonl`
2. The Tauri app polls `poll_notifications` every **2 seconds** — reads and truncates the file
3. `App.svelte` processes each notification: shows a toast, refreshes the sidebar, and if the affected feature is open in a tab, refreshes its data and optionally navigates to the AI tab (for plan submissions)

There is no push mechanism — 2s is the minimum notification latency. The `plan_id` field on a notification causes the UI to auto-navigate to the pending plan in the AI tab.

### Frontend components

- `App.svelte` — root layout with sidebar + detail view, notification polling, storage setup
- `Sidebar.svelte` — feature list with search, filtering, drag-to-reorder, feature groups
- `FeatureDetail.svelte` — generic tab renderer driven by module registry
- `CreateFeatureModal.svelte` — feature creation dialog
- `ExportImportModal.svelte` — export/import features dialog
- `SearchBar.svelte` — global FTS5 search
- `MarkdownPreview.svelte` — markdown rendering component (shared)
- `SettingsModal.svelte` — app settings (CLI path, MCP servers, default repositories, integrations, fonts)
- `StorageSelector.svelte` / `StorageSetup.svelte` — multi-storage management
- `ToastContainer.svelte` — notification toast display
- `StatusBadge.svelte` / `TagBadge.svelte` — status and tag display components
- `src/lib/components/ui/` — shared UI primitives: ConfirmDialog, Dropdown, IconButton, Modal
- `src/lib/modules/knowledge/` — KnowledgePanel (root), KnowledgeFolderTree, KnowledgeEntryEditor

### Tab modules (`src/lib/modules/`)

Each tab is a self-contained module that registers itself via `registerTab()` in `src/lib/modules/registry.ts`:

- **ai/** — AiPanel, SessionList, SessionCard, PlanCard, PlanDetail, ContextEditor, McpServersPanel, SkillsPanel, Terminal
- **links/** — LinksGrid, LinkCard
- **repos/** — RepositoriesPanel
- **tasks-notes/** — TasksNotesPanel, TaskList, NotesEditor
- **files/** — FileBrowser, FileList, FilePreviewPanel, FolderBreadcrumb
- **timeline/** — Timeline

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

### Additional library modules

- `src-tauri/src/export_import.rs` — feature export/import functionality
- `src-tauri/src/jira.rs` — Jira integration module
- `src-tauri/src/terminal.rs` — terminal/PTY state management

## Conventions

- Feature status values: `active` (default), `todo`, `in_progress`, `in_review`, `done`, `blocked`, `paused`
- Plan status values: `pending`, `approved`, `rejected`
- All styling uses CSS classes in `src/app.css` with CSS custom properties — avoid Tailwind utility classes in Svelte templates (unreliable with Tailwind v4 content detection)
- Svelte 5 runes exclusively (`$state`, `$derived`, `$effect`, `$props`) — no legacy stores
- Callback props for events (not `createEventDispatcher`)
- `onclick` syntax (not `on:click`)
- UUIDs for all entity IDs, ISO 8601 strings for dates
- Link types are auto-detected from URLs (github, jira, figma, confluence, slack, notion, linear, etc.)
- Tasks support dual source: `manual` (user-created) or `jira` (external with key/url/status)
- Per-feature MCP server and skill overrides allow enabling/disabling specific servers/skills for individual features
- Features support pinning, archiving, parent-child hierarchy, and group organization

### Testing

- **Rust tests** use `#[cfg(test)]` modules within each source file. `db::test_utils::test_db()` creates a fresh in-memory SQLite DB with the full schema for each test — no cleanup needed.
- **Frontend tests** use Vitest with jsdom environment. Svelte 5 components with Snippet props need test wrapper components (see `IconButtonTestWrapper.svelte` pattern). The `resolve: { conditions: ["browser"] }` setting in `vite.config.ts` is required for Svelte component tests to resolve client-side code.
- Test files live next to the code they test: `*.test.ts` for frontend, `#[cfg(test)] mod tests` for Rust.
