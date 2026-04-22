# Sessions Panel — Design Spec

**Date:** 2026-04-05  
**Status:** Approved

## Overview

A persistent right-side panel in the app shell that shows all Claude sessions across all features globally. Active sessions are visually highlighted. Clicking a row navigates to the session's feature and opens its terminal. Toggled via a new icon rail button.

## Layout

The app shell gains a fourth column to the right of the main content area:

```
[Icon Rail 52px] [Sidebar ~272px] [Main Content flex:1] [Sessions Panel 260px]
```

Panel is 260px fixed width (no resize handle). Visibility toggled by an icon rail button (⚡, positioned above the settings button). Panel open/closed state persisted in `localStorage`.

## Icon Rail Button

- Icon: lightning bolt (⚡) or similar "agents" icon
- Position: bottom of rail, above the settings gear
- When panel is open: button shown with active/accent background
- Badge: green circle showing count of currently active sessions (hidden when 0)

## Panel Structure

### Header
`SESSIONS` label (left) + `● N active` indicator (right, green, hidden when 0 active).

### Active Section
Label: `ACTIVE`. Shows all sessions where the corresponding `claude` process is currently running (detected via `get_running_session_ids`). Sorted by `last_modified` desc.

### Recent Section
Label: `RECENT`. Shows all other DB sessions (not currently active), sorted by `last_modified` desc. All sessions from all features included.

### Session Row

Each row shows:
- Status dot (green pulsing = active, grey = idle)
- Feature name (truncated with ellipsis)
- Branch pill (grey badge)
- Status text: `● Active` (running process detected) / `⊘ Idle` (no running process). Sub-states like Generating/Thinking are not detectable from outside the Claude process.
- Time since last activity (`last_modified` formatted as relative time)
- Model name (e.g. `claude-opus-4-6`)
- Total token count (e.g. `42k tok`) or cost if `costUSD` is available in JSONL

Active rows: brighter colours, green left border, full opacity.  
Idle rows: dimmed text, grey left border, 75% opacity.

**Click behaviour:** navigate to the session's feature (open it in the main view) and switch to the AI tab, then open the terminal for that session.

## Data Architecture

### Approach: Extend `sessionActivity` store (Option B)

The existing `sessionActivity` Svelte store in `src/lib/stores/sessionActivity.svelte.ts` polls every 10 seconds. It will be extended to also fetch and return full session panel data.

### New Tauri command: `get_sessions_panel_data`

Returns `SessionsPanelData`:

```rust
pub struct SessionsPanelData {
    pub sessions: Vec<PanelSession>,
    pub active_count: usize,
}

pub struct PanelSession {
    pub id: String,               // DB session ID
    pub feature_id: String,
    pub feature_name: String,
    pub claude_session_id: Option<String>,
    pub branch: Option<String>,
    pub last_modified: String,    // ISO 8601, from JSONL mtime or DB started_at
    pub is_active: bool,
    pub model: Option<String>,    // from JSONL
    pub total_tokens: Option<u64>,// input + output tokens summed from JSONL
    pub cost_usd: Option<f64>,    // summed costUSD from JSONL if non-null
}
```

### JSONL Stats Cache

To avoid re-parsing large JSONL files on every 10s poll, a `Mutex<HashMap<String, CachedStats>>` is stored in `AppState`:

```rust
pub struct CachedStats {
    pub mtime: SystemTime,
    pub model: Option<String>,
    pub total_tokens: u64,
    pub cost_usd: Option<f64>,
}
```

On each poll: locate JSONL file for session, check mtime. If mtime unchanged, use cache. If changed (or not cached), re-parse and update cache.

### JSONL Parsing for Stats

New function `parse_session_stats(path: &Path) -> CachedStats` in `claude/session_parser.rs`:

- Reads JSONL line by line
- Extracts `model` from first `assistant` message's `message.model` field
- Accumulates `message.usage.input_tokens` + `message.usage.output_tokens` per assistant entry
- Accumulates `costUSD` (root field on assistant entries) when non-null
- Skips non-`assistant` lines
- Hard limit: skip files > 100MB to prevent memory issues

### `get_sessions_panel_data` Implementation

1. Load all sessions from DB across all features (join with `features` table for `feature_name`)
2. Run `get_running_session_ids()` to determine active set
3. For each session with a `claude_session_id`: locate JSONL file in `~/.claude/projects/`, check mtime cache, read stats
4. Return `SessionsPanelData`

### Frontend Store Extension

`sessionActivity.svelte.ts` gains a second derived state:

```typescript
export let panelSessions = $state<PanelSession[]>([]);
export let panelActiveCount = $state(0);
```

The existing 10s poll calls `get_sessions_panel_data` alongside the existing activity command (or replaces it if the new command subsumes the data).

### Frontend Component: `SessionsPanel.svelte`

New component at `src/lib/components/SessionsPanel.svelte`:

- Reads from `sessionActivity` store (`panelSessions`, `panelActiveCount`)
- Renders header, ACTIVE section, RECENT section
- On row click: calls existing navigation to feature + opens AI tab + switches to terminal subtab for that session
- No internal state except scroll position

### App Shell Integration (`App.svelte`)

- New `sessionsPanel` boolean state (initialised from `localStorage`)
- Icon rail button added (⚡, above settings)
- When `sessionsPanel` is true: `SessionsPanel` rendered as fourth column
- Badge on icon rail button shows `panelActiveCount` (hidden when 0)

## Relative Time Formatting

Reuse or add a `formatRelativeTime(iso: string): string` utility in `src/lib/utils.ts` (or equivalent). Format: `just now`, `2m ago`, `1h ago`, `3d ago`.

## Error Handling

- If JSONL file not found for a session: show session row with no model/token data (not an error state)
- If JSONL parse fails mid-file: use whatever was accumulated so far
- If `get_sessions_panel_data` command fails: keep previous panel data, log to console

## Out of Scope

- Resizable panel width
- Filtering/searching sessions in the panel
- Killing/stopping sessions from the panel
- Context menu (like the reference screenshot)
- Token cost computation from pricing tables (only show if `costUSD` is in JSONL)
