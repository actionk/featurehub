# Phase 5 — Content Panels Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Polish the scoped component styles and global CSS for all content panel modules — replacing old tokens, hardcoded values, and inline styles with Phase 1 design tokens.

**Architecture:** Four types of change: (1) scoped `<style>` blocks in Svelte components updated in-place — PlanCard, SessionCard, Timeline; (2) `TasksNotesPanel.svelte` inline styles replaced with CSS classes added to `app.css`; (3) `app.css` link section — fix old accent rgba + font-size tokens; (4) no new Svelte HTML structure changes. All changes are backward-compatible CSS-only (or CSS + wrapper div classes).

**Tech Stack:** CSS custom properties (Phase 1 tokens), Svelte 5, `npm run test` for regression check.

**Spec reference:** `docs/superpowers/specs/2026-04-03-ui-redesign-design.md` — Phase 5 section.

---

## File Map

| File | Change |
|---|---|
| `src/lib/modules/ai/PlanCard.svelte:41-88` | Fix `--bg-secondary` → `--bg-card`, `12.5px` → `--text-sm`, raw transition → token, `6px` → `--radius-md` |
| `src/lib/modules/ai/SessionCard.svelte:211-468` | Fix `6px` → `--radius-md`, raw `0.1s` → `--transition-fast`, confirm overlay radius |
| `src/lib/modules/timeline/Timeline.svelte:116-206` | Fix hardcoded `'JetBrains Mono'` → `--font-mono`, `16px` → `--space-4`, `8px` gaps → `--space-2` |
| `src/lib/modules/tasks-notes/TasksNotesPanel.svelte` | Replace inline styles with `.tn-tasks` / `.tn-notes` CSS classes |
| `src/app.css` (after `.tn-notes` section) | Add `.tn-tasks` / `.tn-notes` CSS |
| `src/app.css` (link section ~3313) | Fix `rgba(91,91,214,0.1)` → `var(--accent-dim)` in focus-within |

---

## Task 1: PlanCard scoped CSS — fix old tokens and hardcoded values

**Files:**
- Modify: `src/lib/modules/ai/PlanCard.svelte:41-88`

- [ ] **Step 1: Read the current style block**

Read `D:\LittleBrushGames\FeatureHub\src\lib\modules\ai\PlanCard.svelte` lines 40–90.

- [ ] **Step 2: Replace the entire `<style>` block**

The current style block (lines 41–88) uses `var(--bg-secondary)` (old token — replaced by `var(--bg-card)` in Phase 1), hardcoded `border-radius: 6px`, `12.5px` font-size, and raw `0.15s` transition values. Replace the ENTIRE `<style>` block with:

```css
<style>
  .plan-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-card);
    cursor: pointer;
    text-align: left;
    width: 100%;
    transition: border-color var(--transition-fast), background var(--transition-fast);
  }
  .plan-card:hover {
    border-color: var(--border-strong);
    background: var(--bg-hover);
  }
  .plan-card--selected {
    border-color: var(--accent);
  }
  .plan-card--pending {
    border-left: 3px solid var(--amber);
  }
  .plan-card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
  }
  .plan-card-status {
    font-size: 10px;
    font-weight: 600;
    padding: 1px 6px;
    border-radius: var(--radius-sm);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }
  .plan-card-time {
    font-size: 10px;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }
  .plan-card-title {
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--text-primary);
    line-height: 1.4;
  }
</style>
```

Changes made:
- `var(--bg-secondary)` → `var(--bg-card)` (old token fix)
- `border-radius: 6px` → `var(--radius-md)`
- `font-size: 12.5px` → `var(--text-sm)` (12px)
- `transition: border-color 0.15s, background 0.15s` → tokens
- `border-radius: 4px` → `var(--radius-sm)`
- `gap: 8px` → `var(--space-2)`
- `padding: 8px 10px` → `var(--space-2) var(--space-3)` (8px 12px)
- `plan-card-time` now uses `var(--font-mono)`
- Removed `var(--border-hover, var(--accent))` fallback hack → `var(--border-strong)` on hover

- [ ] **Step 3: Run tests**

```bash
cd D:\LittleBrushGames\FeatureHub && npm run test
```

Expected: all 34 tests pass.

- [ ] **Step 4: Commit**

```bash
cd D:\LittleBrushGames\FeatureHub && git add src/lib/modules/ai/PlanCard.svelte && git commit -m "style: PlanCard uses design tokens (bg-card, radius, text-sm, font-mono)"
```

