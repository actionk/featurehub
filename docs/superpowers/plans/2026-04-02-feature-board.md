# Feature Board Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Kanban-style feature board as a special workspace tab, showing all non-archived features in four status columns with drag-and-drop, session actions, and stale-card auto-fade.

**Architecture:** The board is a set of new Svelte components (`BoardPanel`, `BoardColumn`, `BoardCard`) living in `src/lib/modules/board/`. The workspace tab system gets a minor extension to support a non-feature "board" tab type. No backend changes — all data comes from existing Tauri commands and stores.

**Tech Stack:** Svelte 5 (runes), TypeScript, HTML5 Drag and Drop API, existing Tauri IPC commands.

**Spec:** `docs/superpowers/specs/2026-04-02-feature-board-design.md`

---

### Task 1: Extend Workspace Tab System for Non-Feature Tabs

**Files:**
- Modify: `src/lib/stores/workspaceTabs.svelte.ts`

The current `WorkspaceTab` interface only has `{ id, featureId }`. We need to support a board tab that has no feature. We'll use a sentinel `featureId` value `"__board__"` and add helper functions. This avoids changing the interface shape (which would break persistence and all consumers).

- [ ] **Step 1: Add board tab constants and helpers**

Add at the top of `src/lib/stores/workspaceTabs.svelte.ts`, after the existing imports/interfaces:

```typescript
export const BOARD_TAB_FEATURE_ID = "__board__";

export function isBoardTab(tab: WorkspaceTab): boolean {
  return tab.featureId === BOARD_TAB_FEATURE_ID;
}
```

- [ ] **Step 2: Add `openBoardTab` function**

Add after the `openTab` function (after line 90):

```typescript
/** Open the board tab, or switch to it if already open. Returns the tab ID. */
export function openBoardTab(): string {
  const existing = tabs.find(t => t.featureId === BOARD_TAB_FEATURE_ID);
  if (existing) {
    activeTabId = existing.id;
    persist();
    return existing.id;
  }
  const tab: WorkspaceTab = { id: generateId(), featureId: BOARD_TAB_FEATURE_ID };
  tabs = [...tabs, tab];
  activeTabId = tab.id;
  persist();
  return tab.id;
}
```

- [ ] **Step 3: Update `pruneInvalidTabs` to preserve board tabs**

The existing `pruneInvalidTabs` filters tabs by valid feature IDs. Board tabs would get pruned since `"__board__"` isn't a real feature ID. Update the filter at line 210:

Change:
```typescript
tabs = tabs.filter(t => validFeatureIds.has(t.featureId));
```

To:
```typescript
tabs = tabs.filter(t => t.featureId === BOARD_TAB_FEATURE_ID || validFeatureIds.has(t.featureId));
```

- [ ] **Step 4: Update `getOpenFeatureIds` to exclude board tab**

Change line 61:
```typescript
return tabs.map(t => t.featureId);
```

To:
```typescript
return tabs.filter(t => t.featureId !== BOARD_TAB_FEATURE_ID).map(t => t.featureId);
```

- [ ] **Step 5: Commit**

```bash
git add src/lib/stores/workspaceTabs.svelte.ts
git commit -m "feat: extend workspace tabs to support board tab type"
```

---

### Task 2: Create BoardCard Component

**Files:**
- Create: `src/lib/modules/board/BoardCard.svelte`

The card component renders a single feature in a column. It handles click-to-open, drag start, session actions, and stale-card rendering.

- [ ] **Step 1: Create the BoardCard component**

Create `src/lib/modules/board/BoardCard.svelte`:

```svelte
<script lang="ts">
  import type { Feature } from "../../api/types";
  import { getActiveCountForFeature } from "../../stores/sessionActivity.svelte";
  import { startNewSession } from "../../api/sessions";

  let {
    feature,
    columnStatus,
    stale = false,
    staleDays = 0,
    onOpen,
    onArchive,
    onDragStart,
    onDragEnd,
  }: {
    feature: Feature;
    columnStatus: string;
    stale?: boolean;
    staleDays?: number;
    onOpen: (featureId: string) => void;
    onArchive?: (featureId: string) => void;
    onDragStart: (e: DragEvent, featureId: string) => void;
    onDragEnd: (e: DragEvent) => void;
  } = $props();

  const statusColors: Record<string, string> = {
    todo: "var(--text-muted)",
    in_progress: "var(--amber)",
    in_review: "var(--blue)",
    done: "var(--green)",
  };

  let borderColor = $derived(statusColors[columnStatus] ?? "var(--text-muted)");
  let activeCount = $derived(getActiveCountForFeature(feature.id));
  let tags = $derived(feature.tags ?? []);
  let taskTotal = $derived(feature.task_count_total ?? 0);
  let taskDone = $derived(feature.task_count_done ?? 0);
  let taskProgress = $derived(taskTotal > 0 ? (taskDone / taskTotal) * 100 : 0);

  function handleClick(e: MouseEvent) {
    onOpen(feature.id);
  }

  function handleSessionAction(e: MouseEvent) {
    e.stopPropagation();
    startNewSession(feature.id);
  }

  function handleArchive(e: MouseEvent) {
    e.stopPropagation();
    onArchive?.(feature.id);
  }

  function handleDragStart(e: DragEvent) {
    onDragStart(e, feature.id);
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="board-card"
  class:board-card--stale={stale}
  style="border-left-color: {borderColor};"
  draggable="true"
  onclick={handleClick}
  ondragstart={handleDragStart}
  ondragend={onDragEnd}
>
  <div class="board-card-header">
    <span class="board-card-title">
      {#if feature.pinned}📌 {/if}{feature.title}
    </span>
    {#if activeCount > 0}
      <span class="board-card-sessions">● {activeCount} {activeCount === 1 ? 'session' : 'sessions'}</span>
    {/if}
  </div>

  {#if tags.length > 0}
    <div class="board-card-tags">
      {#each tags as tag}
        <span class="board-card-tag" style="background: {tag.color}18; color: {tag.color};">{tag.name}</span>
      {/each}
    </div>
  {/if}

  <div class="board-card-footer">
    {#if taskTotal > 0}
      <div class="board-card-progress">
        <div class="board-card-progress-bar">
          <div class="board-card-progress-fill" style="width: {taskProgress}%;"></div>
        </div>
        <span class="board-card-progress-text">{taskDone}/{taskTotal}</span>
      </div>
    {:else if stale}
      <span class="board-card-stale-text">done {staleDays}d ago</span>
    {:else}
      <div></div>
    {/if}

    <div class="board-card-actions">
      {#if stale && onArchive}
        <button class="board-card-btn board-card-btn--archive" onclick={handleArchive}>Archive</button>
      {/if}
      {#if !stale}
        {#if activeCount > 0}
          <button class="board-card-btn board-card-btn--resume" onclick={handleSessionAction}>▶ Resume</button>
        {:else}
          <button class="board-card-btn board-card-btn--start" onclick={handleSessionAction}>▶ Start</button>
        {/if}
      {/if}
    </div>
  </div>

  {#if stale && taskTotal > 0}
    <span class="board-card-stale-text" style="margin-top: 2px;">done {staleDays}d ago</span>
  {/if}
</div>
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/modules/board/BoardCard.svelte
git commit -m "feat: add BoardCard component for feature board"
```

---

### Task 3: Create BoardColumn Component

**Files:**
- Create: `src/lib/modules/board/BoardColumn.svelte`

Each column renders a header (dot + label + count + collapse chevron) and a scrollable list of `BoardCard` components. Handles drag-over and drop events.

- [ ] **Step 1: Create the BoardColumn component**

Create `src/lib/modules/board/BoardColumn.svelte`:

```svelte
<script lang="ts">
  import type { Feature } from "../../api/types";
  import BoardCard from "./BoardCard.svelte";

  let {
    status,
    label,
    color,
    features,
    onOpen,
    onArchive,
    onDragStart,
    onDragEnd,
    onDrop,
  }: {
    status: string;
    label: string;
    color: string;
    features: Feature[];
    onOpen: (featureId: string) => void;
    onArchive?: (featureId: string) => void;
    onDragStart: (e: DragEvent, featureId: string) => void;
    onDragEnd: (e: DragEvent) => void;
    onDrop: (e: DragEvent, targetStatus: string) => void;
  } = $props();

  const STALE_DAYS = 3;
  const MS_PER_DAY = 86400000;

  let collapsed = $state(false);
  let dragOver = $state(false);

  // Persist collapse state in localStorage
  const storageKey = `featurehub:board:collapse:${status}`;
  $effect(() => {
    const saved = localStorage.getItem(storageKey);
    if (saved !== null) collapsed = saved === "true";
  });

  function toggleCollapse() {
    collapsed = !collapsed;
    localStorage.setItem(storageKey, String(collapsed));
  }

  function isStale(feature: Feature): boolean {
    if (status !== "done") return false;
    const updatedAt = new Date(feature.updated_at).getTime();
    const daysSince = (Date.now() - updatedAt) / MS_PER_DAY;
    return daysSince >= STALE_DAYS;
  }

  function staleDays(feature: Feature): number {
    const updatedAt = new Date(feature.updated_at).getTime();
    return Math.floor((Date.now() - updatedAt) / MS_PER_DAY);
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
    dragOver = true;
  }

  function handleDragLeave(e: DragEvent) {
    // Only clear if leaving the column (not entering a child)
    const related = e.relatedTarget as HTMLElement | null;
    if (related && (e.currentTarget as HTMLElement).contains(related)) return;
    dragOver = false;
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    dragOver = false;
    onDrop(e, status);
  }
</script>

<div
  class="board-column"
  class:board-column--drag-over={dragOver}
  ondragover={handleDragOver}
  ondragleave={handleDragLeave}
  ondrop={handleDrop}
  role="list"
>
  <div class="board-column-header">
    <div class="board-column-header-left">
      <span class="board-column-dot" style="background: {color};"></span>
      <span class="board-column-label" style="color: {color};">{label}</span>
      <span class="board-column-count">{features.length}</span>
    </div>
    <button class="board-column-collapse" onclick={toggleCollapse}>
      {collapsed ? '›' : '⌄'}
    </button>
  </div>

  {#if !collapsed}
    <div class="board-column-cards">
      {#each features as feature (feature.id)}
        <BoardCard
          {feature}
          columnStatus={status}
          stale={isStale(feature)}
          staleDays={staleDays(feature)}
          {onOpen}
          onArchive={status === "done" ? onArchive : undefined}
          {onDragStart}
          {onDragEnd}
        />
      {/each}
    </div>
  {/if}
</div>
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/modules/board/BoardColumn.svelte
git commit -m "feat: add BoardColumn component with drag-and-drop zones"
```

---

### Task 4: Create BoardPanel Component

**Files:**
- Create: `src/lib/modules/board/BoardPanel.svelte`

The main board component. Fetches features, partitions into columns, handles drag-and-drop state, filters, and stats.

- [ ] **Step 1: Create the BoardPanel component**

Create `src/lib/modules/board/BoardPanel.svelte`:

```svelte
<script lang="ts">
  import type { Feature, Tag, FeatureGroup } from "../../api/types";
  import { getFeatures, updateFeature, setFeatureArchived, getFeatureGroups } from "../../api/features";
  import { getTags } from "../../api/tags";
  import { getAllActiveSessionCounts } from "../../stores/sessionActivity.svelte";
  import { subscribe } from "../../stores/events.svelte";
  import { switchToFeature } from "../../stores/workspaceTabs.svelte";
  import BoardColumn from "./BoardColumn.svelte";

  const COLUMNS = [
    { status: "todo", label: "Todo", color: "var(--text-muted)" },
    { status: "in_progress", label: "In Progress", color: "var(--amber)" },
    { status: "in_review", label: "In Review", color: "var(--blue)" },
    { status: "done", label: "Done", color: "var(--green)" },
  ] as const;

  const BOARD_STATUSES = new Set(["todo", "in_progress", "in_review", "done"]);

  let features = $state<Feature[]>([]);
  let allTags = $state<Tag[]>([]);
  let allGroups = $state<FeatureGroup[]>([]);

  // Filters
  let selectedTagIds = $state<Set<string>>(new Set());
  let selectedGroupIds = $state<Set<string>>(new Set());
  let showTagFilter = $state(false);
  let showGroupFilter = $state(false);

  // Drag state
  let draggedFeatureId = $state<string | null>(null);

  // Filtered features (non-archived)
  let visibleFeatures = $derived.by(() => {
    let result = features.filter(f => !f.archived);
    if (selectedTagIds.size > 0) {
      result = result.filter(f =>
        (f.tags ?? []).some(t => selectedTagIds.has(t.id))
      );
    }
    if (selectedGroupIds.size > 0) {
      result = result.filter(f =>
        f.group_id !== null && selectedGroupIds.has(f.group_id)
      );
    }
    return result;
  });

  // Partition into columns. Non-board statuses (active, blocked, paused) go to todo.
  function columnFeatures(status: string): Feature[] {
    let col: Feature[];
    if (status === "todo") {
      col = visibleFeatures.filter(f => !BOARD_STATUSES.has(f.status) || f.status === "todo");
    } else {
      col = visibleFeatures.filter(f => f.status === status);
    }
    // Sort: pinned first, then sort_order
    return col.sort((a, b) => {
      if (a.pinned !== b.pinned) return a.pinned ? -1 : 1;
      return a.sort_order - b.sort_order;
    });
  }

  // Stats
  let totalCount = $derived(visibleFeatures.length);
  let activeSessionCounts = $derived(getAllActiveSessionCounts());
  let totalActiveSessions = $derived.by(() => {
    const featureIds = new Set(visibleFeatures.map(f => f.id));
    return Object.entries(activeSessionCounts)
      .filter(([id]) => featureIds.has(id))
      .reduce((sum, [, count]) => sum + count, 0);
  });

  // Load data
  async function loadData() {
    const [f, t, g] = await Promise.all([getFeatures(), getTags(), getFeatureGroups()]);
    features = f;
    allTags = t;
    allGroups = g;
  }

  // Initial load + event subscriptions
  $effect(() => {
    loadData();
    const unsub1 = subscribe("features:changed", () => loadData());
    const unsub2 = subscribe("feature:updated", () => loadData());
    return () => { unsub1(); unsub2(); };
  });

  // Drag handlers
  function handleDragStart(e: DragEvent, featureId: string) {
    draggedFeatureId = featureId;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = "move";
      e.dataTransfer.setData("text/plain", featureId);
    }
  }

  function handleDragEnd(_e: DragEvent) {
    draggedFeatureId = null;
  }

  async function handleDrop(_e: DragEvent, targetStatus: string) {
    if (!draggedFeatureId) return;
    const feature = features.find(f => f.id === draggedFeatureId);
    if (!feature) return;

    // Map: if dragging to "todo" column, set actual status to "todo"
    const newStatus = targetStatus;
    if (feature.status !== newStatus) {
      await updateFeature(feature.id, { status: newStatus });
      await loadData();
    }
    draggedFeatureId = null;
  }

  function handleOpen(featureId: string) {
    switchToFeature(featureId);
  }

  async function handleArchive(featureId: string) {
    await setFeatureArchived(featureId, true);
    await loadData();
  }

  // Filter toggles
  function toggleTagFilter(tagId: string) {
    const next = new Set(selectedTagIds);
    if (next.has(tagId)) next.delete(tagId);
    else next.add(tagId);
    selectedTagIds = next;
  }

  function toggleGroupFilter(groupId: string) {
    const next = new Set(selectedGroupIds);
    if (next.has(groupId)) next.delete(groupId);
    else next.add(groupId);
    selectedGroupIds = next;
  }
</script>

<div class="board-panel">
  <div class="board-header">
    <div class="board-header-left">
      <span class="board-header-title">Feature Board</span>
      <span class="board-header-stat">{totalCount} <span class="board-header-stat-label">features</span></span>
      {#if totalActiveSessions > 0}
        <span class="board-header-stat board-header-stat--active">{totalActiveSessions} <span class="board-header-stat-label">active sessions</span></span>
      {/if}
    </div>
    <div class="board-header-right">
      <div class="board-filter-wrapper">
        <button class="board-filter-btn" class:board-filter-btn--active={selectedTagIds.size > 0} onclick={() => showTagFilter = !showTagFilter}>
          Filter by tag {#if selectedTagIds.size > 0}({selectedTagIds.size}){/if} ▾
        </button>
        {#if showTagFilter}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="board-filter-dropdown" onclick={(e) => e.stopPropagation()}>
            {#each allTags as tag}
              <label class="board-filter-option">
                <input type="checkbox" checked={selectedTagIds.has(tag.id)} onchange={() => toggleTagFilter(tag.id)} />
                <span class="board-card-tag" style="background: {tag.color}18; color: {tag.color};">{tag.name}</span>
              </label>
            {/each}
            {#if allTags.length === 0}
              <span class="board-filter-empty">No tags</span>
            {/if}
          </div>
        {/if}
      </div>
      <div class="board-filter-wrapper">
        <button class="board-filter-btn" class:board-filter-btn--active={selectedGroupIds.size > 0} onclick={() => showGroupFilter = !showGroupFilter}>
          Filter by group {#if selectedGroupIds.size > 0}({selectedGroupIds.size}){/if} ▾
        </button>
        {#if showGroupFilter}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="board-filter-dropdown" onclick={(e) => e.stopPropagation()}>
            {#each allGroups as group}
              <label class="board-filter-option">
                <input type="checkbox" checked={selectedGroupIds.has(group.id)} onchange={() => toggleGroupFilter(group.id)} />
                <span>{group.name}</span>
              </label>
            {/each}
            {#if allGroups.length === 0}
              <span class="board-filter-empty">No groups</span>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  </div>

  <div class="board-columns">
    {#each COLUMNS as col}
      <BoardColumn
        status={col.status}
        label={col.label}
        color={col.color}
        features={columnFeatures(col.status)}
        onOpen={handleOpen}
        onArchive={handleArchive}
        onDragStart={handleDragStart}
        onDragEnd={handleDragEnd}
        onDrop={handleDrop}
      />
    {/each}
  </div>
</div>
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/modules/board/BoardPanel.svelte
git commit -m "feat: add BoardPanel with columns, filters, and drag-and-drop"
```

