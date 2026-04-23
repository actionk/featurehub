# Feature List Redesign — Unified Glass Cards (V3/I1) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers-extended-cc:subagent-driven-development (recommended) or superpowers-extended-cc:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace sidebar's flat feature rows + tree-connected session sub-rows with unified glass cards that contain each feature's sessions internally, separated by a dashed divider.

**Architecture:** Pure frontend, CSS + Svelte markup only. Introduce a new `.feature-unit` primitive system (card + unit-head + sub-list + sub-session) alongside existing styles, swap Sidebar markup to use the new system, then delete legacy `.feature-item*` and `.feature-session-*` CSS. No store changes; existing `requestShowOverview()`, `getViewingTerminal()`, `terminalsByFeature` wiring already supports the new structure.

**Tech Stack:** Svelte 5 (runes), CSS custom properties, Aurora Glass design tokens (`--accent` violet, `--cyan`, `--amber`), existing `pulse-amber` keyframes.

**Spec:** `docs/superpowers/specs/2026-04-23-feature-list-redesign-v3-design.md`

---

## File Structure

**Modified:**
- `src/app.css` — add new `.feature-unit` system (append, then later delete legacy blocks)
- `src/lib/components/Sidebar.svelte` — restructure feature row markup in the list render loop (lines 877-1014)

**Unchanged:**
- `src/lib/stores/terminals.svelte.ts` (already exposes `getViewingTerminal`, `requestShowOverview`)
- `src/lib/stores/sessionActivity.svelte.ts` (already exposes `getActiveCountForFeature`, `isAnySessionWaitingForFeature`)
- All Sidebar event handlers: `handleMouseDown`, `handleContextMenu`, `onSelect`, `onSelectTerminal`, `onFinishTerminal`, drag state, group logic — all preserved
- Tests: no component tests exist for Sidebar currently; this plan relies on visual QA + existing Vitest smoke run

---

## Task 1: Add `.feature-unit` CSS primitive system

**Goal:** Introduce the new unified-card CSS without using it yet — appended below existing rules so it's inert.

**Files:**
- Modify: `src/app.css` (append section near EOF, before final polish block)

**Acceptance Criteria:**
- [ ] New classes `.feature-unit`, `.unit-head`, `.sub-list`, `.sub-session` + state modifiers defined
- [ ] `npm run build` passes (Vite)
- [ ] Existing `npm run test -- --run` passes (no regressions)
- [ ] Visual: no change yet — Sidebar still uses old markup

**Verify:** `npm run build && npm run test -- --run` → both succeed

**Steps:**

- [ ] **Step 1: Locate append position in `src/app.css`**

The new CSS goes immediately before the line `/* ===== OVERVIEW DASHBOARD GRID ===== */` at line ~3326, OR at the very end of the file. Append at EOF to win cascade naturally:

```bash
# use Grep or Read to confirm current EOF around line 7141
```

- [ ] **Step 2: Append the `.feature-unit` primitive block to `src/app.css`**

Append this exact block at the end of the file:

