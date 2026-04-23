# Aurora Glass — Full UI Redesign

**Date:** 2026-04-23
**Scope:** Visual redesign of the FeatureHub Tauri app (Svelte 5 frontend). Same feature set, same routes, same data model. Only colors, surfaces, motion, typography, and component visuals change.

## Goal

Replace the current flat dark theme with a cohesive **Aurora Glass** system: cool aurora gradient (indigo → pink → cyan) painted at the app shell level, frosted glass panels with backdrop blur, neon-accented borders/badges, and subtle motion. Modern, distinctive, info-dense, performant in Tauri.

## Non-goals

- No layout changes. Sidebar still on the left, tabs in the same order, modals open the same way.
- No new features, no removed features. All registered tabs (Agents, Links, Repositories, Tasks & Notes, Files, Timeline, Board, Extensions) keep their current behavior.
- No changes to Tauri commands, IPC, DB schema, or MCP tools.
- No accessibility regression — focus rings stay visible, keyboard nav stays.

## Approach

**Token-first.** All visual changes flow through CSS custom properties in `src/app.css`. Components reference tokens, never hardcoded colors. Existing CSS class hooks (`.feature-item-compact`, `.active-sessions-badge`, `.btn`, `.tab`, etc.) keep their selectors — only the rules change. This keeps the diff surgical: one big `app.css` rewrite + targeted class additions on existing markup where new primitives are needed.

**Primitive-driven.** Roughly 10 reusable classes cover ~95% of the surface. New panels/cards/buttons inherit the look automatically.

## Design Tokens

### Surfaces

```css
--bg-base: #050610;            /* outermost (rare) */
--bg-aurora: #07081a;           /* app shell base — gradient sits on top */
--bg-sidebar: rgba(8,9,18,0.35); /* over aurora, with backdrop-filter */
--bg-glass: rgba(20,22,40,0.42);   /* primary card surface */
--bg-glass-soft: rgba(15,16,32,0.5); /* nested/secondary surface */
--bg-input: rgba(20,22,40,0.5);
--bg-overlay: rgba(0,0,0,0.7);     /* modals scrim */
```

### Aurora gradient (single source)

Painted on `#app` once, never re-applied per panel:

```css
--aurora:
  radial-gradient(640px 460px at 0% 0%,   rgba(99,102,241,0.28), transparent 55%),
  radial-gradient(540px 380px at 100% 0%, rgba(236,72,153,0.16), transparent 55%),
  radial-gradient(720px 480px at 50% 130%, rgba(34,211,238,0.16), transparent 55%);
```

### Borders

```css
--border-subtle: rgba(255,255,255,0.06);
--border:        rgba(255,255,255,0.10);
--border-strong: rgba(196,181,253,0.35);
--border-focus:  rgba(196,181,253,0.6);
--border-glow-top: linear-gradient(90deg, transparent, rgba(196,181,253,0.5), transparent);
```

### Accent (indigo → violet → cyan)

```css
--accent: #a78bfa;             /* violet, primary accent */
--accent-hover: #c4b5fd;
--accent-deep: #6366f1;        /* indigo, used in gradients */
--accent-cool: #22d3ee;        /* cyan, secondary accent */
--accent-grad: linear-gradient(135deg, #6366f1, #a78bfa);
--accent-grad-cool: linear-gradient(90deg, #a78bfa, #22d3ee);
--accent-dim: rgba(167,139,250,0.18);
--accent-border: rgba(196,181,253,0.4);
--accent-glow: 0 0 14px rgba(167,139,250,0.35);
--accent-glow-strong: 0 0 22px rgba(167,139,250,0.55);
```

### Semantic colors (status / link types / tags)

Keep the same hue-per-meaning mapping as today, but each color gets a `dim`, `border`, and `glow` token:

| Token base | Hex | Used for |
|---|---|---|
| `--green` | `#22d3ee` (cyan-leaning) | done, active session pulse, success |
| `--amber` | `#fbbf24` | in_progress, awaiting input, warn |
| `--red` | `#f87171` | blocked, error, reject |
| `--violet` | `#a78bfa` | primary accent, plan pending |
| `--pink` | `#ec4899` | tags, secondary highlight |
| `--cyan` | `#67e8f9` | session live, info |
| `--blue` | `#52a9ff` | links, neutral info |

