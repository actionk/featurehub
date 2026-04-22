# Prototype Polish — Full UI Alignment Design

## Goal

Bring the live app to full visual parity with `docs/design-prototype.html`. Every section of the diff has been catalogued; this spec covers all changes.

## Architecture

All changes are CSS + minimal HTML structure. No new Rust, no new components. Two exceptions: (1) `tauri.conf.json` gains `decorations: false` + `transparent: true` for the frameless window effect, and (2) `App.svelte` gains a `.shell-fx` wrapper and `.app-frame` container. Everything else is `src/app.css` and minor template tweaks in existing Svelte files.

## Tech Stack

Svelte 5, TypeScript, Tauri 2, CSS custom properties (`src/app.css`)

---

## Section 1 — Frameless Window + Shell FX

### Tauri config (`src-tauri/tauri.conf.json`)
```json
"decorations": false,
"transparent": true,
"shadow": true
```

### App.svelte structure changes

Add before the `{#if ...}` chain (outside all conditionals, always rendered):

```svelte
<!-- Shell FX — fixed behind everything -->
<div class="shell-fx" aria-hidden="true">
  <div class="ray"></div>
</div>
```

When the main app shell renders (the `{:else}` branch containing `.app-shell`), wrap the entire `.app-shell` div in:

```svelte
<div class="app-frame">
  <div class="app-shell" ...>
    ...
  </div>
</div>
```

Add `data-tauri-drag-region` to the `.icon-rail-logo` div so the window is draggable from the logo area.

Add Tauri window API imports and two window-control buttons inside `.icon-rail`, between the logo and the nav buttons:

```svelte
<div class="icon-rail-win-controls">
  <button class="icon-rail-wc-btn" aria-label="Minimize" onclick={minimizeWindow}>
    <svg width="10" height="2" viewBox="0 0 10 2"><rect width="10" height="1.5" rx="0.75" fill="currentColor"/></svg>
  </button>
  <button class="icon-rail-wc-btn icon-rail-wc-btn--close" aria-label="Close" onclick={closeWindow}>
    <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1 1l8 8M9 1l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
  </button>
</div>
```

Import from `@tauri-apps/api/window`:
```typescript
import { getCurrentWindow } from "@tauri-apps/api/window";
const appWindow = getCurrentWindow();
function minimizeWindow() { appWindow.minimize(); }
function closeWindow() { appWindow.close(); }
```

### CSS — Shell FX + App frame

```css
body {
  background: transparent;
}

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

.app-shell {
  /* already exists — add these: */
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
  flex: 1;
}
```

### CSS — Window controls

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

---

## Section 2 — Global Design Tokens

Replace existing token values in `:root` in `src/app.css`:

```css
/* Border-radius — match prototype scale */
--radius-sm:  7px;    /* was 4px */
--radius-md:  11px;   /* was 6px */
--radius-lg:  15px;   /* was 12px */
--radius-xl:  20px;   /* was 16px */
--radius-2xl: 26px;   /* new — app frame */

/* Border opacity — match prototype */
--border:        rgba(255,255,255,0.06);  /* was 0.08 */
--border-strong: rgba(255,255,255,0.10);  /* was 0.14 */

/* Easing — prototype's --smooth curve */
--transition-fast: 0.12s cubic-bezier(0.16, 1, 0.3, 1);
--transition-base: 0.18s cubic-bezier(0.16, 1, 0.3, 1);

/* Card hover */
--card-hover-shadow: 0 14px 40px rgba(0,0,0,0.38), 0 2px 8px rgba(0,0,0,0.18);
```

Update `body` rule:
```css
body {
  font-size: 13.5px;   /* was 13px */
  line-height: 1.55;   /* was 1.5 — note: already 1.55 from earlier pass, confirm */
}
```

---

## Section 3 — Icon Rail Sizing + Tooltip Arrow

```css
.icon-rail {
  width: 60px;       /* was 52px */
  min-width: 60px;
  padding: 14px 0 10px;
  gap: 3px;          /* was 2px */
}

.icon-rail-logo {
  width: 34px;       /* was 32px */
  height: 34px;
  font-size: 12px;   /* was 11px */
  margin-bottom: 12px; /* was 10px */
}

.icon-rail-btn {
  width: 38px;       /* was 36px */
  height: 38px;
  border-radius: var(--radius-sm); /* now 7px */
}

/* Tooltip: add arrow (::before) alongside existing ::after */
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

/* Shift tooltip text a bit further right to account for arrow */
.icon-rail-btn::after {
  left: calc(100% + 16px); /* was 10px */
}

.icon-rail-sep {
  width: 28px;  /* was 24px */
}
```

