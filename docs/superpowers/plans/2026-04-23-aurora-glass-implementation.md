# Aurora Glass Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers-extended-cc:subagent-driven-development (recommended) or superpowers-extended-cc:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Apply the Aurora Glass redesign (see `docs/superpowers/specs/2026-04-23-aurora-glass-design-system.md`) across every surface of FeatureHub without touching feature behavior.

**Architecture:** Token-first rewrite of `src/app.css` (root tokens + new primitive classes), then surface-by-surface adoption — most surfaces inherit automatically because existing class hooks get redefined to match primitives; a small number of components add primitive classes directly on existing markup.

**Tech Stack:** Svelte 5 (runes) + TypeScript + Tauri 2 + plain CSS in `src/app.css` (CSS custom properties, backdrop-filter, radial-gradient, transform/opacity animations). No new deps.

**Verification model:** Visual work — TDD doesn't apply per-class. Each task verifies via:
1. `cargo check` from `src-tauri/` (passes — no Rust changes)
2. `npm run test` (existing Vitest stays green; snapshot updates allowed)
3. `npm run tauri dev` smoke — open the affected surface, confirm look matches spec, confirm interactions still work, confirm no console errors
4. Spec section reference: each task names the spec section it implements

**Spec reference shorthand:** `[spec §<section>]` points to a section in the design spec. Open the spec alongside the plan; do not duplicate token definitions or full primitive bodies in this plan — copy them verbatim from the spec.

---

## File Structure

**Modified files (CSS — bulk of the work):**
- `src/app.css` (~6600 lines today) — token rewrite + new primitives section + targeted rule rewrites

**Modified files (Svelte — narrow markup additions):**
- `src/App.svelte` — add `.aurora-bg` to root
- `src/lib/components/Sidebar.svelte` — class additions on rows/badges
- `src/lib/components/WorkspaceTabBar.svelte` — class additions on tabs
- `src/lib/components/FeatureDetail.svelte` — class additions on tab bar
- `src/lib/components/StatusBadge.svelte` — switch to `.aurora-pill`
- `src/lib/components/TagBadge.svelte` — switch to `.aurora-pill`
- `src/lib/components/SettingsModal.svelte` — adopt modal primitives
- `src/lib/components/StorageSelector.svelte` / `StorageSetup.svelte`
- `src/lib/components/CreateFeatureModal.svelte` / `ExportImportModal.svelte`
- `src/lib/components/ToastContainer.svelte`
- `src/lib/components/SearchBar.svelte`
- `src/lib/components/SessionsPanel.svelte`, `DashboardPanel.svelte`, `GlobalTimeline.svelte`, `InstalledExtensionsPanel.svelte`, `MarkdownPreview.svelte`, `OpenFgaPreview.svelte`
- `src/lib/components/ui/Modal.svelte`, `ConfirmDialog.svelte`, `Dropdown.svelte`, `IconButton.svelte`
- All module files under `src/lib/modules/{ai,links,repos,tasks-notes,files,timeline,board,knowledge}/`

**New files:** none.

---

## Task 0: Branch + baseline smoke

**Goal:** Branch off, confirm app builds + runs, capture starting state.

**Files:**
- None modified

**Acceptance Criteria:**
- [ ] On a fresh branch `aurora-glass`
- [ ] `npm install` succeeds
- [ ] `cargo check` from `src-tauri/` passes
- [ ] `npm run test` passes
- [ ] `npm run tauri dev` boots the app

**Verify:** see Steps 4–6 below.

**Steps:**

- [ ] **Step 1: Create branch**

```bash
git checkout -b aurora-glass
```

- [ ] **Step 2: Install deps**

```bash
npm install
```

Expected: completes without errors.

- [ ] **Step 3: Rust check**

```bash
cd src-tauri && cargo check && cd ..
```

Expected: `Finished dev` line, no errors.

- [ ] **Step 4: Frontend tests baseline**

```bash
npm run test -- --run
```

Expected: all pass. Note any pre-existing failures — those are not introduced by this work.

- [ ] **Step 5: Dev smoke (manual)**

```bash
npm run tauri dev
```

Open the app, click through each tab on a feature, open Settings, open a modal. Confirm everything renders with current (pre-redesign) look. Close.

- [ ] **Step 6: Commit baseline marker**

```bash
git commit --allow-empty -m "chore: aurora-glass branch start"
```

---

## Task 1: Rewrite design tokens

**Goal:** Replace the `:root` token block in `src/app.css` with the Aurora Glass token set [spec §Design Tokens].

**Files:**
- Modify: `src/app.css:15-116` (the `:root { ... }` block plus the gradient block at `:root` lines ~106-116)

**Acceptance Criteria:**
- [ ] All token names from [spec §Surfaces, §Borders, §Accent, §Semantic colors, §Typography, §Shadows, §Motion] exist in `:root`
- [ ] Backward-compatible names retained: `--accent`, `--bg-card`, `--bg-primary`, `--bg-raised`, `--bg-hover`, `--bg-active`, `--bg-input`, `--green`, `--green-dim`, `--amber`, `--amber-dim`, `--red`, `--red-dim`, `--blue`, `--blue-dim`, `--purple`, `--purple-dim`, `--cyan`, `--cyan-dim`, `--text-primary`, `--text-secondary`, `--text-muted`, `--text-faint`, `--border`, `--border-subtle`, `--border-strong`, `--border-focus`, `--accent-dim`, `--accent-border`, `--accent-glow`, `--accent-hover`, `--shadow-sm`, `--shadow-md`, `--shadow-lg`, `--shadow-xl`, `--shadow-glow`, `--radius-sm`, `--radius-md`, `--radius`, `--radius-lg`, `--radius-xl`, `--radius-2xl`, `--radius-full`, `--space-0..12`, `--text-xs..2xl`, `--font-ui`, `--font-mono`, `--font-size`, `--transition-fast`, `--transition-base`, `--transition-slow`, `--grad-primary`, `--grad-success`, `--grad-warn`, `--grad-cool`, `--card-hover-y`, `--card-hover-shadow`
- [ ] New tokens added: `--bg-aurora`, `--bg-sidebar`, `--bg-glass`, `--bg-glass-soft`, `--aurora`, `--border-glow-top`, `--accent-deep`, `--accent-cool`, `--accent-grad`, `--accent-grad-cool`, `--accent-glow-strong`, `--violet`, `--violet-dim`, `--violet-border`, `--green-border`, `--amber-border`, `--red-border`, `--blue-border`, `--cyan-border`, `--pink`, `--pink-dim`, `--pink-border`, `--shadow-card`, `--shadow-card-hover`, `--shadow-modal`, `--shadow-glow-violet`, `--shadow-glow-cyan`, `--shadow-glow-amber`, `--ease`, `--dur-fast`, `--dur-base`, `--dur-slow`, `--lift`, `--font-weight-normal`, `--font-weight-medium`, `--font-weight-semibold`, `--font-weight-bold`, `--letter-spacing-tight`, `--letter-spacing-wide`
- [ ] Backward-compat token values updated to Aurora palette: `--accent` → `#a78bfa`, `--bg-primary` → `#07081a`, etc. (mapping below)
- [ ] App still boots (`npm run tauri dev`)

**Verify:**
```bash
npm run tauri dev   # smoke; expect aurora-tinted look in some places, broken/inconsistent in others (intentional — primitives come next)
```

**Steps:**

- [ ] **Step 1: Open `src/app.css`, locate the two `:root { ... }` blocks (lines ~15-104 and ~106-116)**

- [ ] **Step 2: Replace both blocks with the unified token block below**

Drop in this exact replacement (copies new tokens from spec, preserves every existing token name with new aurora value):

```css
:root {
  /* ── Surface Colors ── */
  --bg-base: #050610;
  --bg-aurora: #07081a;
  --bg-primary: #07081a;            /* alias of bg-aurora for back-compat */
  --bg-secondary: #0a0c1a;
  --bg-card: #12141f;               /* legacy; new code uses --bg-glass */
  --bg-raised: #161826;
  --bg-hover: rgba(255,255,255,0.04);
  --bg-active: rgba(167,139,250,0.16);
  --bg-input: rgba(20,22,40,0.5);
  --bg-overlay: rgba(0,0,0,0.7);
  --bg-sidebar: rgba(8,9,18,0.35);
  --bg-glass: rgba(20,22,40,0.42);
  --bg-glass-soft: rgba(15,16,32,0.5);

  /* ── Aurora gradient (single source) ── */
  --aurora:
    radial-gradient(640px 460px at 0% 0%,    rgba(99,102,241,0.28), transparent 55%),
    radial-gradient(540px 380px at 100% 0%,  rgba(236,72,153,0.16), transparent 55%),
    radial-gradient(720px 480px at 50% 130%, rgba(34,211,238,0.16), transparent 55%);

  /* ── Borders ── */
  --border-subtle: rgba(255,255,255,0.06);
  --border:        rgba(255,255,255,0.10);
  --border-strong: rgba(196,181,253,0.35);
  --border-focus:  rgba(196,181,253,0.6);
  --border-glow-top: linear-gradient(90deg, transparent, rgba(196,181,253,0.5), transparent);

  /* ── Text ── */
  --text-primary: #e8ecf8;
  --text-secondary: #b8c0d8;
  --text-muted: #6a7390;
  --text-faint: #4a5070;

  /* ── Accent (indigo→violet→cyan) ── */
  --accent: #a78bfa;
  --accent-hover: #c4b5fd;
  --accent-deep: #6366f1;
  --accent-cool: #22d3ee;
  --accent-grad: linear-gradient(135deg, #6366f1, #a78bfa);
  --accent-grad-cool: linear-gradient(90deg, #a78bfa, #22d3ee);
  --accent-dim: rgba(167,139,250,0.18);
  --accent-border: rgba(196,181,253,0.4);
  --accent-glow: 0 0 14px rgba(167,139,250,0.35);
  --accent-glow-strong: 0 0 22px rgba(167,139,250,0.55);

  /* ── Semantic colors ── */
  --green: #22d3ee;          --green-dim: rgba(34,211,238,0.15);    --green-border: rgba(103,232,249,0.45);
  --amber: #fbbf24;          --amber-dim: rgba(251,191,36,0.15);    --amber-border: rgba(252,211,77,0.45);
  --red:   #f87171;          --red-dim:   rgba(248,113,113,0.15);   --red-border:   rgba(248,113,113,0.45);
  --blue:  #52a9ff;          --blue-dim:  rgba(82,169,255,0.15);    --blue-border:  rgba(82,169,255,0.45);
  --cyan:  #67e8f9;          --cyan-dim:  rgba(103,232,249,0.15);   --cyan-border:  rgba(103,232,249,0.45);
  --purple:#a78bfa;          --purple-dim:rgba(167,139,250,0.15);
  --violet:#a78bfa;          --violet-dim:rgba(167,139,250,0.18);   --violet-border:rgba(196,181,253,0.4);
  --pink:  #ec4899;          --pink-dim:  rgba(236,72,153,0.15);    --pink-border:  rgba(244,114,182,0.45);

  /* ── Spacing Scale ── */
  --space-0: 0px;  --space-1: 4px;  --space-2: 8px;  --space-3: 12px;  --space-4: 16px;
  --space-5: 20px; --space-6: 24px; --space-8: 32px; --space-10: 40px; --space-12: 48px;

  /* ── Radius ── */
  --radius-sm:  7px;  --radius-md: 11px;  --radius: 15px;  --radius-lg: 15px;
  --radius-xl:  20px; --radius-2xl: 26px; --radius-full: 9999px;

  /* ── Shadows ── */
  --shadow-sm:        0 1px 3px rgba(0,0,0,0.4);
  --shadow-md:        0 4px 16px rgba(0,0,0,0.5), 0 1px 3px rgba(0,0,0,0.3);
  --shadow-lg:        0 8px 30px rgba(0,0,0,0.6), 0 2px 8px rgba(0,0,0,0.3);
  --shadow-xl:        0 20px 60px rgba(0,0,0,0.7), 0 4px 16px rgba(0,0,0,0.4);
  --shadow-glow:      0 0 20px rgba(167,139,250,0.18);
  --shadow-card:      0 8px 28px rgba(99,102,241,0.18);
  --shadow-card-hover:0 12px 36px rgba(99,102,241,0.28);
  --shadow-modal:     0 30px 80px rgba(0,0,0,0.7);
  --shadow-glow-violet: 0 0 22px rgba(167,139,250,0.45);
  --shadow-glow-cyan:   0 0 18px rgba(34,211,238,0.45);
  --shadow-glow-amber:  0 0 16px rgba(251,191,36,0.40);

  /* ── Typography ── */
  --text-xs: 10.5px; --text-sm: 12px; --text-base: 13px; --text-md: 14px;
  --text-lg: 16px;   --text-xl: 18px; --text-2xl: 22px;
  --font-ui:   'Space Grotesk', -apple-system, BlinkMacSystemFont, sans-serif;
  --font-mono: 'JetBrains Mono', monospace;
  --font-size: 13px;
  --font-weight-normal: 400; --font-weight-medium: 500;
  --font-weight-semibold: 600; --font-weight-bold: 700;
  --letter-spacing-tight: -0.01em;
  --letter-spacing-wide:   0.12em;

  /* ── Motion ── */
  --ease: cubic-bezier(0.16, 1, 0.3, 1);
  --dur-fast: 120ms;  --dur-base: 180ms;  --dur-slow: 320ms;
  --transition-fast: var(--dur-fast)  var(--ease);
  --transition-base: var(--dur-base)  var(--ease);
  --transition-slow: var(--dur-slow)  var(--ease);
  --lift: translateY(-2px);

  /* ── Gradients (legacy aliases) ── */
  --grad-primary: var(--accent-grad);
  --grad-success: linear-gradient(135deg, #22d3ee, #34d399);
  --grad-warn:    linear-gradient(135deg, #fbbf24, #f97316);
  --grad-cool:    var(--accent-grad-cool);

  /* ── Card hover (legacy) ── */
  --card-hover-y:      -2px;
  --card-hover-shadow: var(--shadow-card-hover);
}
```