```css
/* ===========================================================
 * FEATURE UNIT — unified glass cards (spec: feature-list-redesign-v3)
 * =========================================================== */

/* Card container holding one feature + optional sub-list of sessions */
.feature-unit {
  position: relative;
  padding: 8px 10px 10px;
  border-radius: 12px;
  background: rgba(255,255,255,0.028);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  border: 1px solid rgba(255,255,255,0.055);
  transition: background var(--dur-fast) var(--ease),
              box-shadow var(--dur-fast) var(--ease),
              transform var(--dur-fast) var(--ease);
  cursor: pointer;
  width: 100%;
  text-align: left;
  font-family: inherit;
  color: inherit;
}

.feature-unit:hover {
  box-shadow: inset 0 0 0 1px rgba(34,211,238,0.28);
}

.feature-unit--selected {
  background: rgba(167,139,250,0.1);
  box-shadow: inset 0 0 0 1px rgba(167,139,250,0.45),
              0 0 22px rgba(167,139,250,0.14);
}

/* Parent unit highlights violet-subtle when one of its sessions is being viewed */
.feature-unit--has-viewing:not(.feature-unit--selected) {
  background: rgba(167,139,250,0.06);
  box-shadow: inset 0 0 0 1px rgba(167,139,250,0.35);
}

.feature-unit--archived {
  opacity: 0.55;
}

.feature-unit--dragging {
  opacity: 0.35;
}

/* Header row inside the card */
.unit-head {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 22px;
  background: transparent;
  border: none;
  padding: 0;
  color: inherit;
  font-family: inherit;
  cursor: pointer;
  width: 100%;
  text-align: left;
}

.unit-head__title {
  flex: 1;
  min-width: 0;
  font-size: var(--text-sm);
  font-weight: var(--font-weight-medium);
  color: var(--text-secondary);
  letter-spacing: -0.01em;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.25;
  transition: color var(--dur-fast) var(--ease);
}

.feature-unit:hover .unit-head__title {
  color: var(--text-primary);
}

.feature-unit--selected .unit-head__title {
  color: var(--text-primary);
  font-weight: var(--font-weight-semibold);
}

.feature-unit--archived .unit-head__title {
  color: var(--text-muted);
}

.unit-head__right {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

/* Task progress mini-bar reused inside unit-head */
.unit-head__pmb {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}
.unit-head__pmb-track {
  width: 24px;
  height: 3px;
  border-radius: 2px;
  background: rgba(255,255,255,0.08);
  overflow: hidden;
}
.unit-head__pmb-fill {
  height: 100%;
  background: var(--accent-grad-cool);
  border-radius: 2px;
  box-shadow: 0 0 6px rgba(167,139,250,0.5);
  transition: width var(--dur-slow) var(--ease);
}
.unit-head__pmb-label {
  font-size: 9.5px;
  color: var(--text-muted);
  font-variant-numeric: tabular-nums;
  font-family: var(--font-mono);
}

/* Sub-list of sessions inside the card */
.sub-list {
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px dashed rgba(255,255,255,0.06);
  display: flex;
  flex-direction: column;
  gap: 4px;
}

/* Individual session row — darker inset, colored left rail */
.sub-session {
  position: relative;
  padding: 6px 9px;
  border-radius: 7px;
  background: rgba(0,0,0,0.28);
  border-left: 2px solid rgba(34,211,238,0.55);
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 11.5px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: background var(--dur-fast) var(--ease),
              box-shadow var(--dur-fast) var(--ease);
}

.sub-session:hover {
  background: rgba(0,0,0,0.4);
}

.sub-session__dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
  background: var(--cyan);
  box-shadow: 0 0 8px var(--cyan);
}

.sub-session__label {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sub-session__meta {
  font-size: 10px;
  color: var(--text-muted);
  font-variant-numeric: tabular-nums;
  font-family: var(--font-mono);
  flex-shrink: 0;
}

.sub-session__finish {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  border: none;
  border-radius: 3px;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  opacity: 0;
  transition: opacity var(--dur-fast) var(--ease),
              background var(--dur-fast) var(--ease),
              color var(--dur-fast) var(--ease);
}
.sub-session:hover .sub-session__finish { opacity: 1; }
.sub-session__finish:hover { background: var(--red-dim); color: var(--red); }

/* State modifiers */
.sub-session--waiting {
  border-left-color: rgba(245,158,11,0.75);
  background: rgba(245,158,11,0.06);
  color: #fde68a;
}
.sub-session--waiting .sub-session__dot {
  background: var(--amber);
  box-shadow: 0 0 8px var(--amber);
  animation: pulse-amber 2s ease-in-out infinite;
}
.sub-session--waiting:hover {
  background: rgba(245,158,11,0.1);
}

.sub-session--viewing {
  background: rgba(167,139,250,0.2);
  border-left-color: #c4b5fd;
  box-shadow: inset 0 0 0 1px rgba(167,139,250,0.4),
              0 0 12px rgba(167,139,250,0.22);
  color: var(--text-primary);
}
.sub-session--viewing .sub-session__dot {
  background: var(--accent);
  box-shadow: 0 0 8px var(--accent);
}

.sub-session--exited {
  opacity: 0.45;
  border-left-color: rgba(255,255,255,0.1);
  background: rgba(0,0,0,0.14);
}
.sub-session--exited .sub-session__dot {
  background: var(--text-muted);
  box-shadow: none;
}

@media (prefers-reduced-motion: reduce) {
  .sub-session--waiting .sub-session__dot { animation: none; }
  .feature-unit { transition: none; }
}
```