---

### Task 5: Add Board CSS Styles

**Files:**
- Modify: `src/app.css`

Add all board-specific styles using the existing CSS custom properties and dark theme palette.

- [ ] **Step 1: Add board styles to app.css**

Append to the end of `src/app.css`:

```css
/* ── Feature Board ─────────────────────────────────────────────────── */

.board-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.board-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  border-bottom: 1px solid var(--bg-card);
  flex-shrink: 0;
}

.board-header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.board-header-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
}

.board-header-stat {
  font-size: 12px;
  color: var(--text-secondary);
}

.board-header-stat--active {
  color: var(--green);
}

.board-header-stat-label {
  color: var(--text-muted);
}

.board-header-right {
  display: flex;
  gap: 8px;
}

.board-filter-wrapper {
  position: relative;
}

.board-filter-btn {
  background: var(--bg-card);
  color: var(--text-secondary);
  padding: 4px 10px;
  border-radius: var(--radius-sm);
  font-size: 11px;
  border: 1px solid var(--bg-hover);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.board-filter-btn:hover {
  background: var(--bg-hover);
}

.board-filter-btn--active {
  border-color: var(--blue);
  color: var(--blue);
}

.board-filter-dropdown {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 4px;
  background: var(--bg-raised);
  border: 1px solid var(--bg-hover);
  border-radius: var(--radius-md);
  padding: 6px;
  min-width: 160px;
  max-height: 240px;
  overflow-y: auto;
  z-index: 50;
  box-shadow: var(--shadow-lg);
}

.board-filter-option {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 6px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-size: 12px;
  color: var(--text-secondary);
}

.board-filter-option:hover {
  background: var(--bg-hover);
}

.board-filter-empty {
  font-size: 11px;
  color: var(--text-muted);
  padding: 4px 6px;
}

/* Columns */

.board-columns {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr 1fr;
  flex: 1;
  overflow: hidden;
}

.board-column {
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--bg-card);
  overflow: hidden;
}

.board-column:last-child {
  border-right: none;
}

.board-column--drag-over {
  background: rgba(82, 169, 255, 0.04);
}

.board-column-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  flex-shrink: 0;
}

.board-column-header-left {
  display: flex;
  align-items: center;
  gap: 6px;
}

.board-column-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.board-column-label {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 1px;
  font-weight: 600;
}

.board-column-count {
  background: var(--bg-hover);
  color: var(--text-secondary);
  padding: 1px 6px;
  border-radius: 8px;
  font-size: 10px;
}

.board-column-collapse {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 12px;
  padding: 2px 4px;
}

.board-column-collapse:hover {
  color: var(--text-secondary);
}

.board-column-cards {
  flex: 1;
  overflow-y: auto;
  padding: 0 8px 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

/* Cards */

.board-card {
  background: var(--bg-card);
  border-radius: var(--radius-md);
  padding: 10px 12px;
  border-left: 3px solid var(--text-muted);
  cursor: grab;
  transition: opacity var(--transition-base), background var(--transition-fast);
}

.board-card:hover {
  background: var(--bg-raised);
}

.board-card:active {
  cursor: grabbing;
}

.board-card--stale {
  opacity: 0.45;
}

.board-card--stale:hover {
  opacity: 0.7;
}

.board-card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 8px;
}

.board-card-title {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
  line-height: 1.3;
}

.board-card--stale .board-card-title {
  color: var(--text-secondary);
}

.board-card-sessions {
  background: rgba(61, 214, 140, 0.15);
  color: var(--green);
  padding: 2px 7px;
  border-radius: 3px;
  font-size: 9px;
  white-space: nowrap;
  flex-shrink: 0;
}

.board-card-tags {
  display: flex;
  gap: 4px;
  margin-top: 5px;
  flex-wrap: wrap;
}

.board-card-tag {
  padding: 1px 6px;
  border-radius: 3px;
  font-size: 9px;
}

.board-card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 6px;
}

.board-card-progress {
  display: flex;
  align-items: center;
  gap: 6px;
}

.board-card-progress-bar {
  background: var(--bg-hover);
  border-radius: 3px;
  height: 4px;
  width: 50px;
}

.board-card-progress-fill {
  background: var(--green);
  border-radius: 3px;
  height: 4px;
  transition: width var(--transition-base);
}

.board-card-progress-text {
  font-size: 9px;
  color: var(--text-secondary);
}

.board-card-stale-text {
  font-size: 9px;
  color: var(--text-muted);
}

.board-card-actions {
  display: flex;
  gap: 4px;
}

.board-card-btn {
  padding: 2px 7px;
  border-radius: 3px;
  font-size: 9px;
  border: none;
  cursor: pointer;
  transition: opacity var(--transition-fast);
}

.board-card-btn:hover {
  opacity: 0.8;
}

.board-card-btn--start {
  background: var(--bg-hover);
  color: var(--blue);
}

.board-card-btn--resume {
  background: var(--bg-hover);
  color: var(--green);
}

.board-card-btn--archive {
  background: rgba(240, 178, 50, 0.15);
  color: var(--amber);
}
```