- [ ] **Step 3: Save and reload dev server**

```bash
npm run tauri dev
```

Walk every tab. Expect colors shifted toward violet/cyan, surfaces still flat (primitives not in yet).

- [ ] **Step 4: Run tests**

```bash
npm run test -- --run
```

Expected: still pass. If a snapshot test breaks because it asserts a literal color, update the snapshot — these are intentional changes.

- [ ] **Step 5: Commit**

```bash
git add src/app.css
git commit -m "Aurora Glass: rewrite design tokens"
```

---

## Task 2: Add Aurora primitive classes

**Goal:** Append a self-contained `/* ===== AURORA PRIMITIVES ===== */` section to `src/app.css` defining the ten reusable classes [spec §Reusable Primitives].

**Files:**
- Modify: `src/app.css` (append a new section near the top, after the `:root` block but before `* { ... }` reset, OR at end of file as a separate section — pick whichever keeps cascade behavior)

**Acceptance Criteria:**
- [ ] All ten primitives exist verbatim from spec: `.glass-panel` (+`.glass-panel--hover`, `.glass-panel--soft`), `.aurora-pill` (+`--success`, `--warn`, `--danger`, `--muted`, `--no-dot`), `.btn` (+`--primary`, `--ghost`, `--danger`, `--sm`, `--icon`), `.tab-bar` + `.tab` (+`--active`) + `.tab__badge`, `.list-row` (+`--active`), `.input` (+`--search`), `.check` (+`--done`), `.live-dot` (+`--warn`), `.aurora-bg`, `.scrim`
- [ ] `@keyframes pulse-dot` defined (already exists at line ~892 — keep one, delete duplicates)
- [ ] No primitive class collides with an existing selector (verified via grep)
- [ ] App boots without console errors

**Verify:**
```bash
grep -nE '\.glass-panel|\.aurora-pill|\.list-row|\.aurora-bg|\.scrim' src/app.css
# Expect each class appearing in the new section, not duplicated elsewhere
npm run tauri dev
```

**Steps:**

- [ ] **Step 1: Locate insertion point**

Open `src/app.css`. After the `:root { ... }` block (post-Task 1, ends around line 130), add a new section starting at the next blank line. Before the `* { margin: 0; ... }` reset.

- [ ] **Step 2: Insert primitives section**

Paste the following block verbatim:

```css
/* ===========================================================
 * AURORA PRIMITIVES
 * Reusable classes that drive the Aurora Glass redesign.
 * See docs/superpowers/specs/2026-04-23-aurora-glass-design-system.md
 * =========================================================== */

/* App-shell aurora background — apply once on root */
.aurora-bg {
  background: var(--aurora), var(--bg-aurora);
  position: relative;
  isolation: isolate;
}

/* Scrim for modals / dropdowns */
.scrim {
  position: fixed;
  inset: 0;
  background: var(--bg-overlay);
  backdrop-filter: blur(8px);
  z-index: 100;
}

/* ── Glass surfaces ── */
.glass-panel {
  background: var(--bg-glass);
  backdrop-filter: blur(22px) saturate(140%);
  -webkit-backdrop-filter: blur(22px) saturate(140%);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  position: relative;
  overflow: hidden;
}
.glass-panel::before {
  content: '';
  position: absolute;
  left: 0; top: 0; right: 0;
  height: 1px;
  background: var(--border-glow-top);
  pointer-events: none;
}
.glass-panel--hover {
  transition: transform var(--dur-base) var(--ease),
              border-color var(--dur-base) var(--ease),
              box-shadow var(--dur-base) var(--ease);
}
.glass-panel--hover:hover {
  transform: var(--lift);
  border-color: var(--border-strong);
  box-shadow: var(--shadow-card-hover);
}
.glass-panel--soft {
  background: var(--bg-glass-soft);
  backdrop-filter: blur(14px);
  -webkit-backdrop-filter: blur(14px);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  position: relative;
  overflow: hidden;
}

/* ── Pills (status / tag / count badges) ── */
.aurora-pill {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 2px 9px;
  border-radius: var(--radius-full);
  font-size: var(--text-xs);
  font-weight: var(--font-weight-semibold);
  background: var(--accent-dim);
  color: var(--accent-hover);
  border: 1px solid var(--accent-border);
  box-shadow: var(--accent-glow);
  white-space: nowrap;
}
.aurora-pill::before {
  content: '';
  width: 5px; height: 5px;
  border-radius: 50%;
  background: currentColor;
  box-shadow: 0 0 8px currentColor;
  flex-shrink: 0;
}
.aurora-pill--no-dot::before { display: none; }
.aurora-pill--success { background: var(--green-dim);  color: var(--cyan);  border-color: var(--green-border);  box-shadow: var(--shadow-glow-cyan); }
.aurora-pill--warn    { background: var(--amber-dim);  color: var(--amber); border-color: var(--amber-border);  box-shadow: var(--shadow-glow-amber); }
.aurora-pill--danger  { background: var(--red-dim);    color: var(--red);   border-color: var(--red-border);    box-shadow: 0 0 14px rgba(248,113,113,0.35); }
.aurora-pill--info    { background: var(--blue-dim);   color: var(--blue);  border-color: var(--blue-border);   box-shadow: 0 0 14px rgba(82,169,255,0.30); }
.aurora-pill--muted   { background: rgba(255,255,255,0.05); color: var(--text-muted); border-color: var(--border-subtle); box-shadow: none; }
.aurora-pill--sm { padding: 1px 7px; font-size: 9.5px; gap: 4px; }

/* ── Buttons ── */
.btn {
  background: rgba(255,255,255,0.05);
  border: 1px solid var(--border);
  color: var(--text-primary);
  padding: 6px 12px;
  border-radius: var(--radius-sm);
  font-family: var(--font-ui);
  font-size: var(--text-sm);
  font-weight: var(--font-weight-medium);
  cursor: pointer;
  transition: background var(--dur-base) var(--ease),
              border-color var(--dur-base) var(--ease),
              color var(--dur-base) var(--ease),
              box-shadow var(--dur-base) var(--ease),
              transform var(--dur-base) var(--ease);
  display: inline-flex;
  align-items: center;
  gap: 6px;
  line-height: 1.2;
}
.btn:hover { background: var(--accent-dim); border-color: var(--border-strong); }
.btn:active { transform: translateY(0.5px); }
.btn:disabled { opacity: 0.45; cursor: not-allowed; }
.btn--primary {
  background: var(--accent-grad);
  border-color: var(--border-strong);
  color: #fff;
  box-shadow: 0 4px 16px rgba(167,139,250,0.4), inset 0 1px 0 rgba(255,255,255,0.2);
}
.btn--primary:hover { box-shadow: var(--shadow-glow-violet), inset 0 1px 0 rgba(255,255,255,0.25); transform: translateY(-1px); border-color: var(--accent-hover); }
.btn--ghost { background: transparent; border-color: transparent; color: var(--text-secondary); }
.btn--ghost:hover { background: rgba(255,255,255,0.05); color: var(--text-primary); border-color: var(--border-subtle); }
.btn--danger:hover { background: var(--red-dim); border-color: var(--red-border); color: var(--red); }
.btn--sm { padding: 4px 9px; font-size: var(--text-xs); border-radius: 6px; gap: 4px; }
.btn--icon { padding: 6px; aspect-ratio: 1; justify-content: center; }
.btn--icon.btn--sm { padding: 4px; }

/* ── Tabs ── */
.tab-bar {
  display: flex;
  gap: 2px;
  border-bottom: 1px solid var(--border-subtle);
  flex-wrap: wrap;
}
.tab {
  padding: 9px 14px;
  font-size: var(--text-sm);
  color: var(--text-secondary);
  cursor: pointer;
  position: relative;
  border-radius: 6px 6px 0 0;
  transition: color var(--dur-base) var(--ease), background var(--dur-base) var(--ease);
  background: transparent;
  border: none;
  font-family: var(--font-ui);
}
.tab:hover { color: var(--accent-hover); }
.tab--active { color: var(--text-primary); }
.tab--active::after {
  content: '';
  position: absolute;
  left: 8px; right: 8px; bottom: -1px;
  height: 2px;
  background: var(--accent-grad-cool);
  border-radius: 2px;
  box-shadow: 0 0 10px rgba(167,139,250,0.6);
}
.tab__badge {
  display: inline-block;
  margin-left: 5px;
  background: var(--accent-dim);
  color: var(--accent-hover);
  font-size: 9.5px;
  font-weight: var(--font-weight-semibold);
  padding: 1px 5px;
  border-radius: var(--radius-full);
}

/* ── List rows (sidebar features, tasks, files, links, kb entries…) ── */
.list-row {
  padding: 7px 10px;
  border-radius: var(--radius-sm);
  font-size: var(--text-sm);
  color: var(--text-primary);
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  position: relative;
  transition: background var(--dur-base) var(--ease), color var(--dur-base) var(--ease);
}
.list-row:hover { background: rgba(255,255,255,0.04); }
.list-row--active {
  background: var(--accent-dim);
  color: var(--text-primary);
  box-shadow: inset 0 0 0 1px var(--accent-border);
}
.list-row--active::before {
  content: '';
  position: absolute;
  left: -10px;
  top: 20%; bottom: 20%;
  width: 3px;
  background: var(--accent-grad-cool);
  border-radius: 0 3px 3px 0;
  box-shadow: var(--accent-glow-strong);
}

/* ── Inputs ── */
.input {
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 7px 10px;
  color: var(--text-primary);
  font-family: var(--font-ui);
  font-size: var(--text-sm);
  transition: border-color var(--dur-base) var(--ease), box-shadow var(--dur-base) var(--ease);
  width: 100%;
}
.input::placeholder { color: var(--text-muted); }
.input:focus { outline: none; border-color: var(--border-focus); box-shadow: 0 0 0 3px var(--accent-dim); }
.input--search {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}
.input--search input {
  background: transparent;
  border: none;
  outline: none;
  color: inherit;
  font: inherit;
  flex: 1;
  min-width: 0;
}

/* ── Checkbox ── */
.check {
  width: 14px;
  height: 14px;
  border: 1px solid var(--accent-border);
  border-radius: 4px;
  flex-shrink: 0;
  cursor: pointer;
  transition: border-color var(--dur-base) var(--ease), box-shadow var(--dur-base) var(--ease), background var(--dur-base) var(--ease);
  background: transparent;
  display: inline-block;
  position: relative;
}
.check:hover { border-color: var(--accent-hover); box-shadow: var(--accent-glow); }
.check--done {
  background: var(--accent-grad-cool);
  border-color: transparent;
  box-shadow: var(--accent-glow);
}
.check--done::after {
  content: '✓';
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  color: #fff;
  line-height: 1;
}

/* ── Live dot (active session pulse, agent indicator) ── */
.live-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--cyan);
  box-shadow: 0 0 8px var(--cyan);
  animation: pulse-dot 1.6s ease-in-out infinite;
  display: inline-block;
  flex-shrink: 0;
}
.live-dot--warn { background: var(--amber); box-shadow: 0 0 8px var(--amber); }
.live-dot--danger { background: var(--red); box-shadow: 0 0 8px var(--red); }
.live-dot--static { animation: none; }

/* keyframes — single source */
@keyframes pulse-dot {
  0%, 100% { opacity: 1; transform: scale(1); }
  50%      { opacity: 0.55; transform: scale(0.85); }
}
@keyframes cursor-blink {
  50% { opacity: 0; }
}
```

- [ ] **Step 3: Remove duplicate `@keyframes pulse-dot`**

Search the file for any other `@keyframes pulse-dot` definition (one exists around line 892) and delete it. Same for any other primitive class that already exists ad-hoc — leave older selectors in place; they get rewritten in later tasks.

- [ ] **Step 4: Reload + smoke**

```bash
npm run tauri dev
```

Open dev tools console — no CSS warnings. App still renders (existing classes still drive most surfaces).

- [ ] **Step 5: Verify primitives load**