- [ ] **Step 3: Verify build + tests**

Run:
```bash
npm run build
npm run test -- --run
```

Expected: build succeeds, all tests pass. No visual change in the app (new classes aren't used yet).

- [ ] **Step 4: Commit**

```bash
git add src/app.css
git commit -m "feat(aurora): add .feature-unit CSS primitive system"
```

---

## Task 2: Restructure Sidebar markup to use `.feature-unit` system

**Goal:** Replace the per-feature render block in `Sidebar.svelte` to emit a single `.feature-unit` card containing the unit-head + optional `.sub-list` with `.sub-session` rows. Preserve every existing behavior (click, drag, context menu, pin, chevron, terminal click, finish button).

**Files:**
- Modify: `src/lib/components/Sidebar.svelte:882-1011` (the feature-item-wrapper + feature-session-item blocks inside the feature list render loop)

**Acceptance Criteria:**
- [ ] Feature with no sessions renders as single `.feature-unit` with only `.unit-head`
- [ ] Feature with 1+ sessions renders `.feature-unit` with `.unit-head` + `.sub-list` (dashed divider visible)
- [ ] Session state → correct modifier: running (no modifier), `--waiting`, `--viewing`, `--exited`
- [ ] Parent gets `.feature-unit--has-viewing` when any of its terminals matches `viewingTerminalId` and feature is NOT selected (selected wins)
- [ ] Pinned glyph renders inside `.unit-head` before title (only when `feature.pinned`)
- [ ] Tree chevron (`node.hasChildren`) renders inside `.unit-head__right`
- [ ] Task progress mini-bar renders inside `.unit-head__right` when `task_count_total > 0`
- [ ] Click `.unit-head` → calls `onSelect(feature.id)` OR `requestShowOverview()` (existing logic preserved)
- [ ] Click `.sub-session` → calls `onSelectTerminal(featureId, terminalId)`
- [ ] Click `.sub-session__finish` → calls `onFinishTerminal(terminalId, sessionDbId)` and stops propagation
- [ ] Drag behavior works (`handleMouseDown`, `draggingId`, drop target classes still apply to wrapper)
- [ ] Context menu works (`oncontextmenu`)
- [ ] `npm run build && npm run test -- --run` passes
- [ ] Visual: running `npm run tauri dev`, sidebar shows new card-based rows

**Verify:** `npm run build && npm run test -- --run && npm run tauri dev` → builds, tests pass, app boots with new sidebar look

**Steps:**

- [ ] **Step 1: Confirm existing state + helpers**

Read `src/lib/components/Sidebar.svelte:877-1014` to confirm current markup. The render loop has these key elements:
- `<div class="feature-item-wrapper">` — drop-target + drag states (KEEP this wrapper unchanged — it has all the drag-drop classes)
- `<button class="feature-item ...">` — the clickable row (REPLACE with `<div class="feature-unit">` structure)
- `<div class="feature-item-compact list-row">` — inner row with status dot, title, progress, sessions badge, chevron (MOVE contents into `.unit-head`)
- `<div class="feature-session-item">` — per-session sub-row (REPLACE with `<div class="sub-session">` inside `.sub-list`)

The following variables are already in scope:
- `feature`, `node.depth`, `node.hasChildren`, `node.isExpanded`
- `selectedId`, `viewingTerminalId`, `terminalsByFeature`, `draggingId`, `justDropped`, `dropTargetId`, `dropZone`
- Functions: `handleMouseDown`, `handleContextMenu`, `toggleExpanded`, `requestShowOverview`, `getActiveCountForFeature`, `isAnySessionWaitingForFeature`
- Callback props: `onSelect`, `onSelectTerminal`, `onSelectSessions`, `onFinishTerminal`, `onSelectNewTab`

- [ ] **Step 2: Replace lines 882-1011 (the `<div class="feature-item-wrapper">` block and trailing sub-row block) with the new structure**

Apply this edit. Replace the entire block from `<!-- svelte-ignore a11y_no_static_element_interactions -->` (line 882) through the closing `{/if}` for the terminals block (line 1011):

```svelte
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            {@const hasSessions = terminalsByFeature.has(feature.id)}
            {@const sessions = terminalsByFeature.get(feature.id) ?? []}
            {@const hasViewing = !!viewingTerminalId && sessions.some(t => t.terminalId === viewingTerminalId)}
            <div
              class="feature-item-wrapper"
              class:feature-item-drop-above={isOver && dropZone === "above"}
              class:feature-item-drop-child={isOver && dropZone === "child"}
              class:feature-item-drop-below={isOver && dropZone === "below"}
              class:feature-item-dragging={draggingId === feature.id}
              data-feature-id={feature.id}
              style="padding-left: {node.depth * 16}px;"
            >
              <div
                class="feature-unit"
                class:feature-unit--selected={selectedId === feature.id}
                class:feature-unit--has-viewing={hasViewing && selectedId !== feature.id}
                class:feature-unit--archived={feature.archived}
                class:feature-unit--dragging={draggingId === feature.id}
                role="button"
                tabindex="0"
                onmousedown={(e) => handleMouseDown(e, feature)}
                oncontextmenu={(e) => handleContextMenu(e, feature)}
                onclick={() => {
                  if (draggingId || justDropped) return;
                  if (selectedId === feature.id && viewingTerminalId &&
                      sessions.some(t => t.terminalId === viewingTerminalId)) {
                    requestShowOverview();
                    return;
                  }
                  onSelect(feature.id);
                }}
                onkeydown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    onSelect(feature.id);
                  }
                }}
              >
                <div class="unit-head">
                  {#if feature.pinned}
                    <svg class="pin-icon" width="10" height="10" viewBox="0 0 16 16" fill="var(--accent)">
                      <path d="M9.828.722a.5.5 0 01.354.146l4.95 4.95a.5.5 0 01-.707.707l-.707-.707-3.182 3.182a3.5 3.5 0 01-.564.41l-2.05 1.166a.5.5 0 01-.639-.112l-.41-.41-3.96 3.96a.5.5 0 01-.707-.707l3.96-3.96-.41-.41a.5.5 0 01-.112-.639l1.166-2.05a3.5 3.5 0 01.41-.564l3.182-3.182-.707-.707a.5.5 0 01.146-.854z"/>
                    </svg>
                  {/if}
                  <span class="unit-head__title">{feature.title}</span>
                  <div class="unit-head__right">
                    {#if (feature.task_count_total ?? 0) > 0}
                      <div class="unit-head__pmb">
                        <div class="unit-head__pmb-track">
                          <div class="unit-head__pmb-fill" style="width: {Math.round(((feature.task_count_done ?? 0) / (feature.task_count_total ?? 1)) * 100)}%;"></div>
                        </div>
                        <span class="unit-head__pmb-label">{feature.task_count_done ?? 0}/{feature.task_count_total}</span>
                      </div>
                    {/if}
                    {#if getActiveCountForFeature(feature.id) > 0}
                      {@const count = getActiveCountForFeature(feature.id)}
                      {@const waiting = isAnySessionWaitingForFeature(feature.id)}
                      <!-- svelte-ignore a11y_no_static_element_interactions -->
                      <span
                        class="active-sessions-badge aurora-pill"
                        class:aurora-pill--success={!waiting}
                        class:aurora-pill--warn={waiting}
                        title={`${count} active session${count > 1 ? 's' : ''}${waiting ? ' (awaiting input)' : ''} — click to open`}
                        role="button"
                        tabindex="-1"
                        onclick={(e: MouseEvent) => {
                          e.stopPropagation();
                          const terms = terminalsByFeature.get(feature.id);
                          if (terms && terms.length > 0) {
                            onSelectTerminal?.(feature.id, terms[0].terminalId);
                          } else {
                            onSelectSessions?.(feature.id);
                          }
                        }}
                        onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') { e.stopPropagation(); const terms = terminalsByFeature.get(feature.id); if (terms && terms.length > 0) { onSelectTerminal?.(feature.id, terms[0].terminalId); } else { onSelectSessions?.(feature.id); } } }}
                      >
                        <span class="live-dot" class:live-dot--warn={waiting}></span>{count}
                      </span>
                    {/if}
                    {#if node.hasChildren}
                      <!-- svelte-ignore a11y_no_static_element_interactions -->
                      <span class="tree-chevron" class:tree-chevron--expanded={node.isExpanded}
                        onclick={(e: MouseEvent) => { e.stopPropagation(); toggleExpanded(feature.id); }}
                        onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') { e.stopPropagation(); toggleExpanded(feature.id); } }}
                        role="button" tabindex="-1">
                        <svg width="10" height="10" viewBox="0 0 16 16" fill="var(--text-muted)"><path d="M6 3.5l5 4.5-5 4.5V3.5z"/></svg>
                      </span>
                    {/if}
                  </div>
                </div>

                {#if hasSessions}
                  <div class="sub-list">
                    {#each sessions as term (term.terminalId)}
                      {@const isViewing = viewingTerminalId === term.terminalId && selectedId === feature.id}
                      <!-- svelte-ignore a11y_no_static_element_interactions -->
                      <div
                        class="sub-session"
                        class:sub-session--waiting={term.needsInput && !isViewing}
                        class:sub-session--viewing={isViewing}
                        class:sub-session--exited={term.exited}
                        role="button"
                        tabindex="0"
                        onmousedown={(e) => e.stopPropagation()}
                        onclick={(e: MouseEvent) => {
                          e.stopPropagation();
                          onSelectTerminal?.(term.featureId, term.terminalId);
                        }}
                        onkeydown={(e) => { if (e.key === 'Enter') { e.stopPropagation(); onSelectTerminal?.(term.featureId, term.terminalId); } }}
                      >
                        <span class="sub-session__dot"></span>
                        <span class="sub-session__label">{term.label}</span>
                        {#if term.needsInput}
                          <span class="aurora-pill aurora-pill--warn aurora-pill--sm aurora-pill--no-dot">Waiting</span>
                        {:else if term.statusLine}
                          <span class="sub-session__meta">{term.statusLine}</span>
                        {/if}
                        <button
                          class="sub-session__finish"
                          onclick={(e) => { e.stopPropagation(); onFinishTerminal?.(term.terminalId, term.sessionDbId); }}
                          title="Finish session"
                        >
                          <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M3.72 3.72a.75.75 0 011.06 0L8 6.94l3.22-3.22a.749.749 0 011.275.326.749.749 0 01-.215.734L9.06 8l3.22 3.22a.749.749 0 01-.326 1.275.749.749 0 01-.734-.215L8 9.06l-3.22 3.22a.751.751 0 01-1.042-.018.751.751 0 01-.018-1.042L6.94 8 3.72 4.78a.75.75 0 010-1.06z"/></svg>
                        </button>
                      </div>
                    {/each}
                  </div>
                {/if}
              </div>
            </div>
```

Key changes vs. old markup:
- Outer `<div class="feature-item-wrapper">` stays (preserves drag-drop CSS hooks). `node.depth` indentation moves from inner padding to `padding-left` on the wrapper so the whole card nests for child features.
- Replaced `<button class="feature-item">` with `<div class="feature-unit" role="button">` because the card now contains another clickable element (`.unit-head`'s child chevron + `.sub-session` children) — nested buttons aren't valid HTML. The div keeps full keyboard + click semantics via `role`, `tabindex`, `onclick`, `onkeydown`.
- Removed `.feature-item-compact list-row` wrapper — `.unit-head` replaces it.
- Removed `.feature-item-status-dot` (spec explicitly drops status dots — they're hidden in current CSS anyway).
- Sub-session rows now live INSIDE the `.feature-unit` (not as separate sibling `<div>`s), and the old `padding-left: {40 + node.depth * 16}px` status-line extra row is gone — the status line renders inline as `.sub-session__meta` when present.

- [ ] **Step 3: Build + test**

Run:
```bash
npm run build
npm run test -- --run
```

Expected: both pass. Existing Vitest suite doesn't cover Sidebar markup, so any breakage would be a type/compile error.

- [ ] **Step 4: Visual QA — boot the app**

Run `npm run tauri dev`. Manually verify:
1. Every feature row is a rounded glass card
2. Features with running agents show the dashed divider + sub-sessions inside
3. Clicking a feature selects it (violet ring + glow)
4. Clicking a session opens that terminal; viewed session gets violet fill + ring
5. Waiting session: amber rail + pulsing dot + "Waiting" pill
6. Hover over idle card: cyan inner ring
7. Hover over sub-session: background darkens
8. Pinned features show the violet pin glyph
9. Chevron toggles child expansion (for tree features)
10. Right-click opens context menu
11. Drag-to-reorder still works (row becomes 0.35 opacity while dragging, drop indicators appear on other cards)

If any check fails, iterate the markup and re-verify before committing.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/Sidebar.svelte
git commit -m "feat(sidebar): restructure feature rows as unified glass cards"
```

---

## Task 3: Remove legacy `.feature-item*` and `.feature-session-*` CSS

**Goal:** Delete the now-unused CSS blocks for the old flat-row + tree-connector system.

**Files:**
- Modify: `src/app.css` — delete specific ranges

**Acceptance Criteria:**
- [ ] All `.feature-item`, `.feature-item--*`, `.feature-item-compact`, `.feature-item-*` rules removed EXCEPT `.feature-item-wrapper` + `.feature-item-drop-*` + `.feature-item-dragging` (still used by the outer drag-drop wrapper)
- [ ] All `.feature-session-item`, `.feature-session-main`, `.feature-session-finish`, `.feature-session-label`, `.feature-session-status`, `.feature-session-status-line` rules removed
- [ ] `.sidebar-terminal-input-dot`, `.sidebar-terminal-input-badge`, `.sidebar-terminal-dot-wrap` removed (only used by old markup)
- [ ] `@keyframes pulse-amber` preserved (still used by new `.sub-session--waiting`)
- [ ] `.list-row` / `.list-row--active` preserved (used by other primitives across the app)
- [ ] `npm run build` passes; app visually identical to end of Task 2; no console errors
- [ ] Grep for `feature-item[^-]` and `feature-session-` in `src/` returns 0 matches (aside from wrapper-drag classes)

**Verify:** `npm run build && npm run test -- --run` → passes; grep `feature-session-` in `src/` returns only the drag-wrapper references

**Steps:**

- [ ] **Step 1: Grep to confirm which classes are still referenced anywhere**

```bash
# Keep these — still used by the wrapper
# .feature-item-wrapper
# .feature-item-drop-above
# .feature-item-drop-below
# .feature-item-drop-child
# .feature-item-dragging
```

Run Grep for `feature-item[^-]|feature-item-compact|feature-item-title|feature-item-status-dot|feature-item-pmb|feature-item-content|feature-item-row|feature-item-tags|feature-item-tag-dot|feature-item-ticket|feature-item-time|feature-item--detailed|feature-item--selected|feature-item--archived|feature-item--session-active|feature-session-item|feature-session-main|feature-session-finish|feature-session-label|feature-session-status|feature-session-status-line|sidebar-terminal-input-dot|sidebar-terminal-input-badge|sidebar-terminal-dot-wrap|sidebar-terminal-label` inside `src/` (excluding `src/app.css`).

Expected: zero matches after Task 2 (markup no longer uses them). If any match appears, that's a missed reference — fix the markup first.

- [ ] **Step 2: Delete CSS blocks in `src/app.css`**

Using Edit tool, remove these rule blocks (find by grepped line numbers, which may shift as edits accumulate — re-grep as needed):

**Block A — classic feature-item styles (around lines 614-765):**
Delete everything from `.feature-item {` through the end of `.feature-item-time { ... margin-left: auto; }`, BUT keep `.feature-item-wrapper {` and the `.feature-item-wrapper.feature-item-drop-*` and `.feature-item-wrapper.feature-item-dragging` and `.feature-item-wrapper.feature-item-drop-child > .feature-item` rules. The drop-child selector can be changed to `> .feature-unit` to target the new container:

```css
/* Keep, but retarget */
.feature-item-wrapper.feature-item-drop-child > .feature-unit {
  background: rgba(167,139,250,0.18);
  box-shadow: inset 0 0 0 1px rgba(167,139,250,0.45);
}
```

**Block B — feature-session styles (around lines 4804-4930):**
Delete `.feature-session-item`, `.feature-session-item:hover`, `.feature-session-item--exited`, `.feature-session-item--exited .feature-session-main`, `.feature-session-item--input...`, `.feature-session-item--viewing...`, `.feature-session-main`, `.feature-session-item:hover .feature-session-main`, `.feature-session-finish`, `.feature-session-item:hover .feature-session-finish`, `.feature-session-finish:hover`, `.feature-session-label`, `.feature-session-status`, `.feature-session-status-line`.

Keep `.sidebar-terminal-dot-wrap`, `.sidebar-terminal-input-dot`, `@keyframes pulse-amber`, `.sidebar-terminal-label` only if they're still referenced by something else — grep confirms. If unreferenced, delete them too. `@keyframes pulse-amber` is referenced by new CSS — KEEP it.

**Block C — aurora polish feature-item overrides (around lines 7077-7141):**
Delete `.feature-item { padding-top: 4px... }` through `.feature-session-label { font-size: 11.5px; }` — all override blocks targeting the old classes.

- [ ] **Step 3: Build + visual smoke**

```bash
npm run build
npm run test -- --run
npm run tauri dev
```

Expected: build passes, tests pass, sidebar renders exactly as at end of Task 2 (no regression). No console warnings about unknown CSS.

- [ ] **Step 4: Grep double-check**

Run Grep for patterns matching old classes in `src/`:

```
pattern: (feature-item[^-w]|feature-item-(compact|title|status|pmb|content|row|tags|tag-dot|ticket|time|detailed)|feature-item--(selected|archived|session-active)|feature-session-(item|main|finish|label|status))
```

Expected: zero matches.

- [ ] **Step 5: Commit**

```bash
git add src/app.css
git commit -m "chore(aurora): remove legacy feature-item + feature-session CSS"
```

---

## Self-Review

**Spec coverage:**
- Glass card surface — Task 1 (primitive) + Task 2 (markup)
- Parent header with pinned glyph, title, right chip slot — Task 2 markup
- Task progress mini-bar — Task 1 CSS `.unit-head__pmb*` + Task 2 markup
- Sub-list with dashed divider — Task 1 CSS `.sub-list` + Task 2 markup
- Sub-session states (running/waiting/viewing/exited) — Task 1 CSS `.sub-session--*` + Task 2 markup
- `.feature-unit--has-viewing` modifier — Task 2 logic (`hasViewing && selectedId !== feature.id`)
- Click to open terminal / select feature / requestShowOverview — Task 2 preserves existing handlers
- Reduced-motion — Task 1 CSS `@media` block
- Drag-drop, context menu, pin, chevron, group management — Task 2 preserves
- Legacy cleanup — Task 3

**Placeholder scan:** None.

**Type consistency:** `.feature-unit`, `.unit-head`, `.sub-list`, `.sub-session` used consistently across Tasks 1-3. State modifier naming: `--selected`, `--has-viewing`, `--archived`, `--dragging` on `.feature-unit`; `--waiting`, `--viewing`, `--exited` on `.sub-session`.

---
