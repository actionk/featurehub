<script lang="ts">
  import type { FileEntry, Folder } from "../../api/tauri";
  import { deleteFile, openFile, revealFile, getFilePath, renameFile, deleteFolder, renameFolder } from "../../api/tauri";
  import { formatFileSize } from "../../utils/format";

  let {
    files,
    folders,
    selectedFileId,
    onSelectFile,
    onAddFiles,
    onCreateFolder,
    onFilesChanged,
    onMoveFile,
    dragOver,
    newFolderParentId,
    showNewFolderInput,
    newFolderName,
    onNewFolderNameChange,
    onSubmitNewFolder,
    onCancelNewFolder,
  }: {
    files: FileEntry[];
    folders: Folder[];
    selectedFileId: string | null;
    onSelectFile: (file: FileEntry) => void;
    onAddFiles: () => void;
    onCreateFolder: () => void;
    onFilesChanged: () => void;
    onMoveFile: (fileId: string, folderId: string | null) => void;
    dragOver: boolean;
    newFolderParentId: string | null;
    showNewFolderInput: boolean;
    newFolderName: string;
    onNewFolderNameChange: (name: string) => void;
    onSubmitNewFolder: () => void;
    onCancelNewFolder: () => void;
  } = $props();

  let hoveredFileId = $state<string | null>(null);
  let hoveredFolderId = $state<string | null>(null);
  let renamingFolderId = $state<string | null>(null);
  let renamingFileId = $state<string | null>(null);
  let renameInput = $state("");
  let expandedFolders = $state<Set<string>>(new Set());

  // Context menu
  let contextMenu = $state<{ x: number; y: number; file: FileEntry } | null>(null);

  function handleContextMenu(e: MouseEvent, file: FileEntry) {
    e.preventDefault();
    e.stopPropagation();
    contextMenu = { x: e.clientX, y: e.clientY, file };
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  $effect(() => {
    if (contextMenu) {
      const handler = () => closeContextMenu();
      window.addEventListener("click", handler);
      window.addEventListener("contextmenu", handler);
      return () => {
        window.removeEventListener("click", handler);
        window.removeEventListener("contextmenu", handler);
      };
    }
  });

  async function handleRevealFile(file: FileEntry) {
    try {
      await revealFile(file.id);
    } catch (err) {
      console.error("Failed to reveal file:", err);
    }
  }

  async function handleCopyPath(file: FileEntry) {
    try {
      const path = await getFilePath(file.id);
      await navigator.clipboard.writeText(path);
    } catch (err) {
      console.error("Failed to copy path:", err);
    }
  }

  // File filter
  let filterQuery = $state("");
  let filterInputEl = $state<HTMLInputElement | null>(null);

  let filterLower = $derived(filterQuery.trim().toLowerCase());

  // When filtering, determine which files match and which folders contain matches
  let matchingFileIds = $derived.by(() => {
    if (!filterLower) return null;
    return new Set(files.filter(f => f.filename.toLowerCase().includes(filterLower)).map(f => f.id));
  });

  // Folders that contain matching files (recursively)
  let matchingFolderIds = $derived.by(() => {
    if (!matchingFileIds) return null;
    const result = new Set<string>();
    // For each matching file, walk up the folder chain
    for (const file of files) {
      if (!matchingFileIds.has(file.id)) continue;
      let folderId = file.folder_id;
      while (folderId) {
        if (result.has(folderId)) break;
        result.add(folderId);
        const folder = folders.find(f => f.id === folderId);
        folderId = folder?.parent_id ?? null;
      }
    }
    // Also include folders whose name matches
    for (const folder of folders) {
      if (folder.name.toLowerCase().includes(filterLower)) {
        let fid: string | null = folder.id;
        while (fid) {
          if (result.has(fid)) break;
          result.add(fid);
          const f = folders.find(fo => fo.id === fid);
          fid = f?.parent_id ?? null;
        }
      }
    }
    return result;
  });

  // Custom drag state (mouse-based, not HTML5 DnD)
  let draggingFileId = $state<string | null>(null);
  let dragGhost = $state<{ x: number; y: number; filename: string } | null>(null);
  let dropTargetFolderId = $state<string | null>(null);
  let dragStartPos = $state<{ x: number; y: number } | null>(null);
  let pendingDragFile = $state<FileEntry | null>(null);
  let justDropped = $state(false);
  const DRAG_THRESHOLD = 5;

  function autoFocus(node: HTMLInputElement) {
    node.focus();
    node.select();
  }

  function toggleFolder(folderId: string) {
    if (justDropped) return;
    const next = new Set(expandedFolders);
    if (next.has(folderId)) {
      next.delete(folderId);
    } else {
      next.add(folderId);
    }
    expandedFolders = next;
  }

  function getChildFolders(parentId: string | null): Folder[] {
    let result = folders.filter((f) => f.parent_id === parentId);
    if (matchingFolderIds) {
      result = result.filter(f => matchingFolderIds!.has(f.id));
    }
    return result;
  }

  function getChildFiles(folderId: string | null): FileEntry[] {
    let result = files.filter((f) => f.folder_id === folderId);
    if (matchingFileIds) {
      result = result.filter(f => matchingFileIds!.has(f.id));
    }
    return result;
  }

  function hasChildren(folderId: string): boolean {
    return folders.some((f) => f.parent_id === folderId) || files.some((f) => f.folder_id === folderId);
  }

  async function handleDeleteFile(e: MouseEvent, file: FileEntry) {
    e.stopPropagation();
    try {
      await deleteFile(file.id);
      onFilesChanged();
    } catch (err) {
      console.error("Failed to delete file:", err);
    }
  }

  async function handleOpenFile(file: FileEntry) {
    try {
      await openFile(file.id);
    } catch (err) {
      console.error("Failed to open file:", err);
    }
  }

  async function handleDeleteFolder(e: MouseEvent, folder: Folder) {
    e.stopPropagation();
    try {
      await deleteFolder(folder.id);
      onFilesChanged();
    } catch (err) {
      console.error("Failed to delete folder:", err);
    }
  }

  function startRenameFolder(folder: Folder) {
    renamingFolderId = folder.id;
    renameInput = folder.name;
  }

  async function finishRename() {
    if (!renamingFolderId) return;
    const name = renameInput.trim();
    if (name) {
      try {
        await renameFolder(renamingFolderId, name);
        onFilesChanged();
      } catch (err) {
        console.error("Failed to rename folder:", err);
      }
    }
    renamingFolderId = null;
    renameInput = "";
  }

  function handleRenameKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") finishRename();
    if (e.key === "Escape") { renamingFolderId = null; renamingFileId = null; renameInput = ""; }
  }

  function startRenameFile(file: FileEntry) {
    renamingFileId = file.id;
    renameInput = file.filename;
  }

  async function finishFileRename() {
    if (!renamingFileId) return;
    const name = renameInput.trim();
    if (name) {
      try {
        await renameFile(renamingFileId, name);
        onFilesChanged();
      } catch (err) {
        console.error("Failed to rename file:", err);
      }
    }
    renamingFileId = null;
    renameInput = "";
  }

  function handleFileRenameKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") finishFileRename();
    if (e.key === "Escape") { renamingFileId = null; renameInput = ""; }
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
  }

  // ── Custom mouse drag ─────────────────────────────────────────────

  function handleFileMouseDown(e: MouseEvent, file: FileEntry) {
    if (e.button !== 0) return;
    if ((e.target as HTMLElement).closest("button")) return;
    e.preventDefault();
    dragStartPos = { x: e.clientX, y: e.clientY };
    pendingDragFile = file;

    window.addEventListener("mousemove", handleGlobalMouseMove);
    window.addEventListener("mouseup", handleGlobalMouseUp);
  }

  function handleGlobalMouseMove(e: MouseEvent) {
    if (pendingDragFile && dragStartPos && !draggingFileId) {
      const dx = e.clientX - dragStartPos.x;
      const dy = e.clientY - dragStartPos.y;
      if (Math.abs(dx) > DRAG_THRESHOLD || Math.abs(dy) > DRAG_THRESHOLD) {
        draggingFileId = pendingDragFile.id;
        dragGhost = { x: e.clientX, y: e.clientY, filename: pendingDragFile.filename };
      }
    }
    if (draggingFileId && dragGhost) {
      dragGhost = { ...dragGhost, x: e.clientX, y: e.clientY };

      const el = document.elementFromPoint(e.clientX, e.clientY);
      const folderEl = el?.closest("[data-folder-id]") as HTMLElement | null;
      dropTargetFolderId = folderEl?.dataset.folderId ?? null;
    }
  }

  function handleGlobalMouseUp(_e: MouseEvent) {
    window.removeEventListener("mousemove", handleGlobalMouseMove);
    window.removeEventListener("mouseup", handleGlobalMouseUp);

    if (draggingFileId && dropTargetFolderId) {
      // Auto-expand the target folder so the moved file is visible
      if (!expandedFolders.has(dropTargetFolderId)) {
        const next = new Set(expandedFolders);
        next.add(dropTargetFolderId);
        expandedFolders = next;
      }
      onMoveFile(draggingFileId, dropTargetFolderId);
      // Suppress the click event that fires after mouseup on the folder row
      justDropped = true;
      requestAnimationFrame(() => { justDropped = false; });
    }

    draggingFileId = null;
    dragGhost = null;
    dropTargetFolderId = null;
    dragStartPos = null;
    pendingDragFile = null;
  }

  function handleNewFolderKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") onSubmitNewFolder();
    if (e.key === "Escape") onCancelNewFolder();
  }