In dev tools, inspect any `<div>`, manually add class `glass-panel` via the elements panel, confirm the glass effect applies.

- [ ] **Step 6: Commit**

```bash
git add src/app.css
git commit -m "Aurora Glass: add primitive classes"
```

---

## Task 3: App shell + icon rail + aurora background

**Goal:** Apply `.aurora-bg` once at the app root, retire the old `.shell-fx` ray effects, glass-up the icon rail.

**Files:**
- Modify: `src/App.svelte` (root container)
- Modify: `src/app.css` — `.app-frame`, `.app-shell`, `.shell-fx`, `.icon-rail*` (lines ~173-300+)

**Acceptance Criteria:**
- [ ] `<div id="app">` (or the outermost `.app-frame`/`.app-shell`) carries class `aurora-bg`
- [ ] Aurora gradient visible behind sidebar and main content
- [ ] `.shell-fx` removed from DOM and from `app.css` (the rays are gone)
- [ ] Icon rail uses glass background (`var(--bg-sidebar)` + `backdrop-filter: blur(18px)`)
- [ ] Icon rail buttons have hover/active states using accent color
- [ ] No layout shift vs pre-redesign

**Verify:** `npm run tauri dev`; aurora visible; icon rail has frosted look; clicking icon-rail buttons still toggles panes.

**Steps:**

- [ ] **Step 1: Find and read the root container in `App.svelte`**

```bash
grep -n 'app-frame\|app-shell\|shell-fx' src/App.svelte
```

Identify the outermost wrapper element (usually a `<div class="app-frame">` or similar).

- [ ] **Step 2: Add `aurora-bg` to the root**

Edit `src/App.svelte`. Add `aurora-bg` to the outermost div's class list. If a `<div class="shell-fx">…</div>` element exists, delete it.

- [ ] **Step 3: Update `.app-frame` and `.app-shell` in `src/app.css`**

Find the existing rules (around lines 203-227) and replace bodies with:

```css
.app-frame {
  position: relative;
  z-index: 1;
  height: 100vh;
  display: flex;
  align-items: stretch;
  box-sizing: border-box;
  background: var(--bg-aurora);
}

.app-shell {
  display: flex;
  flex: 1;
  min-width: 0;
  opacity: 1;
  transition: opacity var(--dur-fast) var(--ease);
  background: transparent;
  overflow: hidden;
}

.app-shell.storage-switching { opacity: 0; }
```

- [ ] **Step 4: Delete `.shell-fx` rules**

Find `.shell-fx` (around line 175) and `.shell-fx::before` and `.ray` and `@keyframes ray-drift` — delete them. They are obsolete.

- [ ] **Step 5: Update `.icon-rail`**

Find `.icon-rail` rule (around line 231) and replace with:

```css
.icon-rail {
  width: 60px;
  min-width: 60px;
  height: 100%;
  background: var(--bg-sidebar);
  backdrop-filter: blur(18px) saturate(140%);
  -webkit-backdrop-filter: blur(18px) saturate(140%);
  border-right: 1px solid var(--border-subtle);
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 14px 0 10px;
  gap: 3px;
  user-select: none;
  flex-shrink: 0;
  z-index: 10;
  position: relative;
}
```

- [ ] **Step 6: Update `.icon-rail-btn` hover**

Replace the existing rule body for `.icon-rail-btn`:

```css
.icon-rail-btn {
  width: 38px;
  height: 38px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background var(--dur-fast) var(--ease), color var(--dur-fast) var(--ease), border-color var(--dur-fast) var(--ease);
  color: var(--text-muted);
  border: 1px solid transparent;
  background: none;
  flex-shrink: 0;
  position: relative;
}
.icon-rail-btn:hover { background: var(--accent-dim); color: var(--accent-hover); border-color: var(--accent-border); }
.icon-rail-btn--on { color: var(--accent-hover); background: var(--accent-dim); border-color: var(--accent-border); box-shadow: var(--accent-glow); }
```

- [ ] **Step 7: Update logo glow**

Replace `.icon-rail-logo`:

```css
.icon-rail-logo {
  width: 34px;
  height: 34px;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 8px;
  flex-shrink: 0;
  cursor: default;
  background: var(--accent-grad);
  border: 1px solid var(--accent-border);
  box-shadow: var(--accent-glow);
}
```

- [ ] **Step 8: Smoke test**

```bash
npm run tauri dev
```

Confirm: aurora background visible; icon rail frosted; buttons highlight on hover; logo has subtle glow; no layout shift.

- [ ] **Step 9: Commit**

```bash
git add src/App.svelte src/app.css
git commit -m "Aurora Glass: app shell aurora background + glass icon rail"
```

---

## Task 4: Sidebar adoption

**Goal:** Apply primitives to `Sidebar.svelte` — feature rows become `.list-row`, status dot becomes a `.live-dot--static` colored variant, active-sessions badge becomes `.aurora-pill--success` with `.live-dot`, search becomes `.input--search`.

**Files:**
- Modify: `src/lib/components/Sidebar.svelte` (markup additions only — no logic change)
- Modify: `src/app.css` — rules under `.sidebar`, `.feature-item-compact`, `.feature-item-status-dot`, `.feature-item-pmb*`, `.active-sessions-badge`, `.active-sessions-dot`, `.tree-chevron`, search input, section titles, group headers

**Acceptance Criteria:**
- [ ] Sidebar has glass background via `var(--bg-sidebar)` + blur
- [ ] Each feature row visually matches the H1-v3 mockup: status dot + title + mini progress + sessions badge + chevron
- [ ] Active feature row has glowing left rail
- [ ] Sessions badge pulses; warn variant (yellow) when any session is `WaitingForInput`
- [ ] Search box uses `.input--search`
- [ ] Drag reorder still works
- [ ] Feature group headers + collapsing still works
- [ ] `npm run test` still passes (or snapshots updated)

**Verify:**
```bash
npm run tauri dev
# walk: scroll sidebar, hover features, click active feature, drag reorder, search, expand/collapse groups
```

**Steps:**

- [ ] **Step 1: Read the sidebar feature row markup**

```bash
grep -n 'feature-item-compact\|active-sessions-badge\|feature-item-status-dot' src/lib/components/Sidebar.svelte
```

Locate the row rendering (around line 921). Note the existing class structure — it stays.

- [ ] **Step 2: Update `Sidebar.svelte` — search box**

Find the search input element near the top of the sidebar template. Add class `input input--search` on the wrapper. Example:

```svelte
<div class="sidebar-search input input--search">
  <span class="search-icon">⌕</span>
  <input type="text" placeholder="Search features" bind:value={searchQuery} />
  <span class="kbd">⌘K</span>
</div>
```

(Keep existing event handlers unchanged.)

- [ ] **Step 3: Update `Sidebar.svelte` — feature row**

In the `.feature-item-compact` element, add `list-row` and conditionally `list-row--active`:

```svelte
<div
  class="feature-item-compact list-row"
  class:list-row--active={feature.id === selectedFeatureId}
  ...existing handlers...
>
  ...existing children unchanged...
</div>
```

- [ ] **Step 4: Update `Sidebar.svelte` — sessions badge**

Locate the `<span class="active-sessions-badge">` element. Add `aurora-pill aurora-pill--success` to its class list and replace the inner `<span class="active-sessions-dot">` with `<span class="live-dot"></span>`. For warn state, conditionally swap to `aurora-pill--warn` + `live-dot--warn` based on whether any session for the feature is `WaitingForInput` (helper exists in `sessionActivity.svelte.ts`).

```svelte
{#if getActiveCountForFeature(feature.id) > 0}
  {@const count = getActiveCountForFeature(feature.id)}
  {@const waiting = isAnySessionWaitingForFeature(feature.id)}
  <span
    class="active-sessions-badge aurora-pill"
    class:aurora-pill--success={!waiting}
    class:aurora-pill--warn={waiting}
    title="{count} active session{count > 1 ? 's' : ''}{waiting ? ' (awaiting input)' : ''}"
    ...existing handlers...
  >
    <span class="live-dot" class:live-dot--warn={waiting}></span>{count}
  </span>
{/if}
```

If `isAnySessionWaitingForFeature` doesn't exist, add this helper to `src/lib/stores/sessionActivity.svelte.ts`:

```ts
export function isAnySessionWaitingForFeature(featureId: string): boolean {
  return panelSessions.some(
    (s) => s.feature_id === featureId && s.status === "WaitingForInput",
  );
}
```

Import it at the top of `Sidebar.svelte` alongside `getActiveCountForFeature`.

- [ ] **Step 5: Rewrite sidebar CSS rules in `src/app.css`**

Find the `.sidebar` block (search for `\.sidebar\s*{` — the outer container). Replace with:

```css
.sidebar {
  background: var(--bg-sidebar);
  backdrop-filter: blur(18px) saturate(140%);
  -webkit-backdrop-filter: blur(18px) saturate(140%);
  border-right: 1px solid var(--border-subtle);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
}
```

Remove or override stale background colors elsewhere in `.sidebar` rules.

- [ ] **Step 6: Rewrite `.feature-item-compact`**

Locate the existing rule. Replace body with primitive-aligned styling. Because the element now has both `feature-item-compact` and `list-row`, lean on the primitive and only add what's specific:

```css
.feature-item-compact {
  /* primitive classes provide most styling */
  font-weight: var(--font-weight-medium);
}
.feature-item-compact .feature-item-title {
  flex: 1;
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
```

- [ ] **Step 7: Rewrite `.feature-item-status-dot`**

```css
.feature-item-status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
  /* color comes from inline style="background: ..." driven by statusColors[status] */
  box-shadow: 0 0 6px currentColor;
}
```

Note: existing markup sets `style="background: {color}"` — the `box-shadow: 0 0 6px currentColor` would need `color: <same>;` too. Update the inline style to set both `background` and `color` to the same value:

In `Sidebar.svelte`, change:

```svelte
<span class="feature-item-status-dot" style="background: {statusColors[feature.status] ?? 'var(--text-muted)'};"></span>
```

to:

```svelte
<span class="feature-item-status-dot" style="background: {statusColors[feature.status] ?? 'var(--text-muted)'}; color: {statusColors[feature.status] ?? 'var(--text-muted)'};"></span>
```

- [ ] **Step 8: Rewrite mini progress bar `.feature-item-pmb*`**

```css
.feature-item-pmb { display: flex; align-items: center; gap: 4px; flex-shrink: 0; }
.feature-item-pmb-track {
  width: 24px; height: 3px;
  border-radius: 2px;
  background: rgba(255,255,255,0.08);
  overflow: hidden;
}
.feature-item-pmb-fill {
  height: 100%;
  background: var(--accent-grad-cool);
  border-radius: 2px;
  box-shadow: 0 0 6px rgba(167,139,250,0.5);
  transition: width var(--dur-slow) var(--ease);
}
.feature-item-pmb-label {
  font-size: 9.5px;
  color: var(--text-muted);
  font-variant-numeric: tabular-nums;
  font-family: var(--font-mono);
}
```

- [ ] **Step 9: Rewrite `.active-sessions-badge` + `.active-sessions-dot`**

Both classes now defer to `.aurora-pill` and `.live-dot`. Override what's still needed:

```css
.active-sessions-badge {
  font-size: 9.5px;
  font-weight: var(--font-weight-bold);
  cursor: pointer;
  transition: background var(--dur-fast) var(--ease), transform var(--dur-fast) var(--ease);
  flex-shrink: 0;
  padding: 0 6px;
  height: 16px;
}
.active-sessions-badge:hover { transform: scale(1.06); }
.active-sessions-dot { /* legacy selector; keep no-op for any old markup */ }
```

- [ ] **Step 10: Rewrite section titles + group headers**

Find rules for sidebar section labels (e.g. "Pinned", "In progress", "Backlog") — they may use a class like `.sidebar-section-title` or `.feature-section-header`. Apply uppercase wide-spaced label style:

```css
.sidebar-section-title, .feature-section-title {
  font-size: 10.5px;
  text-transform: uppercase;
  letter-spacing: var(--letter-spacing-wide);
  color: var(--text-muted);
  padding: 8px 10px 6px;
  font-weight: var(--font-weight-bold);
}
```

(Use the actual class name from the source file. If multiple, list all.)

- [ ] **Step 11: Tree chevron**

```css
.tree-chevron {
  color: var(--text-faint);
  font-size: 10px;
  flex-shrink: 0;
  transition: transform var(--dur-base) var(--ease), color var(--dur-base) var(--ease);
}
.tree-chevron:hover { color: var(--accent-hover); }
.tree-chevron--expanded { transform: rotate(90deg); }
```

- [ ] **Step 12: Smoke test**

```bash
npm run tauri dev
```

Walk: scroll, hover, click, drag-reorder, expand/collapse groups, search, pin/unpin (right-click menu).

- [ ] **Step 13: Tests**

```bash
npm run test -- --run
```

Update any snapshots that fail.

- [ ] **Step 14: Commit**

```bash
git add src/lib/components/Sidebar.svelte src/lib/stores/sessionActivity.svelte.ts src/app.css
git commit -m "Aurora Glass: sidebar adopts list-row + aurora-pill primitives"
```

