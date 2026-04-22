# Prototype Polish — Full UI Alignment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring the live app to full visual parity with `docs/design-prototype.html` — frameless window, shell FX, rounded gradient frame, larger radii everywhere, smoother easing, richer card hover effects, and content-level size/spacing fixes.

**Architecture:** All changes are CSS + minimal App.svelte HTML structure + one Tauri config update. No new components, no Rust changes. Six CSS-focused tasks followed by one structural task for the frameless window. Tests confirm no regressions after each task.

**Tech Stack:** Svelte 5, TypeScript, Tauri 2, CSS custom properties (`src/app.css`)

---

## File Map

| File | Changes |
|------|---------|
| `src-tauri/tauri.conf.json` | `decorations: false`, `transparent: true` |
| `src/App.svelte` | Shell FX div, app-frame wrapper, window controls, drag region, resize offset constants |
| `src/app.css` | CSS tokens, shell FX, app frame, icon rail, bento grid/cards, stat cards, title, session card, live ring, plans border |

---

### Task 1: Global CSS token updates

**Files:**
- Modify: `src/app.css` (`:root` block lines 4–92, `html, body` rule ~line 108)

- [ ] **Step 1: Update radius scale in `:root`**

Find the `/* ── Radius ── */` block (~line 61) and replace:

```css
  /* ── Radius ── */
  --radius-sm:  7px;
  --radius-md:  11px;
  --radius:     15px;
  --radius-lg:  15px;
  --radius-xl:  20px;
  --radius-2xl: 26px;
  --radius-full: 9999px;
```

- [ ] **Step 2: Update border opacity in `:root`**

Find the `/* ── Borders ── */` block (~line 16) and replace these two lines:

```css
  --border:        rgba(255,255,255,0.06);
  --border-strong: rgba(255,255,255,0.10);
```

- [ ] **Step 3: Update transition easing in `:root`**

Find the `/* ── Transitions ── */` block (~line 88) and replace:

```css
  /* ── Transitions ── */
  --transition-fast: 0.12s cubic-bezier(0.16, 1, 0.3, 1);
  --transition-base: 0.18s cubic-bezier(0.16, 1, 0.3, 1);
  --transition-slow: 0.32s cubic-bezier(0.16, 1, 0.3, 1);
```

- [ ] **Step 4: Update card hover shadow in the second `:root` block**

Find the `/* ── Card hover system ── */` block (~line 101) and replace:

```css
  /* ── Card hover system ── */
  --card-hover-y:      -2px;
  --card-hover-shadow: 0 14px 40px rgba(0,0,0,0.38), 0 2px 8px rgba(0,0,0,0.18);
```

- [ ] **Step 5: Update body font size and line-height**

Find the `html, body {` rule (~line 108). Change these two properties:

```css
  font-size: 13.5px;
  line-height: 1.55;
```

(Keep all other properties — `-webkit-font-smoothing`, `height`, `width`, etc. — unchanged.)

Also change `--font-size: 13px;` in `:root` to `--font-size: 13.5px;`.

- [ ] **Step 6: Run tests**

```bash
cd /d/LittleBrushGames/FeatureHub && npm run test
```

Expected: 38 tests pass across 6 files.

- [ ] **Step 7: Commit**

```bash
git add src/app.css
git commit -m "style: update design tokens to match prototype (radius, borders, easing, font)"
```

---

### Task 2: Shell FX + App frame CSS

**Files:**
- Modify: `src/app.css` (after `#app` rule ~line 158; update `html, body` background; update `.app-shell`)

- [ ] **Step 1: Make body background transparent**

Find `html, body {` rule (~line 108). Change `background: var(--bg-primary);` to `background: transparent;`.

- [ ] **Step 2: Add shell FX and app frame CSS**

Find the `/* ===== CORE LAYOUT =====` comment (~line 160). Insert the following block **before** it:

