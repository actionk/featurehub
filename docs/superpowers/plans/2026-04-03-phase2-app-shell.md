# Phase 2 — App Shell Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Polish the app shell visual layer — workspace tab bar, sidebar footer, search overlay, and toast notifications — using Phase 1 tokens, with minimal Svelte HTML changes.

**Architecture:** Primarily CSS changes in `src/app.css`. Two targeted Svelte changes: (1) `WorkspaceTabBar.svelte` active tab emoji is removed for cleaner gradient title, (2) `SearchBar.svelte` gets a keyboard shortcut footer row. No new components. No routing changes.

**Tech Stack:** CSS custom properties (Phase 1 tokens), Svelte 5, `npm run tauri dev` to verify, `npm run test` for regression check.

**Spec reference:** `docs/superpowers/specs/2026-04-03-ui-redesign-design.md` — Phase 2 section.

---

## File Map

| File | Change |
|---|---|
| `src/app.css:975-1104` | WorkspaceTabBar CSS: active tab gradient title, status dot tokens, context menu shadow |
| `src/app.css:873-895` | Sidebar footer: btn-new-feature → gradient style |
| `src/app.css:3754-3825` | Search overlay: command palette improvements |
| `src/lib/components/WorkspaceTabBar.svelte` | CSS-only: no HTML change needed (gradient applied via CSS class) |
| `src/lib/components/SearchBar.svelte` | Add keyboard shortcut footer row to search-box HTML |

---

## Task 1: Workspace tab bar — active tab gradient title and status dot tokens

**Files:**
- Modify: `src/app.css:975-1104`

The active tab currently shows `color: var(--text-primary)` with an `inset 0 -2px 0 var(--accent)` bottom border. We want the active tab title to use gradient text, and status dots to use semantic color tokens instead of hardcoded hex.

- [ ] **Step 1: Add gradient title to active tab**

Find `.workspace-tab--active` in `src/app.css` (around line 1009) and replace it:

Current:
```css
.workspace-tab--active {
  color: var(--text-primary);
  background: var(--bg-primary);
  box-shadow: inset 0 -2px 0 var(--accent);
}
```

Replace with:
```css
.workspace-tab--active {
  color: var(--text-primary);
  background: var(--bg-primary);
  box-shadow: inset 0 -2px 0 var(--accent);
}

.workspace-tab--active .workspace-tab-title {
  background-image: var(--grad-primary);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}
```

- [ ] **Step 2: Replace hardcoded status dot colors with semantic tokens**

Find the `.workspace-tab-status[data-status="..."]` rules (around lines 1035-1040) and replace:

Current:
```css
.workspace-tab-status[data-status="in_progress"] { background: var(--accent); }
.workspace-tab-status[data-status="todo"] { background: #6b7280; }
.workspace-tab-status[data-status="in_review"] { background: #f59e0b; }
.workspace-tab-status[data-status="done"] { background: #22c55e; }
.workspace-tab-status[data-status="blocked"] { background: #ef4444; }
.workspace-tab-status[data-status="paused"] { background: #a855f7; }
```

Replace with:
```css
.workspace-tab-status[data-status="active"]      { background: var(--accent); }
.workspace-tab-status[data-status="in_progress"] { background: var(--amber); }
.workspace-tab-status[data-status="todo"]        { background: var(--text-muted); }
.workspace-tab-status[data-status="in_review"]   { background: var(--blue); }
.workspace-tab-status[data-status="done"]        { background: var(--green); }
.workspace-tab-status[data-status="blocked"]     { background: var(--red); }
.workspace-tab-status[data-status="paused"]      { background: var(--purple); }
```

- [ ] **Step 3: Fix hardcoded context menu shadow**

Find `.workspace-tab-context` (around line 1076) and replace `box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4)` with `box-shadow: var(--shadow-lg)`.

- [ ] **Step 4: Verify in dev server**

Open the app. Open 2+ features so the workspace tab bar is visible. The active tab title should show a blue→violet gradient text. Status dots should match the semantic colors.

- [ ] **Step 5: Commit**