---

## Task 5: Workspace tab bar + Feature detail header

**Goal:** Tabs (workspace tabs and feature tabs) use `.tab-bar` + `.tab--active` with the glowing underline. Feature detail header uses `.aurora-pill` for status, `.btn`/`.btn--primary` for actions.

**Files:**
- Modify: `src/lib/components/WorkspaceTabBar.svelte`
- Modify: `src/lib/components/FeatureDetail.svelte`
- Modify: `src/app.css` — `.workspace-tab*`, `.feature-detail-header*`, `.tab-button`, `.tabs-bar`, etc.

**Acceptance Criteria:**
- [ ] Workspace tabs render with the glow underline on active tab; close button is `.btn--icon.btn--ghost`
- [ ] Feature tab bar uses `.tab-bar` with `.tab--active`; tab badges use `.tab__badge`
- [ ] Feature header status pill uses `.aurora-pill` with variant matching status
- [ ] Resume / +New session buttons use `.btn` and `.btn--primary`
- [ ] Switching tabs is smooth (180ms underline animation)

**Verify:** `npm run tauri dev`; click through tabs; open multiple workspace tabs; switch between them.

**Steps:**

- [ ] **Step 1: WorkspaceTabBar markup**

Edit `src/lib/components/WorkspaceTabBar.svelte`. The container becomes a `.tab-bar`:

```svelte
<div class="workspace-tab-bar tab-bar">
  {#each tabs as tab (tab.id)}
    <button
      class="workspace-tab tab"
      class:tab--active={tab.id === activeTabId}
      onclick={() => onSelect(tab.id)}
    >
      {tab.title}
      <span class="btn btn--icon btn--ghost btn--sm" onclick={(e) => { e.stopPropagation(); onClose(tab.id); }}>×</span>
    </button>
  {/each}
  <button class="btn btn--icon btn--ghost btn--sm" onclick={onNewTab}>+</button>
</div>
```

(Adjust to match real prop names.)

- [ ] **Step 2: FeatureDetail tab bar**

Edit `src/lib/components/FeatureDetail.svelte`. Find the tab strip (renders the registered tabs). Wrap in `.tab-bar`, apply `.tab` + `.tab--active` to each tab button. Badges from `getBadges(ctx)` get `.tab__badge`:

```svelte
<div class="feature-detail-tabs tab-bar">
  {#each tabs as tab (tab.id)}
    <button
      class="tab"
      class:tab--active={tab.id === activeTabId}
      onclick={() => setActive(tab.id)}
    >
      {tab.emoji ?? ''} {tab.label}
      {#each tab.getBadges(ctx) as badge}
        <span class="tab__badge">{badge.value}</span>
      {/each}
    </button>
  {/each}
</div>
```

- [ ] **Step 3: Feature header**

In `FeatureDetail.svelte`, the header section that renders title + status + meta. Status pill becomes `.aurora-pill` with variant:

```svelte
<span class="aurora-pill" class:aurora-pill--success={status === 'done'} class:aurora-pill--warn={status === 'in_progress' || status === 'in_review'} class:aurora-pill--danger={status === 'blocked'}>{status}</span>
```

Header action buttons → `.btn` and the primary action → `.btn--primary`.

- [ ] **Step 4: Rewrite related CSS in `src/app.css`**

Locate any of these rules and slim them down (primitives provide the look):

```css
.workspace-tab-bar, .feature-detail-tabs, .tabs-bar { /* primitives drive */ }
.workspace-tab { /* primitive .tab drives base */ }
.workspace-tab .close { font-size: 13px; line-height: 1; }
.feature-detail-header {
  display: flex;
  align-items: flex-start;
  gap: 14px;
  padding: 14px 18px;
}
.feature-detail-title {
  font-size: var(--text-2xl);
  font-weight: var(--font-weight-semibold);
  letter-spacing: var(--letter-spacing-tight);
}
.feature-detail-meta {
  color: var(--text-secondary);
  font-size: var(--text-sm);
  display: flex;
  gap: 10px;
  align-items: center;
  flex-wrap: wrap;
  margin-top: 4px;
}
.feature-detail-actions { margin-left: auto; display: flex; gap: 8px; }
```

(Use the actual selectors from the source — `grep` first.)

- [ ] **Step 5: Smoke test**

```bash
npm run tauri dev
```

Click each tab; confirm underline glows + slides smoothly. Open a second feature in a new workspace tab.

- [ ] **Step 6: Tests + commit**

```bash
npm run test -- --run
git add src/lib/components/WorkspaceTabBar.svelte src/lib/components/FeatureDetail.svelte src/app.css
git commit -m "Aurora Glass: workspace + feature tabs adopt tab primitive"
```

---

## Task 6: Agents tab (AI module)

**Goal:** All children of `AiPanel` adopt primitives — sessions, plans, context editor, MCP servers panel, skills panel, terminal.

**Files:**
- Modify: `src/lib/modules/ai/AiPanel.svelte`
- Modify: `src/lib/modules/ai/SessionList.svelte`
- Modify: `src/lib/modules/ai/SessionCard.svelte`
- Modify: `src/lib/modules/ai/PlanCard.svelte`
- Modify: `src/lib/modules/ai/PlanDetail.svelte`
- Modify: `src/lib/modules/ai/ContextEditor.svelte`
- Modify: `src/lib/modules/ai/McpServersPanel.svelte`
- Modify: `src/lib/modules/ai/SkillsPanel.svelte`
- Modify: `src/lib/modules/ai/Terminal.svelte`
- Modify: `src/app.css` — relevant module classes

**Acceptance Criteria:**
- [ ] SessionCard is a `.glass-panel.glass-panel--hover`. Status uses `.live-dot` (cyan/amber/muted). Model name in monospace, accent color
- [ ] PlanCard is a `.glass-panel`. Status pill matches plan state (pending=warn, approved=success, rejected=danger). Approve=`.btn--primary`, Reject=`.btn`, Discuss=`.btn--ghost`
- [ ] ContextEditor textarea uses `.input`, sits inside a `.glass-panel`
- [ ] MCP server rows are `.list-row` with toggle (`.check`)
- [ ] Skills rows are `.glass-panel--soft`
- [ ] Terminal has dark inner bg, monospace, blinking cursor
- [ ] Polling and approve/reject/discuss flows still work

**Verify:** `npm run tauri dev`; open a feature, switch to Agents tab; trigger a plan submission via MCP; approve/reject; open Terminal sub-pane.

**Steps:**

- [ ] **Step 1: SessionCard**

Edit `src/lib/modules/ai/SessionCard.svelte`. Outer wrapper: add `glass-panel glass-panel--hover` to the card class list. Replace the active indicator span with `<span class="live-dot" class:live-dot--warn={isWaitingForInput} class:live-dot--static={!isActive}></span>`. Model name span: add `style="font-family: var(--font-mono); color: var(--accent-hover);"` or use a new class `.session-card__model` defined in `app.css`.

CSS additions:
```css
.session-card { padding: 12px 14px; }
.session-card__model { font-family: var(--font-mono); font-size: var(--text-xs); color: var(--accent-hover); }
.session-card__meta { font-family: var(--font-mono); font-size: var(--text-xs); color: var(--text-muted); }
```

- [ ] **Step 2: PlanCard + PlanDetail**

Edit both files. Wrapper gets `glass-panel`. Status badge uses pill variants (pending → warn, approved → success, rejected → danger). Action buttons get `.btn`/`.btn--primary`/`.btn--ghost` classes.

```svelte
<div class="plan-card glass-panel">
  <div class="plan-card__header">
    <span class="aurora-pill" class:aurora-pill--warn={status === 'pending'} class:aurora-pill--success={status === 'approved'} class:aurora-pill--danger={status === 'rejected'}>{status}</span>
    <span class="plan-card__title">{plan.title}</span>
  </div>
  <div class="plan-card__body">{plan.body}</div>
  {#if status === 'pending'}
    <div class="plan-card__actions">
      <button class="btn btn--primary" onclick={onApprove}>Approve</button>
      <button class="btn" onclick={onReject}>Reject</button>
      <button class="btn btn--ghost" onclick={onDiscuss}>Discuss</button>
    </div>
  {/if}
</div>
```

CSS:
```css
.plan-card { padding: 14px; }
.plan-card__header { display: flex; align-items: center; gap: 10px; margin-bottom: 8px; }
.plan-card__title { font-weight: var(--font-weight-semibold); }
.plan-card__body { font-size: var(--text-sm); color: var(--text-secondary); line-height: 1.55; }
.plan-card__actions { display: flex; gap: 8px; margin-top: 12px; }
```

- [ ] **Step 3: ContextEditor**

Wrap textarea in `.glass-panel`. Textarea gets `.input` class. Toolbar buttons (save, preview-toggle) get `.btn` `.btn--sm`.

```svelte
<div class="context-editor glass-panel">
  <div class="context-editor__toolbar">
    <button class="btn btn--sm" onclick={togglePreview}>{showPreview ? 'Edit' : 'Preview'}</button>
    <button class="btn btn--primary btn--sm" onclick={save}>Save</button>
  </div>
  {#if showPreview}
    <MarkdownPreview content={value} />
  {:else}
    <textarea class="input context-editor__textarea" bind:value></textarea>
  {/if}
</div>
```

CSS:
```css
.context-editor { padding: 12px; display: flex; flex-direction: column; gap: 10px; height: 100%; }
.context-editor__toolbar { display: flex; gap: 8px; justify-content: flex-end; }
.context-editor__textarea { flex: 1; resize: none; font-family: var(--font-mono); font-size: var(--text-base); line-height: 1.6; min-height: 280px; }
```

- [ ] **Step 4: McpServersPanel**

Each server row → `.list-row` with name + transport + toggle. Toggle is a `.check` (square checkbox; OK for binary on/off here).

```svelte
{#each servers as server}
  <div class="list-row mcp-server-row">
    <span class="check" class:check--done={server.enabled} onclick={() => toggle(server.id)}></span>
    <span class="mcp-server-row__name">{server.name}</span>
    <span class="aurora-pill aurora-pill--muted aurora-pill--sm">{server.transport}</span>
    <span style="margin-left:auto" class="btn btn--icon btn--ghost btn--sm" onclick={() => edit(server.id)}>✎</span>
  </div>
{/each}
```

- [ ] **Step 5: SkillsPanel**

Each skill row → `.glass-panel--soft` with title + toggle + edit button.

```svelte
{#each skills as skill}
  <div class="skill-row glass-panel--soft">
    <span class="check" class:check--done={skill.enabled} onclick={() => toggle(skill.id)}></span>
    <span class="skill-row__name">{skill.name}</span>
    <span class="skill-row__desc">{skill.description}</span>
    <button class="btn btn--icon btn--ghost btn--sm" onclick={() => edit(skill.id)}>✎</button>
  </div>
{/each}
```

CSS:
```css
.skill-row { display: flex; align-items: center; gap: 10px; padding: 10px 12px; }
.skill-row__name { font-weight: var(--font-weight-semibold); font-size: var(--text-sm); }
.skill-row__desc { color: var(--text-secondary); font-size: var(--text-xs); flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
```

- [ ] **Step 6: Terminal**

Locate the terminal container in `src/lib/modules/ai/Terminal.svelte`. Wrap with `.glass-panel` (or just style the existing element):

```css
.terminal {
  background: rgba(0,0,0,0.4);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-primary);
  padding: 10px 12px;
  border-radius: var(--radius-md);
  position: relative;
  overflow: hidden;
}
.terminal::after {
  content: '';
  position: absolute;
  inset: 0;
  background: repeating-linear-gradient(0deg, rgba(0,0,0,0) 0, rgba(0,0,0,0) 2px, rgba(0,0,0,0.06) 3px);
  pointer-events: none;
}
.terminal__cursor {
  display: inline-block;
  width: 7px;
  height: 14px;
  background: var(--accent-cool);
  vertical-align: text-bottom;
  animation: cursor-blink 1s step-end infinite;
}
```

(If Terminal uses xterm.js or similar, only style the wrapper.)

- [ ] **Step 7: AiPanel layout**

Layout container — make sure inner spacing uses primitives. No glass on `AiPanel` outer wrapper itself (it sits in the tab content area which is transparent).

- [ ] **Step 8: Smoke + tests**

```bash
npm run tauri dev   # start a session via fh; submit a plan via MCP; approve
npm run test -- --run
```

- [ ] **Step 9: Commit**

```bash
git add src/lib/modules/ai/ src/app.css
git commit -m "Aurora Glass: agents tab adopts glass + pill + btn primitives"
```

---

## Task 7: Tasks & Notes + Files modules

**Goal:** Apply primitives to TasksNotesPanel/TaskList/NotesEditor and FileBrowser/FileList/FilePreviewPanel/FolderBreadcrumb.