```css
/* ===== SHELL FX ===== */

.shell-fx {
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 0;
  overflow: hidden;
}

.shell-fx::before {
  content: '';
  position: absolute;
  inset: 0;
  background:
    radial-gradient(ellipse 70% 55% at 75% -5%,  rgba(77,124,255,0.08)  0%, transparent 55%),
    radial-gradient(ellipse 50% 45% at 5%  80%,  rgba(139,92,246,0.06)  0%, transparent 50%),
    radial-gradient(ellipse 35% 35% at 95% 95%,  rgba(34,211,238,0.04)  0%, transparent 50%);
}

.ray {
  position: absolute;
  inset: 0;
  opacity: 0.02;
  background:
    linear-gradient(122deg, transparent 28%, rgba(77,124,255,1) 46%, transparent 50%),
    linear-gradient(122deg, transparent 52%, rgba(139,92,246,0.7) 64%, transparent 68%);
  animation: ray-drift 24s ease-in-out infinite alternate;
}

@keyframes ray-drift {
  from { transform: translateX(-12px) rotate(-0.2deg); }
  to   { transform: translateX(12px)  rotate(0.2deg); }
}

/* ===== APP FRAME ===== */

.app-frame {
  position: relative;
  z-index: 1;
  height: 100vh;
  padding: 10px;
  display: flex;
  align-items: stretch;
  box-sizing: border-box;
}
```

- [ ] **Step 3: Update `.app-shell` to fill the frame and show gradient border**

Find `.app-shell {` (~line 162). Replace the entire rule with:

```css
.app-shell {
  display: flex;
  flex: 1;
  min-width: 0;
  opacity: 1;
  transition: opacity 0.15s ease;
  border-radius: var(--radius-2xl);
  border: 1px solid transparent;
  background-image:
    linear-gradient(var(--bg-primary), var(--bg-primary)),
    linear-gradient(135deg,
      rgba(77,124,255,0.28) 0%,
      rgba(139,92,246,0.18) 40%,
      rgba(34,211,238,0.10) 100%);
  background-origin: border-box;
  background-clip: padding-box, border-box;
  box-shadow: 0 48px 120px rgba(0,0,0,0.75);
  overflow: hidden;
}

.app-shell.storage-switching {
  opacity: 0;
}
```

- [ ] **Step 4: Run tests**

```bash
cd /d/LittleBrushGames/FeatureHub && npm run test
```

Expected: 38 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/app.css
git commit -m "style: add shell FX background and app frame gradient border"
```

---

### Task 3: Tauri config + App.svelte structural changes

**Files:**
- Modify: `src-tauri/tauri.conf.json` (window config lines 13–21)
- Modify: `src/App.svelte` (imports ~line 1, constants ~line 79, template ~line 392)

- [ ] **Step 1: Update Tauri window config**

In `src-tauri/tauri.conf.json`, replace the `"windows"` array content:

```json
    "windows": [
      {
        "title": "Feature Hub",
        "width": 1200,
        "height": 800,
        "center": true,
        "decorations": false,
        "transparent": true,
        "shadow": true
      }
    ],
```

- [ ] **Step 2: Add Tauri window API import to App.svelte**

At the top of the `<script>` block in `src/App.svelte`, after the existing imports, add:

```typescript
  import { getCurrentWindow } from "@tauri-apps/api/window";
```

- [ ] **Step 3: Add window control functions and update resize constants**

Find the `// Resizable sidebar` block (~line 75). Replace the constants and add window functions:

```typescript
  // Resizable sidebar
  const SIDEBAR_MIN = 200;
  const SIDEBAR_MAX = 500;
  const SIDEBAR_DEFAULT = 272;
  const ICON_RAIL_WIDTH = 60;
  const APP_FRAME_PADDING = 10;
  let sidebarWidth = $state(
    parseInt(localStorage.getItem("featurehub:sidebarWidth") || "") || SIDEBAR_DEFAULT
  );
  let isResizing = $state(false);

  function onResizeStart(e: MouseEvent) {
    e.preventDefault();
    isResizing = true;
    const onMove = (ev: MouseEvent) => {
      const w = Math.min(SIDEBAR_MAX, Math.max(SIDEBAR_MIN, ev.clientX - ICON_RAIL_WIDTH - APP_FRAME_PADDING));
      sidebarWidth = w;
    };
    const onUp = () => {
      isResizing = false;
      localStorage.setItem("featurehub:sidebarWidth", String(sidebarWidth));
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }

  const appWindow = getCurrentWindow();
  function minimizeWindow() { appWindow.minimize(); }
  function closeWindow() { appWindow.close(); }
```

