# Phase 7 — Detail Header & Sidebar Polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the visual gap between the running app and the design target (Image #9) by adding stat chips to the feature header, "Updated X ago" timestamps, last-word gradient title, live session timer with summary, session history durations, token breakdown placeholder, sidebar status dots, and filter tab consolidation.

**Architecture:** Eight focused tasks — all Svelte + CSS changes, no Rust/backend changes needed. Stat chips derive from existing `tasks`, `plans`, `activeSessionCount` state already in `FeatureDetail.svelte`. Session timer uses a reactive `now` clock driven by `setInterval` in `AiPanel.svelte`. Token breakdown is placeholder UI (backend tracking not yet implemented). Sidebar filter "Active" maps to `in_progress | in_review` statuses.

**Tech Stack:** Svelte 5 runes (`$state`, `$derived`, `$effect`), CSS custom properties, `npm run test` for regression check.

**Spec reference:** `docs/superpowers/specs/2026-04-03-ui-redesign-design.md`

---

## File Map

| File | Change |
|---|---|
| `src/lib/components/FeatureDetail.svelte:1-10` | Import `formatRelativeTime` |
| `src/lib/components/FeatureDetail.svelte:90-93` | Add `tasksDone`, `pendingPlanCount` derived values |
| `src/lib/components/FeatureDetail.svelte:396-399` | Title last-word gradient split |
| `src/lib/components/FeatureDetail.svelte:416-417` | Add stat chips HTML |
| `src/lib/components/FeatureDetail.svelte:419-421` | Add "Updated X ago" near ticket ID |
| `src/lib/modules/ai/AiPanel.svelte:1-25` | Import `formatRelativeTime`, `formatDuration` |
| `src/lib/modules/ai/AiPanel.svelte:43-51` | Add `now` clock state, session elapsed derived |
| `src/lib/modules/ai/AiPanel.svelte:397-425` | Active session card — ACTIVE NOW, summary, timer |
| `src/lib/modules/ai/AiPanel.svelte:460-476` | Session history — relative time + duration |
| `src/lib/modules/ai/AiPanel.svelte:503-561` | Insights — token breakdown section |
| `src/lib/utils/format.ts` | Add `formatElapsed(iso, now)` function |
| `src/lib/components/Sidebar.svelte:143` | `statusFilter` stays as-is |
| `src/lib/components/Sidebar.svelte:225-231` | Replace 5 filters with All/Active/Done |
| `src/lib/components/Sidebar.svelte:233-238` | Add `active`, `blocked`, `paused` to `statusColors` |
| `src/lib/components/Sidebar.svelte:241-263` | Update `statusCounts` + `filteredFeatures` for "active" |
| `src/lib/components/Sidebar.svelte:921-953` | Add status dot to feature item compact |
| `src/app.css` | Add `.detail-stat-chips`, `.detail-stat-chip`, `.feature-item-status-dot`, `.bento-session-summary`, `.bento-session-timer`, `.bento-history-meta`, `.bento-token-breakdown` CSS |

---

## Task 1: Feature header stat chips

Show Done count, active Agents count, and pending Plans count as compact chips in the top-right of the feature header.

**Files:**
- Modify: `src/lib/components/FeatureDetail.svelte`
- Modify: `src/app.css`

- [ ] **Step 1: Add derived values in `FeatureDetail.svelte`**

After line 93 (`let activeSessionCount = $derived(getActiveCountForFeature(featureId));`), add:

```svelte
let tasksDone = $derived((tasks ?? []).filter(t => t.done).length);
let pendingPlanCount = $derived((plans ?? []).filter(p => p.status === 'pending').length);
```

- [ ] **Step 2: Add stat chips HTML in `detail-header-row1`**

In `FeatureDetail.svelte`, find the `detail-header-row1` div (line 390). After the status-dropdown-wrapper closing `</div>` at line 416, before the closing `</div>` of `detail-header-row1` at line 417, insert:

```svelte
      <div class="detail-stat-chips">
        <span class="detail-stat-chip detail-stat-chip--green">
          <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M13.3 4.3L6 11.6 2.7 8.3l1.4-1.4L6 8.8l5.9-5.9z"/></svg>
          {tasksDone} Done
        </span>
        <span class="detail-stat-chip {activeSessionCount > 0 ? 'detail-stat-chip--accent' : ''}">
          <span class="detail-stat-live-dot {activeSessionCount > 0 ? 'detail-stat-live-dot--on' : ''}"></span>
          {activeSessionCount} {activeSessionCount === 1 ? 'Agent' : 'Agents'}
        </span>
        {#if pendingPlanCount > 0}
          <span class="detail-stat-chip detail-stat-chip--amber">
            <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M8 1L10 6H15L11 9.5L12.5 14.5L8 11.5L3.5 14.5L5 9.5L1 6H6Z"/></svg>
            {pendingPlanCount} {pendingPlanCount === 1 ? 'Plan' : 'Plans'}
          </span>
        {/if}
      </div>
```

- [ ] **Step 3: Add CSS for stat chips in `app.css`**

In `src/app.css`, find the `/* ===== DETAIL HEADER ===== */` section (around line 1115). After the `.detail-header-row2` block (after line 1171), add:

```css
.detail-stat-chips {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-shrink: 0;
}

.detail-stat-chip {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  padding: 3px 8px;
  border-radius: var(--radius-md);
  font-size: var(--text-xs);
  font-weight: 600;
  background: var(--bg-hover);
  color: var(--text-muted);
  border: 1px solid var(--border);
  white-space: nowrap;
}

.detail-stat-chip--green {
  background: var(--green-dim);
  color: var(--green);
  border-color: transparent;
}

.detail-stat-chip--accent {
  background: var(--accent-dim);
  color: var(--accent);
  border-color: transparent;
}

.detail-stat-chip--amber {
  background: var(--amber-dim);
  color: var(--amber);
  border-color: transparent;
}

.detail-stat-live-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--text-muted);
  flex-shrink: 0;
}

.detail-stat-live-dot--on {
  background: var(--green);
  box-shadow: 0 0 4px var(--green);
}
```

- [ ] **Step 4: Run tests**

```bash
npm run test
```

Expected: 34 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/FeatureDetail.svelte src/app.css
git commit -m "feat: add stat chips (Done/Agents/Plans) to feature detail header"
```

---

## Task 2: Header "Updated X ago"

Show how recently the feature was updated, adjacent to the ticket ID.

**Files:**
- Modify: `src/lib/components/FeatureDetail.svelte`

- [ ] **Step 1: Import `formatRelativeTime` in `FeatureDetail.svelte`**

Add the import at the top of the `<script>` block. Find the existing imports from `"../api/tauri"` (line 3). Add after line 13 (`} from "../api/tauri";`):

```svelte
  import { formatRelativeTime } from "../utils/format";