- [ ] **Step 2: Commit**

```bash
git add src/app.css
git commit -m "feat: add CSS styles for feature board"
```

---

### Task 6: Wire Board Tab into App.svelte

**Files:**
- Modify: `src/App.svelte`

Add the board tab rendering in the workspace tab loop. When the active tab is a board tab, render `BoardPanel` instead of `FeatureDetail`.

- [ ] **Step 1: Add import for BoardPanel and board tab helpers**

At the top of `src/App.svelte`, add these imports alongside the existing ones:

```typescript
import BoardPanel from "./lib/modules/board/BoardPanel.svelte";
import { isBoardTab, BOARD_TAB_FEATURE_ID } from "./lib/stores/workspaceTabs.svelte";
```

- [ ] **Step 2: Update the tab rendering loop**

Replace the `{#each workspaceTabs as tab}` block (lines 450-468) with:

```svelte
{#each workspaceTabs as tab (tab.id)}
  {@const isActive = tab.id === activeTabId}
  <div class="tab-content-wrapper" style:display={isActive ? '' : 'none'}>
    {#if isBoardTab(tab)}
      <BoardPanel />
    {:else}
      <FeatureDetail
        featureId={tab.featureId}
        {isActive}
        onDeleted={handleDeleted}
        onUpdated={loadFeatures}
        onSessionsChanged={() => refreshSessionActivity()}
        {refreshFeatureId}
        onRefreshHandled={() => refreshFeatureId = null}
        {pendingPlanId}
        {pendingPlanFeatureId}
        onPendingPlanHandled={() => { pendingPlanId = null; pendingPlanFeatureId = null; }}
        initialTab={initialTabTargetId === tab.featureId ? initialTab : null}
        onInitialTabHandled={() => { initialTab = null; initialTabTargetId = null; }}
      />
    {/if}
  </div>
{/each}
```