For each: `--{color}-dim` (~0.15 alpha bg), `--{color}-border` (~0.45 alpha), `--{color}-glow` (box-shadow).

### Typography

Unchanged: Space Grotesk UI, JetBrains Mono code. Adjust weights:

```css
--font-weight-normal: 400;
--font-weight-medium: 500;
--font-weight-semibold: 600;
--font-weight-bold: 700;
--letter-spacing-tight: -0.01em;  /* large titles */
--letter-spacing-wide:  0.12em;   /* uppercase labels */
```

### Radius

Unchanged tokens. Cards default to `--radius-md` (11px); pills `--radius-full`.

### Shadows

```css
--shadow-card: 0 8px 28px rgba(99,102,241,0.18);
--shadow-card-hover: 0 12px 36px rgba(99,102,241,0.28);
--shadow-modal: 0 30px 80px rgba(0,0,0,0.7);
--shadow-glow-violet: 0 0 22px rgba(167,139,250,0.45);
--shadow-glow-cyan:   0 0 18px rgba(34,211,238,0.45);
--shadow-glow-amber:  0 0 16px rgba(251,191,36,0.4);
```

### Motion

```css
--ease: cubic-bezier(0.16, 1, 0.3, 1);
--dur-fast: 120ms;
--dur-base: 180ms;
--dur-slow: 320ms;
--lift: translateY(-2px);  /* card hover lift */
```

Animations: `pulse-dot` (active session, 1.6s), `cursor-blink` (terminal, 1s), `aurora-drift` (optional, 14s — disabled by default to save GPU).

## Reusable Primitives

These ~10 classes are the foundation. Existing components either gain these classes or have their existing classes redefined to look like these.

### `.glass-panel`

The default surface for cards, panels, sessions, plan cards, knowledge entries, file rows, link cards, settings sections, modals, dropdowns, board columns.

```css
.glass-panel {
  background: var(--bg-glass);
  backdrop-filter: blur(22px) saturate(140%);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  position: relative;
  overflow: hidden;
}
.glass-panel::before {
  content: '';
  position: absolute; left: 0; top: 0; right: 0; height: 1px;
  background: var(--border-glow-top);
}
.glass-panel--hover { transition: transform var(--dur-base) var(--ease), border-color var(--dur-base) var(--ease), box-shadow var(--dur-base) var(--ease); }
.glass-panel--hover:hover { transform: var(--lift); border-color: var(--border-strong); box-shadow: var(--shadow-card-hover); }
.glass-panel--soft { background: var(--bg-glass-soft); backdrop-filter: blur(14px); border-radius: var(--radius-sm); }
```

### `.aurora-pill`

Status badges, tag badges, count chips, session badges, plan-status badges. Variants for each semantic color.

```css
.aurora-pill {
  display: inline-flex; align-items: center; gap: 5px;
  padding: 2px 9px; border-radius: var(--radius-full);
  font-size: var(--text-xs); font-weight: var(--font-weight-semibold);
  background: var(--accent-dim); color: var(--accent-hover);
  border: 1px solid var(--accent-border);
  box-shadow: var(--accent-glow);
}
.aurora-pill::before { content: ''; width: 5px; height: 5px; border-radius: 50%; background: currentColor; box-shadow: 0 0 8px currentColor; }
.aurora-pill--success { background: var(--green-dim); color: var(--cyan); border-color: var(--green-border); box-shadow: var(--shadow-glow-cyan); }
.aurora-pill--warn    { background: var(--amber-dim); color: var(--amber); border-color: var(--amber-border); box-shadow: var(--shadow-glow-amber); }
.aurora-pill--danger  { background: var(--red-dim); color: var(--red); border-color: var(--red-border); }
.aurora-pill--muted   { background: rgba(255,255,255,0.05); color: var(--text-muted); border-color: var(--border-subtle); box-shadow: none; }
.aurora-pill--no-dot::before { display: none; }
```

