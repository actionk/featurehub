# Feature List Redesign — Unified Glass Cards (V3/I1)

**Status:** Approved
**Date:** 2026-04-23
**Depends on:** Existing Aurora Glass design system (`docs/superpowers/specs/2026-04-23-aurora-glass-design-system.md`)

## Goal

Replace the current sidebar feature list with a unified glass-card surface where each feature and its running agent sessions live inside one rounded container, separated by a thin dashed divider. Sessions render as darker insets with colored left rails indicating their state.

## Why

The current list shows features as flat rows with sessions appearing as indented sub-rows with tree connectors. This reads as two stacked groups of items rather than one coherent unit. The user finds the current visualization messy — too many small containers with borders, too much noise around inactive features, unclear hierarchy between a feature and its sessions.

The unified card treatment:
- Visually binds a feature to its sessions (they're inside it, not next to it)
- Scales down idle features (just a header) and scales up hot features (header + sub-list)
- Uses Aurora palette semantically: cyan = live, amber = waiting, violet = selected
- Removes tree connector complexity

## Design

### Surface

Each feature is rendered as a `.feature-unit` glass card:

- Background: `rgba(255,255,255,0.028)`
- Backdrop blur: `blur(8px)` with `-webkit-backdrop-filter` fallback
- Border: `1px solid rgba(255,255,255,0.055)`
- Border radius: `12px`
- Padding: `8px 10px 10px`
- Transition: `all 0.15s var(--ease)`

Cards stack with `6px` gap in a flex-column list. The existing aurora-bg radial gradient shows through.

### Parent header (`.unit-head`)

A single row inside the card containing:

- **Pinned glyph** (left, optional): 10px violet push-pin icon, only when `feature.pinned`
- **Title**: `13px`, `--text-secondary` when idle, `--text-primary` semibold when selected
- **Right-aligned chip slot** (optional): one pill — either task progress (`3/7`, violet) or agent count (`◉ N`, cyan). Agent count wins when the feature has running sessions. Waiting badge (`⚠`, amber) can appear alongside agent count.

The header is the click target for selecting the feature.

### Sub-list (sessions)

Rendered only when `terminalsByFeature.has(feature.id)`. Structure:

```
.feature-unit
  .unit-head                    ← parent header
  .sub-list                     ← margin-top: 8px, padding-top: 8px, border-top: 1px dashed rgba(255,255,255,0.06)
    .sub-session                ← per terminal
    .sub-session.sub-session--wait
    .sub-session.sub-session--viewing
```

Each `.sub-session`:

- Background: `rgba(0,0,0,0.28)`
- Padding: `6px 9px`
- Radius: `7px`
- Border-left: `2px solid var(--rail-color)`
- Gap between rows: `4px`
- Font-size: `11.5px`

Contents (left to right):

1. Status dot (6px, colored per state)
2. Label `agent-N — <status line>` — flex, ellipsis truncate
3. Runtime meta (`2m`, `12m`) in muted mono, right-aligned
4. Optional `waiting` amber pill (only when waiting)
5. Finish button (existing behavior, hover-only, unchanged)

### State matrix

Three session states, with a fourth "exited" for legacy:

| State | Rail color | Dot | Text | Background |
|---|---|---|---|---|
| Running | `rgba(34,211,238,0.55)` | cyan with glow | normal | `rgba(0,0,0,0.28)` |
| Waiting for input | `rgba(245,158,11,0.75)` | amber pulsing | `#fde68a` | `rgba(245,158,11,0.06)` |
| Viewing (user inside terminal) | `#c4b5fd` | violet with glow | white | `rgba(167,139,250,0.2)` + inner ring + outer glow |
| Exited | `rgba(255,255,255,0.1)` | muted | faded | `rgba(0,0,0,0.14)` |

When a sub-session is in the **viewing** state, the parent `.feature-unit` gains a `.feature-unit--has-viewing` modifier:

- Subtle violet outline: `inset 0 0 0 1px rgba(167,139,250,0.35)`
- Slight violet tint: `background: rgba(167,139,250,0.06)`

This signals "you are looking inside this feature" without mimicking the full selected state.

### Hover / selection

**Card hover** (idle, not selected): `box-shadow: inset 0 0 0 1px rgba(34,211,238,0.28)`.

**Card selected** (`.feature-unit--selected`):
- Background: `rgba(167,139,250,0.1)`
- Inner ring: `inset 0 0 0 1px rgba(167,139,250,0.45)`
- Outer glow: `0 0 22px rgba(167,139,250,0.14)`
- Title: `--text-primary` semibold

**Sub-row hover**: background darkens to `rgba(0,0,0,0.4)`.

### Group headers

Unchanged structure — uppercase caps `10px`, `--text-muted`, `0.08em letter-spacing`, no container.

### Click behavior

Existing logic preserved:

- Click `.unit-head` → `onSelect(feature.id)`
- If already selected AND one of its sessions is being viewed → `requestShowOverview()` (already implemented)
- Click `.sub-session` → `onSelectTerminal(feature.id, terminalId)`
- Ctrl/Cmd+click unit-head → open in new workspace tab (existing)

### Motion & accessibility

- Transitions: `120-150ms` with `var(--ease)`
- Amber pulse: existing `@keyframes pulse-amber` (2s loop)
- `@media (prefers-reduced-motion: reduce)`: skip pulse, skip outer glow transition, keep static color tokens

### Focus

Keyboard focus on card or sub-row: `box-shadow: 0 0 0 2px var(--accent), 0 0 12px var(--accent-dim)`. Uses the existing Aurora focus token.

## Files

### Modified

- `src/app.css`
  - Replace `.feature-item` / `.feature-item--*` / `.feature-item-compact` / `.feature-session-item` / `.feature-session-*` rule blocks with `.feature-unit` / `.unit-head` / `.sub-list` / `.sub-session` system
  - Remove tree-connector rules (`::before`/`::after` guides in feature-session-item)
  - Remove status-stripe rules on feature-item
  - Keep `@keyframes pulse-amber`, keep aurora tokens
- `src/lib/components/Sidebar.svelte`
  - Restructure feature row markup: parent `<div class="feature-unit">` wrapping `<button class="unit-head">` and optional `<div class="sub-list">`
  - Move current sub-row terminal markup inside `.feature-unit` as `.sub-session` elements
  - Drop existing tree depth padding on sub-rows (replaced by card padding)
  - Apply `.feature-unit--has-viewing` modifier when any terminal of this feature === `viewingTerminalId`
- `src/lib/stores/terminals.svelte.ts` — no change

### Unchanged

- `src/lib/stores/sessionActivity.svelte.ts` — `isAnySessionWaitingForFeature` still used for header badge
- Callback prop shape: `onSelect`, `onSelectTerminal`, `onFinishTerminal` all unchanged
- Context menu, drag-drop, group management — untouched

## Acceptance

- [ ] Feature with no sessions renders as a single-row glass card
- [ ] Feature with 1+ sessions renders card with dashed divider and sub-list below
- [ ] Sub-session shows correct rail color and dot per state (running/waiting/viewing)
- [ ] Parent card gets violet outline when one of its sessions is in viewing state
- [ ] Hover on card shows cyan inner ring
- [ ] Selected card shows violet inner ring + outer glow
- [ ] Click card selects feature; click sub-session opens that terminal
- [ ] Click selected feature card while viewing a terminal → returns to overview (existing behavior preserved)
- [ ] Drag-to-reorder features still works
- [ ] Context menu still works
- [ ] Group collapse/expand still works
- [ ] Pinned features show violet pin glyph
- [ ] `prefers-reduced-motion` disables pulse + glow transitions
- [ ] No console errors; no regressions in existing frontend tests

## Out of scope

- I4 collapse-on-unselect behavior (sessions always visible when they exist)
- Multi-session summary chips on parent (chip slot holds at most agent count + waiting badge)
- Sparkline / activity history
- Kanban-by-state reorganization
