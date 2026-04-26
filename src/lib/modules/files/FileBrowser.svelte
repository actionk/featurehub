<script lang="ts">
  import type { FileEntry, Folder } from "../../api/tauri";
  import { getFiles, getFolders, addFiles, createFolder, moveFile, syncWorkspaceFiles } from "../../api/tauri";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { setToolbarActions, clearToolbarActions } from "../../stores/tabToolbar.svelte";
  import { onDestroy } from "svelte";
  import type { TabContext } from "../registry";
  import FileList from "./FileList.svelte";
  import FilePreviewPanel from "./FilePreviewPanel.svelte";

  let { featureId }: TabContext = $props();

  let files = $state<FileEntry[]>([]);
  let folders = $state<Folder[]>([]);
  let loading = $state(true);
  let selectedFile = $state<FileEntry | null>(null);
  let dragOver = $state(false);
  let showNewFolderInput = $state(false);
  let newFolderName = $state("");
  let newFolderParentId = $state<string | null>(null);

  // Resizable preview panel (persisted, defaults to 70% of container)
  const savedWidth = localStorage.getItem("featurehub:previewWidth");
  let previewWidth = $state(savedWidth ? Number(savedWidth) : Math.round(window.innerWidth * 0.7));
  let isResizing = $state(false);
  let browserEl: HTMLDivElement | undefined = $state();

  let resizeCleanup: (() => void) | null = null;

  function onResizeStart(e: MouseEvent) {
    e.preventDefault();
    isResizing = true;
    const startX = e.clientX;
    const startWidth = previewWidth;

    function onMouseMove(ev: MouseEvent) {
      const containerWidth = browserEl?.clientWidth ?? 800;
      let newWidth = startWidth - (ev.clientX - startX);
      newWidth = Math.max(200, Math.min(newWidth, containerWidth * 0.7));
      previewWidth = newWidth;
    }

    function cleanup() {
      isResizing = false;
      localStorage.setItem("featurehub:previewWidth", String(previewWidth));
      window.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("mouseup", cleanup);
      resizeCleanup = null;
    }

    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("mouseup", cleanup);
    resizeCleanup = cleanup;
  }

  $effect(() => {
    // Clear stale preview when switching features
    selectedFile = null;
    loadData();
    const interval = setInterval(syncAndRefresh, 4000);
    return () => {
      clearInterval(interval);
      // Clean up any in-progress resize listeners on unmount
      resizeCleanup?.();
    };
  });

  // Listen for Tauri native drag-drop events
  $effect(() => {
    const unlisten = getCurrentWebview().onDragDropEvent(async (event) => {
      if (event.payload.type === "over") {
        dragOver = true;
      } else if (event.payload.type === "drop") {
        dragOver = false;
        const paths = event.payload.paths;
        if (paths.length > 0) {
          try {
            await addFiles(featureId, paths, null);
            await loadData();
          } catch (e) {
            console.error("Failed to add dropped files:", e);
          }
        }
      } else {
        dragOver = false;
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  });

  // Background sync + refresh without showing loading state (used by periodic poll)
  async function syncAndRefresh() {
    try {
      await syncWorkspaceFiles(featureId);
    } catch (e) {
      console.error("Failed to sync workspace files:", e);
    }
    try {
      const [f, d] = await Promise.all([getFiles(featureId), getFolders(featureId)]);
      files = f;
      folders = d;
      // Update selected file with fresh data (e.g. size changed on disk)
      if (selectedFile) {
        const fresh = f.find((file) => file.id === selectedFile!.id);
        if (fresh && fresh.size !== selectedFile!.size) {
          selectedFile = fresh;
        }
      }
    } catch (e) {
      console.error("Failed to refresh files:", e);
    }
  }

  async function loadData() {
    loading = true;
    try {
      try {
        await syncWorkspaceFiles(featureId);
      } catch (e) {
        console.error("Failed to sync workspace files:", e);
      }
      const [f, d] = await Promise.all([
        getFiles(featureId),
        getFolders(featureId),
      ]);
      files = f;
      folders = d;

      // Restore persisted file selection
      if (!selectedFile) {
        const savedFileId = localStorage.getItem(`featurehub:selectedFile:${featureId}`);
        if (savedFileId) {
          const found = f.find((file) => file.id === savedFileId);
          if (found) selectedFile = found;
          else localStorage.removeItem(`featurehub:selectedFile:${featureId}`);
        }
      }
    } catch (e) {
      console.error("Failed to load files:", e);
    } finally {
      loading = false;
    }
  }

  async function handleAddFiles() {
    try {
      const selected = await open({
        multiple: true,
        title: "Select files to attach",
      });
      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        if (paths.length > 0) {
          await addFiles(featureId, paths, null);
          await loadData();
        }
      }
    } catch (e) {
      console.error("Failed to add files:", e);
    }
  }

  async function handleCreateFolder() {
    showNewFolderInput = true;
    newFolderName = "New Folder";
    newFolderParentId = null;
  }

  async function submitNewFolder() {
    const name = newFolderName.trim();
    if (!name) {
      showNewFolderInput = false;
      return;
    }
    try {
      await createFolder(featureId, newFolderParentId, name);
      await loadData();
    } catch (e) {
      console.error("Failed to create folder:", e);
    }
    showNewFolderInput = false;
    newFolderName = "";
  }

  function cancelNewFolder() {
    showNewFolderInput = false;
    newFolderName = "";
  }

  function handleSelectFile(file: FileEntry) {
    selectedFile = file;
    localStorage.setItem(`featurehub:selectedFile:${featureId}`, file.id);
  }

  function handleClosePreview() {
    selectedFile = null;
    localStorage.removeItem(`featurehub:selectedFile:${featureId}`);
  }

  async function handleMoveFile(fileId: string, folderId: string | null) {
    try {
      await moveFile(fileId, folderId);
      await loadData();
    } catch (e) {
      console.error("Failed to move file:", e);
    }
  }

  // Register toolbar actions
  $effect(() => {
    setToolbarActions("files", [
      {
        id: "add-files",
        label: "Add Files",
        icon: '<svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor"><path d="M8 1a1 1 0 011 1v5h5a1 1 0 110 2H9v5a1 1 0 11-2 0V9H2a1 1 0 010-2h5V2a1 1 0 011-1z"/></svg>',
        onClick: handleAddFiles,
        title: "Upload files to this feature",
      },
      {
        id: "new-folder",
        label: "New Folder",
        icon: '<svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor"><path d="M1 3.5A1.5 1.5 0 012.5 2h3.379a1.5 1.5 0 011.06.44L8.062 3.5H13.5A1.5 1.5 0 0115 5v7.5a1.5 1.5 0 01-1.5 1.5h-11A1.5 1.5 0 011 12.5v-9z"/></svg>',
        onClick: handleCreateFolder,
        title: "Create a new folder",
      },
    ]);
  });
  onDestroy(() => clearToolbarActions("files"));

</script>

{#if loading}
  <div style="text-align: center; padding: 16px; font-size: 12px; color: var(--text-muted);">Loading files...</div>
{:else}
  <div class="file-browser" class:file-browser--resizing={isResizing} class:file-browser__drop-zone--active={dragOver} bind:this={browserEl}>
    <div class="file-browser__list glass-panel">
      <FileList
        {files}
        {folders}
        selectedFileId={selectedFile?.id ?? null}
        onSelectFile={handleSelectFile}
        onAddFiles={handleAddFiles}
        onCreateFolder={handleCreateFolder}
        onFilesChanged={loadData}
        onMoveFile={handleMoveFile}
        {dragOver}
        {newFolderParentId}
        {showNewFolderInput}
        {newFolderName}
        onNewFolderNameChange={(name) => newFolderName = name}
        onSubmitNewFolder={submitNewFolder}
        onCancelNewFolder={cancelNewFolder}
      />
    </div>

    {#if selectedFile}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="file-browser__divider" onmousedown={onResizeStart}></div>
      <div class="file-browser__preview glass-panel" style="width: {previewWidth}px;">
        <FilePreviewPanel file={selectedFile} onClose={handleClosePreview} onFileSaved={loadData} />
      </div>
    {/if}
  </div>
{/if}