- [ ] **Step 4: Update App.svelte template — shell FX div + app-frame wrapper**

Find the `{:else}` branch (~line 398). It currently starts with:

```svelte
{:else}
  <div class="app-shell" class:resizing={isResizing} class:storage-switching={storageSwitching}>
```

Replace with:

```svelte
{:else}
  <div class="shell-fx" aria-hidden="true">
    <div class="ray"></div>
  </div>
  <div class="app-frame">
  <div class="app-shell" class:resizing={isResizing} class:storage-switching={storageSwitching}>
```

Then find the closing `</div>` of `.app-shell` (the one that closes the entire app-shell — it's the last `</div>` before the `{/if}` that closes the `{:else if !storageChecked}` chain). Add a closing `</div>` after it for `.app-frame`:

```svelte
  </div><!-- .app-shell -->
  </div><!-- .app-frame -->
{/if}
```

- [ ] **Step 5: Add drag region + window controls to icon rail**

Find the `.icon-rail` div in the template. Add `data-tauri-drag-region` to the logo div, and add a window controls block between the logo and the first nav button:

```svelte
    <div class="icon-rail">
      <div class="icon-rail-logo" data-tauri-drag-region>FH</div>
      <div class="icon-rail-win-controls">
        <button class="icon-rail-wc-btn" aria-label="Minimize" onclick={minimizeWindow}>
          <svg width="10" height="2" viewBox="0 0 10 2" fill="currentColor"><rect width="10" height="1.5" rx="0.75"/></svg>
        </button>
        <button class="icon-rail-wc-btn icon-rail-wc-btn--close" aria-label="Close" onclick={closeWindow}>
          <svg width="9" height="9" viewBox="0 0 9 9" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><path d="M1 1l7 7M8 1l-7 7"/></svg>
        </button>
      </div>

      <button class="icon-rail-btn" class:icon-rail-btn--on={activeView === 'dashboard'} ...
```

- [ ] **Step 6: Run tests**

```bash
cd /d/LittleBrushGames/FeatureHub && npm run test
```

Expected: 38 tests pass. (The Tauri window change can only be visually verified with `npm run tauri dev`.)

- [ ] **Step 7: Run Rust check**

```bash
cd /d/LittleBrushGames/FeatureHub/src-tauri && cargo check
```

Expected: `Finished` with no errors.

- [ ] **Step 8: Commit**

```bash
cd /d/LittleBrushGames/FeatureHub
git add src-tauri/tauri.conf.json src/App.svelte
git commit -m "feat: frameless transparent window with shell FX, drag region, window controls"
```

---

### Task 4: Icon rail sizing + tooltip arrow + window control CSS

**Files:**
- Modify: `src/app.css` (`.icon-rail` block ~line 176, `.icon-rail-btn::after` ~line 225)

- [ ] **Step 1: Update icon rail container sizing**

Find `.icon-rail {` (~line 176). Replace the rule:

```css
.icon-rail {
  width: 60px;
  min-width: 60px;
  height: 100%;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 14px 0 10px;
  gap: 3px;
  user-select: none;
  flex-shrink: 0;
  z-index: 10;
}
```

- [ ] **Step 2: Update icon rail logo sizing**

Find `.icon-rail-logo {` (~line 192). Replace the rule:

```css
.icon-rail-logo {
  width: 34px;
  height: 34px;
  background: var(--grad-primary);
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 700;
  color: #fff;
  letter-spacing: -0.04em;
  margin-bottom: 8px;
  flex-shrink: 0;
  box-shadow: 0 2px 12px rgba(77, 124, 255, 0.3);
  cursor: default;
}
```

- [ ] **Step 3: Update icon rail button sizing and add tooltip arrow**

Find `.icon-rail-btn {` (~line 209). Replace the rule and add `::before` for the arrow:

```css
.icon-rail-btn {
  width: 38px;
  height: 38px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background var(--transition-fast), color var(--transition-fast);
  color: var(--text-muted);
  border: 1px solid transparent;
  background: none;
  flex-shrink: 0;
  position: relative;
}

/* Tooltip arrow */
.icon-rail-btn::before {
  content: '';
  position: absolute;
  left: calc(100% + 6px);
  top: 50%;
  transform: translateY(-50%) scale(0.85);
  border: 5px solid transparent;
  border-right-color: var(--border-strong);
  pointer-events: none;
  opacity: 0;
  transition: opacity var(--transition-fast), transform var(--transition-fast);
  z-index: 200;
}

.icon-rail-btn:hover::before {
  opacity: 1;
  transform: translateY(-50%) scale(1);
}
```

- [ ] **Step 4: Update tooltip text offset to clear the arrow**

Find `.icon-rail-btn::after {` (~line 225). Change the `left` value:

```css
  left: calc(100% + 16px);
```

(was `calc(100% + 10px)`)

- [ ] **Step 5: Update separator width**

Find `.icon-rail-sep {` (~line 267). Change:

```css
  width: 28px;
  margin: 6px 0;
```

- [ ] **Step 6: Add window control button CSS**

Find the end of the icon rail section (after `.icon-rail-spacer { flex: 1; }`). Add:

```css
.icon-rail-win-controls {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  margin-bottom: var(--space-2);
}

.icon-rail-wc-btn {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  border: 1px solid var(--border);
  background: var(--bg-raised);
  color: var(--text-faint);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all var(--transition-fast);
  padding: 0;
  flex-shrink: 0;
}

.icon-rail-wc-btn:hover {
  color: var(--text-secondary);
  border-color: var(--border-strong);
  background: var(--bg-hover);
}

.icon-rail-wc-btn--close:hover {
  background: rgba(248, 113, 113, 0.15);
  border-color: rgba(248, 113, 113, 0.3);
  color: #f87171;
}
```

- [ ] **Step 7: Run tests**

```bash
cd /d/LittleBrushGames/FeatureHub && npm run test
```

Expected: 38 tests pass.

- [ ] **Step 8: Commit**

```bash
git add src/app.css
git commit -m "style: icon rail 60px, tooltip arrow, window control buttons"
```

---

### Task 5: Bento grid gap + card polish

**Files:**
- Modify: `src/app.css` (`.bento` ~line 1791, `.bento-card` ~line 1804)

- [ ] **Step 1: Reduce bento grid gap**

Find `.bento {` (~line 1791). Change `gap: var(--space-4);` to:

```css
  gap: 11px;
```

- [ ] **Step 2: Add position, overflow, hover transform, and inner sheen to bento cards**

Find `.bento-card {` (~line 1804). Replace the rule:

```css
.bento-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 17px;
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  transition: border-color var(--transition-fast), transform var(--transition-fast), box-shadow var(--transition-fast);
  min-height: 0;
  position: relative;
  overflow: hidden;
}

.bento-card::after {
  content: '';
  position: absolute;
  top: 0; left: 0; right: 0;
  height: 40px;
  background: linear-gradient(to bottom, rgba(255,255,255,0.035), transparent);
  border-radius: var(--radius-lg) var(--radius-lg) 0 0;
  pointer-events: none;
  z-index: 0;
}

.bento-card:hover {
  border-color: var(--border-strong);
  transform: translateY(-2px);
  box-shadow: var(--card-hover-shadow);
}
```

- [ ] **Step 3: Remove position + overflow from `.bento-card--live`** (now inherited from `.bento-card`)

Find `.bento-card--live {` (~line 1946). Remove the two lines `position: relative;` and `overflow: hidden;` from that rule since they're now on the base `.bento-card`.

- [ ] **Step 4: Run tests**

```bash
cd /d/LittleBrushGames/FeatureHub && npm run test
```

Expected: 38 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/app.css
git commit -m "style: bento grid 11px gap, card hover lift, inner sheen"
```

---

### Task 6: Content detail CSS

**Files:**
- Modify: `src/app.css` (six targeted rule updates)

- [ ] **Step 1: Feature title — 24px**

Find `.detail-header-title {` (~line 1264). Change:

```css
  font-size: 24px;
  line-height: 1.15;
```

- [ ] **Step 2: Stat card — larger icon, number, padding, hover**

Find `.detail-stat-card {` (~line 1344). Change the padding line:

```css
  padding: 9px 13px;
```

Find `.detail-stat-card:hover {` (~line 1370). Change:

```css
.detail-stat-card:hover {
  transform: translateY(-2px);
  border-color: var(--border-strong);
  box-shadow: 0 8px 24px rgba(0,0,0,0.3);
}
```

Find `.detail-stat-card-icon {` (~line 1375). Change:

```css
  width: 30px;
  height: 30px;
```

Find `.detail-stat-card-num {` (~line 1403). Change:

```css
  font-size: 20px;
```

- [ ] **Step 3: Session card text sizes**

Find `.bento-session-title {` (~line 2023). Change:

```css
  font-size: 15px;
  font-weight: 600;
```

Find `.bento-session-summary {` (~line 2060). Change:

```css
  font-size: 12.5px;
```

- [ ] **Step 4: Live ring — wider pulse**

Find `@keyframes bento-pulse {` (~line 2018). Replace:

```css
@keyframes bento-pulse {
  0%, 100% { box-shadow: 0 0 0 0   rgba(52, 211, 153, 0.4); }
  50%       { box-shadow: 0 0 0 8px rgba(52, 211, 153, 0); }
}
```

- [ ] **Step 5: Plans bento card — amber left border**

Find `.bento-card--warn .bento-title {` (~line 2114). Insert a new rule **before** it:

```css
.bento-card--warn {
  border-left: 2.5px solid var(--amber);
}

```

- [ ] **Step 6: Meta row gap**

Find `.detail-meta-row {` (~line 1422). Change `gap: var(--space-2);` to:

```css
  gap: 7px;
```

- [ ] **Step 7: Run tests**

```bash
cd /d/LittleBrushGames/FeatureHub && npm run test
```

Expected: 38 tests pass.

- [ ] **Step 8: Commit**

```bash
git add src/app.css
git commit -m "style: feature title 24px, stat cards, session card text, live ring, plans border"
```

---

### Task 7: Final verification

**Files:** None modified.

- [ ] **Step 1: Run frontend tests**

```bash
cd /d/LittleBrushGames/FeatureHub && npm run test
```

Expected: 38 tests pass across 6 test files.

- [ ] **Step 2: Run Rust check**

```bash
cd /d/LittleBrushGames/FeatureHub/src-tauri && cargo check
```

Expected: `Finished` with no errors.

- [ ] **Step 3: Confirm git log is clean**

```bash
cd /d/LittleBrushGames/FeatureHub && git log --oneline -8
```

Expected: 6 new commits visible (tasks 1–6), all on master.

---

## Self-Review

**Spec coverage:**
- ✅ Frameless window: `decorations: false`, `transparent: true` — Task 3
- ✅ Shell FX (rays + radial gradients): `.shell-fx`, `.ray`, `@keyframes ray-drift` — Task 2
- ✅ App frame (gradient border, box-shadow, radius 26px): `.app-frame`, `.app-shell` update — Task 2
- ✅ Window controls (minimize + close): template + CSS — Tasks 3, 4
- ✅ Drag region: `data-tauri-drag-region` on logo — Task 3
- ✅ Border-radius scale updated: Task 1
- ✅ Border opacity updated: Task 1
- ✅ Easing curves updated: Task 1
- ✅ Body font 13.5px: Task 1
- ✅ Card hover shadow updated: Task 1
- ✅ Icon rail 60px, buttons 38px, logo 34px: Task 4
- ✅ Tooltip arrow `::before`: Task 4
- ✅ Separator 28px: Task 4
- ✅ Window control button CSS: Task 4
- ✅ Bento grid gap 11px: Task 5
- ✅ Bento card padding 17px, position + overflow, hover transform + shadow: Task 5
- ✅ Bento card inner sheen `::after`: Task 5
- ✅ `position: relative; overflow: hidden` removed from `--live` (now on base): Task 5
- ✅ Feature title 24px: Task 6
- ✅ Stat card icon 30px, number 20px, padding 9px 13px: Task 6
- ✅ Session card title 15px, summary 12.5px: Task 6
- ✅ Live ring pulse 8px: Task 6
- ✅ Plans card amber left border: Task 6
- ✅ Meta row gap 7px: Task 6

**Placeholder scan:** No TBDs. All values are exact CSS.

**Type consistency:** No shared types across tasks — pure CSS changes.