### `.btn` + variants

Used for every button (header actions, Approve/Reject, Resume, +New session, settings buttons, modal actions, file actions).

```css
.btn {
  background: rgba(255,255,255,0.05);
  border: 1px solid var(--border);
  color: var(--text-primary);
  padding: 6px 12px; border-radius: var(--radius-sm);
  font-size: var(--text-sm); cursor: pointer;
  transition: all var(--dur-base) var(--ease);
}
.btn:hover { background: var(--accent-dim); border-color: var(--border-strong); }
.btn--primary {
  background: var(--accent-grad); border-color: var(--border-strong);
  box-shadow: 0 4px 16px rgba(167,139,250,0.4), inset 0 1px 0 rgba(255,255,255,0.2);
}
.btn--primary:hover { box-shadow: var(--shadow-glow-violet); transform: translateY(-1px); }
.btn--ghost { background: transparent; border-color: transparent; }
.btn--danger:hover { background: var(--red-dim); border-color: var(--red-border); color: var(--red); }
.btn--sm { padding: 4px 9px; font-size: var(--text-xs); border-radius: 6px; }
.btn--icon { padding: 6px; aspect-ratio: 1; display: inline-flex; align-items: center; justify-content: center; }
```

### `.tab-bar` + `.tab`

Feature tabs, settings modal tabs, knowledge folder tabs, any tabs.

```css
.tab-bar { display: flex; gap: 2px; border-bottom: 1px solid var(--border-subtle); }
.tab {
  padding: 9px 14px; font-size: var(--text-sm); color: var(--text-secondary);
  cursor: pointer; position: relative; border-radius: 6px 6px 0 0;
  transition: color var(--dur-base) var(--ease);
}
.tab:hover { color: var(--accent-hover); }
.tab--active { color: var(--text-primary); }
.tab--active::after {
  content: ''; position: absolute; left: 8px; right: 8px; bottom: -1px;
  height: 2px; background: var(--accent-grad-cool); border-radius: 2px;
  box-shadow: 0 0 10px rgba(167,139,250,0.6);
}
.tab__badge {
  display: inline-block; margin-left: 5px;
  background: var(--accent-dim); color: var(--accent-hover);
  font-size: 9.5px; padding: 1px 5px; border-radius: var(--radius-full);
}
```

### `.list-row`

Sidebar feature rows, task list items, knowledge folder entries, file list rows, link rows, repo rows, MCP server rows, skill rows, settings list items.

```css
.list-row {
  padding: 7px 10px; border-radius: var(--radius-sm);
  font-size: var(--text-sm); cursor: pointer;
  display: flex; align-items: center; gap: 8px; position: relative;
  transition: background var(--dur-base) var(--ease), color var(--dur-base) var(--ease);
}
.list-row:hover { background: rgba(255,255,255,0.04); }
.list-row--active {
  background: var(--accent-dim); color: var(--text-primary);
  box-shadow: inset 0 0 0 1px var(--accent-border);
}
.list-row--active::before {
  content: ''; position: absolute; left: -10px; top: 20%; bottom: 20%;
  width: 3px; background: var(--accent-grad-cool); border-radius: 0 3px 3px 0;
  box-shadow: var(--accent-glow-strong);
}
```

### `.input` / `.input--search`

All text inputs, search bars, settings fields.

```css
.input {
  background: var(--bg-input); border: 1px solid var(--border);
  border-radius: var(--radius-sm); padding: 7px 10px;
  color: var(--text-primary); font-size: var(--text-sm);
  transition: border-color var(--dur-base) var(--ease), box-shadow var(--dur-base) var(--ease);
}
.input:focus { outline: none; border-color: var(--border-focus); box-shadow: 0 0 0 3px var(--accent-dim); }
.input--search { display: flex; align-items: center; gap: 6px; }
```

### `.check`

Task checkboxes (TaskList), settings toggles (compact form), file selection.

