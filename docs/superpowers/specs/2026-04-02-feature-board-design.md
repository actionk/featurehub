# Feature Board Design Spec

## Overview

A Kanban-style board view that shows all non-archived features organized by status columns. Lives as a special workspace tab in the right panel, alongside regular feature tabs. Primary use: bird's-eye triage with drag-to-reorder and quick access to Claude sessions.

## Board Structure

### Workspace Tab

- The board is a **special workspace tab** that appears in the workspace tab bar alongside feature tabs
- Opened via a "Board" button in the sidebar header
- Closeable like any feature tab
- Uses a distinct tab type (not tied to a `featureId`) — the workspace tab system needs a minor extension to support non-feature tabs
- Tab label: "📋 Board" (or just "Board" if `show_tab_emojis` is off)

### Header Bar

- **Title**: "Feature Board"
- **Stats**: Total feature count, active session count (from the session activity polling store)
- **Filters** (right side):
  - Filter by tag (dropdown, multi-select)
  - Filter by group (dropdown, multi-select)
- Filters are session-only state (not persisted)

### Columns

Four equal-width columns, left to right:

| Column | Status Value | Color | Dot Color |
|--------|-------------|-------|-----------|
| Todo | `todo` | `#55556a` (muted) | `--text-muted` |
| In Progress | `in_progress` | `#f0b232` (amber) | `--amber` |
| In Review | `in_review` | `#52a9ff` (blue) | `--blue` |
| Done | `done` | `#3dd68c` (green) | `--green` |

Each column has:
- Colored status dot + uppercase label + count badge
- Collapse chevron (toggle to hide column content, persisted in localStorage)
- Independent vertical scroll for overflow
- Columns separated by subtle `1px solid var(--bg-card)` borders

**Features with non-board statuses** (`active`, `blocked`, `paused`) appear in the **Todo** column as a catch-all. The `active` status is the default for new features, so they naturally land in Todo.

### Feature Cards

Each card displays:

1. **Title** — with 📌 pin emoji prefix if pinned
2. **Active session badge** (top-right) — green badge showing "● N sessions" if the feature has active Claude sessions. Hidden if no active sessions.
3. **Tags** — row of tag badges (same style as sidebar), shown if feature has tags
4. **Task progress** — progress bar + "X/Y tasks" count. Hidden if feature has 0 tasks.
5. **Quick session action** (bottom-right):
   - "▶ Start" (blue) — if no sessions exist for this feature. Launches new Claude session via `fh` CLI.
   - "▶ Resume" (green) — if sessions exist. Resumes most recent session.

Card styling:
- `background: var(--bg-card)` with `border-radius: var(--radius-md)`
- Left border: 3px solid, colored by column status
- Padding: 10px 12px
- Gap between cards: 6px
- Cursor: `grab` (draggable)

**Sort order within columns**: Pinned features first, then by `sort_order`.

### Done Column — Stale Card Behavior

Features in the Done column that have been done for **3+ days** get special treatment:

- Card fades to **45% opacity**
- Title text color shifts to `--text-secondary`
- An **"Archive" button** appears (amber background) — clicking sets `archived = true`
- A "done Xd ago" timestamp shows when the feature was completed
- Dragging a stale card back to another column removes the stale state (status change resets the timer)

**Staleness is calculated from the feature's `updated_at` timestamp** — when the status was last changed to `done`. No new database column needed; `updated_at` is already set on every status change.

## Interactions

### Drag & Drop

- Cards can be dragged between columns to change their status
- Drop indicator shows a horizontal line where the card will land
- On drop: call `update_feature` with the new status value
- Reordering within a column updates `sort_order`
- Use HTML5 drag-and-drop API (no external library)

### Click to Open

- Clicking a card (not on a button) opens the feature in a workspace tab, same behavior as clicking in the sidebar
- If the feature is already open in a tab, focuses that tab

### Session Actions

- **"Start" button**: Calls the same session launch flow as the sidebar/AI tab. Opens a terminal tab or triggers `fh start`.
- **"Resume" button**: Resumes the most recent session for the feature.
- These buttons stop event propagation (don't trigger card click/open).

### Archive Action

- The "Archive" button on stale Done cards calls `set_archived(feature_id, true)`
- Card animates out of the board
- Archived features are accessible via the sidebar's existing "Done" filter

### Filters

- **Tag filter**: Multi-select dropdown. When active, only features with at least one selected tag are shown across all columns.
- **Group filter**: Multi-select dropdown. When active, only features in selected groups are shown.
- Filters combine with AND logic (must match both tag and group filters).
- Filter state is ephemeral (resets on tab close).

## Data Flow

### Loading

1. Board tab mounts → calls `get_features()` to get all features (same as sidebar)
2. Features are partitioned into columns by `status` field
3. Active session counts come from the `sessionActivity` store (already polls every 10s)
4. Task counts come from `task_count_total` / `task_count_done` fields on FeatureSummary
5. Tags come from the `tags` field on FeatureSummary

### Refresh

- Board listens to the same event bus notifications as the sidebar
- When a feature is mutated (via MCP, detail view, or board drag), the board re-fetches `get_features()`
- Session activity updates automatically via the polling store

### No New Backend Commands

The board uses only existing Tauri commands:
- `get_features` — list all features
- `update_feature` — change status on drag
- `set_archived` — archive stale features
- `get_tags` — for the tag filter dropdown
- `get_feature_groups` — for the group filter dropdown

Session activity data comes from the existing `sessionActivity` store.

## Implementation Scope

### Frontend (new files)

- `src/lib/modules/board/BoardPanel.svelte` — main board component
- `src/lib/modules/board/BoardColumn.svelte` — single column with drop zone
- `src/lib/modules/board/BoardCard.svelte` — feature card
- `src/lib/modules/board/index.ts` — tab registration (does NOT use `registerTab` since this isn't a feature-scoped tab)

### Frontend (modified files)

- `src/lib/stores/workspaceTabs.svelte.ts` — extend to support non-feature tabs (board tab type)
- `src/App.svelte` — render BoardPanel when the active workspace tab is a board tab
- `src/lib/components/Sidebar.svelte` — add "Board" button to sidebar header

### No Backend Changes

All required data and mutations already exist. No new Rust code needed.

## Out of Scope

- WIP limits per column
- Card creation from the board (use sidebar or MCP)
- Inline editing of feature details on cards
- Column reordering or customization
- Persistent filter presets
- Board for a single feature's sub-tasks (this is a storage-level board)
