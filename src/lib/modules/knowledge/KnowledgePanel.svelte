<script lang="ts">
  import type { KnowledgeEntry, KnowledgeFolder } from "../../api/types";
  import {
    getAllKnowledgeEntries,
    getKnowledgeFolders,
    createKnowledgeEntry,
    createKnowledgeFolder,
    deleteKnowledgeFolder,
    renameKnowledgeFolder,
  } from "../../api/knowledge";
  import KnowledgeFolderTree from "./KnowledgeFolderTree.svelte";
  import KnowledgeEntryEditor from "./KnowledgeEntryEditor.svelte";

  let {
    onClose,
  }: {
    onClose: () => void;
  } = $props();

  let entries = $state<KnowledgeEntry[]>([]);
  let folders = $state<KnowledgeFolder[]>([]);
  let selectedEntryId = $state<string | null>(null);
  let selectedFolderId = $state<string | null>(null);
  let loading = $state(true);

  let selectedEntry = $derived(
    selectedEntryId ? entries.find(e => e.id === selectedEntryId) ?? null : null,
  );

  $effect(() => {
    loadData();
  });

  async function loadData() {
    loading = true;
    try {
      const [e, f] = await Promise.all([
        getAllKnowledgeEntries(),
        getKnowledgeFolders(),
      ]);
      entries = e;
      folders = f;
    } catch (e) {
      console.error("Failed to load knowledge base:", e);
    } finally {
      loading = false;
    }
  }

  async function handleCreateEntry(folderId: string | null) {
    try {
      const entry = await createKnowledgeEntry({
        title: "New Entry",
        content: "",
        folderId,
      });
      await loadData();
      selectedEntryId = entry.id;
    } catch (e) {
      console.error("Failed to create entry:", e);
    }
  }

  async function handleCreateFolder(parentId: string | null) {
    try {
      await createKnowledgeFolder("New Folder", parentId);
      await loadData();
    } catch (e) {
      console.error("Failed to create folder:", e);
    }
  }

  async function handleDeleteFolder(id: string) {
    if (!confirm("Delete this folder? Entries will be moved to the parent folder.")) return;
    try {
      await deleteKnowledgeFolder(id);
      if (selectedFolderId === id) selectedFolderId = null;
      await loadData();
    } catch (e) {
      console.error("Failed to delete folder:", e);
    }
  }

  async function handleRenameFolder(id: string, name: string) {
    try {
      await renameKnowledgeFolder(id, name);
      await loadData();
    } catch (e) {
      console.error("Failed to rename folder:", e);
    }
  }

  function handleEntryDeleted() {
    selectedEntryId = null;
    loadData();
  }
</script>

<div class="kb-panel knowledge-panel">
  <div class="kb-panel-header">
    <h2 class="kb-panel-title">Knowledge Base</h2>
    <button class="btn btn--sm btn--icon" onclick={onClose} title="Close">
      <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M3.72 3.72a.75.75 0 011.06 0L8 6.94l3.22-3.22a.75.75 0 111.06 1.06L9.06 8l3.22 3.22a.75.75 0 11-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 01-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 010-1.06z"/></svg>
    </button>
  </div>
  <div class="kb-panel-body">
    <div class="glass-panel knowledge-panel__tree">
      <KnowledgeFolderTree
        {folders}
        {entries}
        {selectedEntryId}
        {selectedFolderId}
        onSelectEntry={(id) => selectedEntryId = id}
        onSelectFolder={(id) => selectedFolderId = id}
        onCreateEntry={handleCreateEntry}
        onCreateFolder={handleCreateFolder}
        onDeleteFolder={handleDeleteFolder}
        onRenameFolder={handleRenameFolder}
      />
    </div>
    <div class="glass-panel knowledge-panel__editor kb-panel-editor">
      <KnowledgeEntryEditor
        entry={selectedEntry}
        onSaved={loadData}
        onDeleted={handleEntryDeleted}
      />
    </div>
  </div>
</div>

<style>
  .kb-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .kb-panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 16px;
    border-bottom: 1px solid var(--border);
  }
  .kb-panel-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }
  .kb-panel-body {
    display: flex;
    flex: 1;
    overflow: hidden;
  }
  .kb-panel-editor {
    flex: 1;
    overflow-y: auto;
  }
</style>
