# Phase 4 — Bento Grid & Tab Bar Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the Agents tab's 2-column grid with a 3-column bento dashboard and add keyboard shortcut hints to the feature tab bar.

**Architecture:** Two distinct changes: (1) CSS-only tab shortcut hints via `.sk` span and hover rules in `app.css` plus one HTML change in `FeatureDetail.svelte`; (2) Bento grid — new `.bento` and `.bento-card` CSS classes in `app.css` plus AiPanel.svelte restructure replacing `.ai-grid` with a 6-card bento layout. All bento data (tasks, sessions, plans, links) comes from the existing TabContext that is already passed to AiPanel. MCP/Skills/Context config panels stay as collapsible sections below the bento. No backend changes.

**Tech Stack:** CSS custom properties (Phase 1 tokens), Svelte 5 runes, `npm run test` for regression check.

**Spec reference:** `docs/superpowers/specs/2026-04-03-ui-redesign-design.md` — Phase 4 section.

---

## File Map

| File | Change |
|---|---|
| `src/app.css` (after line ~1504) | Add `.sk` CSS, `.bento` grid CSS, all bento card variant CSS |
| `src/lib/components/FeatureDetail.svelte:497-504` | Add `<span class="sk">` inside tab button |
| `src/lib/modules/ai/AiPanel.svelte` | Add derived values + helper; replace `.ai-grid` overview with `.bento` grid |

---

## Task 1: Tab shortcut hints

**Files:**
- Modify: `src/app.css` (around line 1242, after `.tab-btn:hover`)
- Modify: `src/lib/components/FeatureDetail.svelte` (around line 497)

- [ ] **Step 1: Add `.sk` CSS**

Read `src/app.css` around lines 1242–1248 to find the `.tab-btn:hover` rule. Add the following CSS immediately after the closing `}` of `.tab-btn:hover`:

```css
.sk {
  font-size: 9px;
  font-family: var(--font-mono);
  color: var(--text-muted);
  opacity: 0;
  transition: opacity var(--transition-fast);
  margin-left: var(--space-1);
}

.tab-btn:hover .sk {
  opacity: 0.5;
}
```

- [ ] **Step 2: Add `<span class="sk">` in FeatureDetail.svelte**

Read `src/lib/components/FeatureDetail.svelte` around lines 493–510. Find the tab button HTML which looks like:

```svelte
<button class="tab-btn {activeTab === tab.id ? 'tab-btn--active' : ''}" 
  onclick={() => { switchTab(tab.id); }}
  title="{tab.label} ({tab.shortcutKey})">
  {showEmojis ? `${tab.emoji} ${tab.label}` : tab.label}
  {#each tab.getBadges(tabContext) as badge}
    <span class="tab-count {badge.style === 'active' ? 'tab-count--active' : ''}"
      style="{badge.style === 'warning' ? 'background: var(--amber); color: #000;' : ''}"
      title={badge.title ?? ''}>{badge.text}</span>
  {/each}
</button>
```

Replace with (`.sk` span added after the badges loop):

```svelte
<button class="tab-btn {activeTab === tab.id ? 'tab-btn--active' : ''}" 
  onclick={() => { switchTab(tab.id); }}
  title="{tab.label} ({tab.shortcutKey})">
  {showEmojis ? `${tab.emoji} ${tab.label}` : tab.label}
  {#each tab.getBadges(tabContext) as badge}
    <span class="tab-count {badge.style === 'active' ? 'tab-count--active' : ''}"
      style="{badge.style === 'warning' ? 'background: var(--amber); color: #000;' : ''}"
      title={badge.title ?? ''}>{badge.text}</span>
  {/each}
  <span class="sk">{tab.shortcutKey}</span>
</button>
```

- [ ] **Step 3: Run tests**

```bash
cd D:\LittleBrushGames\FeatureHub && npm run test
```

Expected: all tests pass.

- [ ] **Step 4: Commit**

