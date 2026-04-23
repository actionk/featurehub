<script lang="ts">
  import type { Folder } from "../../api/tauri";
  import { renameFolder } from "../../api/tauri";

  let {
    folders,
    currentFolderId,
    onNavigate,
    onFoldersChanged,
  }: {
    folders: Folder[];
    currentFolderId: string | null;
    onNavigate: (folderId: string | null) => void;
    onFoldersChanged?: () => void;
  } = $props();

  let renamingId = $state<string | null>(null);
  let renameInput = $state("");

  function autoFocus(node: HTMLInputElement) {
    node.focus();
    node.select();
  }

  let breadcrumbPath = $derived.by(() => {
    if (!currentFolderId) return [];
    const path: Folder[] = [];
    let id: string | null = currentFolderId;
    while (id) {
      const folder = folders.find((f) => f.id === id);
      if (!folder) break;
      path.unshift(folder);
      id = folder.parent_id;
    }
    return path;
  });

  function startRename(folder: Folder) {
    renamingId = folder.id;
    renameInput = folder.name;
  }

  async function finishRename() {
    if (!renamingId) return;
    const name = renameInput.trim();
    if (name) {
      try {
        await renameFolder(renamingId, name);
        onFoldersChanged?.();
      } catch (err) {
        console.error("Failed to rename folder:", err);
      }
    }
    renamingId = null;
    renameInput = "";
  }

  function handleRenameKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") finishRename();
    if (e.key === "Escape") { renamingId = null; renameInput = ""; }
  }
</script>

<div class="breadcrumb folder-breadcrumb">
  <button class="breadcrumb__item breadcrumb__item--clickable aurora-pill aurora-pill--no-dot aurora-pill--muted" onclick={() => onNavigate(null)}>
    <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M2 2h5l1 1h6v10H2V2zm1 1v9h10V4H7.5L6.5 3H3z"/></svg>
    Files
  </button>
  {#each breadcrumbPath as crumb, i}
    <span class="breadcrumb__separator folder-breadcrumb__sep">/</span>
    {#if renamingId === crumb.id}
      <input
        type="text"
        class="form-input breadcrumb__rename-input"
        bind:value={renameInput}
        onblur={finishRename}
        onkeydown={handleRenameKeydown}
        use:autoFocus
      />
    {:else if i < breadcrumbPath.length - 1}
      <button
        class="breadcrumb__item breadcrumb__item--clickable aurora-pill aurora-pill--no-dot aurora-pill--muted"
        onclick={() => onNavigate(crumb.id)}
        ondblclick={() => startRename(crumb)}
      >
        {crumb.name}
      </button>
    {:else}
      <span
        class="breadcrumb__item breadcrumb__item--current aurora-pill aurora-pill--no-dot"
        role="button"
        tabindex="0"
        ondblclick={() => startRename(crumb)}
      >
        {crumb.name}
      </span>
    {/if}
  {/each}
</div>