</script>

<div>
  <div class="section-title">
    Attached Files
    <div style="display: flex; gap: 4px;">
      <button class="btn-add" onclick={onCreateFolder}>+ Folder</button>
      <button class="btn-add" onclick={onAddFiles}>+ Add</button>
    </div>
  </div>

  {#if files.length > 0 || folders.length > 0}
    <div class="file-filter">
      <svg class="file-filter__icon" width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
        <path d="M11.5 7a4.5 4.5 0 1 1-9 0 4.5 4.5 0 0 1 9 0zm-.82 4.74a6 6 0 1 1 1.06-1.06l3.04 3.04-1.06 1.06-3.04-3.04z"/>
      </svg>
      <input
        bind:this={filterInputEl}
        type="text"
        class="file-filter__input"
        placeholder="Filter files..."
        bind:value={filterQuery}
        onkeydown={(e) => { if (e.key === 'Escape') { filterQuery = ''; filterInputEl?.blur(); } }}
      />
      {#if filterQuery}
        <button class="file-filter__clear" onclick={() => { filterQuery = ''; }} aria-label="Clear filter">
          <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M4.5 3.1L8 6.6l3.5-3.5 1.4 1.4L9.4 8l3.5 3.5-1.4 1.4L8 9.4l-3.5 3.5-1.4-1.4L6.6 8 3.1 4.5z"/></svg>
        </button>
      {/if}
    </div>
  {/if}

  {#if files.length === 0 && folders.length === 0}
    <div class="drop-zone {dragOver ? 'drop-zone--active' : ''}"
      ondragover={(e) => { e.preventDefault(); }}
      ondrop={handleDrop}
      onclick={onAddFiles}
      onkeydown={(e) => { if (e.key === 'Enter') onAddFiles(); }}
      role="button"
      tabindex="0"
      aria-label="File drop zone"
    >
      <div style="font-size: 20px; margin-bottom: 6px; opacity: 0.5;">+</div>
      Drop files here or click to add
    </div>
  {:else}
    <div class="drop-zone drop-zone--compact {dragOver ? 'drop-zone--active' : ''}"
      ondragover={(e) => { e.preventDefault(); }}
      ondrop={handleDrop}
      onclick={onAddFiles}
      onkeydown={(e) => { if (e.key === 'Enter') onAddFiles(); }}
      role="button"
      tabindex="0"
      aria-label="File drop zone"
    >
      Drop files here or click to add
    </div>
  {/if}

  <div class="tree">
    {#snippet folderTree(parentId: string | null, depth: number)}
      {#each getChildFolders(parentId) as folder (folder.id)}
        {@const expanded = filterLower ? true : expandedFolders.has(folder.id)}
        {@const hasKids = hasChildren(folder.id)}
        <div
          class="tree-row tree-row--folder {dropTargetFolderId === folder.id ? 'tree-row--drop-target' : ''}"
          style="--depth: {depth};"
          data-folder-id={folder.id}
          onclick={() => toggleFolder(folder.id)}
          ondblclick={() => startRenameFolder(folder)}
          onmouseenter={() => (hoveredFolderId = folder.id)}
          onmouseleave={() => (hoveredFolderId = null)}
          role="treeitem"
          tabindex="0"
          aria-selected={false}
          aria-expanded={expanded}
          onkeydown={(e) => { if (e.key === 'Enter') toggleFolder(folder.id); }}
        >
          <span class="tree-chevron {hasKids ? '' : 'tree-chevron--hidden'}">
            <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"
              style="transform: rotate({expanded ? '90deg' : '0deg'}); transition: transform 0.12s;">
              <path d="M6 3l5 5-5 5z"/>
            </svg>
          </span>
          <svg class="tree-icon" width="14" height="14" viewBox="0 0 16 16" fill="var(--amber)">
            {#if expanded}
              <path d="M1 4h3l1 1h5l1-1h3v1l-2 8H3L1 5V4zm2 1l1.7 7h6.6L13 5h-2.5l-1-1H5.5l-1 1H3z"/>
            {:else}
              <path d="M2 2h5l1 1h6v10H2V2zm1 1v9h10V4H7.5L6.5 3H3z"/>
            {/if}
          </svg>
          {#if renamingFolderId === folder.id}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div onclick={(e) => e.stopPropagation()} ondblclick={(e) => e.stopPropagation()} onkeydown={() => {}} style="flex: 1; min-width: 0;">
              <input
                type="text"
                class="form-input tree-rename-input"
                bind:value={renameInput}
                onblur={finishRename}
                onkeydown={handleRenameKeydown}
                use:autoFocus
              />
            </div>
          {:else}
            <span class="tree-name">{folder.name}</span>
          {/if}
          {#if hoveredFolderId === folder.id && renamingFolderId !== folder.id}
            <button class="btn-ghost" style="flex-shrink: 0;"
              onclick={(e) => { e.stopPropagation(); startRenameFolder(folder); }} aria-label="Rename folder">
              <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M12.1 1.3a1 1 0 011.4 0l1.2 1.2a1 1 0 010 1.4L5.8 12.8l-3.5.9.9-3.5z"/></svg>
            </button>
            <button class="btn-ghost" style="color: var(--red); flex-shrink: 0;"
              onclick={(e) => handleDeleteFolder(e, folder)} aria-label="Delete folder">
              <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor"><path d="M4.5 3.1L8 6.6l3.5-3.5 1.4 1.4L9.4 8l3.5 3.5-1.4 1.4L8 9.4l-3.5 3.5-1.4-1.4L6.6 8 3.1 4.5z"/></svg>
            </button>
          {/if}
        </div>
        {#if expanded}
          {#if showNewFolderInput && newFolderParentId === folder.id}
            <div class="tree-row tree-row--folder" style="--depth: {depth + 1};">
              <span class="tree-chevron tree-chevron--hidden">
                <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M6 3l5 5-5 5z"/></svg>
              </span>
              <svg class="tree-icon" width="14" height="14" viewBox="0 0 16 16" fill="var(--amber)">
                <path d="M2 2h5l1 1h6v10H2V2zm1 1v9h10V4H7.5L6.5 3H3z"/>
              </svg>
              <input
                type="text"
                class="form-input tree-rename-input"
                value={newFolderName}
                oninput={(e) => onNewFolderNameChange(e.currentTarget.value)}
                onblur={onSubmitNewFolder}
                onkeydown={handleNewFolderKeydown}
                use:autoFocus
              />
            </div>
          {/if}
          {@render folderTree(folder.id, depth + 1)}
          {#each getChildFiles(folder.id) as file (file.id)}
            {@render fileRow(file, depth + 1)}
          {/each}
        {/if}
      {/each}
    {/snippet}

    {#snippet fileRow(file: FileEntry, depth: number)}
      <div
        class="tree-row tree-row--file {selectedFileId === file.id ? 'tree-row--selected' : ''} {draggingFileId === file.id ? 'tree-row--dragging' : ''}"
        style="--depth: {depth};"
        onmousedown={(e) => handleFileMouseDown(e, file)}
        onclick={() => { if (!draggingFileId) onSelectFile(file); }}
        oncontextmenu={(e) => handleContextMenu(e, file)}
        onmouseenter={() => (hoveredFileId = file.id)}
        onmouseleave={() => (hoveredFileId = null)}
        ondblclick={() => startRenameFile(file)}
        role="treeitem"
        tabindex="0"
        aria-selected={selectedFileId === file.id}
        onkeydown={(e) => { if (e.key === 'Enter') onSelectFile(file); }}
      >
        <span class="tree-chevron tree-chevron--hidden">
          <svg width="10" height="10" viewBox="0 0 16 16"><path d="M6 3l5 5-5 5z"/></svg>
        </span>
        <svg class="tree-icon" width="14" height="14" viewBox="0 0 16 16" fill="var(--text-muted)">
          <path d="M4 1a1 1 0 00-1 1v12a1 1 0 001 1h8a1 1 0 001-1V5l-4-4H4zm5 0v4h4"/>
        </svg>
        {#if renamingFileId === file.id}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div onclick={(e) => e.stopPropagation()} ondblclick={(e) => e.stopPropagation()} onkeydown={() => {}} style="flex: 1; min-width: 0;">
            <input
              type="text"
              class="form-input tree-rename-input"
              bind:value={renameInput}
              onblur={finishFileRename}
              onkeydown={handleFileRenameKeydown}
              use:autoFocus
            />
          </div>
        {:else}
          <span class="tree-name tree-name--file">{file.filename}</span>
          <span class="tree-size">{formatFileSize(file.size)}</span>
        {/if}
        {#if hoveredFileId === file.id && !draggingFileId && renamingFileId !== file.id}
          <button class="btn-ghost" style="flex-shrink: 0;"
            onclick={(e) => { e.stopPropagation(); startRenameFile(file); }} aria-label="Rename file">
            <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M12.1 1.3a1 1 0 011.4 0l1.2 1.2a1 1 0 010 1.4L5.8 12.8l-3.5.9.9-3.5z"/></svg>
          </button>
          <button class="btn-ghost" style="color: var(--red); flex-shrink: 0;"
            onclick={(e) => handleDeleteFile(e, file)} aria-label="Delete file">
            <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor"><path d="M4.5 3.1L8 6.6l3.5-3.5 1.4 1.4L9.4 8l3.5 3.5-1.4 1.4L8 9.4l-3.5 3.5-1.4-1.4L6.6 8 3.1 4.5z"/></svg>
          </button>
        {/if}
      </div>
    {/snippet}

    {#if showNewFolderInput && newFolderParentId === null}
      <div class="tree-row tree-row--folder" style="--depth: 0;">
        <span class="tree-chevron tree-chevron--hidden">
          <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M6 3l5 5-5 5z"/></svg>
        </span>
        <svg class="tree-icon" width="14" height="14" viewBox="0 0 16 16" fill="var(--amber)">
          <path d="M2 2h5l1 1h6v10H2V2zm1 1v9h10V4H7.5L6.5 3H3z"/>
        </svg>
        <input
          type="text"
          class="form-input tree-rename-input"
          value={newFolderName}
          oninput={(e) => onNewFolderNameChange(e.currentTarget.value)}
          onblur={onSubmitNewFolder}
          onkeydown={handleNewFolderKeydown}
          use:autoFocus
        />
      </div>
    {/if}

    {@render folderTree(null, 0)}

    {#each getChildFiles(null) as file (file.id)}
      {@render fileRow(file, 0)}
    {/each}

    {#if filterLower && getChildFolders(null).length === 0 && getChildFiles(null).length === 0}
      <div style="padding: 8px 12px; font-size: 11px; color: var(--text-muted);">No files match "{filterQuery.trim()}"</div>
    {/if}
  </div>

</div>

{#if contextMenu}
  <div class="context-menu" style="left: {contextMenu.x}px; top: {contextMenu.y}px;">
    <button class="context-menu-item" onclick={() => { handleOpenFile(contextMenu!.file); closeContextMenu(); }}>
      <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M6.22 8.72a.75.75 0 0 0 1.06 1.06l5.22-5.22v1.69a.75.75 0 0 0 1.5 0V2.75A.75.75 0 0 0 13.25 2h-3.5a.75.75 0 0 0 0 1.5h1.69L6.22 8.72zM3.5 4A1.5 1.5 0 0 0 2 5.5v7A1.5 1.5 0 0 0 3.5 14h7a1.5 1.5 0 0 0 1.5-1.5v-3a.75.75 0 0 0-1.5 0v3h-7v-7h3a.75.75 0 0 0 0-1.5h-3z"/></svg>
      Open
    </button>
    <button class="context-menu-item" onclick={() => { handleRevealFile(contextMenu!.file); closeContextMenu(); }}>
      <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M1 4h3l1 1h5l1-1h3v1l-2 8H3L1 5V4zm2 1l1.7 7h6.6L13 5h-2.5l-1-1H5.5l-1 1H3z"/></svg>
      Reveal in Explorer
    </button>
    <button class="context-menu-item" onclick={() => { handleCopyPath(contextMenu!.file); closeContextMenu(); }}>
      <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M0 6.75C0 5.784.784 5 1.75 5h1.5a.75.75 0 0 1 0 1.5h-1.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-1.5a.75.75 0 0 1 1.5 0v1.5A1.75 1.75 0 0 1 9.25 16h-7.5A1.75 1.75 0 0 1 0 14.25v-7.5z"/><path d="M5 1.75C5 .784 5.784 0 6.75 0h7.5C15.216 0 16 .784 16 1.75v7.5A1.75 1.75 0 0 1 14.25 11h-7.5A1.75 1.75 0 0 1 5 9.25v-7.5zm1.75-.25a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25h-7.5z"/></svg>
      Copy Path
    </button>
    <div class="context-menu-separator"></div>
    <button class="context-menu-item" onclick={() => { startRenameFile(contextMenu!.file); closeContextMenu(); }}>
      <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M12.1 1.3a1 1 0 011.4 0l1.2 1.2a1 1 0 010 1.4L5.8 12.8l-3.5.9.9-3.5z"/></svg>
      Rename
    </button>
    <button class="context-menu-item context-menu-item--danger" onclick={(e) => { handleDeleteFile(e, contextMenu!.file); closeContextMenu(); }}>
      <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M11 1.75V3h2.25a.75.75 0 0 1 0 1.5H2.75a.75.75 0 0 1 0-1.5H5V1.75C5 .784 5.784 0 6.75 0h2.5C10.216 0 11 .784 11 1.75zM6.5 1.75v1.25h3V1.75a.25.25 0 0 0-.25-.25h-2.5a.25.25 0 0 0-.25.25zM4.5 6a.75.75 0 0 1 .75.75v6.5a.75.75 0 0 1-1.5 0v-6.5A.75.75 0 0 1 4.5 6zm3.75.75a.75.75 0 0 0-1.5 0v6.5a.75.75 0 0 0 1.5 0v-6.5zM10.5 6a.75.75 0 0 1 .75.75v6.5a.75.75 0 0 1-1.5 0v-6.5A.75.75 0 0 1 10.5 6z"/><path d="M3.2 4.5l.7 9.1a1.75 1.75 0 0 0 1.74 1.65h4.72a1.75 1.75 0 0 0 1.74-1.65l.7-9.1H3.2z"/></svg>
      Delete
    </button>
  </div>
{/if}

{#if dragGhost}
  <div class="drag-ghost" style="left: {dragGhost.x}px; top: {dragGhost.y}px;">
    <svg width="12" height="12" viewBox="0 0 16 16" fill="var(--text-muted)"><path d="M4 1a1 1 0 00-1 1v12a1 1 0 001 1h8a1 1 0 001-1V5l-4-4H4zm5 0v4h4"/></svg>
    {dragGhost.filename}
  </div>
{/if}
