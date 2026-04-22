# Agents Tab Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the AI tab's bento grid with a unified sessions card (active session + history + insights in one card) and inline config rows (MCP, Skills, Context) replacing the three full-block panels.

**Architecture:** All changes are inside `AiPanel.svelte`'s overview section (the `{:else}` branch when `!activeTerminalId && !selectedPlan`). The terminal overlay, plan detail overlay, and all handler functions remain untouched. MCP/Skills data is loaded directly in AiPanel for the pills display; toggling is handled inline with the same Tauri API calls the existing panels use. Context opens via the existing `Modal` component.

**Tech Stack:** Svelte 5 runes, TypeScript, CSS custom properties in `src/app.css`, Tauri IPC (`getFeatureMcpServers`, `setFeatureMcpServer`, `getFeatureSkills`, `setFeatureSkill`), Chart.js (sparkline already exists, just moves into the card footer).

---

## File Map

| File | What changes |
|------|-------------|
| `src/lib/modules/ai/AiPanel.svelte` | Add state/imports, replace bento HTML + config blocks with new sessions card + config rows |
| `src/app.css` | Remove ~300 lines of bento-* CSS, add ~200 lines of sc-* CSS |
| `src/lib/modules/ai/AiPanel.test.ts` | New — smoke + sessions-slot rendering tests |