---

## Task 2: SessionCard scoped CSS — token alignment

**Files:**
- Modify: `src/lib/modules/ai/SessionCard.svelte:211-468`

- [ ] **Step 1: Read the current style block**

Read `D:\LittleBrushGames\FeatureHub\src\lib\modules\ai\SessionCard.svelte` lines 210–470.

- [ ] **Step 2: Replace the entire `<style>` block**

Replace the ENTIRE `<style>` block (lines 211–468) with the following. Changes are: `border-radius: 6px` → `var(--radius-md)`, raw `0.1s` transitions → `var(--transition-fast)`, `gap: 8px` → `var(--space-2)`, `font-size: 12px` → `var(--text-sm)`, confirm overlay radius `5px` → `var(--radius-md)`, confirm button radius `4px` → `var(--radius-sm)`:

```css
<style>
  .sc {
    position: relative;
    display: flex;
    align-items: center;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-card);
    overflow: hidden;
    transition: border-color var(--transition-fast);
  }
  .sc:hover {
    border-color: var(--accent-border);
  }
  .sc--active {
    border-color: color-mix(in srgb, var(--green) 30%, var(--border));
    background: color-mix(in srgb, var(--green) 5%, var(--bg-card));
  }
  .sc--active:hover {
    border-color: color-mix(in srgb, var(--green) 45%, var(--border));
  }
  .sc--active .sc__main:hover {
    background: color-mix(in srgb, var(--green) 8%, var(--bg-card));
  }

  .sc__main {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 6px 10px;
    border: none;
    background: transparent;
    color: inherit;
    font-family: inherit;
    text-align: left;
    cursor: pointer;
    transition: background var(--transition-fast);
  }
  .sc__main:hover {
    background: var(--bg-hover);
  }
  .sc__main--static {
    cursor: default;
  }
  .sc__main--static:hover {
    background: transparent;
  }
  .sc__main--disabled {
    cursor: default;
  }
  .sc__main--disabled:hover {
    background: transparent;
  }

  .sc__dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    flex-shrink: 0;
    background: var(--text-muted);
    opacity: 0.4;
  }
  .sc__dot--live {
    background: var(--green);
    opacity: 1;
    box-shadow: 0 0 5px var(--green);
  }

  .sc__title {
    font-size: var(--text-sm);
    font-weight: 550;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
    flex-shrink: 1;
  }

  .sc__edit {
    font-size: var(--text-sm);
    font-weight: 550;
    color: var(--text-primary);
    background: var(--bg-input);
    border: 1px solid var(--accent);
    border-radius: var(--radius-sm);
    padding: 1px 5px;
    flex: 1;
    min-width: 0;
    font-family: inherit;
    outline: none;
  }

  .sc__meta {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-shrink: 0;
    font-size: 10.5px;
    color: var(--text-muted);
    font-family: var(--font-mono);
    margin-left: auto;
  }

  .sc__action-icon {
    flex-shrink: 0;
    color: var(--text-muted);
    opacity: 0;
    transition: opacity var(--transition-fast);
    display: flex;
    align-items: center;
  }
  .sc:hover .sc__action-icon {
    opacity: 0.6;
  }
  .sc__main:hover .sc__action-icon {
    opacity: 1;
    color: var(--accent);
  }

  .sc__external-badge {
    font-size: 9.5px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--amber);
    flex-shrink: 0;
    white-space: nowrap;
  }

  .sc__badge {
    font-size: 10px;
    font-weight: 550;
    white-space: nowrap;
    flex-shrink: 0;
  }
  .sc__badge--amber { color: var(--amber); }
  .sc__badge--red { color: var(--red); }

  .sc__copy {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    align-self: stretch;
    flex-shrink: 0;
    border: none;
    border-left: 1px solid var(--border);
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    opacity: 0;
    transition: opacity var(--transition-fast), background var(--transition-fast), color var(--transition-fast);
  }
  .sc:hover .sc__copy {
    opacity: 1;
  }
  .sc__copy:hover {
    background: var(--bg-hover);
    color: var(--accent);
  }

  .sc__finish {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    align-self: stretch;
    flex-shrink: 0;
    border: none;
    border-left: 1px solid var(--border);
    background: transparent;
    color: var(--green);
    cursor: pointer;
    opacity: 0;
    transition: opacity var(--transition-fast), background var(--transition-fast), color var(--transition-fast);
  }
  .sc:hover .sc__finish {
    opacity: 1;
  }
  .sc__finish:hover {
    background: color-mix(in srgb, var(--green) 15%, transparent);
    color: var(--green);
  }

  .sc__unlink {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    align-self: stretch;
    flex-shrink: 0;
    border: none;
    border-left: 1px solid var(--border);
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    opacity: 0;
    transition: opacity var(--transition-fast), background var(--transition-fast), color var(--transition-fast);
  }
  .sc:hover .sc__unlink {
    opacity: 1;
  }
  .sc__unlink:hover {
    background: var(--red-dim);
    color: var(--red);
  }

  .sc__confirm-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    background: color-mix(in srgb, var(--bg-card) 95%, transparent);
    border-radius: var(--radius-md);
    z-index: 1;
  }

  .sc__confirm-text {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .sc__confirm-yes,
  .sc__confirm-no {
    font-size: 10.5px;
    font-weight: 600;
    font-family: inherit;
    padding: 2px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .sc__confirm-yes {
    background: color-mix(in srgb, var(--red) 15%, transparent);
    color: var(--red);
    border-color: color-mix(in srgb, var(--red) 30%, transparent);
  }
  .sc__confirm-yes:hover {
    background: color-mix(in srgb, var(--red) 25%, transparent);
  }

  .sc__confirm-no {
    background: transparent;
    color: var(--text-muted);
  }
  .sc__confirm-no:hover {
    background: var(--bg-hover);
    color: var(--text-secondary);
  }
</style>
```

