<script lang="ts">
  import type { Link } from "../../api/tauri";
  import { addLink, updateLink, deleteLink } from "../../api/tauri";
  import { getLinkTypeFromUrl, getLinkTypeInfo } from "../../utils/linkTypes";
  import LinkCard from "./LinkCard.svelte";
  import { setToolbarActions, clearToolbarActions } from "../../stores/tabToolbar.svelte";
  import { onDestroy } from "svelte";
  import type { TabContext } from "../registry";

  let { featureId, feature, onRefresh: onLinksChanged }: TabContext = $props();
  let links = $derived(feature.links ?? []);

  let editingLink = $state<Link | null>(null);
  let formTitle = $state("");
  let formUrl = $state("");
  let formDescription = $state("");
  let showEditForm = $state(false);

  let quickUrl = $state("");
  let clipboardText = $state("");
  let inputEl = $state<HTMLInputElement | null>(null);

  // Per-block sorting
  type SortKey = "date" | "name" | "status";
  const SORT_STORAGE_PREFIX = "fh-links-sort-";

  // Cache of per-type sort keys (lazy-loaded from localStorage)
  const sortCache: Record<string, SortKey> = {};

  // Reactive trigger — incremented on sort change to re-derive sorted lists
  let sortVersion = $state(0);

  function getSortKey(type: string): SortKey {
    if (sortCache[type]) return sortCache[type];
    const saved = localStorage.getItem(SORT_STORAGE_PREFIX + type) as SortKey | null;
    const key = saved || "date";
    sortCache[type] = key;
    return key;
  }

  function setSortBy(type: string, key: SortKey) {
    sortCache[type] = key;
    localStorage.setItem(SORT_STORAGE_PREFIX + type, key);
    sortVersion++;
  }

  function sortLinks(items: Link[], type: string): Link[] {
    // Read sortVersion to create reactive dependency
    void sortVersion;
    const sortKey = getSortKey(type);
    const sorted = [...items];
    switch (sortKey) {
      case "name":
        sorted.sort((a, b) => a.title.localeCompare(b.title));
        break;
      case "status": {
        sorted.sort((a, b) => {
          const sa = (a.metadata as any)?.status ?? "";
          const sb = (b.metadata as any)?.status ?? "";
          return sa.localeCompare(sb) || a.title.localeCompare(b.title);
        });
        break;
      }
      case "date":
      default:
        sorted.sort((a, b) => b.created_at.localeCompare(a.created_at));
        break;
    }
    return sorted;
  }

  function getSortOptions(_type: string): [string, string][] {
    return [["date", "Date"], ["name", "Name"]];
  }

  let groupedLinks = $derived.by(() => {
    const groups: Record<string, Link[]> = {};
    for (const link of links) {
      const type = link.link_type;
      if (!groups[type]) groups[type] = [];
      groups[type].push(link);
    }
    const order = ["github-pr", "github-issue", "github", "gitlab", "linear", "figma", "notion", "slite", "google-doc", "gdocs", "slack", "discord", "trello", "stackoverflow", "other"];
    const sorted: [string, Link[]][] = [];
    for (const type of order) {
      if (groups[type]) {
        sorted.push([type, sortLinks(groups[type], type)]);
        delete groups[type];
      }
    }
    for (const [type, grpLinks] of Object.entries(groups)) {
      sorted.push([type, sortLinks(grpLinks, type)]);
    }
    return sorted;
  });

  async function readClipboard() {
    try {
      const text = await navigator.clipboard.readText();
      if (text && (text.startsWith("http://") || text.startsWith("https://"))) {
        clipboardText = text;
      } else {
        clipboardText = "";
      }
    } catch {
      clipboardText = "";
    }
  }

  function handleQuickFocus() { readClipboard(); }
  function handleQuickBlur() { clipboardText = ""; }

  function handleQuickKeydown(e: KeyboardEvent) {
    if (e.key === "Tab" && !quickUrl && clipboardText) {
      e.preventDefault();
      quickUrl = clipboardText;
    } else if (e.key === "Enter") {
      e.preventDefault();
      submitQuickLink();
    } else if (e.key === "Escape") {
      quickUrl = "";
      inputEl?.blur();
    }
  }

  async function handleQuickPaste(e: ClipboardEvent) {
    const text = (e.clipboardData?.getData("text/plain") ?? e.clipboardData?.getData("text") ?? "").trim();
    if (text.startsWith("http://") || text.startsWith("https://")) {
      e.preventDefault();
      submitQuickLink(text);
    }
  }

  async function submitQuickLink(urlOverride?: string) {
    const url = (urlOverride ?? quickUrl).trim();
    if (!url) return;

    const linkType = getLinkTypeFromUrl(url);
    let title = url;
    try {
      const u = new URL(url);
      title = u.hostname.replace("www.", "") + (u.pathname !== "/" ? u.pathname : "");
    } catch {}

    quickUrl = "";
    clipboardText = "";
    try {
      await addLink(featureId, title, url, linkType);
      onLinksChanged?.();
    } catch (e) {
      console.error("Failed to add link:", e);
    }
  }

  function resetEditForm() {
    formTitle = "";
    formUrl = "";
    formDescription = "";
    showEditForm = false;
    editingLink = null;
  }

  async function handleEditSubmit() {
    const url = formUrl.trim();
    const title = formTitle.trim();
    if (!url) return;
    const linkType = getLinkTypeFromUrl(url);
    try {
      if (editingLink) {
        await updateLink(editingLink.id, title || url, url, linkType, formDescription.trim() || null);
      }
      resetEditForm();
      onLinksChanged?.();
    } catch (e) {
      console.error("Failed to save link:", e);
    }
  }

  async function handleDelete(link: Link) {
    try {
      await deleteLink(link.id);
      onLinksChanged?.();
    } catch (e) {
      console.error("Failed to delete link:", e);
    }
  }

  function handleEdit(link: Link) {
    editingLink = link;
    formTitle = link.title;
    formUrl = link.url;
    formDescription = link.description ?? "";
    showEditForm = true;
  }

  function handleEditKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") handleEditSubmit();
    if (e.key === "Escape") resetEditForm();
  }

  // Register toolbar actions
  $effect(() => {
    setToolbarActions("links", [
      {
        id: "add-link",
        label: "Add Link",
        icon: '<svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor"><path d="M8 1a1 1 0 011 1v5h5a1 1 0 110 2H9v5a1 1 0 11-2 0V9H2a1 1 0 010-2h5V2a1 1 0 011-1z"/></svg>',
        onClick: () => inputEl?.focus(),
        title: "Focus the URL input to add a link",
      },
    ]);
  });
  onDestroy(() => clearToolbarActions("links"));
