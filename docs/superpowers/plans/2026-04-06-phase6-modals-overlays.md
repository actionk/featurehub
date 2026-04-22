# Phase 6 — Modals & Overlays Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the UI redesign by polishing all modal and overlay components — shared chrome, `CreateFeatureModal`, `SettingsModal`, `SearchBar`, `ConfirmDialog`, and `Dropdown` — replacing hardcoded values with design tokens, aligning animations with the slide-up + fade-in spec, and adding missing CSS classes for inline style extraction.

**Architecture:** Five categories of change:
1. `app.css` global sections (`MODAL`, `DROPDOWN`, `SEARCH`, `CREATE FEATURE MODAL`) — update existing rules in-place, no class renames.
2. `Modal.svelte` — no HTML changes; the backdrop/animation spec is already wired to `.modal-backdrop` / `.modal-content` global classes.
3. `ConfirmDialog.svelte` — fix two hardcoded colour values in its `<style>` block (`#f06669`, `#fff`).
4. `CreateFeatureModal.svelte` — extract inline `style=` blocks into new CSS classes added to `app.css`.
5. `Dropdown.svelte` — upgrade its scoped `.dropdown-panel` to match the spec's `backdrop-filter: blur(8px)` + dark surface; add a global `.dropdown-separator` class.
6. `SettingsModal.svelte` — fix two remaining hardcoded values in its scoped `<style>` block; add `.settings-font-preview` class for the live font preview row.
7. `SearchBar.svelte` — extract inline styles to CSS classes in `app.css`.

**Tech Stack:** CSS custom properties (Phase 1 tokens), Svelte 5 scoped styles, `npm run tauri dev` for smoke test.

**Spec reference:** `docs/superpowers/specs/2026-04-03-ui-redesign-design.md` — Phase 6 section.

---

## File Map

| File | Section | Change |
|---|---|---|
| `src/app.css` ~line 2359 | `/* ===== MODAL =====*/` | Backdrop `rgba(0,0,0,0.6)`, blur `4px`, slide-up animation |
| `src/app.css` ~line 2318 | `/* ===== DROPDOWN ===== */` | Upgrade `.dropdown` + `.dropdown-panel` to `blur(8px)` dark surface; add `.dropdown-separator` |
| `src/app.css` ~line 4337 | `/* ===== SEARCH ===== */` | `search-box` border-radius to `--radius-xl`, animation tokens, inline style extraction |
| `src/app.css` ~line 4050 | `/* ===== CREATE FEATURE MODAL ===== */` | Add `.create-modal-header`, `.create-modal-body`, `.create-modal-footer`, `.create-error-msg` |
| `src/app.css` ~line 4036 | `.form-input:focus` | Fix hardcoded `rgba(91,91,214,0.15)` → `var(--accent-glow)` |
| `src/lib/components/ui/Modal.svelte` | (no changes) | All styling via global classes — no edits needed |
| `src/lib/components/ui/ConfirmDialog.svelte` | `<style>` block | Fix `#f06669` → `color-mix(in srgb, var(--red) 80%, white)`, `#fff` → `white` |
| `src/lib/components/CreateFeatureModal.svelte` | HTML template | Replace inline `style=` blocks with CSS classes |
| `src/lib/components/ui/Dropdown.svelte` | scoped `<style>` | Upgrade `.dropdown-panel` to blur + `--bg-card` surface |
| `src/lib/components/SettingsModal.svelte` | scoped `<style>` | Fix `var(--yellow)` → `var(--amber)`, `'JetBrains Mono', monospace` → `var(--font-mono)`, `var(--text)` → `var(--text-primary)` |
| `src/lib/components/SearchBar.svelte` | HTML template | Extract inline styles to CSS classes |

---

## Task 1: Shared modal chrome — `app.css` MODAL section

**Files:**
- Modify: `src/app.css` (lines ~2359–2414)