- [ ] **Step 3: Run tests**

```bash
cd D:\LittleBrushGames\FeatureHub && npm run test
```

Expected: all 34 tests pass.

- [ ] **Step 4: Commit**

```bash
cd D:\LittleBrushGames\FeatureHub && git add src/lib/modules/ai/SessionCard.svelte && git commit -m "style: SessionCard uses design tokens (radius, transition, font-mono)"
```

---

## Task 3: Timeline scoped CSS — design token alignment

**Files:**
- Modify: `src/lib/modules/timeline/Timeline.svelte:116-206`

- [ ] **Step 1: Read the current style block**

Read `D:\LittleBrushGames\FeatureHub\src\lib\modules\timeline\Timeline.svelte` lines 115–210.

- [ ] **Step 2: Replace the entire `<style>` block**

Changes: hardcoded `'JetBrains Mono', monospace` → `var(--font-mono)`, `margin-top: 16px` → `var(--space-4)`, `margin-bottom: 4px` → `var(--space-1)`, `gap: 8px` → `var(--space-2)`, `font-size: 12px` → `var(--text-sm)`, `font-size: 11px` label stays as-is (between xs/sm).

```css
<style>
  .tl {
    padding: 0;
  }

  .tl-day {
    margin-top: var(--space-4);
  }

  .tl-day--first {
    margin-top: 0;
  }

  .tl-day-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-1);
  }

  .tl-day-label {
    font-size: 11px;
    font-weight: 700;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    white-space: nowrap;
  }

  .tl-day-line {
    flex: 1;
    height: 1px;
    background: var(--border);
  }

  .tl-day-count {
    font-size: 10px;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  .tl-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 3px 0;
    min-height: 24px;
  }

  .tl-time {
    font-size: 10px;
    color: var(--text-muted);
    font-family: var(--font-mono);
    width: 42px;
    flex-shrink: 0;
    text-align: right;
  }

  .tl-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .tl-verb {
    font-size: 11px;
    font-weight: 600;
    flex-shrink: 0;
    min-width: 52px;
  }

  .tl-title {
    font-size: var(--text-sm);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }

  .tl-detail {
    font-size: 10.5px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-shrink: 1;
    min-width: 0;
  }
</style>
```

- [ ] **Step 3: Run tests**

```bash
cd D:\LittleBrushGames\FeatureHub && npm run test
```

Expected: all 34 tests pass.

- [ ] **Step 4: Commit**

```bash
cd D:\LittleBrushGames\FeatureHub && git add src/lib/modules/timeline/Timeline.svelte && git commit -m "style: Timeline uses design tokens (font-mono, spacing tokens, text-sm)"
```

---

## Task 4: TasksNotesPanel — replace inline styles with CSS classes

**Files:**
- Modify: `src/lib/modules/tasks-notes/TasksNotesPanel.svelte`
- Modify: `src/app.css` (add new CSS after the Tasks section)