- [ ] **Step 3: Commit**

```bash
git add src/App.svelte
git commit -m "feat: render BoardPanel for board workspace tabs"
```

---

### Task 7: Update WorkspaceTabBar to Display Board Tab

**Files:**
- Modify: `src/lib/components/WorkspaceTabBar.svelte`

The tab bar uses `getFeatureTitle(tab.featureId)` to render tab labels. Board tabs need special handling since they don't have a feature.

- [ ] **Step 1: Add board tab import and update title logic**

In `WorkspaceTabBar.svelte`, add the import:

```typescript
import { isBoardTab } from "../stores/workspaceTabs.svelte";
```

Then update the `getFeatureTitle` function (around line 32) to handle board tabs:

```typescript
function getFeatureTitle(tab: WorkspaceTab): string {
  if (isBoardTab(tab)) return "Board";
  const f = features.find(f => f.id === tab.featureId);
  return f?.title || "Untitled";
}
```

Update every call site of `getFeatureTitle` — it currently takes `tab.featureId` as argument. Change all calls to pass `tab` instead:
- The `title` attribute: `title={getFeatureTitle(tab)}`
- The label span: `{getFeatureTitle(tab)}`

Also update the status dot — for board tabs it should not show a feature status. Find where `status` is derived from the feature (around line 37-40) and add a board check:

```typescript
function getFeatureStatus(tab: WorkspaceTab): string {
  if (isBoardTab(tab)) return "";
  const f = features.find(f => f.id === tab.featureId);
  return f?.status || "";
}
```

Update the call from `getFeatureStatus(tab.featureId)` to `getFeatureStatus(tab)`.

- [ ] **Step 2: Commit**

```bash
git add src/lib/components/WorkspaceTabBar.svelte
git commit -m "feat: display board tab label in workspace tab bar"
```

---

### Task 8: Add Board Button to Sidebar

**Files:**
- Modify: `src/lib/components/Sidebar.svelte`
- Modify: `src/App.svelte` (pass callback)

