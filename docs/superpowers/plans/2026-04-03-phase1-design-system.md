# Phase 1 — Design System Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Update `src/app.css` design tokens, fonts, gradient utilities, and base component styles to establish the new FeatureHub visual language — without touching any Svelte component HTML.

**Architecture:** All changes are confined to `src/app.css`. Token values are updated in-place (no renames — existing token names are used throughout 4,919 lines). New tokens and utility classes are added to the existing `:root` block and base style section. Component styles (buttons, cards, badges) are updated to use the new tokens.

**Tech Stack:** CSS custom properties, Space Grotesk (Google Fonts), Svelte 5 / Vite (dev server), `npm run tauri dev` to verify.

**Spec reference:** `docs/superpowers/specs/2026-04-03-ui-redesign-design.md` — Phase 1 section.

---

## File Map

| File | Change |
|---|---|
| `src/app.css:1` | Replace `@import` URL — swap Plus Jakarta Sans for Space Grotesk |
| `src/app.css:4-90` | Update `:root` token values + add new tokens |
| `src/app.css:82` | Update `--font-ui` value |
| `src/app.css:92+` | Add gradient utility classes after `:root` block |
| `src/app.css:106-130` | Update focus ring, selection, scrollbar base styles |
| `src/app.css:1608-1684` | Update button variants |
| `src/app.css:1784-1809` | Update badge styles |
| `src/app.css:2429-2440` | Update `.card` hover system |

---

## Task 1: Swap font import and `--font-ui` token

**Files:**
- Modify: `src/app.css:1`
- Modify: `src/app.css:82`

- [ ] **Step 1: Replace the Google Fonts `@import` on line 1**

Current line 1:
```css
@import url('https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600;700&family=Plus+Jakarta+Sans:wght@400;500;600;700&display=swap');
```

Replace with:
```css
@import url('https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600;700&family=Space+Grotesk:wght@400;500;600;700&display=swap');
```

- [ ] **Step 2: Update `--font-ui` on line 82**

Current:
```css
--font-ui: 'Plus Jakarta Sans', -apple-system, BlinkMacSystemFont, sans-serif;
```

Replace with:
```css
--font-ui: 'Space Grotesk', -apple-system, BlinkMacSystemFont, sans-serif;
```

- [ ] **Step 3: Start dev server and verify font loads**

Run: `npm run tauri dev`

Open the app. The UI font should change — Space Grotesk has a slightly geometric, wider feel compared to Plus Jakarta Sans. If it falls back to system font (antialiased sans-serif), the import URL may be blocked (Tauri WebView2 requires network access for Google Fonts, which is allowed by default in dev mode).

- [ ] **Step 4: Commit**

```bash
git add src/app.css
git commit -m "style: swap font to Space Grotesk"
```

---

## Task 2: Update surface color tokens

**Files:**
- Modify: `src/app.css:6-14`

The surfaces get darker and cooler (more blue-black vs the current grey-black).

- [ ] **Step 1: Replace surface tokens in `:root`**

Find the `/* ── Surface Colors */` block (lines 6–14) and replace with:

```css
  /* ── Surface Colors (layered depth system) ── */
  --bg-base: #08090e;
  --bg-primary: #0d0f18;
  --bg-secondary: #0f1120;
  --bg-card: #12141f;
  --bg-raised: #161826;
  --bg-hover: #181a28;
  --bg-active: #1c1e30;
  --bg-input: #0b0d16;
  --bg-overlay: rgba(0,0,0,0.7);
```

- [ ] **Step 2: Verify in dev server**

The sidebar and main panel background should be noticeably darker and shifted from grey toward deep blue-black.

- [ ] **Step 3: Commit**

```bash
git add src/app.css
git commit -m "style: update surface color tokens — deeper blue-black palette"
```

---

## Task 3: Update text color tokens

**Files:**
- Modify: `src/app.css:22-27`