The panel currently uses raw inline styles. Replace them with named CSS classes.

- [ ] **Step 1: Read TasksNotesPanel.svelte**

Read `D:\LittleBrushGames\FeatureHub\src\lib\modules\tasks-notes\TasksNotesPanel.svelte`.

- [ ] **Step 2: Add CSS to `src/app.css`**

Read `src/app.css` and find `/* ===== TASKS =====` (around line 2039). Add the following CSS block BEFORE that tasks section:

```css
/* ===== TASKS & NOTES PANEL ===== */

.tn-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.tn-tasks {
  flex-shrink: 0;
  padding-bottom: var(--space-3);
  border-bottom: 1px solid var(--border);
}

.tn-notes {
  flex: 1;
  min-height: 0;
  display: flex;
  padding-top: var(--space-3);
}
```

- [ ] **Step 3: Update TasksNotesPanel.svelte template**

The current template is:
```svelte
<div bind:this={panelEl}>
  <div style="flex-shrink: 0; padding-bottom: 12px; border-bottom: 1px solid var(--border); margin-bottom: 0;">
    <TaskList {featureId} {tasks} {onTasksChanged} />
  </div>
  <div style="flex: 1; min-height: 0; display: flex; padding-top: 12px;">
    <NotesEditor {featureId} {note} {onNoteChanged} />
  </div>
</div>
```

Replace with:
```svelte
<div class="tn-panel" bind:this={panelEl}>
  <div class="tn-tasks">
    <TaskList {featureId} {tasks} {onTasksChanged} />
  </div>
  <div class="tn-notes">
    <NotesEditor {featureId} {note} {onNoteChanged} />
  </div>
</div>
```

- [ ] **Step 4: Run tests**

```bash
cd D:\LittleBrushGames\FeatureHub && npm run test
```

Expected: all 34 tests pass.

- [ ] **Step 5: Commit**

```bash
cd D:\LittleBrushGames\FeatureHub && git add src/app.css src/lib/modules/tasks-notes/TasksNotesPanel.svelte && git commit -m "style: TasksNotesPanel uses CSS classes instead of inline styles"
```

---

## Task 5: LinksGrid CSS — fix old accent rgba in app.css

**Files:**
- Modify: `src/app.css` (around line 3327)

- [ ] **Step 1: Find and fix the link-add-bar focus-within rgba**

Read `src/app.css` around line 3313–3340. Find the `.link-add-bar:focus-within` rule:

```css
.link-add-bar:focus-within {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px rgba(91,91,214,0.1);
}
```

Replace with (the old accent rgba `91,91,214` doesn't match the Phase 1 accent `#4d7cff` = `77,124,255`):

```css
.link-add-bar:focus-within {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-dim);
}
```

- [ ] **Step 2: Run tests**

```bash
cd D:\LittleBrushGames\FeatureHub && npm run test
```

Expected: all 34 tests pass.

- [ ] **Step 3: Commit**

```bash
cd D:\LittleBrushGames\FeatureHub && git add src/app.css && git commit -m "style: fix link-add-bar focus ring to use --accent-dim token"
```

---

## Task 6: Final Phase 5 verification

- [ ] **Step 1: Run tests**

```bash
cd D:\LittleBrushGames\FeatureHub && npm run test
```

Expected: all 34 tests pass.

- [ ] **Step 2: Rust check**

```bash
cd D:\LittleBrushGames\FeatureHub\src-tauri && cargo check
```

Expected: no errors.

- [ ] **Step 3: Visual smoke test checklist**

1. Agents tab → plan cards: use card background (not old `--bg-secondary`), proper radius ✓
2. Agents tab → session list: session cards use radius-md, smooth transitions ✓
3. Timeline tab: timestamps and day labels use mono font token ✓
4. Tasks & Notes tab: layout unchanged visually (spacing is equivalent 12px ≈ space-3) ✓
5. Links tab: input focus ring uses accent-dim (correct blue, not old purple-blue) ✓
6. No layout breaks anywhere ✓

- [ ] **Step 4: Commit if any smoke test fixes needed**

```bash
cd D:\LittleBrushGames\FeatureHub && git add src/app.css src/lib/modules/ai/PlanCard.svelte src/lib/modules/ai/SessionCard.svelte src/lib/modules/timeline/Timeline.svelte && git commit -m "style: phase 5 smoke test fixes"
```