```bash
cd D:\LittleBrushGames\FeatureHub && git add src/app.css src/lib/components/FeatureDetail.svelte && git commit -m "style: add keyboard shortcut hint to feature tab buttons"
```

---

## Task 2: Bento grid CSS framework

**Files:**
- Modify: `src/app.css` — insert before the `/* ===== TASKS =====` comment (around line 1508)

- [ ] **Step 1: Find insertion point**

Read `src/app.css` around lines 1504–1515. Find the line `/* ===== TASKS =====` (or nearby). Insert the new bento section immediately before it.

- [ ] **Step 2: Add bento CSS**

Insert the following block at the found location:

```css
/* ===== BENTO GRID ===== */

.bento {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  grid-template-rows: auto auto auto;
  grid-template-areas:
    "tasks session plans"
    "tasks history links"
    "insights insights insights";
  gap: var(--space-4);
  align-items: stretch;
  padding: var(--space-4) var(--space-6);
}

.bento-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: var(--space-4);
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  transition: border-color var(--transition-fast);
  min-height: 0;
}

.bento-card:hover {
  border-color: var(--border-strong);
}

.bento-card--span2 {
  grid-row: span 2;
}

.bento-card--full {
  grid-column: 1 / -1;
}

.bento-header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-shrink: 0;
}

.bento-title {
  font-size: 10px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
  flex: 1;
}

.bento-badge {
  background: var(--bg-raised);
  color: var(--text-muted);
  font-size: 10px;
  font-family: var(--font-mono);
  padding: 1px 6px;
  border-radius: var(--radius-full);
  border: 1px solid var(--border);
}

.bento-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  padding: var(--space-4) 0;
}

.bento-empty-icon {
  font-size: 20px;
  opacity: 0.3;
}

.bento-empty-text {
  font-size: var(--text-xs);
  color: var(--text-muted);
}
```

- [ ] **Step 3: Run tests**

```bash
cd D:\LittleBrushGames\FeatureHub && npm run test
```

Expected: all tests pass.

- [ ] **Step 4: Commit**

```bash
cd D:\LittleBrushGames\FeatureHub && git add src/app.css && git commit -m "style: add bento grid CSS framework"
```

---

## Task 3: Bento card variant CSS

**Files:**
- Modify: `src/app.css` — append to the bento section added in Task 2

- [ ] **Step 1: Find the end of the bento CSS block**

Read `src/app.css` and find the `.bento-empty-text` rule you just added. Append all of the following after its closing `}`.

- [ ] **Step 2: Add tasks card CSS**

```css
/* Bento: Tasks card */
.bento-progress-track {
  height: 3px;
  background: var(--bg-raised);
  border-radius: var(--radius-full);
  overflow: hidden;
  flex-shrink: 0;
}

.bento-progress-fill {
  height: 100%;
  background: var(--grad-success);
  border-radius: var(--radius-full);
  transition: width var(--transition-base);
}

.bento-task-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow: hidden;
}

.bento-task-item {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: 3px 0;
}

.bento-task-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--border-strong);
  flex-shrink: 0;
}

.bento-task-dot--done {
  background: var(--green);
}

.bento-task-text {
  font-size: var(--text-xs);
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.bento-task-item--done .bento-task-text {
  color: var(--text-muted);
  text-decoration: line-through;
}

.bento-task-more {
  font-size: 10px;
  color: var(--text-muted);
  padding: 2px 0;
}
```

- [ ] **Step 3: Add live session card CSS**