**Files:**
- Modify: `src/lib/modules/tasks-notes/TasksNotesPanel.svelte`, `TaskList.svelte`, `NotesEditor.svelte`
- Modify: `src/lib/modules/files/FileBrowser.svelte`, `FileList.svelte`, `FilePreviewPanel.svelte`, `FolderBreadcrumb.svelte`
- Modify: `src/app.css` — `.tasks-notes-panel`, `.task-list`, `.task-row`, `.task-checkbox`, `.notes-editor`, `.file-browser`, `.file-list`, `.file-row`, `.file-preview`, `.folder-breadcrumb`

**Acceptance Criteria:**
- [ ] Tasks & Notes: two columns each in a `.glass-panel`. Task rows use `.check`/`.check--done`. Done tasks fade + strikethrough. NotesEditor textarea uses `.input`
- [ ] Files: `.glass-panel` outer container. FolderBreadcrumb chips use `.aurora-pill--muted` (current folder violet). FileList rows use `.list-row` with file-type colored dot. Selected row uses `.list-row--active`
- [ ] Code preview has gradient left border on code blocks; image preview centered with violet drop shadow; binary preview metadata in `.glass-panel--soft`
- [ ] Drag-drop zone: dashed `var(--accent-border)` border that brightens to `var(--border-focus)` on dragenter
- [ ] Add task / add file actions still work

**Verify:** `npm run tauri dev`; toggle tasks; edit notes; navigate folders; preview files; drag-drop a file.

**Steps:**

- [ ] **Step 1: TaskList — task rows**

```svelte
{#each tasks as task}
  <div class="task-row" class:task-row--done={task.done}>
    <span class="check" class:check--done={task.done} onclick={() => toggle(task.id)}></span>
    <span class="task-row__title">{task.title}</span>
    {#if task.source === 'jira'}
      <a href={task.external_url} class="aurora-pill aurora-pill--info aurora-pill--sm">{task.external_key}</a>
    {/if}
    <button class="btn btn--icon btn--ghost btn--sm" onclick={() => del(task.id)}>×</button>
  </div>
{/each}
```

CSS:
```css
.task-row { display: flex; align-items: center; gap: 10px; padding: 6px 8px; border-radius: var(--radius-sm); transition: background var(--dur-fast) var(--ease); }
.task-row:hover { background: rgba(255,255,255,0.04); }
.task-row__title { flex: 1; font-size: var(--text-sm); }
.task-row--done .task-row__title { color: var(--text-muted); text-decoration: line-through; }
```

- [ ] **Step 2: NotesEditor**

Wrap textarea in `.glass-panel`. Textarea uses `.input`. Toolbar buttons use `.btn--icon.btn--sm`.

```svelte
<div class="notes-editor glass-panel">
  <div class="notes-editor__toolbar">
    <button class="btn btn--icon btn--sm">B</button>
    <button class="btn btn--icon btn--sm">I</button>
    <button class="btn btn--sm" onclick={togglePreview}>Preview</button>
  </div>
  {#if previewMode}
    <MarkdownPreview content={value} />
  {:else}
    <textarea class="input notes-editor__textarea" bind:value></textarea>
  {/if}
</div>
```

CSS:
```css
.notes-editor { padding: 12px; display: flex; flex-direction: column; gap: 10px; height: 100%; }
.notes-editor__toolbar { display: flex; gap: 6px; }
.notes-editor__textarea { flex: 1; min-height: 280px; resize: none; font-family: var(--font-mono); font-size: var(--text-base); line-height: 1.6; border: none; box-shadow: none; }
.notes-editor__textarea:focus { box-shadow: none; }
```

- [ ] **Step 3: TasksNotesPanel layout**

Two columns in `.glass-panel`s:

```svelte
<div class="tasks-notes-panel">
  <div class="glass-panel tasks-column"><TaskList ... /></div>
  <div class="glass-panel notes-column"><NotesEditor ... /></div>
</div>
```

CSS:
```css
.tasks-notes-panel { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; height: 100%; padding: 12px; }
.tasks-column, .notes-column { padding: 12px; display: flex; flex-direction: column; gap: 8px; min-height: 0; }
```

- [ ] **Step 4: FolderBreadcrumb**

Each crumb is `.aurora-pill--muted`; current folder is `.aurora-pill` (violet). Separator `›`.

```svelte
<div class="folder-breadcrumb">
  {#each path as crumb, i}
    <button class="aurora-pill aurora-pill--no-dot" class:aurora-pill--muted={i < path.length - 1} onclick={() => navigateTo(crumb.id)}>{crumb.name}</button>
    {#if i < path.length - 1}<span class="folder-breadcrumb__sep">›</span>{/if}
  {/each}
</div>
```

CSS:
```css
.folder-breadcrumb { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; padding: 8px 10px; }
.folder-breadcrumb__sep { color: var(--text-faint); font-size: var(--text-sm); }
```

- [ ] **Step 5: FileList rows**

```svelte
{#each files as file}
  <div class="file-row list-row" class:list-row--active={file.id === selectedId} onclick={() => select(file.id)}>
    <span class="file-row__type-dot" style="background: {fileTypeColor(file.ext)}; color: {fileTypeColor(file.ext)}"></span>
    <span class="file-row__name">{file.name}</span>
    <span class="file-row__size">{formatBytes(file.size)}</span>
  </div>
{/each}
```

CSS + helper:
```css
.file-row__type-dot { width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0; box-shadow: 0 0 6px currentColor; }
.file-row__name { flex: 1; min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.file-row__size { font-family: var(--font-mono); font-size: var(--text-xs); color: var(--text-muted); }
```

`fileTypeColor` lives in a util — add to `src/lib/utils/format.ts`:

```ts
export function fileTypeColor(ext: string): string {
  const map: Record<string, string> = {
    md: 'var(--amber)', ts: 'var(--cyan)', tsx: 'var(--cyan)', js: 'var(--amber)', jsx: 'var(--amber)',
    svelte: 'var(--pink)', rs: 'var(--accent)', toml: 'var(--text-muted)', json: 'var(--green)',
    css: 'var(--blue)', html: 'var(--red)', sql: 'var(--violet)', py: 'var(--cyan)',
    png: 'var(--pink)', jpg: 'var(--pink)', svg: 'var(--violet)', gif: 'var(--pink)',
  };
  return map[ext.toLowerCase()] ?? 'var(--text-muted)';
}
```

- [ ] **Step 6: FilePreviewPanel**

```css
.file-preview { padding: 12px; height: 100%; }
.file-preview pre {
  background: var(--bg-glass-soft);
  border: 1px solid var(--border-subtle);
  border-left: 3px solid var(--accent);
  border-radius: var(--radius-sm);
  padding: 12px;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  line-height: 1.55;
  overflow: auto;
}
.file-preview img {
  max-width: 100%;
  max-height: 100%;
  border-radius: var(--radius-md);
  box-shadow: 0 12px 36px rgba(167,139,250,0.25);
  display: block;
  margin: 0 auto;
}
.file-preview__binary {
  padding: 16px;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--text-secondary);
}
```

Wrap binary metadata in `<div class="glass-panel--soft file-preview__binary">`.

- [ ] **Step 7: FileBrowser drag-drop zone**

```css
.file-browser__drop-zone {
  border: 2px dashed var(--accent-border);
  border-radius: var(--radius-md);
  padding: 24px;
  text-align: center;
  color: var(--text-muted);
  transition: border-color var(--dur-fast) var(--ease), background var(--dur-fast) var(--ease);
}
.file-browser__drop-zone--active {
  border-color: var(--border-focus);
  background: var(--accent-dim);
  color: var(--accent-hover);
}
```

Toggle `--active` class via Svelte ondragenter / ondragleave handlers (already present).

- [ ] **Step 8: FileBrowser layout**

```css
.file-browser { display: grid; grid-template-columns: 240px 1fr; gap: 12px; height: 100%; padding: 12px; }
.file-browser__tree, .file-browser__list, .file-browser__preview { padding: 8px; }
.file-browser__tree.glass-panel, .file-browser__list.glass-panel, .file-browser__preview.glass-panel { /* wrapper handles look */ }
```

Add `.glass-panel` to the three panes in markup.

- [ ] **Step 9: Smoke + tests + commit**

```bash
npm run tauri dev
npm run test -- --run
git add src/lib/modules/tasks-notes/ src/lib/modules/files/ src/lib/utils/format.ts src/app.css
git commit -m "Aurora Glass: tasks-notes + files modules"
```

---

## Task 8: Links + Repositories + Timeline + Board modules

**Goal:** Smaller modules — apply primitives.

**Files:**
- Modify: `src/lib/modules/links/LinksGrid.svelte`, `LinkCard.svelte`
- Modify: `src/lib/modules/repos/RepositoriesPanel.svelte`
- Modify: `src/lib/modules/timeline/Timeline.svelte`
- Modify: `src/lib/components/GlobalTimeline.svelte`
- Modify: `src/lib/modules/board/BoardPanel.svelte`, `BoardColumn.svelte`, `BoardCard.svelte`
- Modify: `src/app.css` — relevant module classes

**Acceptance Criteria:**
- [ ] LinkCard is `.glass-panel.glass-panel--hover`. Type icon in colored circle (color = link type). Type label is `.aurora-pill--muted`. Edit/delete reveal on hover via `.btn--icon.btn--ghost`. "Add link" is `.btn--primary`
- [ ] Repo row is `.glass-panel`. Clone status pill: cloned=success, cloning=warn, failed=danger. Branches as `.list-row`s. "Open in editor" is `.btn--sm`
- [ ] Timeline events are `.glass-panel--soft` rows. Connector line uses `var(--accent-border)`. Event-type dot is colored `.live-dot--static`
- [ ] Board columns are `.glass-panel`. Cards are `.glass-panel.glass-panel--hover`. Drag-active state: `var(--shadow-glow-violet)`. Drop target: `var(--border-focus)` border + `var(--accent-dim)` bg
- [ ] All click/drag interactions still work

**Verify:** `npm run tauri dev`; for each tab: hover cards, click actions, drag a board card, scroll timeline.

**Steps:**

- [ ] **Step 1: LinkCard**

```svelte
<div class="link-card glass-panel glass-panel--hover">
  <div class="link-card__icon" style="background: {linkTypeColor(link.type)}; color: {linkTypeColor(link.type)}; box-shadow: 0 0 12px currentColor"></div>
  <div class="link-card__body">
    <a href={link.url} class="link-card__title">{link.title || link.url}</a>
    <div class="link-card__meta">
      <span class="aurora-pill aurora-pill--muted aurora-pill--sm">{link.type}</span>
      <span class="link-card__url">{shortUrl(link.url)}</span>
    </div>
  </div>
  <div class="link-card__actions">
    <button class="btn btn--icon btn--ghost btn--sm" onclick={() => edit(link.id)}>✎</button>
    <button class="btn btn--icon btn--ghost btn--sm" onclick={() => del(link.id)}>×</button>
  </div>
</div>
```

CSS:
```css
.link-card { padding: 12px; display: flex; gap: 12px; align-items: center; }
.link-card__icon { width: 28px; height: 28px; border-radius: 50%; flex-shrink: 0; }
.link-card__body { flex: 1; min-width: 0; }
.link-card__title { color: var(--text-primary); font-weight: var(--font-weight-semibold); display: block; }
.link-card__title:hover { color: var(--accent-hover); }
.link-card__meta { display: flex; align-items: center; gap: 8px; margin-top: 4px; }
.link-card__url { color: var(--text-muted); font-size: var(--text-xs); font-family: var(--font-mono); }
.link-card__actions { display: flex; gap: 4px; opacity: 0; transition: opacity var(--dur-base) var(--ease); }
.link-card:hover .link-card__actions { opacity: 1; }
```

Add `linkTypeColor` to `src/lib/utils/format.ts` (mirror existing detection — github/jira/figma/confluence/slack/notion/linear/etc).

- [ ] **Step 2: LinksGrid**

```css
.links-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 12px; padding: 12px; }
.links-grid__add { /* primary button at top */ }
```

Top-level "+ Add link" → `.btn.btn--primary`.

- [ ] **Step 3: RepositoriesPanel**

```svelte
{#each repos as repo}
  <div class="repo-row glass-panel">
    <div class="repo-row__head">
      <span class="repo-row__name">{repo.label || repo.path}</span>
      <span class="aurora-pill" class:aurora-pill--success={repo.clone_status === 'cloned'} class:aurora-pill--warn={repo.clone_status === 'cloning'} class:aurora-pill--danger={repo.clone_status === 'failed'}>{repo.clone_status}</span>
    </div>
    <div class="repo-row__path">{repo.path}</div>
    {#if repo.branches?.length}
      <div class="repo-row__branches">
        {#each repo.branches as br}
          <div class="list-row" class:list-row--active={br.current}>
            <span class="status-dot" style="background: {br.current ? 'var(--cyan)' : 'var(--text-faint)'}; color: inherit"></span>
            {br.name}
          </div>
        {/each}
      </div>
    {/if}
    <div class="repo-row__actions">
      <button class="btn btn--sm" onclick={() => openInEditor(repo)}>Open in editor</button>
    </div>
  </div>
{/each}
```

