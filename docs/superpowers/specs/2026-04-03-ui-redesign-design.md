# FeatureHub UI Redesign — Global Spec

**Date:** 2026-04-03  
**Status:** Approved  
**Approach:** CSS-first, tokens down. Six phases, each independently shippable. One global spec; per-phase implementation plans derived from it.

---

## Vision

FeatureHub is a developer tool for managing features and AI agent sessions. The redesign targets the aesthetic and UX of a high-quality developer tool — not a SaaS dashboard, not a consumer product.

Design language: **dark, dense, precise.** Gradients and glow used for hierarchy and live-state signaling, not decoration. Every element earns its place.

### Core Principles

- **Density over whitespace** — developers want to see more at once; generous padding wastes screen real estate
- **Hierarchy through light** — brighter = more important, dimmer = context/metadata
- **Gradient as signal** — gradients mark active, live, or high-attention states (active agent sessions, pending plans, key metrics). Not used on static body text.
- **Mono for data** — IDs, timestamps, branch names, token counts, durations always use `--font-mono`
- **Consistent interactive feedback** — all interactive surfaces: hover lifts (`translateY(-2px)`), focus rings, smooth transitions
- **Empty states** — every list/grid that can be empty has a styled empty state with icon, title, and hint

### Visual Reference

`docs/design-prototype.html` is the primary visual reference. It is a direction, not a pixel-perfect spec — the real implementation will refine details as live data and interactions are encountered.

---

## Design System Tokens

The single source of truth is `src/app.css`. All tokens are CSS custom properties on `:root`. No component defines its own colors — everything references tokens.

### Typography

- **UI font:** Space Grotesk (Google Fonts), weights 400/500/600/700 (max 700 — Space Grotesk has no 800)
- **Mono font:** keep existing stack (`'JetBrains Mono', 'Fira Code', monospace`)
- **Base size:** 13px (dense developer tool default)
- **Letter spacing:** headings use `-0.03em`; body uses `0` or `-0.01em`
- **Line height:** body `1.5`; dense lists `1.3`

### Color Palette

Dark theme. All values are CSS custom properties.

**Token naming strategy:** Phase 1 updates the *values* of existing tokens in place — it does not rename them. The existing names (`--bg-base`, `--text-primary`, `--border-subtle`, etc.) are used throughout 4,919 lines of `app.css` and all component styles. Renaming would require a global find/replace across the entire codebase with high regression risk. New tokens added in Phase 1 (e.g. gradient utilities, glow shadows) use new names.

**Surfaces (darkest → lightest):**
| Existing token | New value | Use |
|---|---|---|
| `--bg-base` | `#08090e` | App background, outermost layer |
| `--bg-primary` | `#0d0f18` | Primary surface (sidebar, panels) |
| `--bg-card` | `#12141f` | Cards, raised elements |
| `--bg-hover` | `#181a28` | Hover states, active backgrounds |
| `--border-subtle` | `rgba(255,255,255,0.07)` | Default border |
| `--border` | `rgba(255,255,255,0.13)` | Emphasized borders |

**Text:**
| Existing token | New value | Use |
|---|---|---|
| `--text-primary` | `#e8ecf8` | Primary text (headings, labels) |
| `--text-secondary` | `#b8c0d8` | Secondary text (descriptions) |
| `--text-muted` | `#5a6480` | Muted text (metadata, timestamps) |

**Accent (primary blue):**
| Token | Value |
|---|---|
| `--accent` | `#4d7cff` |
| `--accent-dim` | `rgba(77,124,255,0.15)` |
| `--accent-glow` *(new)* | `rgba(77,124,255,0.25)` |

**Semantic colors:**
| Token | Value | Dim variant |
|---|---|---|
| `--green` | `#34d399` | `rgba(52,211,153,0.15)` |
| `--amber` | `#fbbf24` | `rgba(251,191,36,0.15)` |
| `--red` | `#f87171` | `rgba(248,113,113,0.15)` |
| `--violet` | `#a78bfa` | `rgba(167,139,250,0.15)` |
| `--cyan` | `#22d3ee` | `rgba(34,211,238,0.15)` |