```css
/* Bento: Live session card */
.bento-card--live {
  background: linear-gradient(135deg, rgba(13, 15, 24, 0.95), rgba(18, 20, 31, 0.98));
  border-color: rgba(77, 124, 255, 0.3);
  box-shadow: 0 0 0 1px rgba(77, 124, 255, 0.08), inset 0 0 40px rgba(77, 124, 255, 0.03);
}

.bento-card--live:hover {
  border-color: rgba(77, 124, 255, 0.45);
}

.bento-live-pill {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  background: rgba(52, 211, 153, 0.12);
  color: var(--green);
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.08em;
  padding: 2px 7px;
  border-radius: var(--radius-full);
  border: 1px solid rgba(52, 211, 153, 0.25);
}

.bento-live-ring {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--green);
  animation: bento-pulse 2s ease-in-out infinite;
}

@keyframes bento-pulse {
  0%, 100% { box-shadow: 0 0 0 0 rgba(52, 211, 153, 0.4); }
  50%       { box-shadow: 0 0 0 4px rgba(52, 211, 153, 0); }
}

.bento-session-title {
  font-size: var(--text-sm);
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.bento-session-meta {
  font-size: 10px;
  font-family: var(--font-mono);
  color: var(--text-muted);
}

.bento-session-open {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  background: var(--accent-dim);
  color: var(--accent);
  border: 1px solid var(--accent-border);
  border-radius: var(--radius-md);
  padding: 4px var(--space-3);
  font-size: var(--text-xs);
  font-weight: 500;
  font-family: inherit;
  cursor: pointer;
  transition: all var(--transition-fast);
  margin-top: auto;
}

.bento-session-open:hover {
  background: var(--accent);
  color: var(--bg-primary);
}

.bento-start-btn {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  background: var(--grad-primary);
  color: #fff;
  border: none;
  border-radius: var(--radius-md);
  padding: 5px var(--space-3);
  font-size: var(--text-xs);
  font-weight: 600;
  font-family: inherit;
  cursor: pointer;
  transition: opacity var(--transition-fast);
  margin-top: var(--space-2);
}

.bento-start-btn:hover:not(:disabled) {
  opacity: 0.88;
}

.bento-start-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
```

- [ ] **Step 4: Add warn plan card CSS**

```css
/* Bento: Warn plan card */
.bento-card--warn .bento-title {
  background-image: var(--grad-warn);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.bento-warn-badge {
  background: rgba(251, 191, 36, 0.12);
  color: var(--amber);
  font-size: 10px;
  font-family: var(--font-mono);
  padding: 1px 6px;
  border-radius: var(--radius-full);
  border: 1px solid rgba(251, 191, 36, 0.25);
}

.bento-plan-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.bento-plan-item {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: 5px var(--space-2);
  background: transparent;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-family: inherit;
  transition: all var(--transition-fast);
  width: 100%;
  text-align: left;
}

.bento-plan-item:hover {
  background: var(--bg-hover);
  border-color: var(--amber);
}

.bento-plan-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--amber);
  flex-shrink: 0;
}

.bento-plan-title {
  flex: 1;
  font-size: var(--text-xs);
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.bento-plan-arrow {
  font-size: 10px;
  color: var(--text-muted);
}
```

- [ ] **Step 5: Add session history + links card CSS**

```css
/* Bento: Session history card */
.bento-session-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.bento-history-item {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: 4px var(--space-2);
  background: transparent;
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  font-family: inherit;
  transition: background var(--transition-fast);
  width: 100%;
  text-align: left;
}

.bento-history-item:hover {
  background: var(--bg-hover);
}

.bento-history-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--border-strong);
  flex-shrink: 0;
}

.bento-history-dot--live {
  background: var(--green);
  box-shadow: 0 0 0 2px rgba(52, 211, 153, 0.2);
}

.bento-history-title {
  flex: 1;
  font-size: var(--text-xs);
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.bento-history-date {
  font-size: 9px;
  font-family: var(--font-mono);
  color: var(--text-muted);
  flex-shrink: 0;
}

/* Bento: Links card */
.bento-link-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.bento-link-item {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: 4px var(--space-2);
  border-radius: var(--radius-md);
  text-decoration: none;
  transition: background var(--transition-fast);
}

.bento-link-item:hover {
  background: var(--bg-hover);
}

.bento-link-item:hover .bento-link-arrow {
  opacity: 1;
}

.bento-link-type {
  font-size: 9px;
  font-family: var(--font-mono);
  color: var(--accent);
  background: var(--accent-dim);
  padding: 1px 4px;
  border-radius: 3px;
  flex-shrink: 0;
  text-transform: lowercase;
  max-width: 48px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.bento-link-name {
  flex: 1;
  font-size: var(--text-xs);
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.bento-link-arrow {
  font-size: 10px;
  color: var(--text-muted);
  opacity: 0;
  transition: opacity var(--transition-fast);
  flex-shrink: 0;
}
```

