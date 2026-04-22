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
  }: {
    feature: Feature;
    columnStatus: string;
    stale?: boolean;
    staleDays?: number;
    onOpen: (featureId: string) => void;
    onArchive?: (featureId: string) => void;
    onDragStart: (e: MouseEvent, featureId: string) => void;
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

  function handleStart(e: MouseEvent) {
    e.stopPropagation();
    startNewSession(feature.id);
  }

  function handleResume(e: MouseEvent) {
    e.stopPropagation();
    onOpen(feature.id);
  }

  function handleArchive(e: MouseEvent) {
    e.stopPropagation();
    onArchive?.(feature.id);
  }

  function handleMouseDown(e: MouseEvent) {
    if (e.button !== 0) return;
    if ((e.target as HTMLElement).closest(".board-card-btn")) return;
    e.preventDefault();
    onDragStart(e, feature.id);
  }
</script>

<div
  class="board-card"
  class:board-card--stale={stale}
  style="border-left-color: {borderColor};"
  data-feature-id={feature.id}
  onclick={handleClick}
  onkeydown={(e) => { if (e.key === 'Enter') handleClick(e as unknown as MouseEvent); }}
  onmousedown={handleMouseDown}
  role="button"
  tabindex="0"
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
          <button class="board-card-btn board-card-btn--resume" onclick={handleResume}>▶ Resume</button>
        {:else}
          <button class="board-card-btn board-card-btn--start" onclick={handleStart}>▶ Start</button>
        {/if}
      {/if}
    </div>
  </div>

  {#if stale && taskTotal > 0}
    <span class="board-card-stale-text" style="margin-top: 2px;">done {staleDays}d ago</span>
  {/if}
</div>