```css
.check { width: 14px; height: 14px; border: 1px solid var(--accent-border); border-radius: 4px; flex-shrink: 0; cursor: pointer; transition: all var(--dur-base) var(--ease); }
.check:hover { border-color: var(--accent-hover); box-shadow: var(--accent-glow); }
.check--done { background: var(--accent-grad-cool); border-color: transparent; box-shadow: var(--accent-glow); }
.check--done::after { content: '✓'; display: flex; align-items: center; justify-content: center; height: 100%; font-size: 10px; color: #fff; }
```

### `.live-dot`

Active session indicator, live agent indicator, anything pulsing. Already present as `.active-sessions-dot` — keep that selector, redefine.

```css
.live-dot { width: 6px; height: 6px; border-radius: 50%; background: var(--cyan); box-shadow: 0 0 8px var(--cyan); animation: pulse-dot 1.6s ease-in-out infinite; }
.live-dot--warn { background: var(--amber); box-shadow: 0 0 8px var(--amber); }
@keyframes pulse-dot { 0%, 100% { opacity: 1; transform: scale(1); } 50% { opacity: 0.55; transform: scale(0.85); } }
```

### `.aurora-bg`

Applied once on `#app`. Paints the gradient. No other surface re-paints aurora.

```css
.aurora-bg { background: var(--aurora), var(--bg-aurora); position: relative; isolation: isolate; }
```

### `.scrim`

Modal/overlay backdrop. Heavier blur on content behind.

```css
.scrim { position: fixed; inset: 0; background: var(--bg-overlay); backdrop-filter: blur(8px); z-index: 100; }
```

## Surface-by-Surface Application

For each existing surface, list which primitives it adopts. No layout changes.

### App shell (`#app`)
- Add `.aurora-bg` class. Aurora gradient paints once here.
- Existing `.shell-fx` ray/glow effects: remove (now redundant).

### Sidebar (`Sidebar.svelte`)
- Outer container: `background: var(--bg-sidebar); backdrop-filter: blur(18px);`
- Search box: `.input--search`
- Section headers (`.feature-section-title` or equivalent): apply uppercase + `letter-spacing: var(--letter-spacing-wide)` + `color: var(--text-muted)`.
- Each feature row (`.feature-item-compact`): becomes a `.list-row`. Active state uses `.list-row--active` (glowing left rail).
- Status dot (`.feature-item-status-dot`): keep selector; redefine to `.live-dot` style with semantic color from `statusColors[status]`.
- Mini progress bar (`.feature-item-pmb-track` / `.feature-item-pmb-fill`): track is `rgba(255,255,255,0.08)`; fill uses `var(--accent-grad-cool)` with subtle glow.
- Active sessions badge (`.active-sessions-badge`): becomes `.aurora-pill.aurora-pill--success` with `.live-dot` inside. Yellow variant (`.aurora-pill--warn`) when any session is `WaitingForInput`.
- Tree chevron: violet on hover.
- Drag handle: subtle, appears on hover.
- Feature group headers: `.glass-panel--soft` style, collapsible.

### WorkspaceTabBar (`WorkspaceTabBar.svelte`)
- Tabs use `.tab` styling. Active tab gets the glowing underline.
- Close button: `.btn--icon.btn--ghost`.
- "+" new tab button: `.btn--icon`.

### FeatureDetail (`FeatureDetail.svelte`)
- Header: title with `letter-spacing: var(--letter-spacing-tight)`. Status pill uses `.aurora-pill`.
- Action buttons: `.btn` / `.btn--primary`.
- Tab bar: `.tab-bar` with tab badges from `getBadges(ctx)`.
- Each tab's panel container: no glass (transparent over aurora). Cards within tabs use `.glass-panel`.

