<script lang="ts">
  import type { FileEntry, FilePreview } from "../../api/tauri";
  import { previewFile, openFile, saveFileContent } from "../../api/tauri";
  import { formatFileSize } from "../../utils/format";
  import { getOpenFgaEnabled } from "../../stores/settings.svelte";
  import MarkdownPreview from "../../components/MarkdownPreview.svelte";
  import OpenFgaPreview from "../../components/OpenFgaPreview.svelte";

  let {
    file,
    onClose,
    onFileSaved,
  }: {
    file: FileEntry;
    onClose: () => void;
    onFileSaved?: () => void;
  } = $props();

  let preview = $state<FilePreview | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let cachedFileId = $state<string | null>(null);
  let cachedFileSize = $state<number | null>(null);

  let editing = $state(false);
  let editContent = $state("");
  let saving = $state(false);
  let dirty = $state(false);

  $effect(() => {
    if (file.id !== cachedFileId) {
      editing = false;
      dirty = false;
      htmlRendered = true;
      loadPreview();
    } else if (file.size !== cachedFileSize && !editing) {
      // File content changed on disk — reload preview
      loadPreview();
    }
  });

  async function loadPreview() {
    loading = true;
    error = null;
    try {
      preview = await previewFile(file.id);
      cachedFileId = file.id;
      cachedFileSize = file.size;
    } catch (e) {
      error = String(e);
      preview = null;
    } finally {
      loading = false;
    }
  }

  let isMarkdown = $derived(
    /\.md$/i.test(file.filename)
  );

  let openfgaEnabled = $derived(getOpenFgaEnabled());

  let isOpenFga = $derived(
    openfgaEnabled && /\.(fga|openfga)$/i.test(file.filename)
  );

  let isHtml = $derived(
    /\.html?$/i.test(file.filename)
  );

  let htmlRendered = $state(true);

  let isEditable = $derived(
    preview?.preview_type === "text" && !preview?.truncated
  );

  async function handleOpenExternal() {
    try {
      await openFile(file.id);
    } catch (e) {
      console.error("Failed to open file:", e);
    }
  }

  function startEditing() {
    if (!preview?.content) return;
    editContent = preview.content;
    editing = true;
    dirty = false;
  }

  function cancelEditing() {
    editing = false;
    dirty = false;
  }

  async function saveFile() {
    saving = true;
    try {
      await saveFileContent(file.id, editContent);
      // Update preview content in place
      if (preview) {
        preview = { ...preview, content: editContent };
      }
      editing = false;
      dirty = false;
      onFileSaved?.();
    } catch (e) {
      console.error("Failed to save file:", e);
    } finally {
      saving = false;
    }
  }

  function handleEditorInput(e: Event) {
    const textarea = e.target as HTMLTextAreaElement;
    editContent = textarea.value;
    dirty = true;
  }

  function handleEditorKeydown(e: KeyboardEvent) {
    // Ctrl+S / Cmd+S to save
    if ((e.ctrlKey || e.metaKey) && e.key === "s") {
      e.preventDefault();
      if (dirty) saveFile();
    }
    // Escape to cancel
    if (e.key === "Escape") {
      e.preventDefault();
      cancelEditing();
    }
    // Tab inserts tab character
    if (e.key === "Tab") {
      e.preventDefault();
      const textarea = e.target as HTMLTextAreaElement;
      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;
      editContent = editContent.substring(0, start) + "\t" + editContent.substring(end);
      dirty = true;
      // Restore cursor position after Svelte updates the textarea
      requestAnimationFrame(() => {
        textarea.selectionStart = textarea.selectionEnd = start + 1;
      });
    }
  }
</script>