**Gradients (named, reusable):**
| Token | Value | Use |
|---|---|---|
| `--grad-primary` | `linear-gradient(135deg, #4d7cff, #7b6fff)` | Primary action, key headings |
| `--grad-success` | `linear-gradient(135deg, #34d399, #22d3ee)` | Positive metrics |
| `--grad-warn` | `linear-gradient(135deg, #fbbf24, #f97316)` | Pending/warning states |
| `--grad-cool` | `linear-gradient(135deg, #22d3ee, #4d7cff)` | Secondary headings, links |

**Gradient text utility classes** (applied to `<span>` elements, never block elements):
- `.gt.gt-p` — primary gradient text
- `.gt.gt-s` — success gradient text
- `.gt.gt-w` — warn gradient text
- `.gt.gt-c` — cool gradient text

Rules: `display: inline-block; background-image: <gradient>; -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text;`

### Spacing Scale

`--sp-1: 4px` through `--sp-12: 48px` (multiples of 4px). Used consistently — no hardcoded px values in component styles.

### Border Radius

- `--r-sm: 4px` — inputs, small chips
- `--r-md: 6px` — buttons, badges
- `--r-lg: 10px` — cards
- `--r-xl: 14px` — modals, panels

### Shadows

- `--shadow-sm` — subtle lift (card default)
- `--shadow-md` — card hover state
- `--shadow-lg` — floating panels, dropdowns
- `--shadow-glow` — `0 0 20px rgba(77,124,255,0.12)` — active/accent elements

### Card Hover System

Consistent across all card types via CSS variables:
```css
--card-hover-y: -2px;
--card-hover-shadow: var(--shadow-md);
```
Every `.card:hover` applies `transform: translateY(var(--card-hover-y))` + `box-shadow: var(--card-hover-shadow)`.

### Transitions

- `--t-fast: 120ms ease` — hover color changes
- `--t-base: 200ms ease` — transforms, opacity
- `--t-slow: 320ms ease` — panel slides, modals

---

## Phase 1 — Design System

**Scope:** `src/app.css` tokens and base element styles only. No Svelte component HTML changes.

**Deliverables:**
1. Replace `:root` token block with new palette, typography, spacing, radius, shadow tokens
2. Add Space Grotesk `@import` from Google Fonts
3. Add gradient utility classes (`.gt`, `.gt-p`, `.gt-s`, `.gt-w`, `.gt-c`)
4. Update base element styles: `body`, `button`, `input`, `textarea`, `select`
5. Update `.btn` variants to use new tokens
6. Update `.badge` / `.status-badge` / `.tag-badge` to use new tokens
7. Update `.card` base styles (hover system, shadows, radius)
8. Update `scrollbar` styles to match dark theme
9. Remove any hardcoded hex values outside `:root`

**Success criteria:** App loads, looks visibly different (darker, better typography), no regressions in component layout.

---

## Phase 2 — App Shell

**Scope:** Sidebar, workspace tab bar, global chrome. Svelte files: `App.svelte`, `Sidebar.svelte`, `WorkspaceTabBar.svelte`, `ToastContainer.svelte`, `StorageSelector.svelte`, `SearchBar.svelte`.

**Sidebar changes:**
- Narrow icon-only left rail (48px) with tooltip system (`data-tip` + CSS `::after` pseudo-elements, no JS)
- Each nav icon shows label tooltip on hover + keyboard shortcut
- Feature list items: tighter cards with status dot, title, task progress pip, tag dots
- Sprint/group section headers with collapse toggle
- "New Feature" button pinned to sidebar footer
- Active feature highlighted with accent-dim background + left border accent

**Workspace tab bar:**
- Tabs for open features + Board tab
- Active tab: gradient text label
- Close button appears on hover
- Drag-to-reorder (already implemented via mouse events — keep behavior, update styles)