**Do NOT modify:** `McpServersPanel.svelte`, `SkillsPanel.svelte`, `ContextEditor.svelte` (they become unused in the overview; don't delete them — other code may reference them later).

---

## Task 1: Add new CSS classes for the sessions card

**Files:**
- Modify: `src/app.css` (append after existing bento block, around line 1900)

- [ ] **Step 1: Append sc-* CSS to app.css**

Add this block after the last `.bento-*` rule (search for `.bento-token-count` — add after its closing brace):

```css
/* ─── Sessions Card (Agents tab redesign) ─── */

.sc-panel {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: var(--space-4) var(--space-6) var(--space-6);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.sessions-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  overflow: hidden;
}

.sc-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px 9px;
  border-bottom: 1px solid var(--border-subtle);
}

.sc-header-title {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--text-muted);
}

.sc-header-count {
  background: rgba(255,255,255,0.05);
  border-radius: 20px;
  padding: 1px 7px;
  font-size: 10px;
  color: var(--text-muted);
}

.sc-header-actions {
  margin-left: auto;
  display: flex;
  gap: 4px;
}

.sc-icon-btn {
  width: 22px;
  height: 22px;
  background: rgba(255,255,255,0.04);
  border: none;
  border-radius: 5px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  cursor: pointer;
  transition: background var(--transition-fast), color var(--transition-fast);
  padding: 0;
}

.sc-icon-btn:hover:not(:disabled) {
  background: rgba(255,255,255,0.08);
  color: var(--text-secondary);
}

.sc-icon-btn:disabled {
  opacity: 0.4;
  cursor: default;
}

/* active slot — expanded (1 active session) */
.sc-active {
  padding: 12px 14px 13px;
  background: linear-gradient(135deg, rgba(0,195,115,0.06) 0%, transparent 65%);
  border-bottom: 1px solid rgba(0,195,115,0.12);
  position: relative;
}

.sc-active::before {
  content: '';
  position: absolute;
  top: 0; left: 0; right: 0;
  height: 1px;
  background: linear-gradient(90deg, rgba(0,210,130,0.45), rgba(0,210,130,0.1), transparent);
}

.sc-active-row1 {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.sc-live-pill {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  background: rgba(0,210,130,0.1);
  border: 1px solid rgba(0,210,130,0.2);
  border-radius: 20px;
  padding: 2px 8px 2px 6px;
  font-size: 9.5px;
  font-weight: 700;
  letter-spacing: 0.07em;
  color: #3de89a;
  text-transform: uppercase;
}

.sc-live-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--green);
  box-shadow: 0 0 5px rgba(0,210,130,0.8);
  animation: sc-blink 2s ease-in-out infinite;
}

@keyframes sc-blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.35; }
}

.sc-active-timer {
  font-size: 10.5px;
  color: var(--text-muted);
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 4px;
}

.sc-active-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  line-height: 1.4;
  margin-bottom: 3px;
}

.sc-active-summary {
  font-size: 11.5px;
  color: var(--text-muted);
  line-height: 1.5;
  margin-bottom: 10px;
}

.sc-open-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  background: rgba(77,124,255,0.14);
  border: 1px solid rgba(77,124,255,0.28);
  color: #8aadff;
  border-radius: 6px;
  padding: 5px 12px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: background var(--transition-fast);
}

.sc-open-btn:hover {
  background: rgba(77,124,255,0.22);
}

/* active slot — compact (2nd+ active session) */
.sc-active-compact {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 8px 14px;
  background: linear-gradient(90deg, rgba(0,195,115,0.04) 0%, transparent 70%);
  border-bottom: 1px solid rgba(0,195,115,0.08);
  position: relative;
}

.sc-active-compact::before {
  content: '';
  position: absolute;
  top: 0; left: 0; bottom: 0;
  width: 2px;
  background: linear-gradient(180deg, rgba(0,210,130,0.5), rgba(0,210,130,0.1));
}

.sc-compact-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--green);
  box-shadow: 0 0 5px rgba(0,210,130,0.6);
  animation: sc-blink 2s ease-in-out infinite;
  animation-delay: 0.4s;
  flex-shrink: 0;
}

.sc-compact-title {
  flex: 1;
  font-size: 12px;
  font-weight: 500;
  color: #a8c4ba;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
}

.sc-compact-timer {
  font-size: 10.5px;
  color: var(--text-muted);
  flex-shrink: 0;
}

.sc-compact-open {
  background: rgba(77,124,255,0.1);
  border: 1px solid rgba(77,124,255,0.2);
  color: #7a9de8;
  border-radius: 5px;
  padding: 3px 9px;
  font-size: 10px;
  font-weight: 600;
  cursor: pointer;
  flex-shrink: 0;
  transition: background var(--transition-fast);
}

.sc-compact-open:hover {
  background: rgba(77,124,255,0.18);
}

/* active slot — no session */
.sc-start-cta {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 11px 14px 12px;
  border-bottom: 1px solid var(--border-subtle);
}

.sc-start-hint {
  font-size: 12px;
  color: var(--text-muted);
}

.sc-start-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: 7px;
  padding: 6px 15px;
  font-size: 11.5px;
  font-weight: 600;
  cursor: pointer;
  margin-left: auto;
  transition: opacity var(--transition-fast);
}

.sc-start-btn:hover:not(:disabled) {
  opacity: 0.88;
}

.sc-start-btn:disabled {
  opacity: 0.5;
  cursor: default;
}

/* past session rows */
.sc-session-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 14px;
  border-bottom: 1px solid rgba(255,255,255,0.03);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.sc-session-row:last-of-type {
  border-bottom: none;
}

.sc-session-row:hover {
  background: rgba(255,255,255,0.025);
}

.sc-session-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: rgba(255,255,255,0.12);
  flex-shrink: 0;
}

.sc-session-name {
  flex: 1;
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.sc-session-when {
  font-size: 10.5px;
  color: var(--text-muted);
  flex-shrink: 0;
}

.sc-session-copy {
  width: 20px;
  height: 20px;
  background: none;
  border: none;
  border-radius: 4px;
  display: none;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  cursor: pointer;
  padding: 0;
  flex-shrink: 0;
  transition: background var(--transition-fast);
}

.sc-session-row:hover .sc-session-copy {
  display: flex;
}

.sc-session-copy:hover {
  background: rgba(255,255,255,0.08);
}

/* insights footer */
.sc-insights {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 14px;
  background: rgba(255,255,255,0.012);
  border-top: 1px solid var(--border-subtle);
}

.sc-insights-stat {
  display: flex;
  flex-direction: column;
  gap: 1px;
  flex-shrink: 0;
}

.sc-insights-val {
  font-size: 16px;
  font-weight: 700;
  color: var(--accent);
  line-height: 1;
}

.sc-insights-label {
  font-size: 9px;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  color: var(--text-muted);
  font-weight: 600;
}

.sc-sparkline {
  flex: 1;
  height: 36px;
  min-width: 0;
}

.sc-range-btns {
  display: flex;
  gap: 3px;
  flex-shrink: 0;
}

.sc-range-btn {
  font-size: 9.5px;
  font-weight: 600;
  padding: 2px 7px;
  border-radius: 20px;
  color: var(--text-muted);
  background: none;
  border: none;
  cursor: pointer;
  transition: background var(--transition-fast), color var(--transition-fast);
}

.sc-range-btn--active {
  background: rgba(77,124,255,0.18);
  color: #8aadff;
}

/* ─── Config rows ─── */

.sc-config {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.sc-config-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 7px 13px;
  border-radius: 7px;
  cursor: pointer;
  transition: background var(--transition-fast);
  position: relative;
}

.sc-config-row:hover {
  background: rgba(255,255,255,0.025);
}

.sc-config-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--text-muted);
  width: 52px;
  flex-shrink: 0;
}

.sc-config-body {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 5px;
  flex-wrap: wrap;
  min-width: 0;
}

.sc-pill {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: rgba(255,255,255,0.05);
  border: 1px solid rgba(255,255,255,0.07);
  border-radius: 20px;
  padding: 2px 8px;
  font-size: 11px;
  color: var(--text-secondary);
}

.sc-pill-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--green);
  box-shadow: 0 0 4px rgba(0,210,130,0.4);
}

.sc-config-add {
  font-size: 11px;
  color: var(--text-muted);
  padding: 2px 4px;
}

.sc-config-caret {
  font-size: 10px;
  color: var(--text-muted);
  flex-shrink: 0;
  transition: transform var(--transition-fast);
}

.sc-config-caret--open {
  transform: rotate(180deg);
}

.sc-ctx-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  background: rgba(255,255,255,0.03);
  border: 1px solid rgba(255,255,255,0.07);
  border-radius: 6px;
  padding: 3px 10px;
  font-size: 11px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.sc-ctx-btn:hover {
  background: rgba(255,255,255,0.06);
}

/* dropdown overlay (shared for MCP + Skills) */
.sc-dropdown {
  position: absolute;
  left: 0; right: 0;
  top: calc(100% + 2px);
  background: var(--bg-elevated, var(--bg-card));
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: 0 8px 24px rgba(0,0,0,0.4);
  z-index: 50;
  padding: 6px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.sc-dropdown-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 12px;
  color: var(--text-secondary);
  background: none;
  border: none;
  width: 100%;
  text-align: left;
  transition: background var(--transition-fast);
}

.sc-dropdown-item:hover {
  background: rgba(255,255,255,0.05);
}

.sc-dropdown-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  border: 1.5px solid rgba(255,255,255,0.2);
  flex-shrink: 0;
}

.sc-dropdown-dot--on {
  background: var(--green);
  border-color: var(--green);
  box-shadow: 0 0 4px rgba(0,210,130,0.4);
}

/* active-count badge in header */
.sc-active-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: rgba(0,210,130,0.1);
  border: 1px solid rgba(0,210,130,0.2);
  border-radius: 20px;
  padding: 1px 8px;
  font-size: 9px;
  font-weight: 700;
  color: #3de89a;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.sc-active-badge-dot {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: var(--green);
}
```

- [ ] **Step 2: Verify app compiles**

```bash
cd src-tauri && cargo check 2>&1 | tail -5
```
(CSS errors show at Vite level, not cargo — just confirm Rust still compiles.)

- [ ] **Step 3: Commit**

```bash
git add src/app.css
git commit -m "style: add sc-* CSS classes for agents tab sessions card"
```

---

## Task 2: Add new state and imports to AiPanel.svelte

**Files:**
- Modify: `src/lib/modules/ai/AiPanel.svelte` (script section, lines 1–30)

- [ ] **Step 1: Add imports**

In the `<script lang="ts">` block, add these imports alongside the existing ones:

```typescript
import type { McpServer, FeatureMcpServer, Skill, FeatureSkill } from "../../api/tauri";
import { getFeatureMcpServers, setFeatureMcpServer, getFeatureSkills, setFeatureSkill } from "../../api/tauri";
import Modal from "../../components/ui/Modal.svelte";
```

Remove these existing imports (they'll no longer be used in the overview):
```typescript
import McpServersPanel from "./McpServersPanel.svelte";
import SkillsPanel from "./SkillsPanel.svelte";
```
Keep `ContextEditor` — it's still used inside the Modal.

- [ ] **Step 2: Add state variables**

After the `let activeSessions = $derived(...)` line, add:

```typescript
let pastSessions = $derived(
  [...sessions]
    .filter(s => s.ended_at || (activeSessions.length > 0 && !activeSessions.includes(s)))
    .sort((a, b) => (sessionLastActive(b) ?? '').localeCompare(sessionLastActive(a) ?? ''))
    .slice(0, 8)
);
```

Wait — `activeSessions` is already `sessions.filter(s => !s.ended_at)`. Past sessions are those with `ended_at`. So:

```typescript
let pastSessions = $derived(
  [...sessions]
    .filter(s => !!s.ended_at)
    .sort((a, b) => (sessionLastActive(b) ?? '').localeCompare(sessionLastActive(a) ?? ''))
    .slice(0, 8)
);
```

- [ ] **Step 3: Add MCP/Skills state**

After the `pastSessions` declaration:

```typescript
// MCP + Skills data for config rows
let mcpAllServers = $state<McpServer[]>([]);
let mcpOverrides = $state<FeatureMcpServer[]>([]);
let skillsAll = $state<Skill[]>([]);
let skillsOverrides = $state<FeatureSkill[]>([]);

let enabledMcpServers = $derived(
  mcpAllServers.filter(srv => {
    const override = mcpOverrides.find(o => o.server_name === srv.name);
    return override ? override.enabled : srv.default_enabled;
  })
);

let enabledSkills = $derived(
  skillsAll.filter(sk => {
    const override = skillsOverrides.find(o => o.skill_id === sk.id);
    return override ? override.enabled : sk.default_enabled;
  })
);

// Config row open/close state
let mcpDropdownOpen = $state(false);
let skillsDropdownOpen = $state(false);
let contextModalOpen = $state(false);
```

- [ ] **Step 4: Add config data loading effect**

After the existing `onMount` block, add:

```typescript
async function loadConfigData() {
  try {
    const [settings, mcpOvr, skOvr] = await Promise.all([
      getCachedSettings(),
      getFeatureMcpServers(featureId),
      getFeatureSkills(featureId),
    ]);
    const extServers: McpServer[] = (settings.extensions ?? [])
      .filter(e => e.enabled && e.mcp_server)
      .map(e => ({ ...e.mcp_server!, name: e.id }));
    mcpAllServers = [...(settings.mcp_servers ?? []), ...extServers];
    mcpOverrides = mcpOvr;
    skillsAll = settings.skills ?? [];
    skillsOverrides = skOvr;
  } catch (e) {
    console.error('Failed to load config data:', e);
  }
}

$effect(() => {
  featureId; // reactive dep — reload when feature changes
  loadConfigData();
});
```

- [ ] **Step 5: Add toggle handlers**

```typescript
async function toggleMcpServer(server: McpServer) {
  const enabled = !!enabledMcpServers.find(s => s.name === server.name);
  await setFeatureMcpServer(featureId, server.name, !enabled);
  const idx = mcpOverrides.findIndex(o => o.server_name === server.name);
  if (idx >= 0) {
    mcpOverrides[idx] = { server_name: server.name, enabled: !enabled };
  } else {
    mcpOverrides = [...mcpOverrides, { server_name: server.name, enabled: !enabled }];
  }
}

async function toggleSkill(skill: Skill) {
  const enabled = !!enabledSkills.find(s => s.id === skill.id);
  await setFeatureSkill(featureId, skill.id, !enabled);
  const idx = skillsOverrides.findIndex(o => o.skill_id === skill.id);
  if (idx >= 0) {
    skillsOverrides[idx] = { skill_id: skill.id, enabled: !enabled };
  } else {
    skillsOverrides = [...skillsOverrides, { skill_id: skill.id, enabled: !enabled }];
  }
}
```

- [ ] **Step 6: Run type check**

```bash
cd D:/LittleBrushGames/FeatureHub && npx tsc --noEmit 2>&1 | head -30
```

Expected: no new errors (there may be pre-existing ones — only fix new ones introduced by this task).

- [ ] **Step 7: Commit**

```bash
git add src/lib/modules/ai/AiPanel.svelte
git commit -m "feat(agents): add MCP/Skills state and config data loading to AiPanel"
```

---

## Task 3: Replace bento HTML with sessions card

**Files:**
- Modify: `src/lib/modules/ai/AiPanel.svelte` (template section, around line 523)

The section to replace is the `{:else}` block inside `{#if !activeTerminalId}` → `{#if selectedPlan} ... {:else} ... {/if}`. Specifically, replace everything from `<div style="flex: 1; min-height: 0; overflow-y: auto;">` down through the closing `</div>` before `{/if}` at the bottom.

- [ ] **Step 1: Write the failing test**

Create `src/lib/modules/ai/AiPanel.test.ts`:

```typescript
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render } from '@testing-library/svelte';
import AiPanel from './AiPanel.svelte';
import type { TabContext } from '../registry';

// Mock all Tauri IPC
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(null),
}));
vi.mock('../../stores/settings.svelte', () => ({
  getCachedSettings: vi.fn().mockResolvedValue({ mcp_servers: [], skills: [], extensions: [] }),
}));
vi.mock('../../stores/sessionActivity.svelte', () => ({
  getPanelSessions: vi.fn().mockReturnValue([]),
}));
vi.mock('../../stores/terminals.svelte', () => ({
  getTerminalsForFeature: vi.fn().mockReturnValue([]),
  addTerminal: vi.fn(),
  removeTerminal: vi.fn(),
  markExited: vi.fn(),
  getPendingViewRequest: vi.fn().mockReturnValue({ version: -1, terminalId: null }),
  getPendingResumeRequest: vi.fn().mockReturnValue({ version: -1, sessionDbId: null }),
  setViewingTerminal: vi.fn(),
}));
vi.mock('../../stores/tabToolbar.svelte', () => ({
  setToolbarActions: vi.fn(),
  clearToolbarActions: vi.fn(),
}));
vi.mock('../../api/tauri', () => ({
  ptySpawnSession: vi.fn(),
  ptyResumeSession: vi.fn(),
  ptyKill: vi.fn(),
  finishEmbeddedSession: vi.fn(),
  detectIdes: vi.fn().mockResolvedValue([]),
  openInIde: vi.fn(),
  getFhCliPath: vi.fn().mockResolvedValue('fh'),
  scanSessions: vi.fn(),
  getFeatureMcpServers: vi.fn().mockResolvedValue([]),
  setFeatureMcpServer: vi.fn(),
  getFeatureSkills: vi.fn().mockResolvedValue([]),
  setFeatureSkill: vi.fn(),
}));

function makeContext(overrides: Partial<TabContext> = {}): TabContext {
  return {
    featureId: 'feat-1',
    feature: {
      id: 'feat-1',
      title: 'Test Feature',
      status: 'active',
      sort_order: 0,
      directories: [],
      links: [],
      tags: [],
    },
    sessions: [],
    plans: [],
    tasks: [],
    note: null,
    allTags: [],
    activeSessionCount: 0,
    pendingPlanId: null,
    onPendingPlanHandled: vi.fn(),
    onSessionsChanged: vi.fn(),
    onRefresh: vi.fn(),
    ...overrides,
  };
}

describe('AiPanel sessions card', () => {
  it('shows Start Session CTA when no active sessions', () => {
    const { container } = render(AiPanel, { props: makeContext({ sessions: [] }) });
    expect(container.querySelector('.sc-start-cta')).toBeTruthy();
    expect(container.querySelector('.sc-start-btn')).toBeTruthy();
    expect(container.querySelector('.sc-active')).toBeFalsy();
  });

  it('shows expanded active card when one session is running', () => {
    const ctx = makeContext({
      sessions: [{
        id: 's1',
        feature_id: 'feat-1',
        claude_session_id: null,
        title: 'Working on auth',
        summary: null,
        started_at: new Date().toISOString(),
        ended_at: null,
        duration_mins: null,
        project_path: null,
        branch: null,
      }],
    });
    const { container } = render(AiPanel, { props: ctx });
    expect(container.querySelector('.sc-active')).toBeTruthy();
    expect(container.querySelector('.sc-live-pill')).toBeTruthy();
    expect(container.querySelector('.sc-start-cta')).toBeFalsy();
  });

  it('shows compact row for second active session', () => {
    const now = new Date().toISOString();
    const ctx = makeContext({
      sessions: [
        { id: 's1', feature_id: 'feat-1', claude_session_id: null, title: 'First', summary: null, started_at: now, ended_at: null, duration_mins: null, project_path: null, branch: null },
        { id: 's2', feature_id: 'feat-1', claude_session_id: null, title: 'Second', summary: null, started_at: now, ended_at: null, duration_mins: null, project_path: null, branch: null },
      ],
    });
    const { container } = render(AiPanel, { props: ctx });
    expect(container.querySelector('.sc-active')).toBeTruthy();
    expect(container.querySelector('.sc-active-compact')).toBeTruthy();
  });
});
```

- [ ] **Step 2: Run tests to confirm they fail**

```bash
cd D:/LittleBrushGames/FeatureHub && npm run test -- AiPanel --run 2>&1 | tail -20
```

Expected: FAIL — `sc-start-cta`, `sc-active` classes not found (bento HTML still in place).

- [ ] **Step 3: Replace the bento HTML section**

Find this block in AiPanel.svelte (around line 523):
```svelte
    <div style="flex: 1; min-height: 0; overflow-y: auto;">
      <div class="bento">
```

Replace everything from that opening `<div style="flex: 1...">` through the closing `</div>` of the config section (just before `{/if}` that closes the `{:else}` — around line 787) with:

```svelte
    <div class="sc-panel">

      <!-- Sessions card -->
      <div class="sessions-card">

        <!-- Header -->
        <div class="sc-header">
          <span class="sc-header-title">Sessions</span>
          {#if sessions.length > 0}
            <span class="sc-header-count">{sessions.length}</span>
          {/if}
          {#if activeSessions.length >= 2}
            <span class="sc-active-badge">
              <span class="sc-active-badge-dot"></span>
              {activeSessions.length} active
            </span>
          {/if}
          <div class="sc-header-actions">
            <button class="sc-icon-btn" onclick={handleStartSession} disabled={launching} title="New session">
              <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M8 1a1 1 0 011 1v5h5a1 1 0 110 2H9v5a1 1 0 11-2 0V9H2a1 1 0 010-2h5V2a1 1 0 011-1z"/></svg>
            </button>
            <button class="sc-icon-btn" onclick={handleCopyStartCommand} title="Copy fh start command">
              {#if copiedStart}
                <svg width="10" height="10" viewBox="0 0 16 16" fill="var(--green)"><path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 0 1 1.06-1.06L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/></svg>
              {:else}
                <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M0 6.75C0 5.784.784 5 1.75 5h1.5a.75.75 0 0 1 0 1.5h-1.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-1.5a.75.75 0 0 1 1.5 0v1.5A1.75 1.75 0 0 1 9.25 16h-7.5A1.75 1.75 0 0 1 0 14.25ZM5 1.75C5 .784 5.784 0 6.75 0h7.5C15.216 0 16 .784 16 1.75v7.5A1.75 1.75 0 0 1 14.25 11h-7.5A1.75 1.75 0 0 1 5 9.25Zm1.75-.25a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25Z"/></svg>
              {/if}
            </button>
            {#if sessions.length > 0}
              <button class="sc-icon-btn" onclick={handleScan} disabled={scanning} title="Scan filesystem for sessions">
                <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M8 2.5a5.487 5.487 0 00-4.131 1.869l1.204 1.204A.25.25 0 014.896 6H1.25A.25.25 0 011 5.75V2.104a.25.25 0 01.427-.177l1.38 1.38A7.001 7.001 0 0115 8a.75.75 0 01-1.5 0A5.5 5.5 0 008 2.5zM1.75 8a.75.75 0 01.75.75 5.5 5.5 0 009.131 4.131l-1.204-1.204A.25.25 0 0110.604 11h3.646a.25.25 0 01.25.25v3.646a.25.25 0 01-.427.177l-1.38-1.38A7.001 7.001 0 011 8.75.75.75 0 011.75 8z"/></svg>
              </button>
            {/if}
          </div>
        </div>

        <!-- Active session slot -->
        {#if activeSessions.length === 0}
          <div class="sc-start-cta">
            <span class="sc-start-hint">No active session</span>
            <button class="sc-start-btn" onclick={handleStartSession} disabled={launching}>
              {launching ? 'Starting…' : '▶ Start Session'}
            </button>
          </div>
        {:else}
          {@const primarySession = activeSessions[0]}
          <div class="sc-active">
            <div class="sc-active-row1">
              <span class="sc-live-pill">
                <span class="sc-live-dot"></span>
                Active Now
              </span>
              {#if sessionElapsed}
                <span class="sc-active-timer">
                  <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M8 1a7 7 0 100 14A7 7 0 008 1zm0 1.5a5.5 5.5 0 110 11 5.5 5.5 0 010-11zM8 4v4.25l2.75 1.75-.75 1.25L7 9V4h1z"/></svg>
                  {sessionElapsed}
                </span>
              {/if}
            </div>
            <div class="sc-active-title">{primarySession.title ?? 'Running session'}</div>
            {#if primarySession.summary}
              <div class="sc-active-summary">{primarySession.summary}</div>
            {/if}
            <button class="sc-open-btn" onclick={() => handleResumeSession(primarySession)}>
              Open Terminal →
            </button>
          </div>
          {#each activeSessions.slice(1) as session (session.id)}
            <div class="sc-active-compact">
              <span class="sc-compact-dot"></span>
              <span class="sc-compact-title">{session.title ?? 'Running session'}</span>
              {#if session.started_at}
                <span class="sc-compact-timer">{formatElapsed(session.started_at, now)}</span>
              {/if}
              <button class="sc-compact-open" onclick={() => handleResumeSession(session)}>Open →</button>
            </div>
          {/each}
        {/if}

        <!-- Past sessions -->
        {#each pastSessions as session (session.id)}
          <div class="sc-session-row" onclick={() => handleResumeSession(session)}>
            <span class="sc-session-dot"></span>
            <span class="sc-session-name">{session.title ?? 'Session'}</span>
            <span class="sc-session-when">
              {#if sessionLastActive(session)}
                {formatRelativeTime(sessionLastActive(session)!)}
              {/if}
            </span>
            {#if session.claude_session_id}
              <button
                class="sc-session-copy"
                onclick={(e) => { e.stopPropagation(); handleCopyResumeCommand(session); }}
                title="Copy resume command"
              >
                {#if copiedResumeId === session.id}
                  <svg width="9" height="9" viewBox="0 0 16 16" fill="var(--green)"><path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 0 1 1.06-1.06L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/></svg>
                {:else}
                  <svg width="9" height="9" viewBox="0 0 16 16" fill="currentColor"><path d="M0 6.75C0 5.784.784 5 1.75 5h1.5a.75.75 0 0 1 0 1.5h-1.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-1.5a.75.75 0 0 1 1.5 0v1.5A1.75 1.75 0 0 1 9.25 16h-7.5A1.75 1.75 0 0 1 0 14.25ZM5 1.75C5 .784 5.784 0 6.75 0h7.5C15.216 0 16 .784 16 1.75v7.5A1.75 1.75 0 0 1 14.25 11h-7.5A1.75 1.75 0 0 1 5 9.25Zm1.75-.25a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25Z"/></svg>
                {/if}
              </button>
            {/if}
          </div>
        {/each}

        <!-- Insights footer -->
        <div class="sc-insights">
          <div class="sc-insights-stat">
            <span class="sc-insights-val">{sessions.length}</span>
            <span class="sc-insights-label">Sessions</span>
          </div>
          <canvas bind:this={sparkCanvas} class="sc-sparkline"></canvas>
          <div class="sc-range-btns">
            {#each (['7d', '14d', '30d'] as const) as range}
              <button
                class="sc-range-btn {insightsRange === range ? 'sc-range-btn--active' : ''}"
                onclick={() => { insightsRange = range; }}
              >{range}</button>
            {/each}
          </div>
        </div>

      </div>

      <!-- Config rows -->
      <div class="sc-config">

        <!-- MCP Servers row -->
        <div class="sc-config-row" onclick={() => { mcpDropdownOpen = !mcpDropdownOpen; skillsDropdownOpen = false; }}>
          <span class="sc-config-label">MCP</span>
          <div class="sc-config-body">
            {#each enabledMcpServers as srv (srv.name)}
              <span class="sc-pill"><span class="sc-pill-dot"></span>{srv.name}</span>
            {/each}
            {#if enabledMcpServers.length === 0}
              <span style="font-size:11px;color:var(--text-muted)">None enabled</span>
            {/if}
          </div>
          {#if mcpAllServers.length > 0}
            <span class="sc-config-caret {mcpDropdownOpen ? 'sc-config-caret--open' : ''}">▾</span>
          {/if}
          {#if mcpDropdownOpen}
            <div class="sc-dropdown" onclick={(e) => e.stopPropagation()}>
              {#each mcpAllServers as srv (srv.name)}
                {@const on = !!enabledMcpServers.find(s => s.name === srv.name)}
                <button class="sc-dropdown-item" onclick={() => toggleMcpServer(srv)}>
                  <span class="sc-dropdown-dot {on ? 'sc-dropdown-dot--on' : ''}"></span>
                  {srv.name}
                </button>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Skills row -->
        <div class="sc-config-row" onclick={() => { skillsDropdownOpen = !skillsDropdownOpen; mcpDropdownOpen = false; }}>
          <span class="sc-config-label">Skills</span>
          <div class="sc-config-body">
            {#each enabledSkills.slice(0, 4) as sk (sk.id)}
              <span class="sc-pill">{sk.name}</span>
            {/each}
            {#if enabledSkills.length > 4}
              <span class="sc-pill">+{enabledSkills.length - 4}</span>
            {/if}
            {#if enabledSkills.length === 0}
              <span style="font-size:11px;color:var(--text-muted)">None enabled</span>
            {/if}
          </div>
          {#if skillsAll.length > 0}
            <span class="sc-config-caret {skillsDropdownOpen ? 'sc-config-caret--open' : ''}">▾</span>
          {/if}
          {#if skillsDropdownOpen}
            <div class="sc-dropdown" onclick={(e) => e.stopPropagation()}>
              {#each skillsAll as sk (sk.id)}
                {@const on = !!enabledSkills.find(s => s.id === sk.id)}
                <button class="sc-dropdown-item" onclick={() => toggleSkill(sk)}>
                  <span class="sc-dropdown-dot {on ? 'sc-dropdown-dot--on' : ''}"></span>
                  {sk.name}
                </button>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Context row -->
        <div class="sc-config-row" onclick={() => { contextModalOpen = true; mcpDropdownOpen = false; skillsDropdownOpen = false; }}>
          <span class="sc-config-label">Context</span>
          <div class="sc-config-body">
            <button class="sc-ctx-btn" onclick|stopPropagation={() => { contextModalOpen = true; }}>
              <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor"><path d="M0 2.75A2.75 2.75 0 012.75 0h10.5A2.75 2.75 0 0116 2.75v10.5A2.75 2.75 0 0113.25 16H2.75A2.75 2.75 0 010 13.25zm2.75-1.25a1.25 1.25 0 00-1.25 1.25v10.5c0 .69.56 1.25 1.25 1.25h10.5c.69 0 1.25-.56 1.25-1.25V2.75c0-.69-.56-1.25-1.25-1.25zM4 5.75A.75.75 0 014.75 5h6.5a.75.75 0 010 1.5h-6.5A.75.75 0 014 5.75zm0 3A.75.75 0 014.75 8h4a.75.75 0 010 1.5h-4A.75.75 0 014 8.75z"/></svg>
              View / Edit Context
            </button>
          </div>
        </div>

      </div>

      <!-- Context modal -->
      <Modal open={contextModalOpen} onClose={() => { contextModalOpen = false; }} width="680px">
        <div style="padding: 20px; display: flex; flex-direction: column; gap: 12px; min-height: 400px;">
          <div style="display:flex;align-items:center;justify-content:space-between;">
            <span style="font-size:14px;font-weight:600;color:var(--text-primary)">Context</span>
            <button onclick={() => { contextModalOpen = false; }} style="background:none;border:none;color:var(--text-muted);cursor:pointer;padding:4px;font-size:13px;">✕</button>
          </div>
          <ContextEditor {featureId} hideHeader />
        </div>
      </Modal>

    </div>
```

- [ ] **Step 4: Run tests**

```bash
cd D:/LittleBrushGames/FeatureHub && npm run test -- AiPanel --run 2>&1 | tail -20
```

Expected: all 3 tests PASS.

- [ ] **Step 5: Run dev server and visually verify**

```bash
npm run tauri dev
```

Open the app, navigate to a feature's Agents tab. Verify:
- Sessions card renders with header
- "No active session" shows Start Session button when no session running
- Config rows show MCP, Skills, Context labels
- Clicking Context row opens modal with ContextEditor
- Clicking MCP row shows dropdown with server toggles (if any servers configured)
- Sparkline chart renders in the insights footer

- [ ] **Step 6: Commit**

```bash
git add src/lib/modules/ai/AiPanel.svelte src/lib/modules/ai/AiPanel.test.ts
git commit -m "feat(agents): replace bento grid with unified sessions card and inline config rows"
```

---

## Task 4: Remove obsolete bento CSS from app.css

**Files:**
- Modify: `src/app.css`

The bento block covers approximately lines 1849–2560. All `.bento-*` classes and the `.ai-block*` classes are now unused.

- [ ] **Step 1: Remove bento CSS block**

In `src/app.css`, find and delete everything from:
```css
.bento {
```
through the end of the last `.bento-*` or `.ai-block*` rule block. The classes to remove include:

`.bento`, `.bento-card`, `.bento-card::after`, `.bento-card:hover`, `.bento-card--span2`, `.bento-card--full`, `.bento-card--tasks`, `.bento-card--tasks:hover`, `.bento-header`, `.bento-title`, `.bento-badge`, `.bento-empty`, `.bento-empty-icon`, `.bento-empty-text`, `.bento-progress-track`, `.bento-progress-fill`, `.bento-task-list`, `.bento-task-item`, `.bento-task-dot`, `.bento-task-dot--done`, `.bento-task-text`, `.bento-task-item--done .bento-task-text`, `.bento-task-more`, `.bento-card--live`, `.bento-card--live:hover`, `.bento-card-content`, `.bento-card-glow`, `.bento-card-dot-field`, `.bento-live-pill`, `.bento-live-ring`, `.bento-session-title`, `.bento-session-meta`, `.bento-session-open`, `.bento-session-open:hover`, `.bento-session-summary`, `.bento-session-footer`, `.bento-session-timer`, `.bento-start-btn`, `.bento-start-btn:hover:not(:disabled)`, `.bento-start-btn:disabled`, `.bento-card--warn`, `.bento-card--warn .bento-title`, `.bento-warn-badge`, `.bento-plan-list`, `.bento-plan-item`, `.bento-plan-item:hover`, `.bento-plan-dot`, `.bento-plan-title`, `.bento-plan-arrow`, `.bento-session-list`, `.bento-history-row`, `.bento-history-row:last-child`, `.bento-history-row:hover .bento-history-copy`, `.bento-history-item`, `.bento-history-item:hover`, `.bento-history-copy`, `.bento-history-copy:hover`, `.bento-icon-btn`, `.bento-icon-btn:hover`, `.bento-icon-btn:disabled`, `.bento-history-dot`, `.bento-history-dot--live`, `.bento-history-title`, `.bento-history-meta`, `.bento-history-when`, `.bento-history-dur`, `.bento-link-list`, `.bento-link-item`, `.bento-link-item:hover`, `.bento-link-item:hover .bento-link-arrow`, `.bento-link-type`, `.bento-link-name`, `.bento-link-arrow`, `.bento-insights-body`, `.bento-stats`, `.bento-stat`, `.bento-stat-value`, `.bento-stat-label`, `.bento-insights-divider`, `.bento-token-breakdown` and any remaining `.bento-token-*` / `.ai-block*` rules.

Search for `ai-block` in app.css and remove those rules too:
```css
.ai-block { ... }
.ai-block-header { ... }
.ai-block-title { ... }
.ai-fold-chevron { ... }
.ai-fold-hint { ... }
```

- [ ] **Step 2: Run dev server smoke check**

```bash
npm run tauri dev
```

Open the Agents tab. Confirm it still renders correctly with no visual regressions (missing styles).

- [ ] **Step 3: Run tests**

```bash
cd D:/LittleBrushGames/FeatureHub && npm run test -- --run 2>&1 | tail -10
```

Expected: all existing tests still pass.

- [ ] **Step 4: Commit**

```bash
git add src/app.css
git commit -m "style: remove obsolete bento-* and ai-block CSS (replaced by sc-* classes)"
```

---

## Task 5: Close dropdowns on outside click

**Files:**
- Modify: `src/lib/modules/ai/AiPanel.svelte`

The MCP and Skills dropdowns currently stay open until the row is clicked again. They should close when clicking outside.

- [ ] **Step 1: Add click-outside handler**

In the AiPanel `<script>` block, add:

```typescript
function handlePanelClick(e: MouseEvent) {
  // Close dropdowns when clicking outside them
  const target = e.target as HTMLElement;
  if (!target.closest('.sc-config-row')) {
    mcpDropdownOpen = false;
    skillsDropdownOpen = false;
  }
}
```

- [ ] **Step 2: Wire it to the panel wrapper**

In the template, change:
```svelte
    <div class="sc-panel">
```
to:
```svelte
    <div class="sc-panel" onclick={handlePanelClick}>
```

- [ ] **Step 3: Verify manually**

Run `npm run tauri dev`. Open MCP dropdown. Click somewhere outside it. Confirm it closes.

- [ ] **Step 4: Commit**

```bash
git add src/lib/modules/ai/AiPanel.svelte
git commit -m "fix(agents): close config dropdowns on outside click"
```

---

## Self-Review

**Spec coverage check:**

| Spec requirement | Covered by |
|-----------------|-----------|
| Remove Links block | Task 3 — not rendered |
| Remove Plans block | Task 3 — not rendered |
| Remove Tasks block | Task 3 — not rendered |
| Sessions card with header | Task 3 |
| Active slot: no session → start CTA | Task 3 |
| Active slot: 1 session → expanded | Task 3 |
| Active slot: 2+ sessions → compact rows | Task 3 |
| Past sessions list | Task 3 |
| Insights footer (sparkline + range) | Task 3 |
| MCP inline row with pills + dropdown | Task 3 |
| Skills inline row with pills + dropdown | Task 3 |
| Context row → modal | Task 3 |
| Remove bento CSS | Task 4 |
| Close dropdowns on outside click | Task 5 |

**Placeholder scan:** No TBDs, no "implement later", all code blocks complete.

**Type consistency:**
- `enabledMcpServers` used in Task 2 and Task 3 ✓
- `enabledSkills` used in Task 2 and Task 3 ✓
- `toggleMcpServer(srv: McpServer)` defined in Task 2, called in Task 3 ✓
- `toggleSkill(sk: Skill)` defined in Task 2, called in Task 3 ✓
- `pastSessions` defined in Task 2, used in Task 3 ✓
- `mcpDropdownOpen`, `skillsDropdownOpen`, `contextModalOpen` defined in Task 2, used in Tasks 3 + 5 ✓
- `loadConfigData()` defined in Task 2, called in `$effect` in Task 2 ✓