Read the current `/* ===== MODAL ===== */` block first. Spec says: backdrop `rgba(0,0,0,0.6)` with `blur(4px)`, slide-up + fade-in entry animation. `.modal-content` currently uses `translateY(-8px)` (slides down from above) — spec says slide up from below (`translateY(8px)`).

- [ ] **Step 1: Update `.modal-backdrop` and `.modal-content` in `app.css`**

Locate the existing `.modal-backdrop` and `.modal-content` rules (around line 2359). Replace the full rules:

For `.modal-backdrop`:
```css
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 50;
  animation: backdropIn var(--transition-base) ease;
}
```

For `.modal-content`:
```css
.modal-content {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: var(--space-8);
  width: 420px;
  max-width: 90%;
  max-height: 85vh;
  overflow-y: auto;
  box-shadow: var(--shadow-xl);
  animation: modalSlideUp var(--transition-base) cubic-bezier(0.16, 1, 0.3, 1);
}
```

Add these keyframes after the existing `@keyframes modalIn` and `@keyframes fadeIn` (keep those — they're referenced elsewhere):
```css
@keyframes backdropIn {
  from { opacity: 0; }
  to   { opacity: 1; }
}

@keyframes modalSlideUp {
  from { opacity: 0; transform: translateY(10px) scale(0.97); }
  to   { opacity: 1; transform: translateY(0) scale(1); }
}
```

- [ ] **Step 2: Fix `.form-input:focus` hardcoded focus glow**

Find `.form-input:focus` (~line 4036):
```css
.form-input:focus {
  border-color: var(--border-focus);
  box-shadow: 0 0 0 3px rgba(91,91,214,0.15);
}
```
Replace with:
```css
.form-input:focus {
  border-color: var(--border-focus);
  box-shadow: 0 0 0 3px var(--accent-glow);
}
```

- [ ] **Step 3: Commit**
```bash
git add src/app.css
git commit -m "phase6: shared modal chrome — slide-up animation, blur(4px) backdrop, accent-glow focus ring"
```

---

## Task 2: Confirm dialog — fix hardcoded colours in scoped styles

**Files:**
- Modify: `src/lib/components/ui/ConfirmDialog.svelte` (scoped `<style>` block)

Read the full file first. Issues to fix:
- `.confirm-btn--danger` sets `color: #fff` — hardcoded white
- `.confirm-btn--danger:hover` uses `background: #f06669` — hardcoded lightened red
- `.confirm-btn--primary` sets `color: #fff` — hardcoded white

- [ ] **Step 1: Replace the `<style>` block in `ConfirmDialog.svelte`**

Replace the full scoped `<style>` block with:

```css
<style>
  .confirm-title {
    font-size: var(--text-lg);
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text-primary);
    margin-bottom: var(--space-3);
  }

  .confirm-body {
    font-size: var(--text-base);
    color: var(--text-secondary);
    line-height: 1.5;
    margin-bottom: var(--space-4);
  }

  .confirm-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
  }

  .confirm-btn {
    padding: 7px 18px;
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
    font-weight: 600;
    font-family: inherit;
    cursor: pointer;
    border: none;
    transition: all var(--transition-fast);
  }

  .confirm-btn--danger {
    background: var(--red);
    color: white;
  }
  .confirm-btn--danger:hover {
    background: color-mix(in srgb, var(--red) 80%, white);
  }

  .confirm-btn--primary {
    background: var(--accent);
    color: white;
  }
  .confirm-btn--primary:hover {
    background: var(--accent-hover);
  }
</style>
```

- [ ] **Step 2: Commit**
```bash
git add src/lib/components/ui/ConfirmDialog.svelte
git commit -m "phase6: ConfirmDialog — replace hardcoded #fff and #f06669 with CSS tokens"
```

---

## Task 3: Dropdown upgrade — `app.css` + `Dropdown.svelte` scoped style

**Files:**
- Modify: `src/app.css` (~lines 2318–2357, the `/* ===== DROPDOWN ===== */` section)
- Modify: `src/lib/components/ui/Dropdown.svelte` (scoped `<style>` block)

Read both files. The global `.dropdown` uses `var(--bg-secondary)` — spec wants `var(--bg-card)` (darker). Spec adds `backdrop-filter: blur(8px)`. There is no `.dropdown-separator` class — add it.

- [ ] **Step 1: Upgrade the global `/* ===== DROPDOWN ===== */` block in `app.css`**

Replace the block from `.dropdown {` through `.dropdown-item:hover` with:

```css
/* ===== DROPDOWN ===== */

.dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  margin-top: var(--space-1);
  padding: var(--space-1);
  border-radius: var(--radius-lg);
  border: 1px solid var(--border-strong);
  background: var(--bg-card);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  box-shadow: var(--shadow-lg);
  z-index: 20;
  min-width: 140px;
  animation: dropdown-in var(--transition-fast) ease;
}

@keyframes dropdown-in {
  from { opacity: 0; transform: translateY(-4px) scale(0.97); }
  to   { opacity: 1; transform: translateY(0) scale(1); }
}

.dropdown-item {
  width: 100%;
  padding: 7px var(--space-3);
  text-align: left;
  font-size: var(--text-sm);
  cursor: pointer;
  border: none;
  border-radius: var(--radius-md);
  background: transparent;
  color: var(--text-primary);
  display: flex;
  align-items: center;
  gap: var(--space-2);
  transition: background var(--transition-fast);
  font-family: inherit;
}

.dropdown-item:hover { background: var(--bg-hover); }

.dropdown-item--danger {
  color: var(--red);
}
.dropdown-item--danger:hover {
  background: var(--red-dim);
  color: var(--red);
}

.dropdown-separator {
  height: 1px;
  background: var(--border);
  margin: var(--space-1) 0;
}
```

- [ ] **Step 2: Upgrade `Dropdown.svelte` scoped `.dropdown-panel`**

Replace the full scoped `<style>` block with:

```css
<style>
  .dropdown-wrapper {
    position: relative;
  }

  .dropdown-panel {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: var(--space-1);
    padding: var(--space-1);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border-strong);
    background: var(--bg-card);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    box-shadow: var(--shadow-lg);
    z-index: 20;
    min-width: 140px;
    animation: dropdown-in var(--transition-fast) ease;
  }

  @keyframes dropdown-in {
    from { opacity: 0; transform: translateY(-4px) scale(0.97); }
    to   { opacity: 1; transform: translateY(0) scale(1); }
  }
</style>
```

- [ ] **Step 3: Commit**
```bash
git add src/app.css src/lib/components/ui/Dropdown.svelte
git commit -m "phase6: Dropdown — blur(8px) backdrop, bg-card surface, border-strong, dropdown-separator class"
```

---

## Task 4: Search / command palette — `app.css` SEARCH section

**Files:**
- Modify: `src/app.css` (~lines 4337–4437, the `/* ===== SEARCH ===== */` section)

Read the current SEARCH section. Issues:
- `.search-overlay` uses `var(--bg-overlay)` (`rgba(0,0,0,0.7)`) — spec says `rgba(0,0,0,0.6)`. Override directly.
- `.search-overlay` animation: upgrade to use `backdropIn var(--transition-fast)`.
- `.search-box` animation: change to `modalSlideUp var(--transition-base)` for consistency with Task 1.
- `padding: 10px` on `.search-result-item` → `var(--space-3)`.
- `font-size: 10.5px` footer hint → `var(--text-xs)`.
- `padding: 8px var(--space-4)` footer → `var(--space-2) var(--space-4)`.
- `gap: 4px` footer hint → `var(--space-1)`.
- `border-radius: 2px` mark → `var(--radius-sm)`.

- [ ] **Step 1: Replace the `/* ===== SEARCH ===== */` block in `app.css`**

Replace everything from `.search-overlay {` through `.search-footer-key {` (inclusive) with:

```css
/* ===== SEARCH ===== */

.search-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 12vh;
  z-index: 100;
  animation: backdropIn var(--transition-fast) ease;
}

.search-box {
  width: 600px;
  max-width: 90%;
  max-height: 480px;
  background: var(--bg-card);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-xl);
  overflow: hidden;
  box-shadow: var(--shadow-xl), 0 0 0 1px var(--accent-border);
  display: flex;
  flex-direction: column;
  animation: modalSlideUp var(--transition-base) cubic-bezier(0.16, 1, 0.3, 1);
}

.search-input {
  flex: 1;
  background: none;
  border: none;
  color: var(--text-primary);
  font-size: var(--text-lg);
  font-family: inherit;
  outline: none;
}

.search-input::placeholder { color: var(--text-muted); }

.search-results {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-2);
  border-top: 1px solid var(--border);
}

.search-result-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3);
  border-radius: var(--radius);
  cursor: pointer;
  transition: background var(--transition-fast);
  width: 100%;
  text-align: left;
  border: none;
  background: transparent;
  color: var(--text-primary);
  font-family: inherit;
}

.search-result-item:hover,
.search-result-item--selected { background: var(--accent-dim); }

.search-result-item mark {
  background: var(--accent-dim);
  color: var(--accent);
  border-radius: var(--radius-sm);
  padding: 0 2px;
}

.search-footer {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  padding: var(--space-2) var(--space-4);
  border-top: 1px solid var(--border);
  background: var(--bg-raised);
  flex-shrink: 0;
}

.search-footer-hint {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  font-size: var(--text-xs);
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

.search-result-icon {
  width: 28px;
  height: 28px;
  border-radius: var(--radius);
  display: grid;
  place-items: center;
  font-size: var(--text-xs);
  font-weight: 700;
  flex-shrink: 0;
}

.search-result-title {
  font-size: var(--text-base);
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--text-primary);
}

.search-result-snippet {
  font-size: var(--text-xs);
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.search-result-type {
  font-size: 10px;
  color: var(--text-muted);
  font-family: var(--font-mono);
}

.search-group-label {
  font-size: 10px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
  padding: var(--space-2) var(--space-3) var(--space-1);
}

.search-empty-state {
  padding: var(--space-8);
  text-align: center;
  font-size: var(--text-base);
  color: var(--text-muted);
}
```

- [ ] **Step 2: Commit**
```bash
git add src/app.css
git commit -m "phase6: SearchBar overlay — rgba(0,0,0,0.6) backdrop, slideUp animation, token cleanup"
```

---

## Task 5: Create Feature modal — extract inline styles to CSS classes

**Files:**
- Modify: `src/lib/components/CreateFeatureModal.svelte` (HTML template)
- Modify: `src/app.css` (`/* ===== CREATE FEATURE MODAL ===== */` section)

Read `CreateFeatureModal.svelte` lines 158–308 first. Inline `style=` blocks to extract:

1. `<h2 style="font-size: 16px; font-weight: 700; letter-spacing: -0.02em; color: var(--text-primary); margin-bottom: 24px;">` → `.create-modal-header`
2. `<div style="display: flex; flex-direction: column; gap: 16px;">` → `.create-modal-body`
3. `<div style="font-size: 12px; color: var(--red); padding: 6px 10px; background: var(--red-dim); border-radius: 6px; border: 1px solid rgba(229,72,77,0.2);">` → `.create-error-msg`
4. `<div style="display: flex; justify-content: flex-end; gap: 8px; margin-top: 24px;">` → `.create-modal-footer`
5. Remove inline padding overrides from cancel/submit buttons (handled in CSS).
6. Remove redundant `max-height: 85vh; overflow-y: auto;` from `.modal-content style=` (already in global rule).

NOTE: `style=` attributes that contain runtime data (chip colors `background: {icon.bg}`) must stay as `style=` — only extract static styles.

- [ ] **Step 1: Add new CSS classes to `app.css` inside the CREATE FEATURE MODAL section**

Read `app.css` around line 4050 to find the `/* ===== CREATE FEATURE MODAL ===== */` section. Prepend these new rules before `.create-status-row`:

```css
.create-modal-header {
  font-size: var(--text-lg);
  font-weight: 700;
  letter-spacing: -0.02em;
  color: var(--text-primary);
  margin-bottom: var(--space-6);
}

.create-modal-body {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.create-modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
  margin-top: var(--space-6);
}

.create-modal-footer .btn-subtle {
  padding: 7px var(--space-4);
}

.create-modal-footer .btn-new {
  width: auto;
  padding: 7px 18px;
}

.create-error-msg {
  font-size: var(--text-sm);
  color: var(--red);
  padding: var(--space-2) var(--space-3);
  background: var(--red-dim);
  border-radius: var(--radius-md);
  border: 1px solid color-mix(in srgb, var(--red) 20%, transparent);
}
```

- [ ] **Step 2: Update `CreateFeatureModal.svelte` HTML template**

Make these surgical replacements:

**`<h2>` title** — change:
```svelte
<h2 style="font-size: 16px; font-weight: 700; letter-spacing: -0.02em; color: var(--text-primary); margin-bottom: 24px;">New Feature</h2>
```
to:
```svelte
<h2 class="create-modal-header">New Feature</h2>
```

**Body wrapper `<div>`** — change:
```svelte
<div style="display: flex; flex-direction: column; gap: 16px;">
```
to:
```svelte
<div class="create-modal-body">
```

**Error message `<div>`** — change:
```svelte
<div style="font-size: 12px; color: var(--red); padding: 6px 10px; background: var(--red-dim); border-radius: 6px; border: 1px solid rgba(229,72,77,0.2);">
```
to:
```svelte
<div class="create-error-msg">
```

**Footer `<div>`** — change:
```svelte
<div style="display: flex; justify-content: flex-end; gap: 8px; margin-top: 24px;">
```
to:
```svelte
<div class="create-modal-footer">
```

**Cancel button** — remove the style attr:
```svelte
<button class="btn-subtle" onclick={onClose}>Cancel</button>
```

**Submit button** — remove the style attr:
```svelte
<button class="btn-new" onclick={handleSubmit} disabled={creating}>
```

**`.modal-content` wrapper** — remove redundant style props, keep only width:
```svelte
<div class="modal-content" style="width: 560px;">
```

- [ ] **Step 3: Commit**
```bash
git add src/app.css src/lib/components/CreateFeatureModal.svelte
git commit -m "phase6: CreateFeatureModal — extract inline styles to CSS classes, token cleanup"
```

---

## Task 6: Settings modal — fix scoped style issues

**Files:**
- Modify: `src/lib/components/SettingsModal.svelte` (scoped `<style>` block)

Read the full `<style>` block at the end of `SettingsModal.svelte`. Issues to fix:
1. `var(--yellow)` is used but undefined in `app.css` `:root` — replace with `var(--amber)`.
2. `font-family: 'JetBrains Mono', monospace` hardcoded in code elements — replace with `var(--font-mono)`.
3. `var(--text)` is used but undefined — replace with `var(--text-primary)`.
4. `border-radius: 3px` → `var(--radius-sm)`, `border-radius: 6px` → `var(--radius-md)`.

- [ ] **Step 1: Fix `.extension-auth-hint` rule**

Find and replace the `.extension-auth-hint` rule:

Current (approximate):
```css
  .extension-auth-hint {
    display: flex;
    gap: 8px;
    font-size: 11.5px;
    line-height: 1.55;
    color: var(--text-secondary);
    background: color-mix(in srgb, var(--yellow) 8%, transparent);
    border: 1px solid color-mix(in srgb, var(--yellow) 25%, transparent);
    border-radius: 6px;
    padding: 10px 12px;
  }
```

Replace with:
```css
  .extension-auth-hint {
    display: flex;
    gap: var(--space-2);
    font-size: 11.5px;
    line-height: 1.55;
    color: var(--text-secondary);
    background: color-mix(in srgb, var(--amber) 8%, transparent);
    border: 1px solid color-mix(in srgb, var(--amber) 25%, transparent);
    border-radius: var(--radius-md);
    padding: var(--space-3);
  }
```

- [ ] **Step 2: Fix `.extension-auth-hint code`**

Current:
```css
  .extension-auth-hint code {
    font-family: 'JetBrains Mono', monospace;
    background: var(--bg-hover);
    padding: 1px 4px;
    border-radius: 3px;
    font-size: 10.5px;
  }
```
Replace with:
```css
  .extension-auth-hint code {
    font-family: var(--font-mono);
    background: var(--bg-hover);
    padding: 1px 4px;
    border-radius: var(--radius-sm);
    font-size: var(--text-xs);
  }
```

- [ ] **Step 3: Fix `.extension-card__hint code`**

Current:
```css
  .extension-card__hint code {
    font-family: 'JetBrains Mono', monospace;
    background: var(--bg-hover);
    padding: 1px 4px;
    border-radius: 3px;
  }
```
Replace with:
```css
  .extension-card__hint code {
    font-family: var(--font-mono);
    background: var(--bg-hover);
    padding: 1px 4px;
    border-radius: var(--radius-sm);
  }
```

- [ ] **Step 4: Fix `.extension-card__section-title` undefined token**

Find and replace `color: var(--text)` with `color: var(--text-primary)`:
```css
  .extension-card__section-title {
    font-size: 11.5px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 4px 0;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
```

- [ ] **Step 5: Add `.settings-font-preview` CSS class**

Add this to the scoped `<style>` block after `.settings-section-title`:
```css
  .settings-font-preview {
    margin-top: var(--space-2);
    padding: var(--space-3) var(--space-4);
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    font-size: var(--text-base);
    color: var(--text-secondary);
    line-height: 1.6;
    transition: font-family var(--transition-base);
  }
```

- [ ] **Step 6: Add font preview HTML to Appearance tab**

In the `{#if activeTab === "appearance"}` block, after each font `<select>` and its description `<div>`, add a preview `<p>`. For UI font selector:
```svelte
<p class="settings-font-preview" style="font-family: {uiFont || 'var(--font-ui)'};">
  The quick brown fox jumps over the lazy dog — 0123456789
</p>
```
For mono font selector:
```svelte
<p class="settings-font-preview" style="font-family: {monoFont || 'var(--font-mono)'};">
  const feature = await getFeature(id); // 0123456789
</p>
```

The `style=` attribute for `font-family` is appropriate here — it's runtime-reactive data (selected font choice), not a static style.

- [ ] **Step 7: Commit**
```bash
git add src/lib/components/SettingsModal.svelte
git commit -m "phase6: SettingsModal — fix var(--yellow)→amber, font-mono tokens, var(--text)→text-primary, font preview"
```

---

## Task 7: SearchBar — extract inline styles to CSS classes

**Files:**
- Modify: `src/lib/components/SearchBar.svelte` (HTML template)
- The CSS classes were already added to `app.css` in Task 4.

Read `SearchBar.svelte`. The inline `style=` attributes to extract (using classes added in Task 4):

1. Icon badge `<div>` — keep only the data-driven `background` and `color` as `style=`; move static sizing/layout to `class="search-result-icon"`.
2. Title `<div>` — extract to `class="search-result-title"`.
3. Snippet `<div>` — extract to `class="search-result-snippet"`.
4. Type `<span>` with hardcoded `font-family: 'JetBrains Mono', monospace` — extract to `class="search-result-type"`.
5. Group label `<div>` — extract to `class="search-group-label"`.
6. Empty state `<div>` — extract to `class="search-empty-state"`.

- [ ] **Step 1: Update `SearchBar.svelte` template**

**Icon badge `<div>`** — change from:
```svelte
<div style="width: 28px; height: 28px; border-radius: 6px; display: grid; place-items: center; font-size: 11px; font-weight: 700; flex-shrink: 0; background: {icon.bg}; color: {icon.color};">
```
to:
```svelte
<div class="search-result-icon" style="background: {icon.bg}; color: {icon.color};">
```

**Title `<div>`** — change from:
```svelte
<div style="font-size: 13px; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; color: var(--text-primary);">
```
to:
```svelte
<div class="search-result-title">
```

**Snippet `<div>`** — change from:
```svelte
<div style="font-size: 11px; color: var(--text-muted); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">
```
to:
```svelte
<div class="search-result-snippet">
```

**Type `<span>`** — change from:
```svelte
<span style="font-size: 10px; color: var(--text-muted); font-family: 'JetBrains Mono', monospace;">{type}</span>
```
to:
```svelte
<span class="search-result-type">{type}</span>
```

**Group label `<div>`** (both occurrences) — change from:
```svelte
<div style="font-size: 10px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.06em; padding: 8px 10px 4px;">
```
to:
```svelte
<div class="search-group-label">
```

**Empty state `<div>`** (both "No results" and "No features yet") — change from:
```svelte
<div style="padding: 32px; text-align: center; font-size: 13px; color: var(--text-muted);">
```
to:
```svelte
<div class="search-empty-state">
```

- [ ] **Step 2: Commit**
```bash
git add src/lib/components/SearchBar.svelte
git commit -m "phase6: SearchBar — extract inline styles to CSS classes, font-mono token, group label"
```

---

## Task 8: Final verification

- [ ] **Step 1: Run frontend tests**
```bash
npm run test
```
Expected: all 34 tests pass.

- [ ] **Step 2: Rust check**
```bash
cd src-tauri && cargo check && cd ..
```
Expected: no errors.

- [ ] **Step 3: Visual smoke test**

With `npm run tauri dev` running:
1. Open a modal (create feature) — verify slide-up animation, `rgba(0,0,0,0.6)` backdrop with blur ✓
2. ConfirmDialog — verify red/accent buttons look correct, no `#fff` warnings ✓
3. Dropdown (click a status badge or three-dot menu) — verify blurred dark surface, smooth open animation ✓
4. Search overlay (Ctrl+T) — verify `rgba(0,0,0,0.6)` backdrop, slide-up box, footer keyboard hints ✓
5. Settings modal — verify Appearance tab font previews update live when font selection changes ✓
6. No visual regressions in: sidebar, workspace tabs, feature list, bento grid ✓

- [ ] **Step 4: Commit any smoke test fixes**
```bash
git add src/app.css src/lib/components/*.svelte
git commit -m "phase6: smoke test fixes"
```

- [ ] **Step 5: Merge to master**
```bash
git checkout master
git merge feat/phase6-modals-overlays --no-ff -m "feat: phase 6 modals and overlays — token-based CSS, slide-up animations, inline style extraction"
git branch -d feat/phase6-modals-overlays
```

---

## Commit Summary

| # | Commit message | Files |
|---|---|---|
| 1 | `phase6: shared modal chrome — slide-up animation, blur(4px) backdrop, accent-glow focus ring` | `app.css` |
| 2 | `phase6: ConfirmDialog — replace hardcoded #fff and #f06669 with CSS tokens` | `ConfirmDialog.svelte` |
| 3 | `phase6: Dropdown — blur(8px) backdrop, bg-card surface, border-strong, dropdown-separator class` | `app.css`, `Dropdown.svelte` |
| 4 | `phase6: SearchBar overlay — rgba(0,0,0,0.6) backdrop, slideUp animation, token cleanup` | `app.css` |
| 5 | `phase6: CreateFeatureModal — extract inline styles to CSS classes, token cleanup` | `app.css`, `CreateFeatureModal.svelte` |
| 6 | `phase6: SettingsModal — fix var(--yellow)→amber, font-mono tokens, var(--text)→text-primary, font preview` | `SettingsModal.svelte` |
| 7 | `phase6: SearchBar — extract inline styles to CSS classes, font-mono token, group label` | `app.css`, `SearchBar.svelte` |
| 8 | `phase6: smoke test fixes` | various |