- [ ] **Step 6: Add insights card CSS**

```css
/* Bento: Insights card */
.bento-insights-body {
  display: flex;
  align-items: stretch;
  gap: var(--space-6);
}

.bento-stats {
  display: flex;
  gap: var(--space-6);
  flex-shrink: 0;
}

.bento-stat {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.bento-stat-value {
  font-size: var(--text-xl);
  font-weight: 700;
  letter-spacing: -0.03em;
  font-family: var(--font-mono);
}

.bento-stat-label {
  font-size: 10px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.bento-insights-divider {
  width: 1px;
  background: var(--border);
  flex-shrink: 0;
  align-self: stretch;
}

.bento-sparkline {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
}

.bento-sparkline-svg {
  width: 100%;
  height: 40px;
  display: block;
}

.bento-range-toggle {
  display: flex;
  gap: 2px;
  margin-left: auto;
}

.bento-range-btn {
  background: transparent;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 2px 6px;
  font-size: 10px;
  font-family: var(--font-mono);
  color: var(--text-muted);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.bento-range-btn:hover {
  color: var(--text-secondary);
  border-color: var(--border-strong);
}

.bento-range-btn--active {
  background: var(--accent-dim);
  color: var(--accent);
  border-color: var(--accent-border);
}
```

- [ ] **Step 7: Run tests**

```bash
cd D:\LittleBrushGames\FeatureHub && npm run test
```

Expected: all tests pass.

- [ ] **Step 8: Commit**

```bash
cd D:\LittleBrushGames\FeatureHub && git add src/app.css && git commit -m "style: add bento card variant CSS (live session, warn plan, insights, history, links)"
```

---

## Task 4: Restructure AiPanel to bento grid

**Files:**
- Modify: `src/lib/modules/ai/AiPanel.svelte`

This task replaces the `.ai-grid` 2-column overview with a `.bento` 3-column grid. The terminal view and plan detail view are NOT changed — only the overview `{:else}` branch of `{#if selectedPlan}`.

**Type reference** (already confirmed in `src/lib/api/types.ts`):
- `Plan`: `{ id, title: string, body, status, session_id, ... }`
- `Session`: `{ id, title: string|null, started_at: string|null, ended_at: string|null, project_path: string|null, ... }`
- `Task`: `{ id, title: string, done: boolean, ... }`
- `Link`: `{ id, title: string, url: string, link_type: string, ... }`

- [ ] **Step 1: Read AiPanel.svelte**

Read `D:\LittleBrushGames\FeatureHub\src\lib\modules\ai\AiPanel.svelte` in full to locate the exact lines for changes.

- [ ] **Step 2: Update TabContext destructuring to include `tasks`**

Find:
```typescript
  let { featureId, feature, sessions, plans, pendingPlanId, onPendingPlanHandled, onSessionsChanged, onRefresh: onPlansChanged }: TabContext = $props();
```

Replace with:
```typescript
  let { featureId, feature, sessions, plans, tasks, pendingPlanId, onPendingPlanHandled, onSessionsChanged, onRefresh: onPlansChanged }: TabContext = $props();
```

- [ ] **Step 3: Add derived values and helper after `let readyDirs = $derived(...)`**

Find the line `let readyDirs = $derived(...)` and add the following block immediately after it:

```typescript
  let activeSessions = $derived(sessions.filter(s => !s.ended_at));
  let links = $derived(feature.links ?? []);
  let tasksDone = $derived(tasks.filter(t => t.done).length);
  let insightsRange = $state<'7d' | '14d' | '30d'>('7d');
  let sparkDays = $derived(insightsRange === '7d' ? 7 : insightsRange === '14d' ? 14 : 30);
  let sparkBuckets = $derived(getSessionBuckets(sessions, sparkDays));
  let sparkMax = $derived(Math.max(...sparkBuckets, 1));
  let sparkPoints = $derived(sparkBuckets.map((v, i) => `${i * 16 + 8},${40 - (v / sparkMax) * 36}`).join(' '));

  function getSessionBuckets(sessions: Session[], days: number): number[] {
    const buckets = new Array(days).fill(0);
    const now = Date.now();
    for (const s of sessions) {
      if (!s.started_at) continue;
      const age = (now - new Date(s.started_at).getTime()) / 86400000;
      const idx = Math.floor(age);
      if (idx >= 0 && idx < days) {
        buckets[days - 1 - idx]++;
      }
    }
    return buckets;
  }
```

- [ ] **Step 4: Replace the overview div**

Find the overview section. It is the `{:else}` branch of `{#if selectedPlan}` and currently starts with:
```svelte
    <div class="ai-grid" style="overflow-y: auto; flex: 1;">
```
and ends just before the `{/if}` that closes the `{:else}` block.

Replace the entire `<div class="ai-grid" ...>...</div>` block (everything from `<div class="ai-grid"` to its closing `</div>`) with the following:

```svelte
    <div style="flex: 1; min-height: 0; overflow-y: auto;">
      <div class="bento">

        <!-- Tasks (col 1, spans rows 1–2) -->
        <div class="bento-card bento-card--span2" style="grid-area: tasks;">
          <div class="bento-header">
            <span class="bento-title">Tasks</span>
            {#if tasks.length > 0}
              <span class="bento-badge">{tasksDone}/{tasks.length}</span>
            {/if}
          </div>
          {#if tasks.length > 0}
            <div class="bento-progress-track">
              <div class="bento-progress-fill" style="width: {Math.round(tasksDone / tasks.length * 100)}%"></div>
            </div>
            <div class="bento-task-list">
              {#each tasks.slice(0, 10) as task (task.id)}
                <div class="bento-task-item {task.done ? 'bento-task-item--done' : ''}">
                  <span class="bento-task-dot {task.done ? 'bento-task-dot--done' : ''}"></span>
                  <span class="bento-task-text">{task.title}</span>
                </div>
              {/each}
              {#if tasks.length > 10}
                <div class="bento-task-more">+{tasks.length - 10} more</div>
              {/if}
            </div>
          {:else}
            <div class="bento-empty">
              <span class="bento-empty-icon">☑</span>
              <span class="bento-empty-text">No tasks yet</span>
            </div>
          {/if}
        </div>

        <!-- Active Session (col 2, row 1) -->
        <div class="bento-card {activeSessions.length > 0 ? 'bento-card--live' : ''}" style="grid-area: session;">
          <div class="bento-header">
            <span class="bento-title">Active Session</span>
            {#if activeSessions.length > 0}
              <span class="bento-live-pill">
                <span class="bento-live-ring"></span>
                LIVE
              </span>
            {/if}
          </div>
          {#if activeSessions.length > 0}
            {@const s = activeSessions[0]}
            <div class="bento-session-title">{s.title ?? 'Running session'}</div>
            {#if s.project_path}
              <div class="bento-session-meta">{s.project_path.split(/[/\\]/).pop()}</div>
            {/if}
            <button class="bento-session-open" onclick={() => handleResumeSession(s)}>
              Open Terminal →
            </button>
          {:else}
            <div class="bento-empty">
              <span class="bento-empty-text">No active session</span>
              <button class="bento-start-btn" onclick={handleStartSession} disabled={launching}>
                {launching ? 'Starting…' : '▶ Start Session'}
              </button>
            </div>
          {/if}
        </div>

        <!-- Pending Plans (col 3, row 1) -->
        <div class="bento-card {pendingPlanCount > 0 ? 'bento-card--warn' : ''}" style="grid-area: plans;">
          <div class="bento-header">
            <span class="bento-title">Plans</span>
            {#if pendingPlanCount > 0}
              <span class="bento-warn-badge">{pendingPlanCount} pending</span>
            {/if}
          </div>
          {#if pendingPlanCount > 0}
            <div class="bento-plan-list">
              {#each pendingPlans.slice(0, 3) as plan (plan.id)}
                <button class="bento-plan-item" onclick={() => { selectedPlan = plan; }}>
                  <span class="bento-plan-dot"></span>
                  <span class="bento-plan-title">{plan.title}</span>
                  <span class="bento-plan-arrow">→</span>
                </button>
              {/each}
            </div>
          {:else}
            <div class="bento-empty">
              <span class="bento-empty-text">No pending plans</span>
            </div>
          {/if}
        </div>

        <!-- Session History (col 2, row 2) -->
        <div class="bento-card" style="grid-area: history;">
          <div class="bento-header">
            <span class="bento-title">Sessions</span>
            {#if sessions.length > 0}
              <span class="bento-badge">{sessions.length}</span>
            {/if}
          </div>
          <div class="bento-session-list">
            {#each sessions.slice(0, 4) as session (session.id)}
              <button class="bento-history-item" onclick={() => handleResumeSession(session)}>
                <span class="bento-history-dot {!session.ended_at ? 'bento-history-dot--live' : ''}"></span>
                <span class="bento-history-title">{session.title ?? 'Session'}</span>
                {#if session.started_at}
                  <span class="bento-history-date">{new Date(session.started_at).toLocaleDateString()}</span>
                {/if}
              </button>
            {/each}
            {#if sessions.length === 0}
              <div class="bento-empty">
                <span class="bento-empty-text">No sessions yet</span>
              </div>
            {/if}
          </div>
        </div>

        <!-- Links (col 3, row 2) -->
        <div class="bento-card" style="grid-area: links;">
          <div class="bento-header">
            <span class="bento-title">Links</span>
            {#if links.length > 0}
              <span class="bento-badge">{links.length}</span>
            {/if}
          </div>
          {#if links.length > 0}
            <div class="bento-link-list">
              {#each links.slice(0, 4) as link (link.id)}
                <a class="bento-link-item" href={link.url} target="_blank" rel="noreferrer">
                  <span class="bento-link-type">{link.link_type}</span>
                  <span class="bento-link-name">{link.title}</span>
                  <span class="bento-link-arrow">↗</span>
                </a>
              {/each}
            </div>
          {:else}
            <div class="bento-empty">
              <span class="bento-empty-text">No links yet</span>
            </div>
          {/if}
        </div>

        <!-- Insights (full width, row 3) -->
        <div class="bento-card bento-card--full" style="grid-area: insights;">
          <div class="bento-header">
            <span class="bento-title">Insights</span>
            <div class="bento-range-toggle">
              {#each (['7d', '14d', '30d'] as const) as range}
                <button
                  class="bento-range-btn {insightsRange === range ? 'bento-range-btn--active' : ''}"
                  onclick={() => { insightsRange = range; }}
                >{range}</button>
              {/each}
            </div>
          </div>
          <div class="bento-insights-body">
            <div class="bento-stats">
              <div class="bento-stat">
                <span class="bento-stat-value gt gt-p">{sessions.length}</span>
                <span class="bento-stat-label">Sessions</span>
              </div>
              <div class="bento-stat">
                <span class="bento-stat-value gt gt-s">{tasksDone}/{tasks.length}</span>
                <span class="bento-stat-label">Tasks done</span>
              </div>
              <div class="bento-stat">
                <span class="bento-stat-value">{links.length}</span>
                <span class="bento-stat-label">Links</span>
              </div>
            </div>
            <div class="bento-insights-divider"></div>
            <div class="bento-sparkline">
              {#if sessions.length > 0}
                <svg class="bento-sparkline-svg" viewBox="0 0 {sparkDays * 16} 40" preserveAspectRatio="none">
                  <defs>
                    <linearGradient id="bento-spark-grad" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="0%" stop-color="#4d7cff" stop-opacity="0.35"/>
                      <stop offset="100%" stop-color="#4d7cff" stop-opacity="0"/>
                    </linearGradient>
                  </defs>
                  <polyline
                    points={sparkPoints}
                    fill="none"
                    stroke="#4d7cff"
                    stroke-width="1.5"
                    stroke-linejoin="round"
                    stroke-linecap="round"
                  />
                  <polygon
                    points={`8,40 ${sparkPoints} ${(sparkDays - 1) * 16 + 8},40`}
                    fill="url(#bento-spark-grad)"
                  />
                </svg>
              {:else}
                <div class="bento-empty" style="flex-direction: row; padding: 0;">
                  <span class="bento-empty-text">No session data</span>
                </div>
              {/if}
            </div>
          </div>
        </div>

      </div>

      <!-- Config panels below bento (MCP, Skills, Context) -->
      <div style="padding: 0 var(--space-6) var(--space-6); display: flex; flex-direction: column; gap: var(--space-4);">
        <div class="ai-block">
          <McpServersPanel {featureId} />
          <SkillsPanel {featureId} />
        </div>
        <div class="ai-block">
          <details>
            <summary class="ai-block-header ai-block-header--foldable" style="cursor: pointer; user-select: none;">
              <svg class="ai-fold-chevron" width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M6 4l4 4-4 4z"/></svg>
              <svg width="13" height="13" viewBox="0 0 16 16" fill="var(--text-muted)" style="flex-shrink: 0;"><path d="M8 1a7 7 0 100 14A7 7 0 008 1zm0 2.5a1 1 0 110 2 1 1 0 010-2zM6.5 7h2v5h-2z"/></svg>
              <span class="ai-block-title">Context</span>
              <span class="ai-fold-hint">Click to expand</span>
            </summary>
            <div style="margin-top: 8px; min-height: 300px; display: flex; flex-direction: column;">
              <ContextEditor {featureId} hideHeader />
            </div>
          </details>
        </div>
      </div>
    </div>
```

