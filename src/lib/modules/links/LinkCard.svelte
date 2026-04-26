<script lang="ts">
  import type { Link } from "../../api/tauri";
  import { open as shellOpen } from "@tauri-apps/plugin-shell";

  let {
    link,
    onDelete,
    onEdit,
  }: {
    link: Link;
    onDelete: (link: Link) => void;
    onEdit: (link: Link) => void;
  } = $props();

  let hovered = $state(false);
  let copied = $state(false);

  async function copyUrl() {
    try {
      await navigator.clipboard.writeText(link.url);
      copied = true;
      setTimeout(() => (copied = false), 1500);
    } catch (e) {
      console.error("Failed to copy URL:", e);
    }
  }

  function truncateUrl(url: string, max = 50): string {
    try {
      const u = new URL(url);
      const display = u.hostname + u.pathname;
      return display.length > max ? display.slice(0, max) + "..." : display;
    } catch {
      return url.length > max ? url.slice(0, max) + "..." : url;
    }
  }

  async function openUrl() {
    try {
      await shellOpen(link.url);
    } catch (e) {
      console.error("Failed to open URL:", e);
    }
  }
</script>

<div
  class="link-row link-card glass-panel glass-panel--hover"
  onmouseenter={() => (hovered = true)}
  onmouseleave={() => (hovered = false)}
  onclick={openUrl}
  ondblclick={(e) => { e.stopPropagation(); onEdit(link); }}
  role="button"
  tabindex="0"
  onkeydown={(e) => { if (e.key === 'Enter') openUrl(); }}
>
  <span class="link-row-title link-card__title">{link.title}</span>
  {#if link.description}
    <span class="link-row-desc">{link.description}</span>
  {/if}
  <span class="link-row-url link-card__url">{truncateUrl(link.url)}</span>
  <div class="link-row-actions link-card__actions link-card__meta">
    <button class="btn-ghost btn btn--icon btn--ghost btn--sm link-copy-btn" class:visible={hovered || copied} onclick={(e) => { e.stopPropagation(); copyUrl(); }} aria-label="Copy link">
      {#if copied}
        <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor" style="color: var(--green);"><path d="M13.5 2.5l-7 7L3 6l-1.5 1.5 5 5 8.5-8.5z"/></svg>
      {:else}
        <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor"><path d="M10 2H4a2 2 0 00-2 2v6h2V4h6V2zm4 4H6a2 2 0 00-2 2v6a2 2 0 002 2h8a2 2 0 002-2V8a2 2 0 00-2-2zm0 8H6V8h8v6z"/></svg>
      {/if}
    </button>
    {#if hovered}
      <button class="btn-ghost btn btn--icon btn--ghost btn--sm" onclick={(e) => { e.stopPropagation(); onEdit(link); }} aria-label="Edit">
        <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor"><path d="M12.1 1.3a1 1 0 011.4 0l1.2 1.2a1 1 0 010 1.4L5.8 12.8l-3.5.9.9-3.5z"/></svg>
      </button>
      <button class="btn-ghost btn btn--icon btn--ghost btn--sm" style="color: var(--red);" onclick={(e) => { e.stopPropagation(); onDelete(link); }} aria-label="Delete">
        <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor"><path d="M4.5 3.1L8 6.6l3.5-3.5 1.4 1.4L9.4 8l3.5 3.5-1.4 1.4L8 9.4l-3.5 3.5-1.4-1.4L6.6 8 3.1 4.5z"/></svg>
      </button>
    {:else}
      <span style="color: var(--text-muted); font-size: 11px; flex-shrink: 0;">&#8599;</span>
    {/if}
  </div>
</div>
