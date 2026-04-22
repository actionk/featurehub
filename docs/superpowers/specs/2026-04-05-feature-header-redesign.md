# Feature Header Redesign

**Date:** 2026-04-05  
**Status:** Approved  
**Scope:** `FeatureDetail.svelte` + `app.css` — header section only

## Goal

Replace the current 3-zone header (large title top-left / stat cards top-right / meta row below) with a compact 2-row layout that is denser, cleaner, and still shows all useful information.

## Layout

### Row 1 — Title + Stats + Actions

```
[  Title: Last Word gradient              ] [ ✓ 0 Done ] [ ● 0 Agents ] | [⎘] [📁] [✏] [🗑]
```

- Title (`flex: 1`, `overflow: hidden`, `text-overflow: ellipsis`) — same gradient-last-word effect
- Stat chips right-aligned, smaller than current cards (15px number, 9.5px uppercase label)
- Each chip has a 2px left accent bar (green for Done, neutral/accent for Agents, amber for Plans)
- Separator (`1px` vertical line) between chips and icon actions
- Icon actions: Copy ID, Copy Path, Edit, Delete — 24×24 ghost buttons
- Delete icon is muted red at rest, brighter red on hover

### Row 2 — Status + Tags + Metadata

```
[ In Progress ] [ + Tag ] · 16d ago · Description truncated to one line…
```

- Status badge (existing `StatusBadge` component, clickable dropdown)
- Tag badges + `+ Tag` button
- Separator dot `·`, then relative timestamp
- Separator dot `·`, then description text — `flex: 1`, single line, `text-overflow: ellipsis`
- If no description, the description segment is omitted

### Header background

Subtle gradient from `color-mix(in srgb, var(--accent) 3%, var(--bg-primary))` to `var(--bg-primary)` — same as current, retained.

## CSS changes

Remove or replace these classes:
- `.detail-stat-chips`, `.detail-stat-card`, `.detail-stat-card-icon`, `.detail-stat-card-num`, `.detail-stat-card-lbl`, `.detail-stat-live-dot` — replaced by new `.detail-chip` system
- `.detail-meta-row` — restructured
- `.detail-header-actions` — merged into row 1 right side

Add:
- `.detail-chip` — base chip (flex, padding, bg, border, border-radius, `position: relative; overflow: hidden`)
- `.detail-chip::before` — 2px left accent bar (absolute positioned)
- `.detail-chip--green`, `--accent`, `--amber` — color variants for `::before` and number color
- `.detail-chip-num` — number style (15px, bold)
- `.detail-chip-lbl` — label style (9.5px, uppercase, muted)
- `.detail-chip-dot` — live indicator dot (5×5px circle)
- `.detail-row1`, `.detail-row2` — the two flex rows inside `.detail-header`

## Svelte template changes

In `FeatureDetail.svelte`:

1. Replace `.detail-header-top` block (title + stat cards) with `.detail-row1` containing:
   - Title (same edit logic)
   - Right group: stat chips + separator + icon buttons
2. Replace `.detail-meta-row` with `.detail-row2` containing:
   - Status dropdown (existing)
   - Tag badges + add button (existing)
   - Separator dot
   - Timestamp
   - Separator dot (only if description exists)
   - Description (only if feature has description, truncated)
3. Remove the separate `<!-- Description -->` block below the header (description is now inline in row 2)

## Behaviour preserved

- Double-click title → edit mode (unchanged)
- Status dropdown (unchanged)
- Tag picker (unchanged)
- Copy ID / Copy Path / Edit / Delete actions (unchanged)
- `pendingPlanCount` chip appears alongside Done/Agents when > 0 (unchanged logic)
- Description edit: double-click on the description segment in row 2 triggers edit mode. A textarea block appears between the header and tab bar (same as current `.detail-description-edit` placement), replacing row 2 visually while editing. On blur or Escape, it collapses back.

## Out of scope

- Tab bar (unchanged)
- Content panels (unchanged)
- Any other component
