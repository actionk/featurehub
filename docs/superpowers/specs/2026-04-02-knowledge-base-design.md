# Knowledge Base — Design Spec

Storage-scoped, hierarchical knowledge base for FeatureHub. Primary consumer is Claude (AI sessions). Entries are markdown documents organized in folders, stored in SQLite, surfaced to Claude via a compact TOC in session instructions + on-demand MCP tools.

## Data Model

Two new tables in `feature-hub.db`, no required `feature_id` FK (storage-scoped):

### `knowledge_folders`

| Column     | Type          | Notes                                              |
|------------|---------------|----------------------------------------------------|
| id         | TEXT PK       | UUID                                               |
| parent_id  | TEXT nullable | FK → knowledge_folders(id) ON DELETE CASCADE       |
| name       | TEXT NOT NULL  | Folder name                                        |
| sort_order | INTEGER       | DEFAULT 0, manual ordering                         |
| created_at | TEXT NOT NULL  | ISO 8601                                           |

Max nesting depth: 5 (enforced in application code, same as Files tab folders).

### `knowledge_entries`

| Column      | Type          | Notes                                              |
|-------------|---------------|----------------------------------------------------|
| id          | TEXT PK       | UUID                                               |
| folder_id   | TEXT nullable | FK → knowledge_folders(id) ON DELETE SET NULL      |
| feature_id  | TEXT nullable | FK → features(id) ON DELETE SET NULL               |
| title       | TEXT NOT NULL  | Entry title                                        |
| description | TEXT NOT NULL DEFAULT '' | One-line summary (used in TOC)            |
| content     | TEXT NOT NULL DEFAULT '' | Full markdown body                        |
| sort_order  | INTEGER       | DEFAULT 0, manual ordering                         |
| created_at  | TEXT NOT NULL  | ISO 8601                                           |
| updated_at  | TEXT NOT NULL  | ISO 8601                                           |

### FTS5 indexing

Add `entity_type = 'knowledge'` rows to the existing `search_index` FTS5 table. Index `title + description + content`. The `entity_id` column stores the entry's UUID; `feature_id` is set to an empty string (storage-scoped entries have no owning feature, but the FTS5 schema requires the column).

Index on create and update, delete on entry removal — same pattern as `index_note`, `index_link`, etc. in `db/search.rs`.

## MCP Integration

### TOC injection in session instructions

When `fh-mcp` builds feature context in `build_feature_context()`, append a "Knowledge Base" section after the existing content. Query all entries ordered by folder path + sort_order and render a compact TOC:

```
## Knowledge Base (12 entries)
Use get_knowledge_entry(id) to read full content.

- [Backend/] Auth flow explained (id: abc-123)
- [Backend/] Database migration patterns (id: def-456)
- [Backend/Migrations/] Column-existence probe pattern (id: mno-345)
- [Infra/] CI/CD pipeline setup (id: ghi-789)
- API rate limiting findings (id: jkl-012)
```

Format: `[folder/path/] title (id: <id>)` — one line per entry, folder path as prefix, root entries have no prefix. Entries with a `description` show it in parentheses after the title if space permits.

This section is always included when a storage has any knowledge entries, regardless of which feature is scoped. It provides awareness without loading content.

### MCP tools

All tools are registered on the `fh-mcp` server alongside existing tools.

**Read tools:**

| Tool                    | Parameters                     | Returns                                    |
|-------------------------|--------------------------------|--------------------------------------------|
| `list_knowledge`        | `folder_id?: string`           | TOC tree: entries with id, title, description, folder path |
| `get_knowledge_entry`   | `id: string`                   | Full entry: title, description, content, folder path, feature link |
| `search_knowledge`      | `query: string`                | FTS5 search results with snippets          |

**Write tools:**

| Tool                        | Parameters                                                        | Returns        |
|-----------------------------|-------------------------------------------------------------------|----------------|
| `create_knowledge_entry`    | `title, content, description?, folder_id?, feature_id?`          | Created entry  |
| `update_knowledge_entry`    | `id, title?, content?, description?, folder_id?, feature_id?`    | Updated entry  |
| `delete_knowledge_entry`    | `id`                                                              | Success        |
| `create_knowledge_folder`   | `name, parent_id?`                                                | Created folder |
| `rename_knowledge_folder`   | `id, name`                                                        | Updated folder |
| `delete_knowledge_folder`   | `id`                                                              | Success        |