```

- [ ] **Step 2: Add "Updated X ago" to `detail-header-row2`**

In `FeatureDetail.svelte`, find `detail-header-row2` (line 418). The ticket ID block is at lines 419-421:

```svelte
      {#if feature.ticket_id}
        <span class="detail-ticket-id">{feature.ticket_id}</span>
        <span class="detail-separator">·</span>
      {/if}
```

After line 421 (after the separator), add:

```svelte
      <span class="detail-updated-at">{formatRelativeTime(feature.updated_at)}</span>
      <span class="detail-separator">·</span>
```

- [ ] **Step 3: Add CSS for `.detail-updated-at` in `app.css`**

In `app.css`, find `.detail-ticket-id` (around line 1173). After its rule block, add:

```css
.detail-updated-at {
  font-size: var(--text-xs);
  color: var(--text-muted);
}
```

- [ ] **Step 4: Run tests**

```bash
npm run test
```

Expected: 34 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/FeatureDetail.svelte src/app.css
git commit -m "feat: show 'Updated X ago' in feature detail header"
```

---

## Task 3: Feature title last-word gradient

The design target shows the last word of the feature title in gradient (e.g. "Auth System **Redesign**"). Remove the whole-title gradient and apply it only to the last word.

**Files:**
- Modify: `src/lib/components/FeatureDetail.svelte`
- Modify: `src/app.css`

- [ ] **Step 1: Remove gradient clip from `.detail-header-title` in `app.css`**

Find `.detail-header-title` (around line 1129). The current rule includes:
```css
  background-image: var(--grad-primary);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
```

Remove those four lines. The result should be:

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

Also add a new class for the last-word gradient span:

```css
.detail-title-last-word {
  background-image: var(--grad-primary);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}
```

- [ ] **Step 2: Wrap last word in a gradient span in `FeatureDetail.svelte`**

Find the h1 title block (lines 395-400):

```svelte
        <h1 class="detail-header-title"
          ondblclick={toggleEditTitle} role="button" tabindex="0"
          onkeydown={(e) => e.key === 'Enter' && toggleEditTitle()} title="Double-click to edit">
          {feature.title}
        </h1>
```

Replace with:

```svelte
        <h1 class="detail-header-title"
          ondblclick={toggleEditTitle} role="button" tabindex="0"
          onkeydown={(e) => e.key === 'Enter' && toggleEditTitle()} title="Double-click to edit">
          {@const li = feature.title.lastIndexOf(' ')}
          {#if li >= 0}{feature.title.slice(0, li + 1)}<span class="detail-title-last-word">{feature.title.slice(li + 1)}</span>{:else}<span class="detail-title-last-word">{feature.title}</span>{/if}
        </h1>
```

- [ ] **Step 3: Run tests**

```bash
npm run test
```

Expected: 34 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/FeatureDetail.svelte src/app.css
git commit -m "style: feature title last word uses gradient text"
```

---

## Task 4: Active session card — ACTIVE NOW + summary + timer

The design target shows: "● ACTIVE NOW" pill label (not "LIVE"), the session's summary text as a description, and an elapsed timer.

**Files:**
- Modify: `src/lib/utils/format.ts`
- Modify: `src/lib/modules/ai/AiPanel.svelte`
- Modify: `src/app.css`

- [ ] **Step 1: Add `formatElapsed` to `src/lib/utils/format.ts`**

Append to `format.ts` after `formatFileSize`:

```typescript
export function formatElapsed(isoStart: string, now: number = Date.now()): string {
  const diffMs = now - new Date(isoStart).getTime();
  if (diffMs < 0) return '0s';
  const diffSec = Math.floor(diffMs / 1000);
  if (diffSec < 60) return `${diffSec}s`;
  const diffMin = Math.floor(diffSec / 60);
  if (diffMin < 60) return `${diffMin}m`;
  const h = Math.floor(diffMin / 60);
  const m = diffMin % 60;
  return m > 0 ? `${h}h ${m}m` : `${h}h`;
}
```

- [ ] **Step 2: Add test for `formatElapsed` in `src/lib/utils/format.test.ts`**

Read `format.test.ts` first to understand existing test structure, then add:

```typescript
describe('formatElapsed', () => {
  it('shows seconds for < 60s', () => {
    const now = Date.now();
    const start = new Date(now - 45000).toISOString();
    expect(formatElapsed(start, now)).toBe('45s');
  });

  it('shows minutes for < 60m', () => {
    const now = Date.now();
    const start = new Date(now - 5 * 60 * 1000).toISOString();
    expect(formatElapsed(start, now)).toBe('5m');
  });

  it('shows hours and minutes', () => {
    const now = Date.now();
    const start = new Date(now - (2 * 3600 + 34 * 60) * 1000).toISOString();
    expect(formatElapsed(start, now)).toBe('2h 34m');
  });

  it('shows hours only when no remaining minutes', () => {
    const now = Date.now();
    const start = new Date(now - 3 * 3600 * 1000).toISOString();
    expect(formatElapsed(start, now)).toBe('3h');
  });
});
```

- [ ] **Step 3: Run tests to verify they pass**

```bash
npm run test
```

Expected: tests now include `formatElapsed` tests, all pass.

- [ ] **Step 4: Import `formatElapsed` and `formatDuration` in `AiPanel.svelte`**

AiPanel.svelte has no format imports currently. Add import after `import { onDestroy } from "svelte";` (line 23):

```svelte
  import { formatElapsed, formatDuration } from '../../utils/format';
```

- [ ] **Step 5: Add reactive `now` clock and `sessionElapsed` in `AiPanel.svelte`**

After the `sparkPoints` derivation (line 50), add:

```svelte
  let now = $state(Date.now());
  $effect(() => {
    const id = setInterval(() => { now = Date.now(); }, 30000);
    return () => clearInterval(id);
  });
  let sessionElapsed = $derived(
    activeSessions.length > 0 && activeSessions[0].started_at
      ? formatElapsed(activeSessions[0].started_at, now)
      : null
  );
```

- [ ] **Step 6: Update active session card HTML in `AiPanel.svelte`**

Find the active session card block (lines 397-425). Replace the entire block:

```svelte
        <!-- Active Session (col 2, row 1) -->
        <div class="bento-card {activeSessions.length > 0 ? 'bento-card--live' : ''}" style="grid-area: session;">
          <div class="bento-header">
            <span class="bento-title">Active Session</span>
            {#if activeSessions.length > 0}
              <span class="bento-live-pill">
                <span class="bento-live-ring"></span>
                ACTIVE NOW
              </span>
            {/if}
          </div>
          {#if activeSessions.length > 0}
            {@const s = activeSessions[0]}
            <div class="bento-session-title">{s.title ?? 'Running session'}</div>
            {#if s.summary}
              <div class="bento-session-summary">{s.summary}</div>
            {/if}
            <div class="bento-session-footer">
              {#if sessionElapsed}
                <span class="bento-session-timer">
                  <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor"><path d="M8 1a7 7 0 100 14A7 7 0 008 1zm0 1.5a5.5 5.5 0 110 11 5.5 5.5 0 010-11zM8 4v4.25l2.75 1.75-.75 1.25L7 9V4h1z"/></svg>
                  {sessionElapsed}
                </span>
              {/if}
              <button class="bento-session-open" onclick={() => handleResumeSession(s)}>
                Open Terminal →
              </button>
            </div>
          {:else}
            <div class="bento-empty">
              <span class="bento-empty-text">No active session</span>
              <button class="bento-start-btn" onclick={handleStartSession} disabled={launching}>
                {launching ? 'Starting…' : '▶ Start Session'}
              </button>
            </div>
          {/if}
        </div>
```

- [ ] **Step 7: Add CSS for new session card elements in `app.css`**

Find the `/* ===== BENTO =====` section. After the existing `.bento-session-open` rule, add:

```css
.bento-session-summary {
  font-size: var(--text-xs);
  color: var(--text-secondary);
  line-height: 1.45;
  margin-top: var(--space-1);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.bento-session-footer {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  margin-top: var(--space-3);
}

.bento-session-timer {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  font-size: var(--text-xs);
  font-family: var(--font-mono);
  color: var(--text-muted);
}
```

- [ ] **Step 8: Run tests**

```bash
npm run test
```

Expected: all pass.

- [ ] **Step 9: Commit**

```bash
git add src/lib/utils/format.ts src/lib/utils/format.test.ts src/lib/modules/ai/AiPanel.svelte src/app.css
git commit -m "feat: active session card shows ACTIVE NOW, summary, and elapsed timer"
```

---

## Task 5: Session history — relative time and duration

Replace raw `toLocaleDateString()` in the session history list with relative time + duration.

**Files:**
- Modify: `src/lib/modules/ai/AiPanel.svelte`
- Modify: `src/app.css`

- [ ] **Step 1: Import `formatRelativeTime` in `AiPanel.svelte`**

Update the format import (already added in Task 4) to also include `formatRelativeTime`:

```svelte
  import { formatElapsed, formatDuration, formatRelativeTime } from '../../utils/format';
```

- [ ] **Step 2: Update session history item HTML in `AiPanel.svelte`**

Find the session history block (lines 460-476). Replace the `<button class="bento-history-item">` content:

```svelte
        <div class="bento-session-list">
          {#each sessions.slice(0, 5) as session (session.id)}
            <button class="bento-history-item" onclick={() => handleResumeSession(session)}>
              <span class="bento-history-dot {!session.ended_at ? 'bento-history-dot--live' : ''}"></span>
              <span class="bento-history-title">{session.title ?? 'Session'}</span>
              <span class="bento-history-meta">
                {#if session.started_at}
                  <span class="bento-history-when">{formatRelativeTime(session.started_at)}</span>
                {/if}
                {#if !session.ended_at && session.started_at}
                  <span class="bento-history-dur">{formatElapsed(session.started_at, now)}</span>
                {:else if session.duration_mins}
                  <span class="bento-history-dur">{formatDuration(session.duration_mins)}</span>
                {/if}
              </span>
            </button>
          {/each}
          {#if sessions.length === 0}
            <div class="bento-empty">
              <span class="bento-empty-text">No sessions yet</span>
            </div>
          {/if}
        </div>
```

(Note: increased slice from 4 to 5 to match design target showing more sessions.)

- [ ] **Step 3: Add CSS for history meta in `app.css`**

After `.bento-history-date` (or wherever the history item CSS is), add:

```css
.bento-history-meta {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-left: auto;
  flex-shrink: 0;
}

.bento-history-when {
  font-size: 10px;
  color: var(--text-muted);
  font-family: var(--font-mono);
}

.bento-history-dur {
  font-size: 10px;
  color: var(--text-muted);
  font-family: var(--font-mono);
}
```

- [ ] **Step 4: Run tests**

```bash
npm run test
```

Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add src/lib/modules/ai/AiPanel.svelte src/app.css
git commit -m "feat: session history shows relative time and duration"
```

---

## Task 6: Insights card — token breakdown section

Add a token breakdown section (Input/Output/Cached bars) as placeholder UI. Token data is not yet tracked in the backend — show the structure with "—" values.

**Files:**
- Modify: `src/lib/modules/ai/AiPanel.svelte`
- Modify: `src/app.css`

- [ ] **Step 1: Update Insights card HTML in `AiPanel.svelte`**

Find the `.bento-insights-body` div (line 516). Replace the entire Insights card block (lines 504-561) with:

```svelte
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
                <span class="bento-stat-label">Agent sessions</span>
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
            <div class="bento-insights-divider"></div>
            <div class="bento-token-breakdown">
              <div class="bento-token-breakdown-title">Token breakdown</div>
              <div class="bento-token-row">
                <span class="bento-token-label">Input</span>
                <div class="bento-token-bar-wrap"><div class="bento-token-bar bento-token-bar--input" style="width: 0%;"></div></div>
                <span class="bento-token-count">—</span>
              </div>
              <div class="bento-token-row">
                <span class="bento-token-label">Output</span>
                <div class="bento-token-bar-wrap"><div class="bento-token-bar bento-token-bar--output" style="width: 0%;"></div></div>
                <span class="bento-token-count">—</span>
              </div>
              <div class="bento-token-row">
                <span class="bento-token-label">Caching</span>
                <div class="bento-token-bar-wrap"><div class="bento-token-bar bento-token-bar--cached" style="width: 0%;"></div></div>
                <span class="bento-token-count">—</span>
              </div>
            </div>
          </div>
        </div>
```

Also add `tasksDone` to the destructured imports from TabContext — it was defined in AiPanel.svelte already at line 45 (`let tasksDone = $derived(tasks.filter(t => t.done).length);`), so it's already available.

- [ ] **Step 2: Add token breakdown CSS in `app.css`**

After the `.bento-insights-divider` rule, add:

```css
.bento-token-breakdown {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  min-width: 160px;
}

.bento-token-breakdown-title {
  font-size: var(--text-xs);
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: var(--space-1);
}

.bento-token-row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.bento-token-label {
  font-size: var(--text-xs);
  color: var(--text-muted);
  width: 44px;
  flex-shrink: 0;
}

.bento-token-bar-wrap {
  flex: 1;
  height: 4px;
  background: var(--bg-hover);
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.bento-token-bar {
  height: 100%;
  border-radius: var(--radius-sm);
  transition: width var(--transition-base);
}

.bento-token-bar--input  { background: var(--accent); }
.bento-token-bar--output { background: var(--violet); }
.bento-token-bar--cached { background: var(--cyan); }

.bento-token-count {
  font-size: var(--text-xs);
  font-family: var(--font-mono);
  color: var(--text-muted);
  width: 36px;
  text-align: right;
  flex-shrink: 0;
}
```

- [ ] **Step 3: Run tests**

```bash
npm run test
```

Expected: all pass.

- [ ] **Step 4: Commit**

```bash
git add src/lib/modules/ai/AiPanel.svelte src/app.css
git commit -m "feat: insights card adds token breakdown placeholder section"
```

---

## Task 7: Sidebar — status dots, complete status colors, filter consolidation

Add a circular status dot to each feature item. Update `statusColors` to cover all status values. Simplify filter tabs to All / Active / Done.

**Files:**
- Modify: `src/lib/components/Sidebar.svelte`
- Modify: `src/app.css`

- [ ] **Step 1: Update `statusColors` in `Sidebar.svelte`**

Find `statusColors` (lines 233-238):

```javascript
  const statusColors: Record<string, string> = {
    todo: "var(--text-muted)",
    in_progress: "var(--amber)",
    in_review: "var(--blue)",
    done: "var(--green)",
  };
```

Replace with:

```javascript
  const statusColors: Record<string, string> = {
    active:      "var(--accent)",
    todo:        "var(--text-muted)",
    in_progress: "var(--amber)",
    in_review:   "var(--blue)",
    done:        "var(--green)",
    blocked:     "var(--red)",
    paused:      "var(--purple)",
  };
```

- [ ] **Step 2: Replace `filters` with 3-item array in `Sidebar.svelte`**

Find `filters` (lines 225-231):

```javascript
  const filters = [
    { value: "all", label: "All" },
    { value: "in_progress", label: "In Progress" },
    { value: "in_review", label: "In Review" },
    { value: "todo", label: "Todo" },
    { value: "done", label: "Done" },
  ];
```

Replace with:

```javascript
  const filters = [
    { value: "all",    label: "All" },
    { value: "active", label: "Active" },
    { value: "done",   label: "Done" },
  ];
```

- [ ] **Step 3: Update `statusCounts` derivation for "active" bucket**

Find `statusCounts` (lines 241-253):

```javascript
  let statusCounts = $derived.by(() => {
    const counts: Record<string, number> = {};
    const nonArchived = features.filter((f) => !f.archived);
    for (const f of filters) {
      if (f.value === "all") {
        counts[f.value] = nonArchived.length;
      } else if (f.value === "done") {
        counts[f.value] = features.filter((ff) => ff.archived || ff.status === "done").length;
      } else {
        counts[f.value] = nonArchived.filter((ff) => ff.status === f.value).length;
      }
    }
    return counts;
  });
```

Replace with:

```javascript
  const ACTIVE_STATUSES = new Set(["active", "in_progress", "in_review"]);

  let statusCounts = $derived.by(() => {
    const counts: Record<string, number> = {};
    const nonArchived = features.filter((f) => !f.archived);
    for (const f of filters) {
      if (f.value === "all") {
        counts[f.value] = nonArchived.length;
      } else if (f.value === "done") {
        counts[f.value] = features.filter((ff) => ff.archived || ff.status === "done").length;
      } else if (f.value === "active") {
        counts[f.value] = nonArchived.filter((ff) => ACTIVE_STATUSES.has(ff.status)).length;
      } else {
        counts[f.value] = nonArchived.filter((ff) => ff.status === f.value).length;
      }
    }
    return counts;
  });
```

- [ ] **Step 4: Update `filteredFeatures` derivation for "active" bucket**

Find `filteredFeatures` (lines 257-263):

```javascript
  let filteredFeatures = $derived(
    statusFilter === "all"
      ? features.filter((f) => !f.archived)
      : statusFilter === "done"
        ? features.filter((f) => f.archived || f.status === "done")
        : features.filter((f) => f.status === statusFilter && !f.archived)
  );
```

Replace with:

```javascript
  let filteredFeatures = $derived(
    statusFilter === "all"
      ? features.filter((f) => !f.archived)
      : statusFilter === "done"
        ? features.filter((f) => f.archived || f.status === "done")
        : statusFilter === "active"
          ? features.filter((f) => ACTIVE_STATUSES.has(f.status) && !f.archived)
          : features.filter((f) => f.status === statusFilter && !f.archived)
  );
```

- [ ] **Step 5: Add status dot to feature item compact template**

Find the `<div class="feature-item-compact">` block (line 921). Currently starts with:

```svelte
                  <div class="feature-item-compact">
                    <span class="feature-item-title">{feature.title}</span>
```

Replace with:

```svelte
                  <div class="feature-item-compact">
                    <span class="feature-item-status-dot" style="background: {statusColors[feature.status] ?? 'var(--text-muted)'};"></span>
                    <span class="feature-item-title">{feature.title}</span>
```

- [ ] **Step 6: Add CSS for status dot in `app.css`**

Find `.feature-item-compact` (around line 435). After the rule, add:

```css
.feature-item-status-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
  transition: background var(--transition-fast);
}
```

- [ ] **Step 7: Run tests**

```bash
npm run test
```

Expected: all pass.

- [ ] **Step 8: Commit**

```bash
git add src/lib/components/Sidebar.svelte src/app.css
git commit -m "feat: sidebar status dots, All/Active/Done filter tabs, complete status colors"
```

---

## Task 8: Final verification and merge

- [ ] **Step 1: Run full test suite**

```bash
npm run test
```

Expected: all 34 tests pass (plus new `formatElapsed` tests).

- [ ] **Step 2: Rust check**

```bash
cd /d/LittleBrushGames/FeatureHub/src-tauri && cargo check
```

Expected: no errors.

- [ ] **Step 3: Visual smoke test checklist**

With `npm run tauri dev` running, verify:

1. Feature header shows "✓ X Done", "⊙ N Agents", "⚡ N Plans" chips top-right — green/accent/amber colors ✓
2. "Updated X ago" appears next to ticket ID ✓
3. Feature title: plain first words + gradient last word ✓
4. Active session card shows "ACTIVE NOW" (not "LIVE"), summary text (2 lines max), elapsed timer in mono ✓
5. Session history items show relative time + duration in mono ✓
6. Insights card has token breakdown section (bars at 0%, "—" values) ✓
7. Sidebar feature items have colored status dots ✓
8. Sidebar filter tabs are now All / Active / Done (3 tabs instead of 5) ✓
9. "Active" filter shows in_progress + in_review + active features ✓
10. No layout regressions ✓

- [ ] **Step 4: Commit any smoke test fixes**

```bash
git add -p
git commit -m "style: phase 7 smoke test fixes"
```

- [ ] **Step 5: Merge to master**

```bash
git checkout master
git merge feat/phase7-detail-sidebar-polish --no-ff -m "feat: phase 7 detail header stat chips, session timer, sidebar polish"
git branch -d feat/phase7-detail-sidebar-polish
```

---

## Out of Scope (Future Phase 8)

The sidebar icon rail (narrow 48px left rail with icon navigation) shown in the design target requires a significant structural refactor of `Sidebar.svelte` and `App.svelte`:
- New 48px icon nav rail component
- Feature list moves to a collapsible panel
- Sprint/Backlog section grouping (currently only "Groups" concept)
- Tooltip system for icon labels

This is scoped as a separate plan due to the structural complexity.