- [ ] **Step 5: Run tests**

```bash
cd D:\LittleBrushGames\FeatureHub && npm run test
```

Expected: all 34 tests pass. AiPanel has no direct test coverage so any test failure points to a TypeScript/import problem — fix before committing.

- [ ] **Step 6: Commit**

```bash
cd D:\LittleBrushGames\FeatureHub && git add src/lib/modules/ai/AiPanel.svelte && git commit -m "feat: restructure Agents tab overview to bento grid"
```

---

## Task 5: Final Phase 4 verification

- [ ] **Step 1: Run tests**

```bash
cd D:\LittleBrushGames\FeatureHub && npm run test
```

Expected: all tests pass.

- [ ] **Step 2: Rust check**

```bash
cd D:\LittleBrushGames\FeatureHub\src-tauri && cargo check
```

Expected: no errors.

- [ ] **Step 3: Visual smoke test checklist**

1. Tab bar: hover a tab → shortcut number fades in subtly at the right end ✓
2. Agents tab > overview: shows 3-column bento grid (not 2-column) ✓
3. Tasks card (left, tall): progress bar, task rows with dots ✓
4. Active Session card (middle-top): "LIVE" pill + pulsing dot when a session is running ✓
5. Pending Plans card (right-top): amber badge + clickable plan rows when plans pending ✓
6. Sessions card (middle-bottom): session history rows ✓
7. Links card (right-bottom): links with type badge + ↗ on hover ✓
8. Insights card (full width, bottom): stat numbers, range toggle, sparkline ✓
9. MCP/Skills/Context: still visible below bento ✓
10. Terminal view: still works (open a session, terminal appears) ✓

- [ ] **Step 4: Commit if any smoke test fixes needed**

```bash
cd D:\LittleBrushGames\FeatureHub && git add src/app.css src/lib/modules/ai/AiPanel.svelte src/lib/components/FeatureDetail.svelte && git commit -m "style: phase 4 smoke test fixes"
```
