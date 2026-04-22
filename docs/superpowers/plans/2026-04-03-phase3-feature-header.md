# Phase 3 — Feature Detail Header Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Update the feature detail header and tab bar — gradient feature title, gradient active tab text, token-based badge colors, ticket ID copy hint — all CSS-only, no Svelte HTML changes.

**Architecture:** All changes in `src/app.css`. Gradient text applied directly to `.detail-header-title` and `.tab-btn--active` via background-clip trick. Tab count badge children get explicit resets so they don't inherit the gradient clip. No component HTML changes needed.

**Tech Stack:** CSS custom properties (Phase 1 tokens), `npm run test` for regression check.

**Spec reference:** `docs/superpowers/specs/2026-04-03-ui-redesign-design.md` — Phase 3 section.

---

## File Map

| File | Change |
|---|---|
| `src/app.css:1129-1141` | `.detail-header-title` — gradient text |
| `src/app.css:1242-1273` | Tab bar — active tab gradient text, badge color tokens, badge resets |
| `src/app.css:1169-1173` | `.detail-ticket-id` — hover copy hint via `::after` |
| `src/app.css:1188-1196` | `.detail-description` — replace hardcoded font-size |

---

## Task 1: Feature title gradient text

**Files:**
- Modify: `src/app.css:1129-1141`

- [ ] **Step 1: Update `.detail-header-title`**

Find `.detail-header-title {` (around line 1129) and replace the entire rule:

Current:
```css
.detail-header-title {
  min-width: 0;
  font-size: var(--text-xl);
  font-weight: 700;
  letter-spacing: -0.03em;
  color: var(--text-primary);
  cursor: pointer;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex-shrink: 1;
  line-height: 1.3;
}
```

Replace with:
```css
.detail-header-title {
  min-width: 0;
  font-size: var(--text-xl);
  font-weight: 700;
  letter-spacing: -0.03em;
  color: var(--text-primary);
  background-image: var(--grad-primary);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  cursor: pointer;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex-shrink: 1;
  line-height: 1.3;
}
```

Note: `color: var(--text-primary)` is kept as a fallback for browsers that don't support background-clip. The `-webkit-text-fill-color: transparent` takes precedence in supported browsers.

- [ ] **Step 2: Verify in dev server**

Open a feature. The feature title in the header should show blue→violet gradient text.

Edge case to check: when title editing is active (clicking the title), the input field `.detail-title-input` should NOT have gradient text — it uses a separate CSS class and is not affected.

- [ ] **Step 3: Commit**

```bash
git add src/app.css
git commit -m "style: feature title uses gradient text"
```

---

## Task 2: Tab bar — active tab gradient text and badge token colors

**Files:**
- Modify: `src/app.css:1242-1273`

The active tab currently shows `color: var(--text-primary)`. We want gradient text on the active tab label, but NOT on the badge children (which need their own background colors).

- [ ] **Step 1: Update `.tab-btn--active`**

Find `.tab-btn--active {` (around line 1242) and replace:

Current:
```css
.tab-btn--active {
  color: var(--text-primary);
  border-bottom-color: var(--accent);
  font-weight: 600;
}
```

Replace with:
```css
.tab-btn--active {
  color: var(--text-primary);
  border-bottom-color: var(--accent);
  font-weight: 600;
  background-image: var(--grad-primary);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}
```

- [ ] **Step 2: Reset gradient on badge children**

The `.tab-count` and `.tab-count--active` children would inherit the gradient clip and show transparent text. Fix by adding reset rules AFTER the `.tab-btn--active` rule:

```css
.tab-btn--active .tab-count,
.tab-btn--active .tab-count--active {
  background-image: none;
  -webkit-background-clip: padding-box;
  -webkit-text-fill-color: initial;
  background-clip: padding-box;
}
```

- [ ] **Step 3: Fix hardcoded green in tab count badge**

Find `.tab-count--active {` (around line 1265) and replace:

Current:
```css
.tab-count--active {
  background: rgba(34, 197, 94, 0.15);
  color: #22c55e;
}

.tab-btn--active .tab-count--active {
  background: #22c55e;
  color: var(--bg-primary);
}
```

