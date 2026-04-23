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
  const DRAG_THRESHOLD = 5;

  let features = $state<Feature[]>([]);
  let allTags = $state<Tag[]>([]);
  let allGroups = $state<FeatureGroup[]>([]);

  // Filters
  let selectedTagIds = $state<Set<string>>(new Set());
  let selectedGroupIds = $state<Set<string>>(new Set());
  let showTagFilter = $state(false);
  let showGroupFilter = $state(false);

  // Mouse-based drag state
  let draggingId = $state<string | null>(null);
  let dragGhost = $state<{ x: number; y: number; title: string } | null>(null);
  let dragStartPos: { x: number; y: number } | null = null;
  let pendingDragFeature: { id: string; title: string } | null = null;
  let dropTargetColumn = $state<string | null>(null);
  let justDropped = false;

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

  // Partition into columns
  function columnFeatures(status: string): Feature[] {
    let col: Feature[];
    if (status === "todo") {
      col = visibleFeatures.filter(f => !BOARD_STATUSES.has(f.status) || f.status === "todo");
    } else {
      col = visibleFeatures.filter(f => f.status === status);
    }
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
  let inFlightCount = $derived(visibleFeatures.filter(f => ['active', 'in_progress', 'in_review'].includes(f.status)).length);
  let doneCount = $derived(visibleFeatures.filter(f => f.status === 'done').length);
  let blockedCount = $derived(visibleFeatures.filter(f => f.status === 'blocked').length);
  let totalTasks = $derived(visibleFeatures.reduce((s, f) => s + (f.task_count_total ?? 0), 0));
  let doneTasks = $derived(visibleFeatures.reduce((s, f) => s + (f.task_count_done ?? 0), 0));
  let taskPct = $derived(totalTasks > 0 ? Math.round(doneTasks / totalTasks * 100) : 0);

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

  // ── Mouse-based drag handlers ──────────────────────────────────────

  function handleDragStart(e: MouseEvent, featureId: string) {
    const feature = features.find(f => f.id === featureId);
    if (!feature) return;
    dragStartPos = { x: e.clientX, y: e.clientY };
    pendingDragFeature = { id: feature.id, title: feature.title };
    window.addEventListener("mousemove", handleGlobalMouseMove);
    window.addEventListener("mouseup", handleGlobalMouseUp);
  }

  function handleGlobalMouseMove(e: MouseEvent) {
    if (pendingDragFeature && dragStartPos && !draggingId) {
      const dx = e.clientX - dragStartPos.x;
      const dy = e.clientY - dragStartPos.y;
      if (Math.abs(dx) > DRAG_THRESHOLD || Math.abs(dy) > DRAG_THRESHOLD) {
        draggingId = pendingDragFeature.id;
        dragGhost = { x: e.clientX, y: e.clientY, title: pendingDragFeature.title };
      }
    }
    if (draggingId && dragGhost) {
      dragGhost = { ...dragGhost, x: e.clientX, y: e.clientY };

      // Hit-test which column we're over
      const el = document.elementFromPoint(e.clientX, e.clientY);
      const columnEl = el?.closest("[data-column-status]") as HTMLElement | null;
      if (columnEl) {
        dropTargetColumn = columnEl.dataset.columnStatus!;
      } else {
        dropTargetColumn = null;
      }
    }
  }

  async function handleGlobalMouseUp() {
    window.removeEventListener("mousemove", handleGlobalMouseMove);
    window.removeEventListener("mouseup", handleGlobalMouseUp);

    if (draggingId && dropTargetColumn) {
      justDropped = true;
      requestAnimationFrame(() => { justDropped = false; });

      const feature = features.find(f => f.id === draggingId);
      if (feature) {
        const newStatus = dropTargetColumn;
        // Determine effective current status (non-board statuses map to "todo")
        const effectiveStatus = BOARD_STATUSES.has(feature.status) ? feature.status : "todo";
        if (effectiveStatus !== newStatus) {
          await updateFeature(feature.id, { status: newStatus });
          await loadData();
        }
      }
    }

    draggingId = null;
    dragGhost = null;
    dragStartPos = null;
    pendingDragFeature = null;
    dropTargetColumn = null;
  }

  function handleOpen(featureId: string) {
    if (justDropped) return;
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

  // Close dropdowns on outside click
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
</script>

<div class="board-panel">
  <div class="board-header">
    <div class="board-header-left">
      <span class="board-header-title">Dashboard</span>
    </div>
    <div class="board-header-right">
      <div class="board-filter-wrapper">
        <button class="board-filter-btn" class:board-filter-btn--active={selectedTagIds.size > 0} onclick={() => showTagFilter = !showTagFilter}>
          Filter by tag {#if selectedTagIds.size > 0}({selectedTagIds.size}){/if} ▾
        </button>
        {#if showTagFilter}
          <div class="board-filter-dropdown" onclick={(e) => e.stopPropagation()} onkeydown={(e) => { if (e.key === 'Escape') showTagFilter = false; }} role="listbox" tabindex="-1">
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
          <div class="board-filter-dropdown" onclick={(e) => e.stopPropagation()} onkeydown={(e) => { if (e.key === 'Escape') showGroupFilter = false; }} role="listbox" tabindex="-1">
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

  <div class="board-stats">
    <div class="board-stat-chip">
      <span class="board-stat-num">{totalCount}</span>
      <span class="board-stat-lbl">features</span>
    </div>
    <div class="board-stat-sep"></div>
    <div class="board-stat-chip board-stat-chip--blue">
      <span class="board-stat-num">{inFlightCount}</span>
      <span class="board-stat-lbl">in flight</span>
    </div>
    <div class="board-stat-chip board-stat-chip--green">
      <span class="board-stat-num">{doneCount}</span>
      <span class="board-stat-lbl">done</span>
    </div>
    {#if blockedCount > 0}
      <div class="board-stat-chip board-stat-chip--red">
        <span class="board-stat-num">{blockedCount}</span>
        <span class="board-stat-lbl">blocked</span>
      </div>
    {/if}
    {#if totalTasks > 0}
      <div class="board-stat-sep"></div>
      <div class="board-stat-chip board-stat-chip--tasks">
        <div class="board-stat-bar">
          <div class="board-stat-bar-fill" style="width: {taskPct}%"></div>
        </div>
        <span class="board-stat-num">{doneTasks}<span class="board-stat-denom">/{totalTasks}</span></span>
        <span class="board-stat-lbl">tasks</span>
      </div>
    {/if}
    {#if totalActiveSessions > 0}
      <div class="board-stat-sep"></div>
      <div class="board-stat-chip board-stat-chip--live">
        <span class="board-stat-live-dot"></span>
        <span class="board-stat-num">{totalActiveSessions}</span>
        <span class="board-stat-lbl">live</span>
      </div>
    {/if}
  </div>

  <div class="board-columns">
    {#each COLUMNS as col}
      <BoardColumn
        status={col.status}
        label={col.label}
        color={col.color}
        features={columnFeatures(col.status)}
        dragOver={dropTargetColumn === col.status && draggingId !== null}
        draggingId={draggingId}
        onOpen={handleOpen}
        onArchive={handleArchive}
        onDragStart={handleDragStart}
      />
    {/each}
  </div>
</div>

{#if dragGhost}
  <div class="board-drag-ghost" style="left: {dragGhost.x}px; top: {dragGhost.y}px;">
    {dragGhost.title}
  </div>
{/if}