CSS:
```css
.repo-row { padding: 12px; margin-bottom: 10px; }
.repo-row__head { display: flex; align-items: center; gap: 10px; margin-bottom: 4px; }
.repo-row__name { font-weight: var(--font-weight-semibold); }
.repo-row__path { font-family: var(--font-mono); font-size: var(--text-xs); color: var(--text-muted); margin-bottom: 8px; }
.repo-row__branches { margin-bottom: 8px; }
.repo-row__actions { display: flex; gap: 6px; }
```

- [ ] **Step 4: Timeline + GlobalTimeline**

```svelte
<div class="timeline">
  {#each events as ev}
    <div class="timeline__event glass-panel--soft">
      <span class="timeline__dot live-dot live-dot--static" style="background: {eventColor(ev.kind)}; color: {eventColor(ev.kind)}"></span>
      <div class="timeline__body">
        <div class="timeline__title">{ev.title}</div>
        <div class="timeline__meta">{formatRelativeTime(ev.timestamp)}</div>
      </div>
    </div>
  {/each}
</div>
```

CSS:
```css
.timeline { position: relative; padding: 12px 0 12px 32px; }
.timeline::before {
  content: '';
  position: absolute;
  left: 18px; top: 16px; bottom: 16px;
  width: 1px;
  background: var(--accent-border);
}
.timeline__event { display: flex; gap: 10px; align-items: flex-start; padding: 10px 12px; margin-bottom: 8px; position: relative; }
.timeline__dot { position: absolute; left: -22px; top: 14px; }
.timeline__title { font-size: var(--text-sm); font-weight: var(--font-weight-medium); }
.timeline__meta { font-family: var(--font-mono); font-size: var(--text-xs); color: var(--text-muted); }
```

Add `eventColor` helper to `format.ts` (kind → semantic color).

- [ ] **Step 5: BoardColumn + BoardCard**

```svelte
<div class="board-column glass-panel" class:board-column--drop-target={isDropTarget}>
  <div class="board-column__header">
    <span class="board-column__title">{column.title}</span>
    <span class="aurora-pill aurora-pill--muted aurora-pill--sm aurora-pill--no-dot">{column.cards.length}</span>
  </div>
  <div class="board-column__body">
    {#each column.cards as card}
      <BoardCard {card} />
    {/each}
  </div>
</div>
```

```svelte
<div class="board-card glass-panel glass-panel--hover" draggable="true" ondragstart={...}>
  <div class="board-card__title">{card.title}</div>
  <div class="board-card__meta">
    {#if card.tags}
      {#each card.tags as tag}
        <TagBadge {tag} />
      {/each}
    {/if}
  </div>
</div>
```

CSS:
```css
.board-panel { display: flex; gap: 12px; padding: 12px; height: 100%; overflow-x: auto; }
.board-column { width: 280px; flex-shrink: 0; padding: 12px; display: flex; flex-direction: column; gap: 8px; }
.board-column__header { display: flex; align-items: center; gap: 8px; text-transform: uppercase; letter-spacing: var(--letter-spacing-wide); font-size: 10.5px; color: var(--text-muted); font-weight: var(--font-weight-bold); }
.board-column__title { flex: 1; }
.board-column__body { display: flex; flex-direction: column; gap: 8px; flex: 1; min-height: 0; overflow-y: auto; }
.board-column--drop-target { border-color: var(--border-focus); background: var(--accent-dim); }
.board-card { padding: 10px 12px; cursor: grab; }
.board-card:active { cursor: grabbing; }
.board-card.dragging { box-shadow: var(--shadow-glow-violet); }
.board-card__title { font-size: var(--text-sm); font-weight: var(--font-weight-medium); margin-bottom: 6px; }
.board-card__meta { display: flex; gap: 4px; flex-wrap: wrap; }
```

- [ ] **Step 6: Smoke + tests + commit**

```bash
npm run tauri dev
npm run test -- --run
git add src/lib/modules/links/ src/lib/modules/repos/ src/lib/modules/timeline/ src/lib/modules/board/ src/lib/components/GlobalTimeline.svelte src/lib/utils/format.ts src/app.css
git commit -m "Aurora Glass: links + repos + timeline + board modules"
```

---

## Task 9: Knowledge module + Search

**Goal:** Apply primitives to KnowledgePanel + KnowledgeFolderTree + KnowledgeEntryEditor and SearchBar.

**Files:**
- Modify: `src/lib/modules/knowledge/KnowledgePanel.svelte`, `KnowledgeFolderTree.svelte`, `KnowledgeEntryEditor.svelte`
- Modify: `src/lib/components/SearchBar.svelte`
- Modify: `src/app.css` — `.knowledge-*`, `.search-bar`, `.search-results`, `.search-result-row`

**Acceptance Criteria:**
- [ ] Knowledge: three-pane layout in `.glass-panel`s. Folder tree rows are `.list-row` with chevron + count. Entry rows are `.list-row` with title + tag pills. Editor: `.input` textarea + MarkdownPreview side-by-side
- [ ] Search bar: `.input--search`. Results dropdown is `.glass-panel` with `.shadow-modal`. Each result is a `.list-row` with entity-type pill + match snippet. Highlighted match terms get `var(--accent-hover)` color
- [ ] Knowledge create/edit/delete folder + entry still works
- [ ] Search debouncing + result navigation still works

**Verify:** `npm run tauri dev`; open Knowledge tab; create folder/entry; search globally.

**Steps:**

- [ ] **Step 1: KnowledgeFolderTree**

```svelte
{#each nodes as node}
  <div class="list-row knowledge-folder-row" class:list-row--active={node.id === selectedFolderId}>
    <span class="tree-chevron" class:tree-chevron--expanded={node.expanded} onclick={() => toggle(node.id)}>›</span>
    <span class="knowledge-folder-row__name" onclick={() => select(node.id)}>{node.name}</span>
    <span class="aurora-pill aurora-pill--muted aurora-pill--sm aurora-pill--no-dot">{node.entry_count}</span>
  </div>
  {#if node.expanded}
    {#each node.children as child}
      <KnowledgeFolderRow node={child} />
    {/each}
  {/if}
{/each}
```

CSS:
```css
.knowledge-folder-row { padding-left: calc(8px + var(--depth, 0) * 14px); }
.knowledge-folder-row__name { flex: 1; min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
```

- [ ] **Step 2: KnowledgePanel layout**

```svelte
<div class="knowledge-panel">
  <div class="glass-panel knowledge-panel__tree"><KnowledgeFolderTree ... /></div>
  <div class="glass-panel knowledge-panel__list">
    {#each entries as entry}
      <div class="list-row knowledge-entry-row" class:list-row--active={entry.id === selectedEntryId} onclick={() => select(entry.id)}>
        <span class="knowledge-entry-row__title">{entry.title}</span>
        {#each entry.tags as tag}
          <TagBadge {tag} />
        {/each}
      </div>
    {/each}
  </div>
  <div class="glass-panel knowledge-panel__editor"><KnowledgeEntryEditor entry={selectedEntry} /></div>
</div>
```

CSS:
```css
.knowledge-panel { display: grid; grid-template-columns: 220px 280px 1fr; gap: 12px; height: 100%; padding: 12px; }
.knowledge-panel__tree, .knowledge-panel__list, .knowledge-panel__editor { padding: 10px; display: flex; flex-direction: column; gap: 6px; min-height: 0; overflow-y: auto; }
.knowledge-entry-row__title { flex: 1; font-size: var(--text-sm); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
```

- [ ] **Step 3: KnowledgeEntryEditor**

Title + description inputs use `.input`. Body textarea + preview share the toolbar pattern from Task 7 NotesEditor. Save uses `.btn--primary`.

- [ ] **Step 4: SearchBar**

```svelte
<div class="search-bar input input--search">
  <span class="search-bar__icon">⌕</span>
  <input bind:value={query} placeholder="Search everything…" oninput={debouncedSearch} />
  <span class="kbd">⌘/</span>
</div>
{#if results.length > 0 && open}
  <div class="search-results glass-panel">
    {#each results as r}
      <div class="list-row search-result-row" onclick={() => navigate(r)}>
        <span class="aurora-pill aurora-pill--muted aurora-pill--sm aurora-pill--no-dot">{r.entity_type}</span>
        <span class="search-result-row__title">{r.title}</span>
        <span class="search-result-row__snippet">{@html r.snippet}</span>
      </div>
    {/each}
  </div>
{/if}
```

CSS:
```css
.search-bar { padding: 6px 10px; }
.search-bar__icon { color: var(--text-muted); }
.kbd { background: rgba(255,255,255,0.06); padding: 1px 5px; border-radius: 4px; font-family: var(--font-mono); font-size: 9.5px; color: var(--text-muted); }
.search-results { position: absolute; top: 100%; left: 0; right: 0; max-height: 60vh; overflow-y: auto; box-shadow: var(--shadow-modal); padding: 6px; z-index: 50; margin-top: 4px; }
.search-result-row__title { flex: 1; min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-size: var(--text-sm); }
.search-result-row__snippet { color: var(--text-muted); font-size: var(--text-xs); margin-left: 8px; min-width: 0; }
.search-result-row mark { background: transparent; color: var(--accent-hover); font-weight: var(--font-weight-semibold); }
```

- [ ] **Step 5: Smoke + tests + commit**

```bash
npm run tauri dev
npm run test -- --run
git add src/lib/modules/knowledge/ src/lib/components/SearchBar.svelte src/app.css
git commit -m "Aurora Glass: knowledge module + search bar"
```

---

## Task 10: Settings + Storage + Modal primitives

**Goal:** All modals use `.scrim` + `.glass-panel`. SettingsModal uses left nav with `.list-row`s and right pane sections in `.glass-panel--soft`. StorageSelector/StorageSetup adopt the same pattern.

**Files:**
- Modify: `src/lib/components/ui/Modal.svelte`, `ConfirmDialog.svelte`, `Dropdown.svelte`, `IconButton.svelte`
- Modify: `src/lib/components/SettingsModal.svelte`
- Modify: `src/lib/components/StorageSelector.svelte`, `StorageSetup.svelte`
- Modify: `src/lib/components/CreateFeatureModal.svelte`, `ExportImportModal.svelte`
- Modify: `src/app.css` — `.modal`, `.modal-scrim`, `.dropdown`, `.icon-button`, `.settings-modal*`, `.storage-*`

**Acceptance Criteria:**
- [ ] All modal scrims use `.scrim` styling (blur-8 + dark overlay)
- [ ] All modal panels use `.glass-panel` with `.shadow-modal`
- [ ] Settings modal: left nav rows are `.list-row`. Active section has glowing rail. Right pane sections in `.glass-panel--soft`. Inputs use `.input`. Toggles use `.check` (or a real switch — see step 4)
- [ ] StorageSelector / StorageSetup: storage rows are `.glass-panel.glass-panel--hover`. Active storage has rail. Buttons primitives
- [ ] CreateFeatureModal / ExportImportModal: standard form layout, `.input`, `.btn--primary`, `.btn`
- [ ] Dropdown floating panel uses `.glass-panel` + `.shadow-modal`
- [ ] IconButton is now an alias for `.btn--icon.btn--ghost`
- [ ] All open/save/cancel flows still work

**Verify:** `npm run tauri dev`; open Settings (every tab); switch storage; create new feature; export/import; trigger ConfirmDialog (e.g. delete a feature); open dropdowns.

**Steps:**

- [ ] **Step 1: Modal.svelte (primitive base)**

```svelte
<div class="scrim" onclick={onClose}>
  <div class="modal glass-panel" onclick={(e) => e.stopPropagation()}>
    <div class="modal__header">
      <h2 class="modal__title">{title}</h2>
      <button class="btn btn--icon btn--ghost btn--sm" onclick={onClose}>×</button>
    </div>
    <div class="modal__body">
      {@render children()}
    </div>
    {#if footer}
      <div class="modal__footer">{@render footer()}</div>
    {/if}
  </div>
</div>
```

CSS:
```css
.modal {
  position: relative;
  z-index: 101;
  width: min(720px, 92vw);
  max-height: 86vh;
  margin: 7vh auto;
  display: flex;
  flex-direction: column;
  padding: 0;
  box-shadow: var(--shadow-modal);
}
.modal__header { display: flex; align-items: center; padding: 14px 18px; border-bottom: 1px solid var(--border-subtle); }
.modal__title { flex: 1; font-size: var(--text-lg); font-weight: var(--font-weight-semibold); letter-spacing: var(--letter-spacing-tight); }
.modal__body { flex: 1; overflow-y: auto; padding: 16px 18px; }
.modal__footer { padding: 12px 18px; border-top: 1px solid var(--border-subtle); display: flex; gap: 8px; justify-content: flex-end; }
```

- [ ] **Step 2: ConfirmDialog.svelte**

Extend Modal. Buttons: Cancel = `.btn`, Confirm = `.btn--primary` (or `.btn--danger` for destructive — pass via `variant` prop).

- [ ] **Step 3: Dropdown.svelte**

The floating panel gets `.glass-panel` + `.shadow-modal`. Each option becomes `.list-row`.