`delete_knowledge_folder` moves child entries to the parent folder (or root if no parent) before deleting, rather than cascading deletes to entries. Child folders are cascade-deleted.

### Notification integration

Write tools call `config::push_notification()` after mutations, same pattern as existing feature mutations. Notification payload includes `entity_type: "knowledge"` and the entry/folder id. The Tauri app's 2s poll picks these up and refreshes the KB panel if open.

## Rust Implementation

### New modules

- `src-tauri/src/db/knowledge.rs` — CRUD for `knowledge_entries` and `knowledge_folders`. Functions: `create_entry`, `update_entry`, `delete_entry`, `get_entry`, `list_entries`, `list_entries_in_folder`, `create_folder`, `rename_folder`, `delete_folder`, `list_folders`, `get_folder_path` (walks parent chain, same pattern as `db/folders.rs`).
- `src-tauri/src/commands/knowledge.rs` — Tauri IPC commands wrapping the db module. Registered in `lib.rs` invoke_handler.

### DB initialization

Add `CREATE TABLE IF NOT EXISTS` statements to `db::initialize()` for both tables. No migrations needed since the tables are new.

### FTS5 integration

Add `index_knowledge_entry` and `delete_knowledge_index` functions to `db/search.rs`, following the existing pattern. Called from `db/knowledge.rs` on create/update/delete.

### MCP server changes

In `src-tauri/src/bin/fh_mcp.rs`:
1. Add tool handlers for all 9 knowledge tools
2. In `build_feature_context()`, append the TOC section by querying `list_entries` + `list_folders` and formatting the compact list

## Frontend

### UI placement

A dedicated icon button in the sidebar header (alongside existing controls) that opens a full-width panel replacing the feature detail view. This keeps the sidebar clean and gives the KB a full editing surface.

The sidebar button shows a book/library icon. When active, the feature list remains visible but no feature is "selected" — the detail area shows the KB panel instead.

### KB panel layout

Two-column layout:

**Left column — folder tree + entry list:**
- Collapsible folder tree (same interaction as Files tab)
- Entries listed under their folders with drag-to-reorder
- "New Entry" and "New Folder" buttons at the top
- Click entry to open in editor

**Right column — entry editor:**
- Title field (text input)
- Description field (text input, one-liner)
- Folder selector (dropdown)
- Feature link selector (dropdown, optional)
- Markdown editor for content (same component as NotesEditor)
- Toggle between edit and preview modes (using existing MarkdownPreview component)
- Auto-save on blur or debounced typing (same pattern as notes)

### Components

New module at `src/lib/modules/knowledge/`:
- `KnowledgePanel.svelte` — root layout (two columns)
- `KnowledgeFolderTree.svelte` — folder tree with expand/collapse
- `KnowledgeEntryList.svelte` — entry list within a folder
- `KnowledgeEntryEditor.svelte` — editor panel

These are NOT tab modules (no `registerTab()`) since the KB is storage-scoped, not feature-scoped. The panel is mounted directly by `App.svelte` when the KB mode is active.

### TypeScript API

New file `src/lib/api/knowledge.ts` with wrapper functions for all Tauri commands:
- `getKnowledgeEntries(folderId?)`, `getKnowledgeEntry(id)`, `searchKnowledge(query)`
- `createKnowledgeEntry(...)`, `updateKnowledgeEntry(...)`, `deleteKnowledgeEntry(id)`
- `getKnowledgeFolders()`, `createKnowledgeFolder(...)`, `renameKnowledgeFolder(...)`, `deleteKnowledgeFolder(id)`

Types added to `src/lib/api/types.ts`:
- `KnowledgeEntry { id, folder_id?, feature_id?, title, description, content, sort_order, created_at, updated_at }`
- `KnowledgeFolder { id, parent_id?, name, sort_order, created_at }`

## Scope exclusions

- No vector/semantic search — FTS5 keyword search is sufficient for now
- No versioning/history — last write wins, consistent with notes/context
- No import-from-disk or export-to-disk (can be added later as a one-time operation)
- No cross-storage KB (each storage has its own, travels with the storage)
- No auto-summarization or AI-generated entries (Claude can write entries manually via MCP tools)
- No tags on knowledge entries (folder organization is sufficient; can add later)