<div class="preview-panel file-preview">
  <div class="preview-header">
    <div class="preview-header__info">
      <div class="preview-header__filename">{file.filename}</div>
      <div class="preview-header__size">{formatFileSize(file.size)}</div>
    </div>
    <div class="preview-header__actions">
      {#if editing}
        <button class="btn-subtle" onclick={cancelEditing}>Cancel</button>
        <button class="btn-accent" onclick={saveFile} disabled={saving || !dirty}>
          {saving ? "Saving..." : "Save"}
        </button>
      {:else}
        {#if isHtml && preview?.preview_type === "text"}
          <button class="btn-subtle" style="display: inline-flex; align-items: center; gap: 5px;" onclick={() => htmlRendered = !htmlRendered}>
            {#if htmlRendered}
              <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M5.6 4L2 8l3.6 4 1-1L3.5 8l3.1-3L5.6 4zm4.8 0l-1 1L12.5 8l-3.1 3 1 1L14 8l-3.6-4z"/></svg>
              Source
            {:else}
              <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M2 3a1 1 0 011-1h10a1 1 0 011 1v10a1 1 0 01-1 1H3a1 1 0 01-1-1V3zm1 0v10h10V3H3z"/></svg>
              Preview
            {/if}
          </button>
        {/if}
        {#if isEditable}
          <button class="btn-subtle" style="display: inline-flex; align-items: center; gap: 5px;" onclick={startEditing}>
            <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M12.1 1.3a1 1 0 011.4 0l1.2 1.2a1 1 0 010 1.4L5.8 12.8l-3.5.9.9-3.5L12.1 1.3zM4.2 10.5l-.5 2 2-.5 8-8-1.5-1.5-8 8z"/></svg>
            Edit
          </button>
        {/if}
        <button class="btn-subtle" style="display: inline-flex; align-items: center; gap: 5px;" onclick={handleOpenExternal}>
          <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M9 2h5v5l-2-2-3 3-2-2 3-3L9 2zM4 4H2v10h10v-2h-1v1H3V5h1V4z"/></svg>
          Open
        </button>
      {/if}
      <button class="btn-ghost" onclick={onClose} aria-label="Close preview">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M4.5 3.1L8 6.6l3.5-3.5 1.4 1.4L9.4 8l3.5 3.5-1.4 1.4L8 9.4l-3.5 3.5-1.4-1.4L6.6 8 3.1 4.5z"/></svg>
      </button>
    </div>
  </div>

  <div class="preview-content">
    {#if loading}
      <div class="preview-loading">Loading preview...</div>
    {:else if error}
      <div class="preview-error">Failed to load preview</div>
    {:else if preview}
      {#if editing}
        <div class="preview-editor-wrapper">
          <textarea
            class="preview-editor"
            value={editContent}
            oninput={handleEditorInput}
            onkeydown={handleEditorKeydown}
            spellcheck="false"
          ></textarea>
          <div class="preview-editor-hint">Ctrl+S to save &middot; Escape to cancel &middot; Tab inserts tab</div>
        </div>
      {:else if preview.preview_type === "text" && isMarkdown}
        <div class="preview-markdown-wrapper">
          <MarkdownPreview content={preview.content ?? ""} />
        </div>
      {:else if preview.preview_type === "text" && isHtml && htmlRendered}
        <div class="preview-html-wrapper">
          <iframe
            srcdoc={preview.content ?? ""}
            title={file.filename}
            class="preview-html"
            sandbox="allow-same-origin"
          ></iframe>
        </div>
      {:else if preview.preview_type === "text" && isOpenFga}
        <div class="preview-code-wrapper">
          {#if preview.truncated}
            <div class="preview-truncated">File truncated (showing first 2 MB)</div>
          {/if}
          <OpenFgaPreview content={preview.content ?? ""} />
        </div>
      {:else if preview.preview_type === "text"}
        <div class="preview-code-wrapper">
          {#if preview.truncated}
            <div class="preview-truncated">File truncated (showing first 2 MB)</div>
          {/if}
          <pre class="preview-code"><code>{preview.content}</code></pre>
        </div>
      {:else if preview.preview_type === "pdf"}
        <div class="preview-pdf-wrapper">
          <iframe
            src="data:application/pdf;base64,{preview.content}"
            title={file.filename}
            class="preview-pdf"
          ></iframe>
        </div>
      {:else if preview.preview_type === "image"}
        {#if preview.mime_type === "image/svg+xml"}
          <div class="preview-image-wrapper">
            <img src="data:image/svg+xml;base64,{btoa(preview.content ?? '')}" alt={file.filename} class="preview-image" />
          </div>
        {:else}
          <div class="preview-image-wrapper">
            <img src="data:{preview.mime_type};base64,{preview.content}" alt={file.filename} class="preview-image" />
          </div>
        {/if}
      {:else}
        <div class="preview-binary glass-panel--soft file-preview__binary">
          <svg width="32" height="32" viewBox="0 0 16 16" fill="var(--text-muted)"><path d="M4 1a1 1 0 00-1 1v12a1 1 0 001 1h8a1 1 0 001-1V5l-4-4H4zm5 0v4h4"/></svg>
          <div>Cannot preview this file type</div>
          <button class="btn-accent" onclick={handleOpenExternal}>Open externally</button>
        </div>
      {/if}
    {/if}
  </div>
</div>