```css
.dropdown__panel {
  position: absolute;
  min-width: 180px;
  padding: 4px;
  z-index: 1000;
  box-shadow: var(--shadow-modal);
}
.dropdown__option { padding: 6px 10px; font-size: var(--text-sm); border-radius: var(--radius-sm); cursor: pointer; }
.dropdown__option:hover { background: var(--accent-dim); color: var(--accent-hover); }
.dropdown__sep { height: 1px; background: var(--border-subtle); margin: 4px 0; }
```

- [ ] **Step 4: IconButton.svelte**

```svelte
<button class="btn btn--icon btn--ghost {size === 'sm' ? 'btn--sm' : ''}" {...$$restProps}>
  {@render children()}
</button>
```

Existing `.icon-button` CSS rules: delete (primitives drive). Update test wrapper if any inline assertions on classes break.

- [ ] **Step 5: SettingsModal**

```svelte
<Modal title="Settings" {onClose}>
  <div class="settings-modal">
    <div class="settings-modal__nav">
      {#each sections as s}
        <div class="list-row" class:list-row--active={s.id === activeSection} onclick={() => setActive(s.id)}>
          <span>{s.icon}</span>
          {s.label}
        </div>
      {/each}
    </div>
    <div class="settings-modal__content">
      {#each visibleSubsections as sub}
        <div class="glass-panel--soft settings-modal__section">
          <h3 class="settings-modal__section-title">{sub.title}</h3>
          {@render sub.body()}
        </div>
      {/each}
    </div>
  </div>
</Modal>
```

CSS:
```css
.settings-modal { display: grid; grid-template-columns: 200px 1fr; gap: 12px; height: 60vh; }
.settings-modal__nav { display: flex; flex-direction: column; gap: 2px; padding: 8px; border-right: 1px solid var(--border-subtle); }
.settings-modal__content { padding: 8px; overflow-y: auto; display: flex; flex-direction: column; gap: 12px; }
.settings-modal__section { padding: 14px; }
.settings-modal__section-title { font-size: 11px; text-transform: uppercase; letter-spacing: var(--letter-spacing-wide); color: var(--text-muted); margin-bottom: 10px; font-weight: var(--font-weight-bold); }
```

For toggle switches, use a styled `.toggle` (slim two-state switch). Add to `app.css`:

```css
.toggle { position: relative; width: 32px; height: 18px; background: rgba(255,255,255,0.08); border-radius: var(--radius-full); cursor: pointer; transition: background var(--dur-base) var(--ease); }
.toggle::after { content: ''; position: absolute; top: 2px; left: 2px; width: 14px; height: 14px; border-radius: 50%; background: var(--text-muted); transition: transform var(--dur-base) var(--ease), background var(--dur-base) var(--ease); }
.toggle--on { background: var(--accent-dim); }
.toggle--on::after { transform: translateX(14px); background: var(--accent-grad-cool); box-shadow: var(--accent-glow); }
```

Replace existing checkbox-style toggles in SettingsModal with `.toggle`.

- [ ] **Step 6: StorageSelector + StorageSetup**

Storage row:
```svelte
<div class="storage-row glass-panel glass-panel--hover" class:list-row--active={storage.id === activeStorageId} onclick={() => select(storage.id)}>
  <div class="storage-row__name">{storage.label}</div>
  <div class="storage-row__path">{storage.path}</div>
</div>
```

(Note: `list-row--active` works on `glass-panel` too because the rail is `::before`-positioned at `left: -10px`.)

CSS:
```css
.storage-row { padding: 12px; margin-bottom: 8px; cursor: pointer; }
.storage-row__name { font-weight: var(--font-weight-semibold); margin-bottom: 4px; }
.storage-row__path { font-family: var(--font-mono); font-size: var(--text-xs); color: var(--text-muted); }
```

- [ ] **Step 7: CreateFeatureModal + ExportImportModal**

Use `Modal` wrapper. Inputs: `.input`. Submit: `.btn--primary`. Cancel: `.btn`.

- [ ] **Step 8: Smoke + tests + commit**

```bash
npm run tauri dev   # open every modal
npm run test -- --run
git add src/lib/components/ui/ src/lib/components/SettingsModal.svelte src/lib/components/StorageSelector.svelte src/lib/components/StorageSetup.svelte src/lib/components/CreateFeatureModal.svelte src/lib/components/ExportImportModal.svelte src/app.css
git commit -m "Aurora Glass: modals + settings + storage selector"
```

---

## Task 11: Toasts + StatusBadge + TagBadge + Markdown previews

**Goal:** ToastContainer, StatusBadge, TagBadge, MarkdownPreview, OpenFgaPreview adopt primitives.

**Files:**
- Modify: `src/lib/components/ToastContainer.svelte`
- Modify: `src/lib/components/StatusBadge.svelte`
- Modify: `src/lib/components/TagBadge.svelte`
- Modify: `src/lib/components/MarkdownPreview.svelte`
- Modify: `src/lib/components/OpenFgaPreview.svelte`
- Modify: `src/app.css` — `.toast*`, `.status-badge`, `.tag-badge`, `.markdown-preview`, `.openfga-preview`

**Acceptance Criteria:**
- [ ] Toasts: small `.glass-panel` (heavier blur), 4px left rail in semantic color, slide-in/out animation
- [ ] StatusBadge: outputs `.aurora-pill` with variant determined by status string
- [ ] TagBadge: `.aurora-pill` with custom inline color from `tag.color`, falls back to default
- [ ] MarkdownPreview: serif-free body, headings with subtle border, code blocks `.glass-panel--soft` with gradient left border, blockquote 3px rail, links violet, mermaid container `.glass-panel--soft`
- [ ] OpenFgaPreview: `.glass-panel--soft` container

**Verify:** Trigger a toast (notification arrival); render a feature with status pills + tags; preview a markdown note with code, headings, blockquote, mermaid block.

**Steps:**

- [ ] **Step 1: StatusBadge.svelte**

```svelte
<script>
  let { status } = $props();
  const variantMap = {
    done: 'success', in_progress: 'warn', in_review: 'warn',
    blocked: 'danger', paused: 'muted', todo: 'muted', active: '',
  };
</script>
<span class="aurora-pill" class={variantMap[status] ? `aurora-pill--${variantMap[status]}` : ''}>{status.replace('_', ' ')}</span>
```

- [ ] **Step 2: TagBadge.svelte**

```svelte
<script>
  let { tag } = $props();
  const style = tag.color
    ? `background: ${hexToAlpha(tag.color, 0.15)}; color: ${tag.color}; border-color: ${hexToAlpha(tag.color, 0.4)}; box-shadow: 0 0 12px ${hexToAlpha(tag.color, 0.3)};`
    : '';
</script>
<span class="aurora-pill aurora-pill--no-dot" {style}>{tag.name}</span>
```

Add `hexToAlpha` to `src/lib/utils/format.ts`:

```ts
export function hexToAlpha(hex: string, alpha: number): string {
  const h = hex.replace('#', '');
  const r = parseInt(h.substring(0, 2), 16);
  const g = parseInt(h.substring(2, 4), 16);
  const b = parseInt(h.substring(4, 6), 16);
  return `rgba(${r},${g},${b},${alpha})`;
}
```

- [ ] **Step 3: ToastContainer.svelte**

```svelte
<div class="toast-container">
  {#each toasts as t (t.id)}
    <div class="toast glass-panel toast--{t.kind}" transition:slide>
      <div class="toast__rail"></div>
      <div class="toast__body">{t.message}</div>
      <button class="btn btn--icon btn--ghost btn--sm" onclick={() => dismiss(t.id)}>×</button>
    </div>
  {/each}
</div>
```

CSS:
```css
.toast-container {
  position: fixed;
  bottom: 16px; right: 16px;
  display: flex; flex-direction: column; gap: 8px;
  z-index: 200;
  pointer-events: none;
}
.toast {
  pointer-events: auto;
  display: flex; align-items: center; gap: 10px;
  padding: 10px 12px 10px 16px;
  min-width: 260px; max-width: 420px;
  backdrop-filter: blur(28px) saturate(160%);
  -webkit-backdrop-filter: blur(28px) saturate(160%);
  position: relative;
  box-shadow: var(--shadow-card-hover);
}
.toast__rail { position: absolute; left: 0; top: 8px; bottom: 8px; width: 3px; border-radius: 0 3px 3px 0; }
.toast--info    .toast__rail { background: var(--blue);  box-shadow: 0 0 12px var(--blue); }
.toast--success .toast__rail { background: var(--cyan);  box-shadow: 0 0 12px var(--cyan); }
.toast--warn    .toast__rail { background: var(--amber); box-shadow: 0 0 12px var(--amber); }
.toast--error   .toast__rail { background: var(--red);   box-shadow: 0 0 12px var(--red); }
.toast__body { flex: 1; font-size: var(--text-sm); }
```

If `transition:slide` isn't ideal for slide-from-right, use Svelte's `fly`:
```svelte
import { fly } from 'svelte/transition';
... transition:fly={{ x: 30, duration: 220 }} ...
```

- [ ] **Step 4: MarkdownPreview.svelte**

CSS only (the component is mostly `{@html marked(content)}`):

```css
.markdown-preview {
  font-family: var(--font-ui);
  color: var(--text-primary);
  line-height: 1.65;
  font-size: var(--text-base);
}
.markdown-preview h1, .markdown-preview h2 { letter-spacing: var(--letter-spacing-tight); padding-bottom: 6px; border-bottom: 1px solid var(--border-subtle); margin: 16px 0 10px; }
.markdown-preview h3, .markdown-preview h4 { margin: 14px 0 6px; }
.markdown-preview p { margin: 0 0 10px; }
.markdown-preview a { color: var(--accent-hover); text-decoration: underline; text-decoration-color: var(--accent-border); text-underline-offset: 2px; }
.markdown-preview a:hover { text-decoration-color: var(--accent-hover); }
.markdown-preview code {
  font-family: var(--font-mono);
  font-size: 0.92em;
  background: var(--accent-dim);
  color: var(--accent-hover);
  padding: 1px 6px;
  border-radius: 4px;
}
.markdown-preview pre {
  background: var(--bg-glass-soft);
  border: 1px solid var(--border-subtle);
  border-left: 3px solid;
  border-image: var(--accent-grad-cool) 1;
  border-radius: var(--radius-sm);
  padding: 12px;
  overflow-x: auto;
  margin: 10px 0;
}
.markdown-preview pre code { background: none; color: var(--text-primary); padding: 0; }
.markdown-preview blockquote {
  border-left: 3px solid;
  border-image: var(--accent-grad-cool) 1;
  padding-left: 12px;
  color: var(--text-secondary);
  margin: 10px 0;
}
.markdown-preview table { border-collapse: collapse; width: 100%; margin: 10px 0; }
.markdown-preview th { text-align: left; text-transform: uppercase; letter-spacing: var(--letter-spacing-wide); font-size: var(--text-xs); color: var(--text-muted); padding: 6px 10px; border-bottom: 1px solid var(--border); }
.markdown-preview td { padding: 6px 10px; border-bottom: 1px solid var(--border-subtle); font-size: var(--text-sm); }
.markdown-preview tr:hover td { background: rgba(255,255,255,0.03); }
.markdown-preview ul, .markdown-preview ol { padding-left: 24px; margin: 6px 0 10px; }
.markdown-preview hr { border: none; border-top: 1px solid var(--border-subtle); margin: 18px 0; }
.markdown-preview .mermaid {
  background: var(--bg-glass-soft);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  padding: 12px;
  margin: 10px 0;
  overflow-x: auto;
}
```

- [ ] **Step 5: OpenFgaPreview.svelte**

```css
.openfga-preview {
  background: var(--bg-glass-soft);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  padding: 12px;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  line-height: 1.55;
  color: var(--text-primary);
}
.openfga-preview .keyword { color: var(--accent-hover); }
.openfga-preview .type-name { color: var(--cyan); }
.openfga-preview .relation { color: var(--amber); }
.openfga-preview .comment { color: var(--text-faint); font-style: italic; }
```

- [ ] **Step 6: Smoke + tests + commit**

```bash
npm run tauri dev   # trigger toast (e.g. submit a plan via fh-mcp); inspect markdown
npm run test -- --run
git add src/lib/components/ToastContainer.svelte src/lib/components/StatusBadge.svelte src/lib/components/TagBadge.svelte src/lib/components/MarkdownPreview.svelte src/lib/components/OpenFgaPreview.svelte src/lib/utils/format.ts src/app.css
git commit -m "Aurora Glass: toasts + badges + markdown preview"
```

---

## Task 12: Storage-level panels (Dashboard / Sessions / InstalledExtensions)

**Goal:** DashboardPanel, SessionsPanel, InstalledExtensionsPanel use primitives.

**Files:**
- Modify: `src/lib/components/DashboardPanel.svelte`
- Modify: `src/lib/components/SessionsPanel.svelte`
- Modify: `src/lib/components/InstalledExtensionsPanel.svelte`
- Modify: `src/app.css` — `.dashboard-*`, `.sessions-panel*`, `.extensions-panel*`

