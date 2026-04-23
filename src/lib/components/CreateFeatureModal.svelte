<script lang="ts">
  import type { Feature, Tag, Repository } from "../api/tauri";
  import { createFeature, addLink, cloneRepository, getTags, toggleTag } from "../api/tauri";
  import { getCachedSettings } from "../stores/settings.svelte";
  import { getLinkTypeFromUrl, getLinkTypeInfo } from "../utils/linkTypes";

  let {
    onClose,
    onCreated,
  }: {
    onClose: () => void;
    onCreated: (feature: Feature) => void;
  } = $props();

  let title = $state("");
  let description = $state("");
  let status = $state("todo");
  let creating = $state(false);
  let error = $state("");
  let inputEl: HTMLInputElement | undefined = $state();

  // Links
  let linkInput = $state("");
  let pendingLinks = $state<{ url: string; title: string; type: string }[]>([]);

  // Repositories
  let defaultRepos = $state<Repository[]>([]);
  let selectedRepoUrls = $state<Set<string>>(new Set());

  // Tags
  let allTags = $state<Tag[]>([]);
  let selectedTagIds = $state<Set<string>>(new Set());

  const statuses = [
    { value: "todo", label: "Todo", color: "var(--text-muted)" },
    { value: "in_progress", label: "In Progress", color: "var(--amber)" },
    { value: "in_review", label: "In Review", color: "var(--blue)" },
  ];

  $effect(() => {
    inputEl?.focus();
  });

  $effect(() => {
    getCachedSettings().then((s) => {
      defaultRepos = s.default_repositories ?? [];
    }).catch(() => {});
    getTags().then((t) => {
      allTags = t;
    }).catch(() => {});
  });

  function addLinkFromInput(urlOverride?: string) {
    const url = (urlOverride ?? linkInput).trim();
    if (!url) return;
    // Validate URL
    try { new URL(url); } catch { error = "Invalid URL"; return; }

    const linkType = getLinkTypeFromUrl(url);
    let linkTitle = url;
    try {
      const u = new URL(url);
      linkTitle = u.hostname.replace("www.", "") + (u.pathname !== "/" ? u.pathname : "");
    } catch {}

    // Don't add duplicates
    if (pendingLinks.some((l) => l.url === url)) return;

    pendingLinks = [...pendingLinks, { url, title: linkTitle, type: linkType }];
    linkInput = "";
    error = "";
  }

  function removePendingLink(url: string) {
    pendingLinks = pendingLinks.filter((l) => l.url !== url);
  }

  function handleLinkKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      e.stopPropagation();
      addLinkFromInput();
    }
  }

  function handleLinkPaste(e: ClipboardEvent) {
    const text = (e.clipboardData?.getData("text/plain") ?? "").trim();
    if (text.startsWith("http://") || text.startsWith("https://")) {
      e.preventDefault();
      addLinkFromInput(text);
    }
  }

  function toggleRepo(url: string) {
    const next = new Set(selectedRepoUrls);
    if (next.has(url)) next.delete(url);
    else next.add(url);
    selectedRepoUrls = next;
  }

  function toggleTagSelection(tagId: string) {
    const next = new Set(selectedTagIds);
    if (next.has(tagId)) next.delete(tagId);
    else next.add(tagId);
    selectedTagIds = next;
  }

  async function handleSubmit() {
    const t = title.trim() || "Untitled Feature";
    creating = true;
    error = "";
    try {
      const feature = await createFeature(t, null, status, description.trim() || null);

      // Add links
      for (const link of pendingLinks) {
        try {
          await addLink(feature.id, link.title, link.url, link.type);
        } catch (e) {
          console.error("Failed to add link:", e);
        }
      }

      // Clone selected repositories (fire and forget — clones happen in background)
      for (const url of selectedRepoUrls) {
        const repo = defaultRepos.find((r) => r.url === url);
        try {
          await cloneRepository(feature.id, url, repo?.name ?? null);
        } catch (e) {
          console.error("Failed to start clone:", e);
        }
      }

      // Toggle tags
      for (const tagId of selectedTagIds) {
        try {
          await toggleTag(feature.id, tagId);
        } catch (e) {
          console.error("Failed to toggle tag:", e);
        }
      }

      onCreated(feature);
    } catch (e) {
      error = String(e);
    } finally {
      creating = false;
    }
  }

  let mouseDownOnBackdrop = false;

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
    if (e.key === "Enter" && !e.shiftKey && !(e.target as HTMLElement)?.classList?.contains("create-link-input")) {
      e.preventDefault();
      handleSubmit();
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="modal-backdrop"
  onmousedown={(e) => { mouseDownOnBackdrop = e.target === e.currentTarget; }}
  onclick={(e) => { if (e.target === e.currentTarget && mouseDownOnBackdrop) onClose(); }}
  onkeydown={handleKeydown}
>
  <div class="modal-content" style="width: 560px;">
    <h2 class="create-modal-header">New Feature</h2>

    <div class="create-modal-body">
      <!-- Title -->
      <div>
        <label class="form-label" for="feat-title">
          Title
        </label>
        <input
          id="feat-title"
          bind:this={inputEl}
          type="text"
          placeholder="Feature title"
          class="form-input input"
          bind:value={title}
        />
      </div>

      <!-- Description -->
      <div>
        <label class="form-label" for="feat-description">Description</label>
        <textarea
          id="feat-description"
          placeholder="Brief description of this feature..."
          class="form-input input"
          style="resize: vertical; min-height: 40px;"
          rows="2"
          bind:value={description}
        ></textarea>
      </div>

      <!-- Status -->
      <div>
        <span class="form-label">Status</span>
        <div class="create-status-row" role="radiogroup" aria-label="Feature status">
          {#each statuses as s}
            <button
              class="create-status-chip{status === s.value ? ' create-status-chip--active' : ''}"
              style="--chip-color: {s.color};"
              onclick={() => { status = s.value; }}
            >
              <span class="create-status-dot" style="background: {s.color};"></span>
              {s.label}
            </button>
          {/each}
        </div>
      </div>

      <!-- Links -->
      <div>
        <label class="form-label" for="create-link-input">Links</label>
        <div class="create-link-bar">
          <input
            id="create-link-input"
            type="url"
            class="create-link-input input"
            placeholder="Paste a URL and press Enter..."
            bind:value={linkInput}
            onkeydown={handleLinkKeydown}
            onpaste={handleLinkPaste}
          />
          {#if linkInput.trim()}
            <button class="create-link-add-btn" onclick={addLinkFromInput}>Add</button>
          {/if}
        </div>
        {#if pendingLinks.length > 0}
          <div class="create-pending-links">
            {#each pendingLinks as link (link.url)}
              {@const info = getLinkTypeInfo(link.type)}
              <div class="create-pending-link">
                <svg width="12" height="12" viewBox="0 0 16 16" fill={info.color}><path d={info.icon}/></svg>
                <span class="create-pending-link-label" title={link.url}>{info.label}: {link.title}</span>
                <button class="create-pending-link-remove" onclick={() => removePendingLink(link.url)} aria-label="Remove link">
                  <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M4.5 3.1L8 6.6l3.5-3.5 1.4 1.4L9.4 8l3.5 3.5-1.4 1.4L8 9.4l-3.5 3.5-1.4-1.4L6.6 8 3.1 4.5z"/></svg>
                </button>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Repositories -->
      {#if defaultRepos.length > 0}
        <div>
          <span class="form-label">Repositories</span>
          <div class="create-dirs-list">
            {#each defaultRepos as repo}
              <label class="create-dir-item">
                <input
                  type="checkbox"
                  checked={selectedRepoUrls.has(repo.url)}
                  onchange={() => toggleRepo(repo.url)}
                />
                <div class="create-dir-info">
                  <span class="create-dir-path">{repo.name ?? repo.url.split("/").pop()?.replace(".git", "") ?? repo.url}</span>
                  <span class="create-dir-desc" title={repo.url}>{repo.url}</span>
                  {#if repo.description}
                    <span class="create-dir-desc">{repo.description}</span>
                  {/if}
                </div>
              </label>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Tags -->
      {#if allTags.length > 0}
        <div>
          <span class="form-label">Tags</span>
          <div class="create-tags-row" role="group" aria-label="Feature tags">
            {#each allTags as tag (tag.id)}
              <button
                class="create-tag-chip{selectedTagIds.has(tag.id) ? ' create-tag-chip--active' : ''}"
                style="--tag-color: {tag.color};"
                onclick={() => toggleTagSelection(tag.id)}
              >
                <span class="create-tag-dot" style="background: {tag.color};"></span>
                {tag.name}
              </button>
            {/each}
          </div>
        </div>
      {/if}

      {#if error}
        <div class="create-error-msg">
          {error}
        </div>
      {/if}
    </div>

    <div class="create-modal-footer">
      <button class="btn btn-subtle" onclick={onClose}>Cancel</button>
      <button class="btn btn--primary btn-new" onclick={handleSubmit} disabled={creating}>
        {creating ? "Creating..." : "Create Feature"}
      </button>
    </div>
  </div>
</div>
