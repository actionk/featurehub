<script lang="ts">
  import type { Note } from "../../api/tauri";
  import { saveNote } from "../../api/tauri";
  import MarkdownPreview from "../../components/MarkdownPreview.svelte";

  let {
    featureId,
    note = null,
    onNoteChanged,
  }: {
    featureId: string;
    note?: Note | null;
    onNoteChanged?: (note: Note | null) => void;
  } = $props();

  type ViewMode = "edit" | "preview" | "split";

  let content = $state("");
  let mode = $state<ViewMode>("split");
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  let saving = $state(false);
  let lastSavedContent = $state("");

  // Sync content from parent prop when note changes (feature switch or refresh)
  $effect(() => {
    content = note?.content ?? "";
    lastSavedContent = content;
  });

  $effect(() => {
    const currentFeatureId = featureId;

    // Flush unsaved notes when the window is closing
    function handleBeforeUnload() {
      if (content !== lastSavedContent) {
        saveNote(currentFeatureId, content);
      }
    }
    window.addEventListener("beforeunload", handleBeforeUnload);

    return () => {
      if (saveTimer) clearTimeout(saveTimer);
      // Save on cleanup (e.g. navigating away from this feature)
      const pendingContent = content;
      const savedContent = lastSavedContent;
      if (pendingContent !== savedContent) {
        saveNote(currentFeatureId, pendingContent);
      }
      window.removeEventListener("beforeunload", handleBeforeUnload);
    };
  });

  async function doSave() {
    if (content === lastSavedContent) return;
    saving = true;
    try {
      const saved = await saveNote(featureId, content);
      lastSavedContent = content;
      onNoteChanged?.(saved);
    } catch (e) {
      console.error("Failed to save note:", e);
    } finally {
      saving = false;
    }
  }

  function handleInput() {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(doSave, 1000);
  }

  function handleBlur() {
    if (saveTimer) clearTimeout(saveTimer);
    doSave();
  }

  let previewContent = $derived(content || "*No notes yet*");

  let saveStatus = $derived(
    saving ? "Saving..." :
    content !== lastSavedContent ? "Unsaved" :
    ""
  );
</script>

<div class="notes-container notes-editor">
  <div class="notes-toolbar notes-editor__toolbar">
    {#each [["edit", "Edit"], ["split", "Split"], ["preview", "Preview"]] as [m, label]}
      <button
        class="notes-toolbar-btn btn btn--sm {mode === m ? 'notes-toolbar-btn--active' : ''}"
        onclick={() => (mode = m as ViewMode)}
      >
        {label}
      </button>
    {/each}
    <div style="flex: 1;"></div>
    {#if saveStatus}
      <span style="font-size: 11px; color: var(--text-muted);">{saveStatus}</span>
    {/if}
  </div>

    <div class="notes-editor">
      {#if mode === "edit" || mode === "split"}
        <textarea
          class="notes-textarea input notes-editor__textarea"
          bind:value={content}
          oninput={handleInput}
          onblur={handleBlur}
          placeholder="Write notes in Markdown..."
        ></textarea>
      {/if}
      {#if mode === "preview" || mode === "split"}
        <div class="notes-preview">
          <MarkdownPreview content={previewContent} />
        </div>
      {/if}
    </div>
</div>