</script>

<div>
  <div class="link-add-bar">
    <svg width="14" height="14" viewBox="0 0 16 16" fill="var(--text-muted)" style="flex-shrink: 0;">
      <path d="M6.354 5.5H4a3 3 0 000 6h3a3 3 0 002.83-4H9a2 2 0 01-2 2H4a2 2 0 110-4h2.354zM9.646 10.5H12a3 3 0 000-6H9a3 3 0 00-2.83 4H7a2 2 0 012-2h3a2 2 0 110 4H9.646z"/>
    </svg>
    <input
      bind:this={inputEl}
      type="url"
      class="link-add-input"
      placeholder={clipboardText ? `Tab to paste: ${clipboardText}` : "Paste a link..."}
      bind:value={quickUrl}
      onfocus={handleQuickFocus}
      onblur={handleQuickBlur}
      onkeydown={handleQuickKeydown}
      onpaste={handleQuickPaste}
    />
  </div>

  <div class="link-sections-grid">
  {#each groupedLinks as [type, typeLinks] (type)}
    {@const info = getLinkTypeInfo(type)}
    <div class="link-section">
      <div class="link-section-header">
        <div style="display: flex; align-items: center; gap: 6px;">
          <svg width="13" height="13" viewBox="0 0 16 16" fill={info.color}><path d={info.icon}/></svg>
          <span class="link-section-title">{info.label}</span>
          <span class="link-section-count">{typeLinks.length}</span>
        </div>
        {#if typeLinks.length > 1}
          <div style="display: flex; align-items: center; gap: 3px;">
            {#each getSortOptions(type) as [key, label]}
              <button
                class="btn-ghost link-sort-btn{getSortKey(type) === key ? ' link-sort-btn--active' : ''}"
                onclick={() => setSortBy(type, key as SortKey)}
              >{label}</button>
            {/each}
          </div>
        {/if}
      </div>
      <div class="links-grid">
        {#each typeLinks as link (link.id)}
          <LinkCard {link} onDelete={handleDelete} onEdit={handleEdit} />
        {/each}
      </div>
    </div>
  {/each}
  </div>

  {#if showEditForm}
    <div class="card" style="margin-top: 8px;">
      <div style="font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; color: var(--text-muted); margin-bottom: 10px;">
        Edit Link
      </div>
      <input
        type="text"
        placeholder="Title (optional)"
        class="form-input"
        style="font-size: 13px; margin-bottom: 8px;"
        bind:value={formTitle}
        onkeydown={handleEditKeydown}
      />
      <input
        type="url"
        placeholder="URL"
        class="form-input"
        style="font-size: 13px; margin-bottom: 8px;"
        bind:value={formUrl}
        onkeydown={handleEditKeydown}
      />
      <input
        type="text"
        placeholder="Description (e.g. Epic ticket, Design doc)"
        class="form-input"
        style="font-size: 13px; margin-bottom: 10px;"
        bind:value={formDescription}
        onkeydown={handleEditKeydown}
      />
      <div style="display: flex; gap: 8px;">
        <button class="btn-accent" style="font-size: 12px; padding: 5px 14px;" onclick={handleEditSubmit}>
          Save
        </button>
        <button class="btn-subtle" onclick={resetEditForm}>
          Cancel
        </button>
      </div>
    </div>
  {/if}
</div>