### Agents tab (`AiPanel.svelte` + children)
- `SessionList` wrapper: transparent column.
- `SessionCard`: `.glass-panel.glass-panel--hover`. Status indicator uses `.live-dot` (cyan/amber/muted). Model name in JetBrains Mono with `color: var(--accent-hover)`.
- `PlanCard`: `.glass-panel`. Status pill (`.aurora-pill--warn` pending, `.aurora-pill--success` approved, `.aurora-pill--danger` rejected). Approve = `.btn--primary`, Reject = `.btn`, Discuss = `.btn--ghost`.
- `PlanDetail`: same panel treatment, larger.
- `ContextEditor`: textarea uses `.input`, full-width inside a `.glass-panel`. Markdown preview pane uses MarkdownPreview (see below).
- `McpServersPanel`: each server is a `.list-row` with toggle (`.check` style or pill). Add server button: `.btn--primary.btn--sm`.
- `SkillsPanel`: each skill is a `.glass-panel--soft` row with toggle and edit button.
- `Terminal`: keep monospace, dark inner background `rgba(0,0,0,0.4)`, with subtle scanline `repeating-linear-gradient` at very low opacity. Cursor uses `cursor-blink` animation.

### Tasks & Notes (`TasksNotesPanel.svelte`)
- Two-column layout unchanged. Each column is a `.glass-panel`.
- `TaskList` rows use `.task` (gap + check + label). Checkboxes use `.check`. Done tasks fade with strikethrough.
- "Add task" inline input: `.input.btn--sm` style.
- `NotesEditor`: textarea uses `.input` (no border, fills panel). Toolbar buttons: `.btn--icon.btn--sm`.
- Markdown preview tab: see MarkdownPreview.

### Files (`FileBrowser.svelte` + children)
- Container: `.glass-panel`. Two-pane layout (folder tree + file list / preview).
- `FolderBreadcrumb`: chips with `.aurora-pill--muted` style; current folder violet.
- `FileList` rows: `.list-row` with file-type icon dot (color by ext: yellow=md, cyan=ts, pink=svelte, etc.). Selected row uses `.list-row--active`.
- `FilePreviewPanel`: code preview gets glassy frame around the highlighted code; image preview centered with subtle violet drop shadow; binary preview shows file metadata in a `.glass-panel--soft`.
- Drag-drop zone: dashed `--accent-border` border that brightens to `--border-focus` with `.aurora-bg` ghost overlay on dragenter.

### Links (`LinksGrid.svelte` / `LinkCard.svelte`)
- Grid layout unchanged.
- Each `LinkCard`: `.glass-panel.glass-panel--hover`. Type icon in colored circle (color = link type). Type label uses `.aurora-pill--muted`.
- Hover reveals edit/delete via `.btn--icon.btn--ghost`.
- "Add link" button: `.btn--primary`.

### Repositories (`RepositoriesPanel.svelte`)
- Each repo: `.glass-panel`. Clone status pill (`.aurora-pill--success` cloned, `.aurora-pill--warn` cloning, `.aurora-pill--danger` failed).
- Branch list: `.list-row`s. Active branch uses `.list-row--active`.
- Open in editor: `.btn--sm`.

### Timeline (`Timeline.svelte`, `GlobalTimeline.svelte`)
- Vertical timeline: events as `.glass-panel--soft` rows, timestamp in JetBrains Mono `color: var(--text-muted)`.
- Connector line uses `var(--accent-border)`.
- Event-type dot: colored `.live-dot` (no pulse) — color by event kind.

### Board (`BoardPanel.svelte` + children)
- Columns: `.glass-panel` (full-height). Column header uppercase `letter-spacing: var(--letter-spacing-wide)`.
- `BoardCard`: `.glass-panel.glass-panel--hover` (smaller padding). Drag-active state: brighter border + `.shadow-glow-violet`.
- Drop target column: `border-color: var(--border-focus); background: var(--accent-dim);`.

### Knowledge (`KnowledgePanel.svelte`, `KnowledgeFolderTree.svelte`, `KnowledgeEntryEditor.svelte`)
- Three-pane (folder tree | entry list | editor) all live in `.glass-panel`s.
- Folder tree rows: `.list-row` with chevron.
- Entry rows: `.list-row` with title + tag pills.
- Editor: `.input` textarea + MarkdownPreview side-by-side.