Replace with:
```css
.tab-count--active {
  background: var(--green-dim);
  color: var(--green);
}

.tab-btn--active .tab-count--active {
  background: var(--green);
  color: var(--bg-primary);
  -webkit-text-fill-color: initial;
  background-image: none;
  -webkit-background-clip: padding-box;
  background-clip: padding-box;
}
```

- [ ] **Step 4: Verify in dev server**

Open a feature and click through tabs. The active tab label text should show gradient. The badge (task count, agent count) on the active tab should still show its own color (NOT gradient).

- [ ] **Step 5: Commit**

```bash
git add src/app.css
git commit -m "style: active feature tab gradient text, green badge token colors"
```

---

## Task 3: Ticket ID — hover copy hint

**Files:**
- Modify: `src/app.css:1169-1173`

The ticket ID (e.g. `AUTH-42`) should subtly reveal a copy hint character on hover.

- [ ] **Step 1: Update `.detail-ticket-id`**

Find `.detail-ticket-id {` (around line 1169) and replace:

Current:
```css
.detail-ticket-id {
  font-size: var(--text-xs);
  color: var(--text-muted);
  font-family: var(--font-mono);
}
```

Replace with:
```css
.detail-ticket-id {
  font-size: var(--text-xs);
  color: var(--text-muted);
  font-family: var(--font-mono);
  cursor: pointer;
  transition: color var(--transition-fast);
}

.detail-ticket-id:hover {
  color: var(--text-secondary);
}

.detail-ticket-id::after {
  content: " ⎘";
  opacity: 0;
  color: var(--accent);
  transition: opacity var(--transition-fast);
}

.detail-ticket-id:hover::after {
  opacity: 1;
}
```

- [ ] **Step 2: Verify in dev server**

Hover over a ticket ID (e.g. `AUTH-42`) in the feature header. A `⎘` copy symbol should fade in to the right of the ID. The text itself should brighten slightly.

- [ ] **Step 3: Commit**

```bash
git add src/app.css
git commit -m "style: ticket ID hover reveals copy hint"
```

---

## Task 4: Fix hardcoded font size in description

**Files:**
- Modify: `src/app.css:1188-1196`

- [ ] **Step 1: Update `.detail-description`**

Find `.detail-description {` (around line 1188). The `font-size: 12.5px` is hardcoded. Replace:

```css
.detail-description {
  padding: 0 var(--space-6) var(--space-1);
  font-size: var(--text-sm);
  line-height: 1.5;
  color: var(--text-secondary);
  cursor: default;
  white-space: pre-wrap;
  word-break: break-word;
}
```

(`--text-sm` is 12px — one step down from the hardcoded 12.5px, which is fine for description text density.)

- [ ] **Step 2: Check for other hardcoded values in lines 1115-1275**

Run: `grep -n "[0-9]\+\(\.[0-9]\+\)\?px" src/app.css | awk -F: '$2 > 1115 && $2 < 1275'`

For any remaining hardcoded px values that should be tokens, replace them. Common ones:
- `2px 8px` in `.detail-title-input` padding → leave as-is (input padding is fine hardcoded)
- `2px solid transparent` in tab border → leave as-is (structural)

- [ ] **Step 3: Commit**

```bash
git add src/app.css
git commit -m "style: use text-sm token for description, audit detail header px values"
```

---

## Task 5: Final Phase 3 verification

- [ ] **Step 1: Run tests**

```bash
npm run test
```

Expected: all tests pass.

- [ ] **Step 2: Rust check**

```bash
cd src-tauri && cargo check && cd ..
```

- [ ] **Step 3: Visual smoke test**

1. Feature title: blue→violet gradient text ✓
2. Active tab: gradient text on label, NOT on badge ✓
3. Green badge (active session count): uses `--green` token color ✓
4. Ticket ID: hover shows ⎘ hint ✓
5. Description: slightly smaller text (12px vs 12.5px) ✓
6. No layout breaks ✓

- [ ] **Step 4: Commit if any smoke test fixes**

```bash
git add src/app.css
git commit -m "style: phase 3 smoke test fixes"
```
