<script lang="ts">
  import type { Feature } from "../../api/types";
  import BoardCard from "./BoardCard.svelte";

  let {
    status,
    label,
    color,
    features,
    dragOver = false,
    onOpen,
    onArchive,
    onDragStart,
  }: {
    status: string;
    label: string;
    color: string;
    features: Feature[];
    dragOver?: boolean;
    onOpen: (featureId: string) => void;
    onArchive?: (featureId: string) => void;
    onDragStart: (e: MouseEvent, featureId: string) => void;
  } = $props();

  const STALE_DAYS = 3;
  const MS_PER_DAY = 86400000;

  let collapsed = $state(false);

  // Load collapse state from localStorage once
  const storageKey = $derived(`featurehub:board:collapse:${status}`);

  $effect.pre(() => {
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
</script>

<div
  class="board-column"
  class:board-column--drag-over={dragOver}
  data-column-status={status}
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
        />
      {/each}
    </div>
  {/if}
</div>