### Search (`SearchBar.svelte`)
- `.input--search` styled. Results dropdown: `.glass-panel` floating with `.shadow-modal`. Result rows: `.list-row` with entity-type pill + match snippet.

### Settings (`SettingsModal.svelte`)
- Modal scrim: `.scrim`.
- Modal container: large `.glass-panel` with `.shadow-modal`.
- Left nav (sections): vertical `.list-row`s.
- Right pane: section content in `.glass-panel--soft` blocks.
- All inputs/dropdowns/toggles use the primitives.

### Modals (`Modal.svelte`, `ConfirmDialog.svelte`, `CreateFeatureModal.svelte`, `ExportImportModal.svelte`)
- All adopt `.scrim` + `.glass-panel`.
- ConfirmDialog actions: `.btn` + `.btn--primary` (or `.btn--danger` for destructive).
- `Dropdown.svelte` panel: `.glass-panel` with `.shadow-modal`.
- `IconButton.svelte`: alias for `.btn--icon.btn--ghost`.

### Toasts (`ToastContainer.svelte`)
- Each toast: small `.glass-panel` (`backdrop-filter: blur(28px)` for stronger separation from background), 4px left rail in semantic color (`var(--cyan)`/`var(--amber)`/`var(--red)`).
- Slide-in from bottom-right with `var(--ease)`. Auto-dismiss fade.

### StatusBadge (`StatusBadge.svelte`)
- Becomes a thin wrapper around `.aurora-pill` with `--variant` selecting the modifier class based on status string.

### TagBadge (`TagBadge.svelte`)
- `.aurora-pill` with custom inline style: `background`, `color`, `border-color` derived from `tag.color` (existing user-picked color). Falls back to `--accent-dim` if no color.

### MarkdownPreview (`MarkdownPreview.svelte`)
- Body: serif-free, `var(--text-primary)`, line-height 1.65.
- Headings: `letter-spacing: var(--letter-spacing-tight)`, h1/h2 with subtle bottom border `var(--border-subtle)`.
- Code blocks: `.glass-panel--soft` with monospace, gradient left border `var(--accent-grad-cool)`. Inline code: violet tint.
- Blockquote: 3px left rail in `var(--accent-grad-cool)`.
- Links: `color: var(--accent-hover); text-decoration-color: var(--accent-border);` underline on hover.
- Mermaid diagrams: container `.glass-panel--soft` with subtle padding.
- Tables: row hover `rgba(255,255,255,0.03)`, header row uppercase + spaced.

### OpenFgaPreview (`OpenFgaPreview.svelte`)
- Same `.glass-panel--soft` container treatment with custom syntax tokens that pull from accent palette.

### DashboardPanel / SessionsPanel / InstalledExtensionsPanel
- Use `.glass-panel` cards. Stats use big numbers (`font-size: var(--text-2xl)`) over uppercase labels with wide letter-spacing.

### Storage selector / setup
- Same `.glass-panel` modal treatment. Storage cards: `.glass-panel.glass-panel--hover`. Active storage gets the glowing left rail (`.list-row--active::before` style).

### Scrollbars
- Width 5px, thumb `var(--accent-border)` on hover, `rgba(255,255,255,0.1)` default. Already configured — keep.

### Focus rings
- Replace current solid blue outline with violet glow: `outline: none; box-shadow: 0 0 0 2px var(--accent-hover), 0 0 0 4px var(--accent-dim);` on `:focus-visible`.

### Selection
- `::selection { background: rgba(167,139,250,0.35); color: var(--text-primary); }`

## Motion Catalog

| Where | What | Duration |
|---|---|---|
| Card hover | `translateY(-2px)` + border + shadow | 180ms ease |
| Button hover | bg + border tint | 120ms ease |
| Primary button hover | lift + glow strengthen | 180ms ease |
| Active session dot | pulse (opacity + scale) | 1.6s loop |
| Terminal cursor | blink | 1s step-end |
| Tab switch | underline slides via `.tab--active::after` re-render | 180ms |
| Tab content swap | fade-in 120ms (Svelte transition) | 120ms |
| Modal open | scrim fade + panel scale `0.96 → 1` | 180ms |
| Toast in/out | slide + fade from bottom-right | 220ms ease |
| Sidebar feature reorder | drag ghost glows; drop animates `transform` 200ms | 200ms |
| Plan approve | brief green flash on card border, then state change | 320ms |
| Notification arrival | sidebar feature row pulses border once | 600ms |