Key change: `--text-secondary` gets much brighter (#8d8da6 → #b8c0d8). This is important — feature names, descriptions, and secondary labels were previously too dim.

- [ ] **Step 1: Replace text tokens in `:root`**

Find the `/* ── Text ── */` block (lines 22–27) and replace with:

```css
  /* ── Text ── */
  --text-primary: #e8ecf8;
  --text-secondary: #b8c0d8;
  --text-muted: #5a6480;
  --text-faint: #3a4060;
```

- [ ] **Step 2: Verify in dev server**

Feature titles and sidebar labels should be slightly brighter/cooler. Secondary text (descriptions, metadata) should be noticeably more readable — it was quite dim before.

- [ ] **Step 3: Commit**

```bash
git add src/app.css
git commit -m "style: brighten text-secondary token for better readability"
```

---

## Task 4: Update border tokens

**Files:**
- Modify: `src/app.css:16-20`

Switching from solid hex borders to `rgba` — this looks better against the new darker backgrounds.

- [ ] **Step 1: Replace border tokens in `:root`**

Find the `/* ── Borders ── */` block (lines 16–20) and replace with:

```css
  /* ── Borders ── */
  --border-subtle: rgba(255,255,255,0.05);
  --border: rgba(255,255,255,0.08);
  --border-strong: rgba(255,255,255,0.14);
  --border-focus: #4d7cff;
```

- [ ] **Step 2: Verify in dev server**

Card and panel borders should look slightly more transparent/integrated rather than having a distinct color. This is subtle but contributes to the glass-like depth.

- [ ] **Step 3: Commit**

```bash
git add src/app.css
git commit -m "style: switch border tokens to rgba for glass-like depth"
```

---

## Task 5: Update accent tokens (purple → blue)

**Files:**
- Modify: `src/app.css:28-32`

This is the most visible single change — the primary accent shifts from indigo-purple (`#7c7cff`) to a saturated blue (`#4d7cff`). Every selected state, button, badge, and focus ring changes.

- [ ] **Step 1: Replace accent tokens in `:root`**

Find the `/* ── Accent ── */` block (lines 28–32) and replace with:

```css
  /* ── Accent ── */
  --accent: #4d7cff;
  --accent-hover: #6b93ff;
  --accent-dim: rgba(77,124,255,0.12);
  --accent-border: rgba(77,124,255,0.22);
  --accent-glow: rgba(77,124,255,0.25);
```

- [ ] **Step 2: Fix hardcoded accent references in base styles**

The `::selection` rule (lines 118-121) and the `.sidebar-logo` gradient (lines 170-171) still use the old `#7c7cff`. Update them:

`::selection` block — change:
```css
  background: rgba(124,124,255,0.3);
```
to:
```css
  background: rgba(77,124,255,0.3);
```

`.sidebar-logo` gradient — change:
```css
  background: linear-gradient(135deg, #7c7cff 0%, #a78bfa 50%, #c4b5fd 100%);
  box-shadow: 0 2px 8px rgba(124,124,255,0.25);
```
to:
```css
  background: var(--grad-primary);
  box-shadow: 0 2px 8px var(--accent-glow);
```

Note: `--grad-primary` is added in Task 7. Run these two tasks back to back before testing.

- [ ] **Step 3: Verify in dev server (after Task 7)**

Selected features in sidebar, active tab indicators, focus rings, and the logo should all be blue instead of purple. The logo gradient will update in Task 7.

- [ ] **Step 4: Commit**

```bash
git add src/app.css
git commit -m "style: shift accent from purple to blue (#4d7cff)"
```

---

## Task 6: Update semantic color tokens

**Files:**
- Modify: `src/app.css:34-46`

- [ ] **Step 1: Replace semantic color tokens in `:root`**

Find the `/* ── Semantic Colors ── */` block (lines 34–46) and replace with:

```css
  /* ── Semantic Colors ── */
  --green: #34d399;
  --green-dim: rgba(52,211,153,0.12);
  --amber: #fbbf24;
  --amber-dim: rgba(251,191,36,0.12);
  --red: #f87171;
  --red-dim: rgba(248,113,113,0.12);
  --blue: #52a9ff;
  --blue-dim: rgba(82,169,255,0.12);
  --purple: #a78bfa;
  --purple-dim: rgba(167,139,250,0.12);
  --cyan: #22d3ee;
  --cyan-dim: rgba(34,211,238,0.12);
```

Changes: green shifts slightly cooler, amber brightens, red softens. Purple and cyan are unchanged.

- [ ] **Step 2: Verify in dev server**

Status badges (active = green, blocked = red, in-progress = amber) should reflect the new colors. Check the feature list sidebar for status dots.

- [ ] **Step 3: Commit**

```bash
git add src/app.css
git commit -m "style: update semantic color tokens"
```

---

## Task 7: Add gradient tokens, card hover vars, and shadow updates

**Files:**
- Modify: `src/app.css:68-72` (shadows)
- Modify: `src/app.css:86-90` (transitions)
- Add after line 90: new gradient + card hover tokens

- [ ] **Step 1: Update shadow tokens (lines 68–72)**

Replace:
```css
  /* ── Shadows ── */
  --shadow-sm: 0 1px 2px rgba(0,0,0,0.3);
  --shadow-md: 0 4px 12px rgba(0,0,0,0.35);
  --shadow-lg: 0 8px 24px rgba(0,0,0,0.45);
  --shadow-xl: 0 20px 60px rgba(0,0,0,0.55);
```

With:
```css
  /* ── Shadows ── */
  --shadow-sm: 0 1px 3px rgba(0,0,0,0.4);
  --shadow-md: 0 4px 16px rgba(0,0,0,0.5), 0 1px 3px rgba(0,0,0,0.3);
  --shadow-lg: 0 8px 30px rgba(0,0,0,0.6), 0 2px 8px rgba(0,0,0,0.3);
  --shadow-xl: 0 20px 60px rgba(0,0,0,0.7), 0 4px 16px rgba(0,0,0,0.4);
  --shadow-glow: 0 0 20px rgba(77,124,255,0.12);
```

- [ ] **Step 2: Update transition tokens (lines 87–89)**

Replace:
```css
  /* ── Transitions ── */
  --transition-fast: 0.1s ease;
  --transition-base: 0.15s ease;
  --transition-slow: 0.25s ease;
```

With:
```css
  /* ── Transitions ── */
  --transition-fast: 0.12s ease;
  --transition-base: 0.2s ease;
  --transition-slow: 0.32s ease;
```

- [ ] **Step 3: Add gradient and card hover tokens after the closing `}` of `:root` (after line 90)**

Insert after line 90 (the closing `}` of `:root`), before the `* { margin: 0...` reset:

```css
:root {
  /* ── Gradients ── */
  --grad-primary: linear-gradient(135deg, #4d7cff, #7b6fff);
  --grad-success: linear-gradient(135deg, #34d399, #22d3ee);
  --grad-warn: linear-gradient(135deg, #fbbf24, #f97316);
  --grad-cool: linear-gradient(135deg, #22d3ee, #4d7cff);

  /* ── Card hover system ── */
  --card-hover-y: -2px;
  --card-hover-shadow: var(--shadow-md);
}
```

(Using a second `:root` block — CSS merges them, no issues.)

- [ ] **Step 4: Verify in dev server**

Open any modal (e.g. Settings). It should have a deeper shadow. Card hover on sidebar feature items should be subtly smoother.

- [ ] **Step 5: Commit**

```bash
git add src/app.css
git commit -m "style: add gradient tokens, card hover vars, deepen shadows"
```

---

## Task 8: Add gradient text utility classes

**Files:**
- Modify: `src/app.css` — insert after the `:root` blocks, before `/* ── Focus ring ──`

These utility classes enable gradient text anywhere in the app. They must be applied to `<span>` (inline) elements — never block elements — or the `transparent` fill becomes invisible.

- [ ] **Step 1: Insert gradient text utilities**

Find the comment `/* ── Focus ring ── */` (around line 106) and insert immediately before it:

```css
/* ── Gradient text utilities ── */
.gt {
  display: inline-block;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}
.gt-p { background-image: var(--grad-primary); }
.gt-s { background-image: var(--grad-success); }
.gt-w { background-image: var(--grad-warn); }
.gt-c { background-image: var(--grad-cool); }
```

- [ ] **Step 2: Verify classes exist**

Open browser devtools (F12 in the Tauri app, or check Vite HMR). Search for `.gt` in the computed styles panel to confirm the classes are present. You won't see them used yet — they'll be wired to components in later phases.

- [ ] **Step 3: Commit**

```bash
git add src/app.css
git commit -m "style: add gradient text utility classes (.gt, .gt-p, .gt-s, .gt-w, .gt-c)"
```

---

## Task 9: Update `.card` hover system

**Files:**
- Modify: `src/app.css:2429-2440`

Replace the flat background-color hover with a consistent lift + shadow hover system.

- [ ] **Step 1: Update `.card` and `.card:hover`**

Find the `/* ===== CARD =====` section (around line 2427) and replace the `.card` and `.card:hover` rules:

Current:
```css
.card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 7px;
  padding: 12px 14px;
  transition: all 0.1s;
}

.card:hover {
  background: var(--bg-hover);
  border-color: var(--accent-border);
}
```

Replace with:
```css
.card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 12px 14px;
  transition: transform var(--transition-base), box-shadow var(--transition-base), border-color var(--transition-base);
}

.card:hover {
  transform: translateY(var(--card-hover-y));
  box-shadow: var(--card-hover-shadow);
  border-color: var(--accent-border);
}
```

- [ ] **Step 2: Verify in dev server**

Navigate to any feature with cards visible (e.g. the Links tab — link cards use `.card`). Hovering a card should produce a subtle upward lift rather than a background color change.

- [ ] **Step 3: Commit**

```bash
git add src/app.css
git commit -m "style: update card hover to lift+shadow system"
```

---

## Task 10: Add `.btn-primary` gradient button variant

**Files:**
- Modify: `src/app.css` — insert after the `/* ===== BUTTONS =====` section

The existing buttons are `.btn-accent` (outline), `.btn-subtle`, `.btn-ghost`, `.btn-icon`. There is no filled gradient primary button. "Start Agent" and other primary CTA buttons will need this.

- [ ] **Step 1: Add `.btn-primary` after the existing `/* ===== BUTTONS ===== */` section**

Find `.btn-accent {` (around line 1610) and insert before it:

```css
.btn-primary {
  padding: 7px 16px;
  border-radius: var(--radius-md);
  cursor: pointer;
  border: none;
  background: var(--grad-primary);
  color: #fff;
  font-size: var(--text-sm);
  font-weight: 600;
  font-family: inherit;
  transition: opacity var(--transition-fast), transform var(--transition-fast);
  white-space: nowrap;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.btn-primary:hover { opacity: 0.88; transform: translateY(-1px); }
.btn-primary:active { opacity: 1; transform: translateY(0); }
.btn-primary:disabled { opacity: 0.4; cursor: not-allowed; transform: none; }
```

- [ ] **Step 2: Verify in dev server**

There are no `.btn-primary` usages yet — they'll be wired in Phase 3. To confirm the class works, temporarily add `class="btn-primary"` to any existing button in a Svelte file, check it looks correct, then revert.

- [ ] **Step 3: Commit**

```bash
git add src/app.css
git commit -m "style: add btn-primary gradient button variant"
```

---

## Task 11: Update scrollbar styles

**Files:**
- Modify: `src/app.css:123-130`

- [ ] **Step 1: Update scrollbar rules**

Find the `/* ── Global scrollbar ── */` block (lines 123–130) and replace with:

```css
/* ── Global scrollbar ── */
::-webkit-scrollbar { width: 5px; height: 5px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb {
  background: rgba(255,255,255,0.1);
  border-radius: 3px;
}
::-webkit-scrollbar-thumb:hover { background: rgba(255,255,255,0.18); }
```

- [ ] **Step 2: Verify in dev server**

Scroll any long list (feature list, session list). The scrollbar thumb should be a subtle translucent white.

- [ ] **Step 3: Commit**

```bash
git add src/app.css
git commit -m "style: update scrollbar to translucent white thumb"
```

---

## Task 12: Update `.status-badge` and `.tag-badge` radius

**Files:**
- Modify: `src/app.css:1792-1809`

Status and tag badges should use the token-based radius rather than hardcoded values.

- [ ] **Step 1: Update badge rules**

Find the `/* ===== BADGES =====` section and update:

Current `.status-badge`:
```css
.status-badge {
  font-size: 10.5px;
  font-weight: 600;
  padding: 2px 9px;
  border-radius: 20px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  display: inline-flex;
  align-items: center;
  gap: 0;
}
```

Replace `.status-badge` with:
```css
.status-badge {
  font-size: 10.5px;
  font-weight: 600;
  padding: 2px 9px;
  border-radius: var(--radius-full);
  letter-spacing: 0.02em;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
```

Current `.tag-badge`:
```css
.tag-badge {
  font-size: 10.5px;
  font-weight: 500;
  padding: 2px 8px;
  border-radius: 4px;
}
```

Replace with:
```css
.tag-badge {
  font-size: 10.5px;
  font-weight: 500;
  padding: 2px 8px;
  border-radius: var(--radius-sm);
}
```

- [ ] **Step 2: Verify in dev server**

Open any feature with tags and a status badge. The status pill should look the same (already pill-shaped). Tags should use the token radius.

- [ ] **Step 3: Commit**

```bash
git add src/app.css
git commit -m "style: update badge radius to use design tokens"
```

---

## Task 13: Final verification pass

- [ ] **Step 1: Run frontend tests**

```bash
npm run test
```

Expected: all tests pass. CSS changes do not affect component logic or test assertions.

- [ ] **Step 2: Run full build check**

```bash
cd src-tauri && cargo check
```

Expected: no errors (Rust is unaffected by CSS changes).

- [ ] **Step 3: Visual smoke test checklist**

With the app running (`npm run tauri dev`), verify each of these manually:

1. Font: UI text uses Space Grotesk (geometric, slightly wide letterforms)
2. Sidebar background: deep blue-black (not grey)
3. Sidebar feature titles: bright, readable
4. Secondary text: visibly brighter than before (#b8c0d8 vs old #8d8da6)
5. Accent color: blue (not purple) — check selected feature in sidebar, active tab, focus rings
6. Logo: uses gradient (blue→violet)
7. Card hover: lifts up 2px + shadow (not background color change)
8. Scrollbar: thin translucent white thumb
9. No layout breaks anywhere

- [ ] **Step 4: Commit if any fixes were made during smoke test**

```bash
git add src/app.css
git commit -m "style: phase 1 smoke test fixes"
```

---

## Self-Review Notes

**Spec coverage check:**
- ✅ Font swap (Space Grotesk) — Task 1
- ✅ Surface color tokens — Task 2
- ✅ Text color tokens — Task 3
- ✅ Border tokens — Task 4
- ✅ Accent tokens (purple → blue) — Task 5
- ✅ Semantic colors — Task 6
- ✅ Gradient tokens — Task 7
- ✅ Card hover vars — Task 7
- ✅ Shadow tokens — Task 7
- ✅ Transition tokens — Task 7
- ✅ Gradient text utilities — Task 8
- ✅ Card hover system — Task 9
- ✅ `.btn-primary` variant — Task 10
- ✅ Scrollbar — Task 11
- ✅ Badge radius — Task 12
- ✅ Test + verification — Task 13

**Type consistency:** No new types introduced — pure CSS.

**Dependency note:** Task 5 references `var(--grad-primary)` in `.sidebar-logo`. This token is added in Task 7. Run Tasks 5 and 7 back to back before starting the dev server, or the logo may briefly show no gradient. Both are committed separately but this is not a functional regression — the fallback is transparent.
