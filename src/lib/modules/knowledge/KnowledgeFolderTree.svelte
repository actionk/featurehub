<script lang="ts">
  import type { KnowledgeFolder, KnowledgeEntry } from "../../api/types";

  let {
    folders,
    entries,
    selectedEntryId,
    selectedFolderId,
    onSelectEntry,
    onSelectFolder,
    onCreateEntry,
    onCreateFolder,
    onDeleteFolder,
    onRenameFolder,
  }: {
    folders: KnowledgeFolder[];
    entries: KnowledgeEntry[];
    selectedEntryId: string | null;
    selectedFolderId: string | null;
    onSelectEntry: (id: string) => void;
    onSelectFolder: (id: string | null) => void;
    onCreateEntry: (folderId: string | null) => void;
    onCreateFolder: (parentId: string | null) => void;
    onDeleteFolder: (id: string) => void;
    onRenameFolder: (id: string, name: string) => void;
  } = $props();

  let expandedFolders = $state<Set<string>>(new Set());
  let renamingFolderId = $state<string | null>(null);
  let renameValue = $state("");

  function toggleExpand(id: string) {
    const next = new Set(expandedFolders);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    expandedFolders = next;
  }

  function childFolders(parentId: string | null): KnowledgeFolder[] {
    return folders.filter(f => f.parent_id === parentId).sort((a, b) => a.sort_order - b.sort_order || a.name.localeCompare(b.name));
  }

  function entriesInFolder(folderId: string | null): KnowledgeEntry[] {
    return entries.filter(e => e.folder_id === folderId).sort((a, b) => a.sort_order - b.sort_order || a.title.localeCompare(b.title));
  }

  function startRename(id: string, currentName: string) {
    renamingFolderId = id;
    renameValue = currentName;
  }

  function commitRename() {
    if (renamingFolderId && renameValue.trim()) {
      onRenameFolder(renamingFolderId, renameValue.trim());
    }
    renamingFolderId = null;
  }
</script>