**Acceptance Criteria:**
- [ ] Dashboard stat cards: `.glass-panel`. Big numbers `var(--text-2xl)` over uppercase wide-spaced labels
- [ ] Active features lists, recent activity: `.list-row`s
- [ ] SessionsPanel: each session row `.glass-panel--soft` with `.live-dot` (matching live state), model name in mono accent, ago in mono muted
- [ ] InstalledExtensionsPanel: each extension `.glass-panel.glass-panel--hover` with name + version + enable toggle + manage button

**Verify:** `npm run tauri dev`; click logo / dashboard icon to view dashboard; open sessions panel; open extensions panel.

**Steps:**

- [ ] **Step 1: DashboardPanel**

```svelte
<div class="dashboard">
  <div class="dashboard__grid">
    <div class="glass-panel dashboard__stat">
      <div class="dashboard__stat-value">{stats.activeFeatures}</div>
      <div class="dashboard__stat-label">Active features</div>
    </div>
    <div class="glass-panel dashboard__stat">
      <div class="dashboard__stat-value">{stats.activeSessions}</div>
      <div class="dashboard__stat-label">Live sessions</div>
    </div>
    <div class="glass-panel dashboard__stat">
      <div class="dashboard__stat-value">{stats.openTasks}</div>
      <div class="dashboard__stat-label">Open tasks</div>
    </div>
    <div class="glass-panel dashboard__stat">
      <div class="dashboard__stat-value">{stats.pendingPlans}</div>
      <div class="dashboard__stat-label">Pending plans</div>
    </div>
  </div>
  <div class="glass-panel dashboard__section">
    <div class="dashboard__section-title">Recent activity</div>
    <GlobalTimeline ... />
  </div>
</div>
```

CSS:
```css
.dashboard { padding: 18px; display: flex; flex-direction: column; gap: 14px; height: 100%; overflow-y: auto; }
.dashboard__grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; }
.dashboard__stat { padding: 14px 16px; }
.dashboard__stat-value { font-size: var(--text-2xl); font-weight: var(--font-weight-bold); letter-spacing: var(--letter-spacing-tight); margin-bottom: 4px; }
.dashboard__stat-label { font-size: 10.5px; text-transform: uppercase; letter-spacing: var(--letter-spacing-wide); color: var(--text-muted); font-weight: var(--font-weight-semibold); }
.dashboard__section { padding: 14px; }
.dashboard__section-title { font-size: 11px; text-transform: uppercase; letter-spacing: var(--letter-spacing-wide); color: var(--text-muted); font-weight: var(--font-weight-bold); margin-bottom: 10px; }
```

- [ ] **Step 2: SessionsPanel**

Each session row → `.glass-panel--soft`. Live dot from store helper.

```svelte
{#each sessions as s}
  <div class="session-list-row glass-panel--soft" onclick={() => open(s)}>
    <span class="live-dot" class:live-dot--warn={s.status === 'WaitingForInput'} class:live-dot--static={!s.is_active}></span>
    <div class="session-list-row__body">
      <div class="session-list-row__title">{s.title || s.claude_session_id}</div>
      <div class="session-list-row__meta">
        <span class="session-list-row__model">{s.model_name}</span>
        <span>{s.feature_title}</span>
        <span class="session-list-row__ago">{formatRelativeTime(s.last_activity)}</span>
      </div>
    </div>
  </div>
{/each}
```

CSS:
```css
.sessions-panel { padding: 12px; display: flex; flex-direction: column; gap: 8px; height: 100%; overflow-y: auto; }
.session-list-row { display: flex; align-items: center; gap: 10px; padding: 10px 12px; cursor: pointer; transition: border-color var(--dur-base) var(--ease); }
.session-list-row:hover { border-color: var(--border-strong); }
.session-list-row__body { flex: 1; min-width: 0; }
.session-list-row__title { font-size: var(--text-sm); font-weight: var(--font-weight-medium); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.session-list-row__meta { display: flex; gap: 10px; font-size: var(--text-xs); color: var(--text-muted); margin-top: 3px; }
.session-list-row__model { font-family: var(--font-mono); color: var(--accent-hover); }
.session-list-row__ago { font-family: var(--font-mono); margin-left: auto; }
```

- [ ] **Step 3: InstalledExtensionsPanel**

```svelte
{#each extensions as ext}
  <div class="extension-row glass-panel glass-panel--hover">
    <div class="extension-row__head">
      <span class="extension-row__name">{ext.manifest.name}</span>
      <span class="aurora-pill aurora-pill--muted aurora-pill--sm aurora-pill--no-dot">v{ext.manifest.version}</span>
    </div>
    <div class="extension-row__desc">{ext.manifest.description}</div>
    <div class="extension-row__actions">
      <span class="toggle" class:toggle--on={ext.enabled} onclick={() => toggle(ext.id)}></span>
      <button class="btn btn--sm" onclick={() => manage(ext.id)}>Manage</button>
    </div>
  </div>
{/each}
```

CSS:
```css
.extensions-panel { padding: 12px; display: flex; flex-direction: column; gap: 10px; }
.extension-row { padding: 12px; }
.extension-row__head { display: flex; align-items: center; gap: 10px; margin-bottom: 6px; }
.extension-row__name { font-weight: var(--font-weight-semibold); }
.extension-row__desc { font-size: var(--text-sm); color: var(--text-secondary); margin-bottom: 10px; }
.extension-row__actions { display: flex; align-items: center; gap: 10px; }
```

- [ ] **Step 4: Smoke + tests + commit**

```bash
npm run tauri dev
npm run test -- --run
git add src/lib/components/DashboardPanel.svelte src/lib/components/SessionsPanel.svelte src/lib/components/InstalledExtensionsPanel.svelte src/app.css
git commit -m "Aurora Glass: dashboard + sessions + extensions panels"
```

---

## Task 13: Focus rings, selection, scrollbars, reduced-motion

**Goal:** Polish global a11y + motion preferences.

**Files:**
- Modify: `src/app.css` — `:focus-visible`, `::selection`, `::-webkit-scrollbar*`, `@media (prefers-reduced-motion)`

**Acceptance Criteria:**
- [ ] `:focus-visible` outline replaced with violet glow box-shadow
- [ ] `::selection` uses violet at 0.35 alpha
- [ ] Scrollbar thumb uses `var(--accent-border)` on hover
- [ ] `@media (prefers-reduced-motion: reduce)` disables `pulse-dot`, `cursor-blink`, card lift, modal scale; keeps fades

**Verify:** Tab through buttons; select text; toggle OS reduce-motion.

**Steps:**

- [ ] **Step 1: Replace focus rules**

Find current `:focus-visible` block (around line 146). Replace:

```css
:focus-visible {
  outline: none;
  box-shadow: 0 0 0 2px var(--accent-hover), 0 0 0 4px var(--accent-dim);
  border-radius: var(--radius-sm);
}
button:focus-visible, .btn:focus-visible, .list-row:focus-visible, .tab:focus-visible {
  box-shadow: 0 0 0 2px var(--accent-hover), 0 0 0 4px var(--accent-dim);
}
```

- [ ] **Step 2: Replace `::selection`**

```css
::selection { background: rgba(167,139,250,0.35); color: var(--text-primary); }
```

- [ ] **Step 3: Update scrollbar**

```css
::-webkit-scrollbar { width: 6px; height: 6px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.08); border-radius: 3px; transition: background var(--dur-base) var(--ease); }
::-webkit-scrollbar-thumb:hover { background: var(--accent-border); }
```

- [ ] **Step 4: Add reduced-motion**

Append to `app.css`:

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.001ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.001ms !important;
  }
  .live-dot, .terminal__cursor { animation: none !important; opacity: 1 !important; }
  .glass-panel--hover:hover { transform: none !important; }
  .btn--primary:hover { transform: none !important; }
}
```

- [ ] **Step 5: Smoke + commit**

```bash
npm run tauri dev   # tab around; select text; OS-level toggle reduce motion
git add src/app.css
git commit -m "Aurora Glass: focus rings, selection, scrollbars, reduced-motion"
```

---

## Task 14: Visual QA pass + cleanup

**Goal:** Walk every surface, fix gaps, remove dead CSS, run full test suite, screenshot for PR.

**Files:**
- Modify: `src/app.css` — delete obsolete rules
- Modify: any component still rendering with stale styling

**Acceptance Criteria:**
- [ ] Every tab on every feature renders without visual glitches
- [ ] All modals open/close cleanly
- [ ] Notifications appear and dismiss
- [ ] No console errors / no missing CSS variable warnings
- [ ] No unused legacy CSS rules left (heuristic: search for selectors used nowhere in `src/`)
- [ ] `npm run test -- --run` passes
- [ ] `cd src-tauri && cargo check` passes

**Verify:** Full manual walkthrough + automated tests.

**Steps:**

- [ ] **Step 1: Manual walkthrough**

```bash
npm run tauri dev
```

Walk:
- Sidebar: scroll, hover, click, drag-reorder, expand groups, search, right-click menus
- Each tab on a feature: AI (sessions, plans, context, MCP, skills, terminal), Tasks & Notes (toggle/add/delete tasks; edit notes; preview), Files (navigate, preview text/image/binary, drag-drop), Links (add, edit, delete), Repos (clone, branches, open in editor), Timeline (scroll), Board (drag cards), Knowledge (folders, entries, edit)
- Modals: Settings (every section), Storage selector, Storage setup, Create feature, Export/Import, Confirm dialog
- Toasts: trigger via `fh` CLI starting a session
- Search: ⌘/ search, navigate to result
- Workspace tabs: open multiple, switch, close

Note any visual bugs. Fix inline, commit per fix:

```bash
git add <file>
git commit -m "Aurora Glass: fix <issue>"
```

- [ ] **Step 2: Find dead CSS**

```bash
grep -E '^\.[a-zA-Z0-9_-]+\s*[,{]' src/app.css | sed -E 's/^(\.[a-zA-Z0-9_-]+).*/\1/' | sort -u > /tmp/all-classes.txt
# For each class, check if used in src/
while read cls; do
  c="${cls#.}"
  if ! grep -qrE "(class|class:|className|classList)[^>]*\b${c}\b" src/lib src/App.svelte 2>/dev/null; then
    echo "UNUSED: $cls"
  fi
done < /tmp/all-classes.txt
```

Review the UNUSED list. For each true unused (not a primitive/semantic state class), delete the rule from `app.css`. Keep all primitive classes even if not yet referenced — they are the public API.

- [ ] **Step 3: Final tests**

```bash
npm run test -- --run
cd src-tauri && cargo check && cd ..
npm run tauri build   # optional — verify production build succeeds
```

- [ ] **Step 4: Screenshot for PR (manual)**

Capture sidebar + open feature with multiple tabs. Save to `docs/superpowers/specs/screenshots/aurora-glass-after.png` (create dir if needed). Commit.

- [ ] **Step 5: Final commit**

```bash
git add docs/superpowers/specs/screenshots/aurora-glass-after.png src/app.css
git commit -m "Aurora Glass: visual QA pass + cleanup"
```

---

## Self-Review Notes

**Spec coverage check:**
- Tokens [§Design Tokens] → Task 1 ✓
- Primitives [§Reusable Primitives] → Task 2 ✓
- App shell [§Surface-by-Surface > App shell] → Task 3 ✓
- Sidebar → Task 4 ✓
- WorkspaceTabBar + FeatureDetail → Task 5 ✓
- Agents tab (AiPanel + 8 children) → Task 6 ✓
- Tasks & Notes → Task 7 ✓
- Files → Task 7 ✓
- Links → Task 8 ✓
- Repositories → Task 8 ✓
- Timeline + GlobalTimeline → Task 8 ✓
- Board → Task 8 ✓
- Knowledge → Task 9 ✓
- Search → Task 9 ✓
- Settings → Task 10 ✓
- Modals (Modal, ConfirmDialog, Dropdown, IconButton) → Task 10 ✓
- CreateFeatureModal, ExportImportModal → Task 10 ✓
- Storage selector / setup → Task 10 ✓
- Toasts → Task 11 ✓
- StatusBadge / TagBadge → Task 11 ✓
- MarkdownPreview / OpenFgaPreview → Task 11 ✓
- DashboardPanel / SessionsPanel / InstalledExtensionsPanel → Task 12 ✓
- Focus / selection / scrollbars / reduced-motion → Task 13 ✓
- Visual QA → Task 14 ✓

**Type consistency:** All primitive class names (`.glass-panel`, `.aurora-pill`, `.btn`, `.tab`, `.list-row`, `.input`, `.check`, `.live-dot`, `.aurora-bg`, `.scrim`) used identically across all tasks. Helper function names (`isAnySessionWaitingForFeature`, `fileTypeColor`, `linkTypeColor`, `eventColor`, `hexToAlpha`) defined once and referenced consistently.

**Placeholder scan:** No "TODO", "TBD", or vague handwaves. Every step shows concrete code or commands.

**Scope:** ~14 tasks, each producing one clean commit. No task touches more than one logical surface.
