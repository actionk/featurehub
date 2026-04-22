<script lang="ts">
  import type { KnowledgeEntry, Feature } from "../../api/types";
  import { updateKnowledgeEntry, deleteKnowledgeEntry } from "../../api/knowledge";
  import MarkdownPreview from "../../components/MarkdownPreview.svelte";

  let {
    entry,
    features = [],
    onSaved,
    onDeleted,
  }: {
    entry: KnowledgeEntry | null;
    features?: Feature[];
    onSaved: () => void;
    onDeleted: () => void;
  } = $props();

  let editTitle = $state("");
  let editDescription = $state("");
  let editContent = $state("");
  let previewMode = $state(false);
  let saveTimer: ReturnType<typeof setTimeout> | null = null;

  // Sync local state when entry changes
  $effect(() => {
    if (entry) {
      editTitle = entry.title;
      editDescription = entry.description;
      editContent = entry.content;
      previewMode = false;
    }
  });

  function scheduleSave() {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(doSave, 800);
  }

  async function doSave() {
    if (!entry) return;
    try {
      await updateKnowledgeEntry({
        id: entry.id,
        title: editTitle,
        description: editDescription,
        content: editContent,
      });
      onSaved();
    } catch (e) {
      console.error("Failed to save knowledge entry:", e);
    }
  }

  async function handleDelete() {
    if (!entry) return;
    if (!confirm(`Delete "${entry.title}"?`)) return;
    try {
      await deleteKnowledgeEntry(entry.id);
      onDeleted();
    } catch (e) {
      console.error("Failed to delete knowledge entry:", e);
    }
  }
</script>

{#if entry}
  <div class="kb-editor">
    <div class="kb-editor-header">
      <input
        class="kb-editor-title"
        bind:value={editTitle}
        oninput={scheduleSave}
        placeholder="Entry title"
      />
      <div class="kb-editor-toolbar">
        <button class="btn-ghost btn-sm" class:active={previewMode} onclick={() => previewMode = !previewMode}>
          {previewMode ? "Edit" : "Preview"}
        </button>
        <button class="btn-ghost btn-sm btn-danger" onclick={handleDelete} title="Delete entry">
          <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M5.5 5.5A.5.5 0 016 6v6a.5.5 0 01-1 0V6a.5.5 0 01.5-.5zm2.5 0a.5.5 0 01.5.5v6a.5.5 0 01-1 0V6a.5.5 0 01.5-.5zm3 .5a.5.5 0 00-1 0v6a.5.5 0 001 0V6z"/><path fill-rule="evenodd" d="M14.5 3a1 1 0 01-1 1H13v9a2 2 0 01-2 2H5a2 2 0 01-2-2V4h-.5a1 1 0 01-1-1V2a1 1 0 011-1H6a1 1 0 011-1h2a1 1 0 011 1h3.5a1 1 0 011 1v1zM4.118 4L4 4.059V13a1 1 0 001 1h6a1 1 0 001-1V4.059L11.882 4H4.118zM2.5 3V2h11v1h-11z"/></svg>
        </button>
      </div>
    </div>
    <input
      class="kb-editor-description"
      bind:value={editDescription}
      oninput={scheduleSave}
      placeholder="One-line description (shown in Claude's TOC)"
    />
    {#if previewMode}
      <div class="kb-editor-preview">
        <MarkdownPreview content={editContent} />
      </div>
    {:else}
      <textarea
        class="kb-editor-content"
        bind:value={editContent}
        oninput={scheduleSave}
        placeholder="Write markdown content..."
      ></textarea>
    {/if}
  </div>
{:else}
  <div class="kb-editor-empty">
    <p>Select an entry or create a new one</p>
  </div>
{/if}

<style>
  .kb-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 12px 16px;
    gap: 8px;
  }
  .kb-editor-header {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .kb-editor-title {
    flex: 1;
    font-size: 18px;
    font-weight: 600;
    background: none;
    border: none;
    color: var(--text-primary);
    padding: 4px 0;
    outline: none;
  }
  .kb-editor-title::placeholder {
    color: var(--text-muted);
  }
  .kb-editor-toolbar {
    display: flex;
    gap: 4px;
  }
  .kb-editor-description {
    font-size: 13px;
    background: none;
    border: none;
    border-bottom: 1px solid var(--border);
    color: var(--text-secondary);
    padding: 4px 0 8px;
    outline: none;
  }
  .kb-editor-description::placeholder {
    color: var(--text-muted);
  }
  .kb-editor-content {
    flex: 1;
    font-family: var(--font-mono);
    font-size: 13px;
    line-height: 1.6;
    background: none;
    border: none;
    color: var(--text-primary);
    resize: none;
    outline: none;
    padding: 0;
  }
  .kb-editor-content::placeholder {
    color: var(--text-muted);
  }
  .kb-editor-preview {
    flex: 1;
    overflow-y: auto;
  }
  .kb-editor-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted);
    font-size: 14px;
  }
  .btn-danger:hover {
    color: var(--status-blocked, #e55);
  }
  .active {
    color: var(--accent);
  }
</style>