<div class="kb-tree">
  <div class="kb-tree-header">
    <span class="kb-tree-title">Knowledge Base</span>
    <div class="kb-tree-actions">
      <button class="btn btn--sm btn--icon" onclick={() => onCreateEntry(null)} title="New entry">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M8 2a.75.75 0 01.75.75v4.5h4.5a.75.75 0 010 1.5h-4.5v4.5a.75.75 0 01-1.5 0v-4.5h-4.5a.75.75 0 010-1.5h4.5v-4.5A.75.75 0 018 2z"/></svg>
      </button>
      <button class="btn btn--sm btn--icon" onclick={() => onCreateFolder(null)} title="New folder">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
          <path d="M1 3.5A1.5 1.5 0 012.5 2h3.879a1.5 1.5 0 011.06.44l1.122 1.12A.5.5 0 008.914 4H13.5A1.5 1.5 0 0115 5.5v7a1.5 1.5 0 01-1.5 1.5h-11A1.5 1.5 0 011 12.5v-9z"/>
        </svg>
      </button>
    </div>
  </div>

  <div class="kb-tree-list">
    {#snippet folderNode(parentId: string | null, depth: number)}
      {#each childFolders(parentId) as folder (folder.id)}
        {@const isExpanded = expandedFolders.has(folder.id)}
        {@const isSelected = selectedFolderId === folder.id}
        {@const folderEntryCount = entriesInFolder(folder.id).length}
        <div class="list-row kb-tree-item kb-tree-folder knowledge-folder-row" class:list-row--active={isSelected} class:selected={isSelected} style="padding-left: {8 + depth * 16}px; --depth: {depth};">
          <button class="tree-chevron kb-tree-expand" class:tree-chevron--expanded={isExpanded} onclick={() => toggleExpand(folder.id)} aria-label="{isExpanded ? 'Collapse' : 'Expand'} folder">
            <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor">
              <path d="M6 4l4 4-4 4"/>
            </svg>
          </button>
          {#if renamingFolderId === folder.id}
            <!-- svelte-ignore a11y_autofocus -->
            <input
              class="input kb-rename-input"
              bind:value={renameValue}
              onblur={commitRename}
              onkeydown={(e) => { if (e.key === 'Enter') commitRename(); if (e.key === 'Escape') { renamingFolderId = null; } }}
              autofocus
            />
          {:else}
            <button class="kb-tree-label knowledge-folder-row__name" onclick={() => onSelectFolder(folder.id)} ondblclick={() => startRename(folder.id, folder.name)}>
              {folder.name}
            </button>
          {/if}
          {#if folderEntryCount > 0}
            <span class="aurora-pill aurora-pill--muted aurora-pill--sm aurora-pill--no-dot">{folderEntryCount}</span>
          {/if}
          <button class="btn btn--sm btn--icon kb-tree-action" onclick={() => onCreateEntry(folder.id)} title="New entry here">+</button>
          <button class="btn btn--sm btn--icon kb-tree-action" onclick={() => onDeleteFolder(folder.id)} title="Delete folder">&times;</button>
        </div>
        {#if isExpanded}
          {#each entriesInFolder(folder.id) as entry (entry.id)}
            <div class="list-row kb-tree-item kb-tree-entry knowledge-entry-row" class:list-row--active={selectedEntryId === entry.id} class:selected={selectedEntryId === entry.id} style="padding-left: {24 + depth * 16}px" onclick={() => onSelectEntry(entry.id)} onkeydown={(e) => { if (e.key === 'Enter') onSelectEntry(entry.id); }} role="button" tabindex="0">
              <span class="kb-entry-icon">📄</span>
              <span class="kb-tree-label knowledge-entry-row__title">{entry.title || "Untitled"}</span>
            </div>
          {/each}
          {@render folderNode(folder.id, depth + 1)}
        {/if}
      {/each}
    {/snippet}

    <!-- Root entries (no folder) -->
    {#each entriesInFolder(null) as entry (entry.id)}
      <div class="list-row kb-tree-item kb-tree-entry knowledge-entry-row" class:list-row--active={selectedEntryId === entry.id} class:selected={selectedEntryId === entry.id} style="padding-left: 8px" onclick={() => onSelectEntry(entry.id)} onkeydown={(e) => { if (e.key === 'Enter') onSelectEntry(entry.id); }} role="button" tabindex="0">
        <span class="kb-entry-icon">📄</span>
        <span class="kb-tree-label knowledge-entry-row__title">{entry.title || "Untitled"}</span>
      </div>
    {/each}

    <!-- Folders at root level -->
    {@render folderNode(null, 0)}
  </div>
</div>

<style>
  .kb-tree {
    display: flex;
    flex-direction: column;
    height: 100%;
    border-right: 1px solid var(--border);
    min-width: 220px;
    max-width: 320px;
    overflow-y: auto;
  }
  .kb-tree-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 8px 4px;
    border-bottom: 1px solid var(--border);
  }
  .kb-tree-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .kb-tree-actions {
    display: flex;
    gap: 2px;
  }
  .kb-tree-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }
  .kb-tree-item {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 3px 8px;
    cursor: pointer;
    font-size: 13px;
    color: var(--text-primary);
    border-radius: 4px;
    margin: 0 4px;
  }
  .kb-tree-item:hover {
    background: var(--bg-hover);
  }
  .kb-tree-item.selected {
    background: var(--bg-selected);
  }
  .kb-tree-expand {
    background: none;
    border: none;
    padding: 2px;
    cursor: pointer;
    color: var(--text-muted);
    display: flex;
    align-items: center;
  }
  .kb-tree-label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .kb-entry-icon {
    font-size: 11px;
    flex-shrink: 0;
  }
  .kb-tree-action {
    opacity: 0;
    font-size: 14px;
    padding: 0 4px;
  }
  .kb-tree-item:hover .kb-tree-action {
    opacity: 0.6;
  }
  .kb-tree-action:hover {
    opacity: 1 !important;
  }
  .kb-rename-input {
    flex: 1;
    font-size: 13px;
    padding: 1px 4px;
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: 3px;
    color: var(--text-primary);
    outline: none;
  }
</style>