Add a board icon button in the sidebar footer next to the Knowledge Base button.

- [ ] **Step 1: Add `onOpenBoard` prop to Sidebar**

In `Sidebar.svelte`, add `onOpenBoard` to the props destructuring (find the `$props()` call):

```typescript
onOpenBoard,
```

And in the type annotation:

```typescript
onOpenBoard?: () => void;
```

- [ ] **Step 2: Add the board button in the sidebar footer**

In the sidebar footer (around line 1045), after the "New Feature" button and before the Knowledge Base button, add:

```svelte
<button class="btn-ghost sidebar-footer-btn" onclick={() => onOpenBoard?.()} title="Feature Board">
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <rect x="3" y="3" width="7" height="7" rx="1"></rect>
    <rect x="14" y="3" width="7" height="7" rx="1"></rect>
    <rect x="3" y="14" width="7" height="7" rx="1"></rect>
    <rect x="14" y="14" width="7" height="7" rx="1"></rect>
  </svg>
</button>
```

- [ ] **Step 3: Wire up in App.svelte**

In `App.svelte`, import `openBoardTab`:

```typescript
import { openBoardTab } from "./lib/stores/workspaceTabs.svelte";
```

Find the `<Sidebar>` component usage and add the prop:

```svelte
onOpenBoard={() => {
  showKnowledge = false;
  openBoardTab();
}}
```

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/Sidebar.svelte src/App.svelte
git commit -m "feat: add board button to sidebar footer"
```

---

### Task 9: Close Filter Dropdowns on Outside Click

**Files:**
- Modify: `src/lib/modules/board/BoardPanel.svelte`

The filter dropdowns need to close when clicking outside. Add a click-outside handler.

- [ ] **Step 1: Add click-outside effect**

In `BoardPanel.svelte`, add an `$effect` that listens for clicks on the document and closes open dropdowns:

```typescript
$effect(() => {
  function handleClickOutside(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest('.board-filter-wrapper')) {
      showTagFilter = false;
      showGroupFilter = false;
    }
  }
  document.addEventListener('click', handleClickOutside);
  return () => document.removeEventListener('click', handleClickOutside);
});
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/modules/board/BoardPanel.svelte
git commit -m "fix: close board filter dropdowns on outside click"
```

---

### Task 10: Test the Board End-to-End

**Files:**
- No new files

Manual verification checklist to run with `npm run tauri dev`.

- [ ] **Step 1: Start dev mode**

```bash
npm run tauri dev
```

- [ ] **Step 2: Verify board tab opens**

Click the board icon (grid icon) in the sidebar footer. Verify:
- A "Board" tab appears in the workspace tab bar
- The board panel renders with header and four columns
- Feature count and active sessions stats are correct

- [ ] **Step 3: Verify feature cards display correctly**

Check that:
- Features appear in the correct status columns
- Features with `active` status appear in the Todo column
- Pinned features appear first in their column
- Tags, task progress, and session badges render correctly

- [ ] **Step 4: Verify drag and drop**

Drag a feature card from one column to another. Verify:
- The column highlights on drag-over
- The feature's status updates on drop
- The sidebar reflects the status change
- The card appears in the new column

- [ ] **Step 5: Verify stale card behavior**

If any features have been in "done" for 3+ days, verify:
- The card is faded (45% opacity)
- An "Archive" button appears
- Clicking "Archive" removes the card from the board

- [ ] **Step 6: Verify filters**

Click "Filter by tag" and select a tag. Verify only features with that tag show. Same for group filter. Verify filters combine (AND logic).

- [ ] **Step 7: Verify card click opens feature**

Click a card (not on a button). Verify the feature opens in a workspace tab alongside the board tab.

- [ ] **Step 8: Verify session buttons**

Click "Start" on a card without sessions — verify session launch. Click "Resume" on a card with sessions — verify session resume.

- [ ] **Step 9: Commit if any fixes were needed**

```bash
git add -A
git commit -m "fix: board integration fixes from manual testing"
```
