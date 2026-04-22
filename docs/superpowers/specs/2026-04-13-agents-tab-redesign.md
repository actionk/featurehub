# Agents Tab Redesign

**Date:** 2026-04-13  
**Status:** Approved

## Summary

Redesign the AI/Agents tab bento layout. Remove Plans and Links (they have dedicated tabs). Remove Tasks (belongs in Tasks & Notes tab). Collapse MCP Servers, Skills, and Context from full blocks into lightweight inline rows. Unify Active Session and Sessions list into a single card.

## What Changes

### Removed from the tab
- **Links** block — already has its own tab
- **Plans** block — will be redesigned separately later
- **Tasks** block — belongs in Tasks & Notes tab

### Removed blocks replaced by compact config
- **MCP Servers block** → inline row (pills + dropdown caret)
- **Skills panel block** → inline row (pill per skill name + dropdown caret)
- **Context collapsible** → inline row with "View / Edit Context" button that opens a subwindow/modal

## New Layout

### Sessions Card (full width)

One card handles all session state. No separate "Active Session" card.

**Header row:** `Sessions` label · count badge · action buttons (new, copy start command, scan)

**Slot 1 — active session state (adaptive):**
- When a session is running: expanded card with green gradient background, top accent line, live pill (`● Active Now`), elapsed timer, title, summary, `Open Terminal →` button
- When no session running: compact row with `No active session` hint and `▶ Start Session` button on the right
- When 2+ sessions running: first session expanded (as above), each additional active session gets a compact row — left accent line, live dot, title, elapsed timer, `Open →` button

**Sessions list:** Past sessions as compact rows — dot · title (truncated) · relative timestamp. Clicking resumes. When there are no past sessions, no empty state message is shown (the active slot already communicates the zero-sessions state via Start Session CTA).

**Insights footer (inside the same card):**
- Session count stat on the left
- Sparkline chart (full flex width)
- 7d / 14d / 30d range toggle on the right

### Config Rows (below sessions card, no card wrapper)

Three floating rows that hover-highlight on mouse-over. Each row: uppercase label (52px fixed width) · content · right action.

| Label | Content | Action |
|-------|---------|--------|
| MCP | Server pills (● jira, ● slite) + `+ add` text | `▾` caret → dropdown |
| Skills | Skill name pills (tdd, refactor, +N) | `▾` caret → dropdown |
| Context | "View / Edit Context" button with doc icon | Opens subwindow/modal |

## States

| State | Active slot shows |
|-------|-----------------|
| No session | Compact row: hint + Start button |
| 1 active | Expanded: live pill, timer, title, summary, Open button |
| 2 active | First expanded + second compact accent row |
| 3+ active | First expanded + N-1 compact accent rows |

## Behaviour Notes

- The existing `selectedPlan` full-screen plan detail view is untouched — it overlays when a plan is selected
- Terminal view (when `activeTerminalId` is set) is untouched — it overlays the whole panel
- The config rows replace the `McpServersPanel`, `SkillsPanel`, and `ContextEditor` components in the overview. The underlying components are still used — they open as dropdowns or a modal/subwindow when triggered from the rows
- Context subwindow: render `ContextEditor` in a modal overlay (reuse existing `Modal` component from `src/lib/components/ui/`)
- MCP and Skills dropdowns: render existing panel components in a dropdown positioned relative to the row

## Files Affected

- `src/lib/modules/ai/AiPanel.svelte` — main layout changes
- `src/app.css` — remove unused bento grid classes, add new session card / config row classes
- `src/lib/modules/ai/McpServersPanel.svelte` — may need a compact/dropdown mode prop
- `src/lib/modules/ai/SkillsPanel.svelte` — may need a compact/dropdown mode prop
- `src/lib/modules/ai/ContextEditor.svelte` — already works standalone; wrap in modal

## Out of Scope

- Plans redesign (separate effort)
- Context subwindow implementation details (keep ContextEditor as-is, just wrap in Modal)
- Any changes to the terminal view or plan detail overlay