---

## Section 4 — Bento Grid + Card Polish

### Grid gap
In `.bento-grid` rule in `src/app.css`:
```css
.bento-grid {
  gap: 11px;  /* was var(--space-4) = 16px */
}
```

### Card base + sheen + hover
```css
.bento-card {
  padding: 17px;       /* was var(--space-4) = 16px */
  border-radius: var(--radius-lg);  /* now 15px via token */
  transition: transform var(--transition-fast), border-color var(--transition-fast), box-shadow var(--transition-fast);
  position: relative;  /* already set for glow/dot-field — confirm */
}

/* Inner top sheen */
.bento-card::after {
  content: '';
  position: absolute;
  top: 0; left: 0; right: 0;
  height: 40px;
  background: linear-gradient(to bottom, rgba(255,255,255,0.035), transparent);
  border-radius: var(--radius-lg) var(--radius-lg) 0 0;
  pointer-events: none;
}

.bento-card:hover {
  transform: translateY(-2px);
  border-color: var(--border-strong);
  box-shadow: var(--card-hover-shadow);
}
```

Note: `.bento-card-glow` and `.bento-card-dot-field` are already `position: absolute` inside `.bento-card`. The `::after` sheen is also `position: absolute`. z-index ordering: glow/dot-field sit at z-index 0 (default), content wrapper at z-index 1, sheen should sit ABOVE content — add `z-index: 2` to the `::after` sheen. Actually sheen should be BELOW content — keep at default z-index but it doesn't need to be above content since it's just a top-edge highlight. Keep `pointer-events: none` and ensure it doesn't obscure the bento-header. Set `z-index: 0` on `::after` if needed.

---

## Section 5 — Stat Cards

```css
.detail-stat-card {
  padding: 9px 13px;   /* was var(--space-2) var(--space-3) = 8px 12px */
}

.detail-stat-card:hover {
  transform: translateY(-2px);  /* was -1px */
}

.detail-stat-card-icon {
  width: 30px;   /* was 26px */
  height: 30px;
}

.detail-stat-card-num {
  font-size: 20px;  /* was var(--text-xl) = 18px */
}
```

---

## Section 6 — Content Details

### Feature title
In `.detail-header-title`:
```css
font-size: 24px;  /* was var(--text-xl) which was 18px */
line-height: 1.15;
```

### Session card text
```css
.bento-session-title {
  font-size: 15px;   /* was var(--text-sm) = 12px */
  font-weight: 600;
}

.bento-session-summary {
  font-size: 12.5px;  /* was var(--text-xs) = 10.5px */
}
```

### Live ring spread
In `@keyframes live-ring-pulse` (or however the ring animation is named):
```css
/* Final keyframe: box-shadow spread 4px → 8px */
to { box-shadow: 0 0 0 8px rgba(52,211,153,0); }
/* was 0 0 0 4px rgba(52,211,153,0) */
```

### Plans bento card left border accent
```css
.bento-card--warn {
  border-left: 2.5px solid var(--amber);
}
```

### Meta row gap
```css
.detail-meta-row {
  gap: 7px;  /* was var(--space-2) = 8px — minor, but matches prototype */
}
```

---

## File Map

| File | Changes |
|------|---------|
| `src-tauri/tauri.conf.json` | `decorations: false`, `transparent: true`, `shadow: true` |
| `src/App.svelte` | Shell FX div, app-frame wrapper, window controls, drag region, Tauri window import |
| `src/app.css` | CSS tokens, shell FX, app frame, icon rail, bento grid/cards, stat cards, content details |

---

## Self-Review

**Placeholder scan:** No TBDs. All values are exact from prototype.

**Internal consistency:** The `--radius-lg` token change from 12px→15px cascades throughout all uses. This is intentional and matches the prototype. The bento card `::after` sheen needs `z-index` consideration — it must sit at or below z-index 1 (content wrapper) to not obscure text.

**Scope check:** This is a single-pass CSS polish — appropriate for one plan.

**Ambiguity:** The `transparent: true` Tauri config requires the OS to support compositing. On Windows 11 this works. The `data-tauri-drag-region` is a Tauri-specific HTML attribute that enables window dragging on frameless windows.
