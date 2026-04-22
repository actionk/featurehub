<script lang="ts">
  import type { Feature } from "../api/tauri";
  import type { WorkspaceTab } from "../stores/workspaceTabs.svelte";
  import { reorderTabs, isBoardTab } from "../stores/workspaceTabs.svelte";

  let {
    tabs,
    activeTabId,
    features,
    onSwitchTab,
    onCloseTab,
    onCloseOtherTabs,
    onCloseAllTabs,
  }: {
    tabs: WorkspaceTab[];
    activeTabId: string | null;
    features: Feature[];
    onSwitchTab: (tabId: string) => void;
    onCloseTab: (tabId: string) => void;
    onCloseOtherTabs: (tabId: string) => void;
    onCloseAllTabs: () => void;
  } = $props();

  // Context menu state
  let contextMenu = $state<{ x: number; y: number; tabId: string } | null>(null);

  // Drag state
  let dragTabId = $state<string | null>(null);
  let dragOverTabId = $state<string | null>(null);
  let dragSide = $state<"left" | "right" | null>(null);

  function getFeatureTitle(tab: WorkspaceTab): string {
    if (isBoardTab(tab)) return "Board";
    const f = features.find(f => f.id === tab.featureId);
    return f?.title || "Untitled";
  }

  function getFeatureStatus(tab: WorkspaceTab): string {
    if (isBoardTab(tab)) return "";
    const f = features.find(f => f.id === tab.featureId);
    return f?.status || "active";
  }

  function handleMouseDown(e: MouseEvent, tabId: string) {
    if (e.button === 1) {
      e.preventDefault();
      onCloseTab(tabId);
    }
  }

  function handleContextMenu(e: MouseEvent, tabId: string) {
    e.preventDefault();
    contextMenu = { x: e.clientX, y: e.clientY, tabId };
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  $effect(() => {
    if (!contextMenu) return;
    const handler = () => closeContextMenu();
    window.addEventListener("click", handler, { once: true });
    return () => window.removeEventListener("click", handler);
  });

  function handleDragStart(e: DragEvent, tabId: string) {
    if (!e.dataTransfer) return;
    closeContextMenu();
    dragTabId = tabId;
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData("text/plain", tabId);
  }

  function handleDragOver(e: DragEvent, tabId: string) {
    if (!dragTabId || dragTabId === tabId) {
      dragOverTabId = null;
      dragSide = null;
      return;
    }
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
    dragOverTabId = tabId;
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    dragSide = e.clientX < rect.left + rect.width / 2 ? "left" : "right";
  }

  function handleDrop(e: DragEvent, targetTabId: string) {
    e.preventDefault();
    if (!dragTabId || dragTabId === targetTabId) return;
    const fromIdx = tabs.findIndex(t => t.id === dragTabId);
    const toTabIdx = tabs.findIndex(t => t.id === targetTabId);
    if (fromIdx < 0 || toTabIdx < 0) return;
    let toIdx = toTabIdx;
    if (dragSide === "right") toIdx = toTabIdx + (fromIdx < toTabIdx ? 0 : 1);
    else toIdx = toTabIdx - (fromIdx < toTabIdx ? 1 : 0);
    if (toIdx !== fromIdx) {
      reorderTabs(fromIdx, toIdx);
    }
    dragTabId = null;
    dragOverTabId = null;
    dragSide = null;
  }

  function handleDragEnd() {
    dragTabId = null;
    dragOverTabId = null;
    dragSide = null;
  }
</script>

<div class="workspace-tab-bar">
  {#each tabs as tab (tab.id)}
    {@const isActive = tab.id === activeTabId}
    {@const status = getFeatureStatus(tab)}
    <div
      class="workspace-tab"
      class:workspace-tab--active={isActive}
      class:workspace-tab--dragging={dragTabId === tab.id}
      class:workspace-tab--drag-left={dragOverTabId === tab.id && dragSide === "left"}
      class:workspace-tab--drag-right={dragOverTabId === tab.id && dragSide === "right"}
      onclick={() => onSwitchTab(tab.id)}
      onkeydown={(e) => { if (e.key === 'Enter') onSwitchTab(tab.id); }}
      onmousedown={(e) => handleMouseDown(e, tab.id)}
      oncontextmenu={(e) => handleContextMenu(e, tab.id)}
      draggable="true"
      ondragstart={(e) => handleDragStart(e, tab.id)}
      ondragover={(e) => handleDragOver(e, tab.id)}
      ondrop={(e) => handleDrop(e, tab.id)}
      ondragend={handleDragEnd}
      title={getFeatureTitle(tab)}
      role="tab"
      tabindex="0"
    >
      <span class="workspace-tab-status" data-status={status}></span>
      <span class="workspace-tab-title">{getFeatureTitle(tab)}</span>
      <button
        class="workspace-tab-close"
        onclick={(e) => { e.stopPropagation(); onCloseTab(tab.id); }}
        title="Close tab"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M3 3l6 6M9 3l-6 6" />
        </svg>
      </button>
    </div>
  {/each}
</div>

{#if contextMenu}
  <div
    class="workspace-tab-context"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => { if (e.key === 'Escape') closeContextMenu(); }}
    role="menu"
    tabindex="-1"
  >
    <button class="workspace-tab-context-item" onclick={() => { onCloseTab(contextMenu!.tabId); closeContextMenu(); }}>Close</button>
    <button class="workspace-tab-context-item" onclick={() => { onCloseOtherTabs(contextMenu!.tabId); closeContextMenu(); }}>Close Others</button>
    <button class="workspace-tab-context-item" onclick={() => { onCloseAllTabs(); closeContextMenu(); }}>Close All</button>
  </div>
{/if}