**Search (command palette):**
- Full-screen overlay with blur backdrop
- Monospace placeholder: `Search features, tasks, links...`
- Results grouped by entity type with type badges
- Keyboard navigation (↑↓ to move, Enter to open, Esc to close)

**Toast notifications:**
- Bottom-right, slide-up animation
- Types: success (green), warning (amber), error (red), info (blue)
- Auto-dismiss with progress bar
- Undo action support (already exists — keep, restyle)

---

## Phase 3 — Feature Detail Header

**Scope:** Feature header area. Svelte file: `FeatureDetail.svelte` (header section) and inline styles.

**Header layout:**
- Feature title: large, gradient text always (not just on hover — it's the primary heading)
- Ticket ID: monospace, dimmed, hover reveals copy hint `⎘`
- Status badge: colored pill matching status color
- Branch name (if available): mono, small, with git icon
- Tag chips: colored, compact
- Description: secondary text, capped at 2 lines with expand option
- Stat chips row: Tasks done/total, Agents (active count), Files — each with gradient number and icon

**Action buttons:**
- "▶ Start Agent" — primary accent button, top-right
- "..." overflow menu for archive, export, etc.

---

## Phase 4 — Bento Grid & Tab Bar

**Scope:** The main content area when a feature is selected. CSS bento layout, tab bar, and all bento card types.

**Tab bar:**
- Compact tabs, active tab has gradient-colored text
- Each tab shows keyboard shortcut hint on hover (`.sk` span, opacity 0→1)
- Badge: pending plan count (amber), active agent count (green pulse dot)

**Bento grid layout:**
- 3-column CSS grid, `grid-template-rows: auto 1fr auto`
- `.content` is a flex column; `.bento` takes `flex: 1`
- Cards use `align-items: stretch` to fill row height

**Bento card types:**

*Tasks card* (`grid-row: span 2`, col 1):
- Card title with gradient text
- Progress bar (no shimmer on static bars — shimmer only on indeterminate)
- Task list with checkbox, done state (strikethrough + dim), hover highlight
- Empty state: icon + "No tasks yet" + hint

*Active Agent session card* (col 2, row 1):
- Dark glass card with gradient border (dual `background-image` layer trick)
- Dot-grid texture overlay (SVG, low opacity)
- "ACTIVE NOW" live pill with pulsing ring
- Session title (plain white, not gradient — better contrast on dark blue)
- Session description body
- Timer (mono font, clock icon)

*Pending Plan card* (col 3, row 1):
- "Pending Plan" title with warn gradient
- Plan title and step list
- Approve / Reject buttons

*Agent History card* (col 2, row 2):
- Session list: dot indicator (green=active, dim=past), title, meta (date + duration mono)
- Empty state

*Links card* (col 3, row 2):
- Link rows: favicon/type icon, name, URL (truncated), `↗` appears on hover
- "+ Add" button in header
- Empty state

*Insights card* (col 1–3, row 3 — full width):
- Three sections separated by vertical dividers:
  1. **Key stats** — Agent sessions (real, from DB), Tasks done/total (real), Time invested (placeholder — backend doesn't track this yet), Total tokens (placeholder)
  2. **Sparkline** — SVG area chart of agent sessions per day over N days (real data from sessions table), peak dot highlighted, x-axis date labels
  3. **Token breakdown** — Input / Output / Cached bars (placeholder — static UI until token tracking is added to backend)
- Time range toggle: 7d / 14d / 30d (drives sparkline range; token breakdown stays static)
- **Note:** Token usage and time-invested data requires future backend work (out of scope for this redesign). Phase 4 renders the Insights card with real data where available and clearly placeholder where not.

---

## Phase 5 — Content Panels

**Scope:** Each tab module's internal layout and styles. Svelte files in `src/lib/modules/*/`.

### Agents Tab (`src/lib/modules/ai/`)

- `AiPanel.svelte`: Layout with session list on left, context/plan/terminal on right
- `SessionCard.svelte`: Consistent with bento session card style
- `PlanCard.svelte`: Matches Pending Plan bento card
- `ContextEditor.svelte`: Textarea with mono font, line numbers optional
- `SkillsPanel.svelte`: Skill cards with description, toggle
- `McpServersPanel.svelte`: Server rows with enable/disable toggle, status dot
- `Terminal.svelte`: Dark terminal, mono font, minimal chrome

### Tasks & Notes Tab (`src/lib/modules/tasks-notes/`)

- `TaskList.svelte`: Matches bento task card style (checkbox, done state, add button)
- `NotesEditor.svelte`: Full-height markdown editor; preview toggle; mono input font

### Links Tab (`src/lib/modules/links/`)

- `LinksGrid.svelte`: Grid of link cards, "+ Add" prominent
- `LinkCard.svelte`: Icon, name, URL, type badge, edit/delete on hover

### Files Tab (`src/lib/modules/files/`)

- `FileBrowser.svelte`: Sidebar tree + main list/preview layout
- `FileList.svelte`: Rows with icon, name, size, date (mono), hover actions
- `FilePreviewPanel.svelte`: Syntax-highlighted preview, image preview, binary fallback
- `FolderBreadcrumb.svelte`: Compact path with `/` separators

### Repos Tab (`src/lib/modules/repos/`)

- `RepositoriesPanel.svelte`: Repo cards with branch, clone status, open-in-IDE action

### Timeline Tab (`src/lib/modules/timeline/`)

- `Timeline.svelte`: Vertical timeline, event dots, mono timestamps, grouped by day

### Knowledge Tab (`src/lib/modules/knowledge/`)

- Sidebar panel (not tab-scoped): folder tree + entry editor
- Entry editor: title input, description, full markdown content area

---

## Phase 6 — Modals & Overlays

**Scope:** All modal and overlay components.

**Shared modal chrome:**
- Dark surface with subtle border
- Backdrop: `rgba(0,0,0,0.6)` with `backdrop-filter: blur(4px)`
- Slide-up + fade-in entry animation
- Close button top-right

**Create Feature modal:**
- Title input (large, prominent)
- Status selector (colored pills)
- Ticket ID, description, tags
- Template/group selector
- Submit button (gradient primary)

**Settings modal:**
- Tab-based sections: General, Fonts, MCP Servers, Extensions, Storage
- Font preview renders live as you change selection
- Token inputs use mono font

**Search / command palette:**
- Full overlay, centered, max-width 600px
- Large input at top
- Results list with type icons, keyboard navigation
- Footer shows available key shortcuts

**Confirm dialog:**
- Compact, centered
- Destructive actions: red confirm button
- Non-destructive: accent confirm button

**Dropdown:**
- `backdrop-filter: blur(8px)` + dark surface
- Items with hover highlight, keyboard navigation
- Separator support

---

## Implementation Order

| Phase | Primary files | Depends on |
|---|---|---|
| 1 — Design System | `src/app.css` (tokens + base) | — |
| 2 — App Shell | `App.svelte`, `Sidebar.svelte`, `WorkspaceTabBar.svelte` | Phase 1 |
| 3 — Feature Header | `FeatureDetail.svelte` (header) | Phase 1 |
| 4 — Bento Grid | `FeatureDetail.svelte` (content), `app.css` (bento) | Phase 1, 3 |
| 5 — Content Panels | `src/lib/modules/*/` | Phase 1, 4 |
| 6 — Modals | `src/lib/components/ui/`, modals | Phase 1 |

Phases 2, 3, and 6 can run in parallel after Phase 1. Phase 4 needs Phase 3 done first. Phase 5 needs Phase 4.

---

## Out of Scope

- New backend features or Rust changes
- New tab modules not already planned
- Responsiveness / mobile layout (Tauri desktop app, fixed window)
- Light theme (dark only)
- Accessibility beyond `focus-visible` keyboard navigation

---

## Testing Criteria (per phase)

- App builds without errors (`npm run tauri dev`)
- No visual regressions in untouched areas
- All interactive states work: hover, focus, active, disabled
- Empty states render correctly
- Existing Vitest frontend tests pass (`npm run test`)