No global aurora-drift animation by default (toggleable via `--motion: rich` later if user wants).

## Accessibility

- All semantic colors meet 4.5:1 contrast on glass background. Primary text on `.glass-panel` is `--text-primary` (#e8ecf8) on rgba(20,22,40,0.42) over aurora — verified ≥7:1.
- Focus ring is always visible (violet glow, not removed for aesthetics).
- `prefers-reduced-motion: reduce` disables `.live-dot` pulse, card lift, modal scale; keeps fades.
- Keyboard nav unchanged. All interactive elements remain `<button>`/`<a>`/`<input>`.

## Performance

- Backdrop-filter is GPU-accelerated in Tauri's webview; tested OK on Windows/macOS.
- Aurora gradient is static; `radial-gradient` cost paid once per repaint. No animated mesh by default.
- Glass panels: limit nesting depth — don't apply `backdrop-filter` inside another `backdrop-filter` panel (degrades to no-op + cost). Use `.glass-panel--soft` for nested cards (no blur on nested level).
- Pulsing `.live-dot`: cheap (transform + opacity). Cap at ~10 simultaneous before degrading to static.

## Implementation strategy

1. **Tokens first.** Rewrite the `:root` block in `src/app.css` (~lines 15–116) with the new token set above. Keep token names backward-compatible where possible (`--accent`, `--bg-card`, `--green`, etc. still exist with new values).
2. **Add primitives.** Append a new section `/* ===== AURORA PRIMITIVES ===== */` defining `.glass-panel`, `.aurora-pill`, `.btn` variants, `.tab`, `.list-row`, `.input`, `.check`, `.live-dot`, `.aurora-bg`, `.scrim`. ~250 lines of CSS.
3. **Redefine existing classes.** Walk `src/app.css` and rewrite the bodies of existing class rules (e.g. `.feature-item-compact`, `.active-sessions-badge`, `.btn`, `.tab-button`, `.modal`, `.card`, etc.) to delegate to or match the primitive look. Selectors stay; rule bodies change.
4. **Markup additions.** In a small number of components, add the primitive class on the existing element so it picks up the new look directly:
   - `App.svelte`: add `.aurora-bg` to `#app` (or root container).
   - `Modal.svelte`: ensure scrim has `.scrim`, panel has `.glass-panel`.
   - `Dropdown.svelte`: add `.glass-panel` to floating panel.
   - `StatusBadge.svelte`, `TagBadge.svelte`: switch to `.aurora-pill` + variant.
5. **Per-tab passes.** For each module folder (ai/, files/, links/, repos/, board/, tasks-notes/, timeline/, knowledge/), audit cards/buttons/inputs to add primitive classes where current ad-hoc classes don't match. Most should pick up via class redefinition in step 3.
6. **Visual QA.** Run `npm run tauri dev`. Walk every tab, every modal, every state (empty/loading/error/active/disabled). Fix gaps.
7. **Tests.** Existing Vitest snapshots may need updating (run `npm run test` and review). No logic changes expected.

Estimated scope: ~600 lines of CSS rewrites in `src/app.css` + small markup additions in ~10 components.

## What stays exactly the same

- All Tauri commands.
- All Svelte component boundaries and props.
- All routes / tab order / shortcut keys.
- All MCP tool definitions.
- Settings schema, DB schema, file storage layout.
- All keyboard shortcuts.
- Notification polling + behavior.
- Knowledge base ingest/search.

## Open decisions deferred to implementation

- Exact gradient stops fine-tuning per real screen sizes (mockup uses fixed pixel positions; production uses responsive units).
- Whether Mermaid diagrams need a custom dark theme override or default works on glass.
- Toast position: keep current (bottom-right) unless user requests change.
