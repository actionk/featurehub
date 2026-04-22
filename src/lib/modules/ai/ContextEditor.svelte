<script lang="ts">
  import { getContext, saveContext } from "../../api/tauri";
  import MarkdownPreview from "../../components/MarkdownPreview.svelte";

  let {
    featureId,
    hideHeader = false,
  }: {
    featureId: string;
    hideHeader?: boolean;
  } = $props();

  type ViewMode = "edit" | "preview" | "split";

  let content = $state("");
  let mode = $state<ViewMode>("split");
  let loading = $state(true);
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  let saving = $state(false);
  let lastSavedContent = $state("");

  $effect(() => {
    const currentFeatureId = featureId;
    loadContext();

    function handleBeforeUnload() {
      if (content !== lastSavedContent) {
        saveContext(currentFeatureId, content);
      }
    }
    window.addEventListener("beforeunload", handleBeforeUnload);

    return () => {
      if (saveTimer) clearTimeout(saveTimer);
      const pendingContent = content;
      const savedContent = lastSavedContent;
      if (pendingContent !== savedContent) {
        saveContext(currentFeatureId, pendingContent);
      }
      window.removeEventListener("beforeunload", handleBeforeUnload);
    };
  });

  async function loadContext() {
    loading = true;
    try {
      const ctx = await getContext(featureId);
      content = ctx?.content ?? "";
      lastSavedContent = content;
    } catch (e) {
      console.error("Failed to load context:", e);
    } finally {
      loading = false;
    }
  }

  async function doSave() {
    if (content === lastSavedContent) return;
    saving = true;
    try {
      await saveContext(featureId, content);
      lastSavedContent = content;
    } catch (e) {
      console.error("Failed to save context:", e);
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

  let previewContent = $derived(content || "*No context yet*");

  let saveStatus = $derived(
    saving ? "Saving..." :
    content !== lastSavedContent ? "Unsaved" :
    ""
  );
</script>

<div class="notes-container">
  <div class="notes-toolbar">
    {#if !hideHeader}
      <svg width="14" height="14" viewBox="0 0 16 16" fill="var(--text-muted)" style="flex-shrink: 0;"><path d="M8 1a7 7 0 100 14A7 7 0 008 1zm0 2.5a1 1 0 110 2 1 1 0 010-2zM6.5 7h2v5h-2z"/></svg>
      <span style="font-size: 11.5px; font-weight: 650; text-transform: uppercase; letter-spacing: 0.04em; color: var(--text-muted);">Context</span>
    {/if}
    <div style="flex: 1;"></div>
    {#each [["edit", "Edit"], ["split", "Split"], ["preview", "Preview"]] as [m, label]}
      <button
        class="notes-toolbar-btn {mode === m ? 'notes-toolbar-btn--active' : ''}"
        onclick={() => (mode = m as ViewMode)}
      >
        {label}
      </button>
    {/each}
    {#if saveStatus}
      <span style="font-size: 11px; color: var(--text-muted); margin-left: 8px;">{saveStatus}</span>
    {/if}
  </div>

  {#if loading}
    <div class="empty-state">
      <div style="font-size: 12px; color: var(--text-muted);">Loading...</div>
    </div>
  {:else}
    <div class="notes-editor">
      {#if mode === "edit" || mode === "split"}
        <textarea
          class="notes-textarea"
          bind:value={content}
          oninput={handleInput}
          onblur={handleBlur}
          placeholder="Write context in Markdown...&#10;&#10;This is injected into every Claude session for this feature.&#10;Use it for requirements, architecture decisions, technical details, conventions..."
        ></textarea>
      {/if}
      {#if mode === "preview" || mode === "split"}
        <div class="notes-preview">
          <MarkdownPreview content={previewContent} />
        </div>
      {/if}
    </div>
  {/if}
</div>