```bash
git add src/app.css
git commit -m "style: active tab gradient title, token-based status dots"
```

---

## Task 2: Sidebar footer — "New Feature" button gradient

**Files:**
- Modify: `src/app.css:873-895`

The `.btn-new-feature` uses `background: var(--accent)` (flat blue). Update it to use the gradient like `.btn-primary`.

- [ ] **Step 1: Update `.btn-new-feature` styling**

Find `.btn-new-feature` (around line 873) and replace the full rule:

Current:
```css
.btn-new-feature {
  flex: 1;
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: var(--radius);
  padding: 8px var(--space-4);
  font-size: var(--text-sm);
  font-weight: 600;
  font-family: inherit;
  cursor: pointer;
  transition: all var(--transition-fast);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  box-shadow: 0 2px 8px var(--accent-glow);
}

.btn-new-feature:hover {
  background: var(--accent-hover);
  box-shadow: 0 4px 12px var(--accent-glow);
}
```

Replace with:
```css
.btn-new-feature {
  flex: 1;
  background: var(--grad-primary);
  color: #fff;
  border: none;
  border-radius: var(--radius-md);
  padding: 8px var(--space-4);
  font-size: var(--text-sm);
  font-weight: 600;
  font-family: inherit;
  cursor: pointer;
  transition: opacity var(--transition-fast), transform var(--transition-fast), box-shadow var(--transition-fast);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  box-shadow: 0 2px 8px var(--accent-glow);
}

.btn-new-feature:hover {
  opacity: 0.88;
  transform: translateY(-1px);
  box-shadow: 0 4px 16px var(--accent-glow);
}
```

- [ ] **Step 2: Verify in dev server**

The "New Feature" button in the sidebar footer should show a blue→violet gradient background that lifts on hover.

- [ ] **Step 3: Commit**

```bash
git add src/app.css
git commit -m "style: btn-new-feature uses gradient background with lift hover"
```

---

## Task 3: Search overlay — command palette improvements

**Files:**
- Modify: `src/app.css:3754-3825`
- Modify: `src/lib/components/SearchBar.svelte`

Improve the search box: darker background, stronger border, and a keyboard shortcut footer.

- [ ] **Step 1: Update search box CSS in `src/app.css`**

Find `.search-box` (around line 3767) and update:

Current:
```css
.search-box {
  width: 600px;
  max-width: 90%;
  max-height: 450px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-xl);
  overflow: hidden;
  box-shadow: var(--shadow-xl);
  display: flex;
  flex-direction: column;
  animation: modalIn 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}
```

Replace with:
```css
.search-box {
  width: 600px;
  max-width: 90%;
  max-height: 480px;
  background: var(--bg-card);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-xl);
  overflow: hidden;
  box-shadow: var(--shadow-xl), 0 0 0 1px rgba(77,124,255,0.06);
  display: flex;
  flex-direction: column;
  animation: modalIn 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}
```

Also find `.search-input` (around line 3781) and add a padding rule to the input row. First, find the existing search input row wrapper. Look for a `.search-input-row` or `.search-box` direct children. Add a new rule after `.search-box`:

```css
.search-input-row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  border-bottom: 1px solid var(--border);
}
```

And add a footer rule for keyboard hints:
```css
.search-footer {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  padding: 8px var(--space-4);
  border-top: 1px solid var(--border);
  background: var(--bg-raised);
}

.search-footer-hint {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 10.5px;
  color: var(--text-muted);
}

.search-footer-key {
  background: var(--bg-hover);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  padding: 1px 5px;
  font-size: 10px;
  font-family: var(--font-mono);
  color: var(--text-muted);
}
```

- [ ] **Step 2: Add footer HTML to `SearchBar.svelte`**

Read `src/lib/components/SearchBar.svelte`. Find the closing tag of the `.search-box` div (it contains `.search-input-row` or equivalent and `.search-results`). Before the closing `</div>` of `.search-box`, add:

```html
  <div class="search-footer">
    <span class="search-footer-hint">
      <kbd class="search-footer-key">↑↓</kbd> navigate
    </span>
    <span class="search-footer-hint">
      <kbd class="search-footer-key">↵</kbd> open
    </span>
    <span class="search-footer-hint">
      <kbd class="search-footer-key">Esc</kbd> close
    </span>
  </div>
```

- [ ] **Step 3: Verify in dev server**

Press Ctrl+T (or the search shortcut). The search box should have a slightly darker background, stronger border, and a subtle footer row with keyboard hints.

- [ ] **Step 4: Run frontend tests**

```bash
npm run test
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/app.css src/lib/components/SearchBar.svelte
git commit -m "style: search overlay — darker box, stronger border, keyboard hints footer"
```

---

## Task 4: Toast notification polish

**Files:**
- Modify: `src/app.css` — toast section

Read the current toast styles (search for `.toast {` or `/* ===== TOAST` in app.css). Update the toast to use `var(--bg-card)` background and `var(--shadow-lg)` shadow consistently.

- [ ] **Step 1: Search for toast styles**

Run: `grep -n "toast" src/app.css` from `D:\LittleBrushGames\FeatureHub` to find the toast CSS section.

- [ ] **Step 2: Update toast background and border**

Find the `.toast` rule. Update `background` to `var(--bg-card)` (if it isn't already) and ensure `border` uses `var(--border)`. If `backdrop-filter` is hardcoded as `blur(8px)`, leave it — that's intentional.

- [ ] **Step 3: Verify toast appearance**

Trigger a toast (e.g., mark a task complete). The toast should appear with the new card background color.

- [ ] **Step 4: Commit (only if changes were made)**

```bash
git add src/app.css
git commit -m "style: toast uses card background token"
```

---

## Task 5: Fix remaining hardcoded colors in app shell CSS

**Files:**
- Modify: `src/app.css`

Search for hardcoded hex/rgba values in the app shell CSS sections (sidebar, workspace tab bar, search) that should be tokens.

- [ ] **Step 1: Search for hardcoded values in app shell range**

Run these to find remaining hardcoded values in lines 150–1110 and 3754–3825:

```bash
grep -n "#[0-9a-fA-F]\{3,6\}\|rgba([0-9]" src/app.css | grep -v "^[0-9]*:.*--" | head -40
```

(This finds hex/rgba values that are NOT inside a variable definition.)

- [ ] **Step 2: Replace any hardcoded values with tokens**

For any found values:
- Background greys → nearest `--bg-*` token
- Border colors → `--border` or `--border-strong`
- Shadow rgba → `--shadow-sm/md/lg`
- Status colors → `--amber`, `--blue`, `--green`, `--red`, `--purple` semantic tokens

Do NOT change values that are in `:root` variable definitions (those are intentional). Only replace usages in component selectors.

- [ ] **Step 3: Run frontend tests**

```bash
npm run test
```

Expected: all tests pass.

- [ ] **Step 4: Commit if any changes**

```bash
git add src/app.css
git commit -m "style: replace hardcoded colors in app shell with design tokens"
```

---

## Task 6: Final Phase 2 verification

- [ ] **Step 1: Run full test suite**

```bash
cd D:\LittleBrushGames\FeatureHub
npm run test
```

Expected: all tests pass.

- [ ] **Step 2: Rust check**

```bash
cd D:\LittleBrushGames\FeatureHub\src-tauri
cargo check
```

Expected: no errors.

- [ ] **Step 3: Visual smoke test**

With `npm run tauri dev` running, check:

1. Active workspace tab: title shows blue→violet gradient text ✓
2. Status dots in workspace tab bar: match semantic colors (amber for in_progress, green for done, etc.) ✓
3. "New Feature" button: gradient background, lifts on hover ✓
4. Search overlay (Ctrl+T): darker box, keyboard hints in footer ✓
5. Toast notifications: appear cleanly with card background ✓
6. No layout breaks anywhere in the app ✓

- [ ] **Step 4: Commit summary if anything was fixed during smoke test**

```bash
git add src/app.css
git commit -m "style: phase 2 smoke test fixes"
```
