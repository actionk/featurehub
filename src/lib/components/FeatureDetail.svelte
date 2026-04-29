<script lang="ts">
  import DOMPurify from "dompurify";
  import { tick, onMount } from "svelte";
  import type { Feature, Tag, Session, Task, Plan, Note } from "../api/tauri";
  import {
    getFeature,
    getFeatureData,
    updateFeature,
    deleteFeature,
    createTag,
    toggleTag,
    getSessions,
  } from "../api/tauri";
  import { getFilesDirectory } from "../api/files";
  import Modal from "./ui/Modal.svelte";
  import TagBadge from "./TagBadge.svelte";
  import { getShowTabEmojis } from "../stores/settings.svelte";
  import { subscribe } from "../stores/events.svelte";
  import { getActiveCountForFeature } from "../stores/sessionActivity.svelte";
  import { getRegisteredTabs } from "../modules/registry";
  import type { TabContext } from "../modules/registry";
  import { getToolbarActions } from "../stores/tabToolbar.svelte";
  import "../modules"; // trigger module registration
  import { extensionTabsReady } from "../modules";
  import { formatRelativeTime } from "../utils/format";
  import { createLatestGuard } from "../utils/asyncGuards";

  let {
    featureId,
    isActive = true,
    onDeleted,
    onUpdated,
    onSessionsChanged,
    refreshFeatureId = null,
    onRefreshHandled,
    pendingPlanId = null,
    pendingPlanFeatureId = null,
    onPendingPlanHandled,
    initialTab = null,
    onInitialTabHandled,
    onOpenSettings,
  }: {
    featureId: string;
    isActive?: boolean;
    onDeleted?: () => void;
    onUpdated?: () => void;
    onSessionsChanged?: () => void;
    refreshFeatureId?: string | null;
    onRefreshHandled?: () => void;
    pendingPlanId?: string | null;
    pendingPlanFeatureId?: string | null;
    onPendingPlanHandled?: () => void;
    initialTab?: string | null;
    onInitialTabHandled?: () => void;
    onOpenSettings?: (tab?: string) => void;
  } = $props();

  let feature = $state<Feature | null>(null);
  let sessions = $state<Session[]>([]);
  let tasks = $state<Task[]>([]);
  let plans = $state<Plan[]>([]);
  let allTags = $state<Tag[]>([]);
  let note = $state<Note | null>(null);
  let loading = $state(true);
  let activeTab = $state("ai");
  let visitedTabs = $state<Set<string>>(new Set(["ai"]));
  let editingTitle = $state(false);
  let titleInput = $state("");
  let descriptionModalOpen = $state(false);
  let descriptionInput = $state("");
  let showTagPicker = $state(false);
  let newTagName = $state("");
  let tagInputEl: HTMLInputElement | undefined = $state();
  let showDeleteConfirm = $state(false);
  let copiedId = $state(false);
  let copiedPath = $state(false);
  const featureDataGuard = createLatestGuard();
  const sessionGuard = createLatestGuard();

  function switchTab(value: string) {
    activeTab = value;
    if (!visitedTabs.has(value)) {
      visitedTabs = new Set([...visitedTabs, value]);
    }
  }

  // Re-derive tabs after extension tabs are registered
  let _extensionsLoaded = $state(false);
  onMount(async () => {
    await extensionTabsReady;
    _extensionsLoaded = true;
  });

  let showEmojis = $derived(getShowTabEmojis());
  // eslint-disable-next-line @typescript-eslint/no-unused-expressions
  let tabs = $derived((_extensionsLoaded, getRegisteredTabs()));
  let activeSessionCount = $derived(getActiveCountForFeature(featureId));
  let tasksDone = $derived((tasks ?? []).filter(t => t.done).length);
  let pendingPlanCount = $derived((plans ?? []).filter(p => p.status === 'pending').length);
  let titleLastSpace = $derived(feature ? feature.title.lastIndexOf(' ') : -1);

  // Build TabContext for tab components
  let tabContext = $derived<TabContext>(feature ? {
    featureId: feature.id,
    feature,
    sessions,
    tasks,
    plans,
    note,
    allTags,
    activeSessionCount,
    pendingPlanId: pendingPlanFeatureId === featureId ? pendingPlanId : null,
    onPendingPlanHandled,
    onRefresh: refresh,
    onSessionsChanged: refreshSessions,
    onOpenSettings,
  } : null as any);

  // Persist active tab per feature
  $effect(() => {
    localStorage.setItem(`featurehub:tab:${featureId}`, activeTab);
  });

  // React to featureId changes (no more {#key} destroy/recreate)
  // Must run BEFORE the initialTab effect so that initialTab can override the persisted tab
  $effect(() => {
    const fid = featureId;
    // Reset UI state
    editingTitle = false;
    descriptionModalOpen = false;
    showTagPicker = false;
    showDeleteConfirm = false;
    copiedId = false;
    copiedPath = false;
    // Restore persisted tab
    const saved = localStorage.getItem(`featurehub:tab:${fid}`);
    const tab = saved || "ai";
    activeTab = tab;
    // Pre-render tabs marked as preload + the active tab
    const preloadIds = getRegisteredTabs().filter(t => t.preload).map(t => t.id);
    visitedTabs = new Set([tab, ...preloadIds]);
    loadFeatureData({ includeSessions: true });
  });

  // Apply initial tab override (e.g. from session panel click or search navigation).
  // Defined AFTER the featureId effect so it runs second and wins on fresh mount.
  $effect(() => {
    if (initialTab) {
      switchTab(initialTab);
      onInitialTabHandled?.();
    }
  });

  async function loadFeatureData({ includeSessions = false } = {}) {
    const fid = featureId;
    const token = featureDataGuard.next();
    loading = true;
    try {
      // Single IPC call for feature + tags + tasks + plans + note (one DB lock)
      const data = await getFeatureData(fid);
      if (!featureDataGuard.isCurrent(token) || fid !== featureId) return;
      feature = data.feature;
      allTags = data.all_tags;
      tasks = data.tasks;
      plans = data.plans;
      note = data.note;
    } catch (e) {
      console.error("Failed to load feature:", e);
    } finally {
      if (featureDataGuard.isCurrent(token) && fid === featureId) {
        loading = false;
      }
    }
    // Sessions loaded separately — they do expensive disk I/O for title scanning
    if (includeSessions && featureDataGuard.isCurrent(token) && fid === featureId) {
      refreshSessions(fid);
    }
  }

  async function refresh() {
    const fid = featureId;
    const token = featureDataGuard.next();
    try {
      const data = await getFeatureData(fid);
      if (!featureDataGuard.isCurrent(token) || fid !== featureId) return;
      feature = data.feature;
      allTags = data.all_tags;
      tasks = data.tasks;
      plans = data.plans;
      note = data.note;
    } catch {}
    onUpdated?.();
  }

  // Refresh when notified by parent (MCP notification)
  $effect(() => {
    if (refreshFeatureId === featureId) {
      loadFeatureData({ includeSessions: true });
      onRefreshHandled?.();
    }
  });

  // Instantly refresh sessions when any session is finished/unlinked
  $effect(() => subscribe("sessions:changed", () => refreshSessions()));

  // Poll for external changes (e.g. MCP adding links/tasks/plans)
  // Only poll when this tab is active to reduce IPC load
  $effect(() => {
    if (!isActive) return;
    const interval = setInterval(async () => {
      const fid = featureId;
      try {
        const fresh = await getFeature(fid);
        if (fid !== featureId) return;
        if (fresh.updated_at !== feature?.updated_at) {
          const token = featureDataGuard.next();
          const data = await getFeatureData(fid);
          if (!featureDataGuard.isCurrent(token) || fid !== featureId) return;
          feature = data.feature;
          allTags = data.all_tags;
          tasks = data.tasks;
          plans = data.plans;
          note = data.note;
          onUpdated?.();
        }
      } catch {}
    }, 5000);
    return () => clearInterval(interval);
  });

  async function refreshSessions(fid = featureId) {
    const token = sessionGuard.next();
    try {
      const freshSessions = await getSessions(fid);
      if (!sessionGuard.isCurrent(token) || fid !== featureId) return;
      sessions = freshSessions;
    } catch {
      if (!sessionGuard.isCurrent(token) || fid !== featureId) return;
      sessions = [];
    }
    if (fid === featureId) {
      onSessionsChanged?.();
    }
  }

  function toggleEditTitle() {
    if (!feature) return;
    if (editingTitle) {
      saveTitle();
    } else {
      titleInput = feature.title;
      editingTitle = true;
    }
  }

  async function saveTitle() {
    if (!feature) return;
    const t = titleInput.trim();
    if (t && t !== feature.title) {
      await updateFeature(feature.id, { title: t });
      await refresh();
    }
    editingTitle = false;
  }

  function handleTitleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") saveTitle();
    if (e.key === "Escape") editingTitle = false;
  }

  function startEditDescription() {
    if (!feature) return;
    descriptionInput = feature.description ?? "";
    descriptionModalOpen = true;
  }

  async function saveDescription() {
    if (!feature) return;
    const val = descriptionInput.trim() || null;
    if (val !== (feature.description ?? null)) {
      await updateFeature(feature.id, { description: val });
      await refresh();
    }
    descriptionModalOpen = false;
  }

  const tagColors = ["#7c5bf5", "#3b82f6", "#22c55e", "#f59e0b", "#ef4444", "#ec4899", "#14b8a6", "#f97316", "#8b5cf6", "#06b6d4"];

  async function handleCreateTag() {
    if (!feature) return;
    const name = newTagName.trim();
    if (!name) return;
    const color = tagColors[Math.floor(Math.random() * tagColors.length)];
    try {
      const tag = await createTag(name, color);
      await toggleTag(feature.id, tag.id);
      newTagName = "";
      showTagPicker = false;
      await refresh();
    } catch (e) {
      console.error("Failed to create tag:", e);
    }
  }

  async function handleToggleTag(tagId: string) {
    if (!feature) return;
    await toggleTag(feature.id, tagId);
    await refresh();
  }

  async function handleRemoveTag(tag: Tag) {
    const wasLast = featureTags.length === 1;
    await handleToggleTag(tag.id);
    if (wasLast) {
      showTagPicker = true;
      await tick();
      tagInputEl?.focus();
    }
  }

  async function handleDelete(cleanupRepos = false) {
    if (!feature) return;
    try { await deleteFeature(feature.id, cleanupRepos); onDeleted?.(); }
    catch (e) { console.error("Failed to delete feature:", e); }
  }

  let featureTags = $derived(feature?.tags ?? []);
  let availableTags = $derived(
    allTags.filter((t) => !featureTags.some((ft) => ft.id === t.id))
  );

  $effect(() => {
    if (showTagPicker) {
      let skipFirst = true;
      function handleClick(e: MouseEvent) {
        if (skipFirst) { skipFirst = false; return; }
        const target = e.target as HTMLElement;
        if (!target.closest(".tag-picker-wrapper")) {
          showTagPicker = false;
        }
      }
      document.addEventListener("click", handleClick, true);
      return () => document.removeEventListener("click", handleClick, true);
    }
  });

  // Tab keyboard shortcuts — generated from registry
  $effect(() => {
    function handleKeydown(e: KeyboardEvent) {
      // Don't trigger when typing in inputs/textareas
      const tag = (e.target as HTMLElement)?.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;
      if (e.ctrlKey || e.metaKey || e.altKey) return;

      const tab = tabs.find((t) => t.shortcutKey === e.key.toLowerCase());
      if (tab) {
        e.preventDefault();
        switchTab(tab.id);
      }
    }
    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  });
</script>

{#if loading && !feature}
  <div class="empty-state">
    <div class="empty-state-loader">
      <div class="skeleton-line" style="width: 60%; height: 20px;"></div>
      <div class="skeleton-line" style="width: 40%; height: 14px; opacity: 0.5;"></div>
      <div class="skeleton-line" style="width: 80%; height: 32px; margin-top: 12px;"></div>
    </div>
  </div>
{:else if !loading && !feature}
  <div class="empty-state">
    <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="var(--text-faint)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="12" cy="12" r="10"/><path d="M8 15h8M9 9h.01M15 9h.01"/>
    </svg>
    <div style="font-size: var(--text-base); color: var(--text-muted);">Feature not found</div>
  </div>
{:else if feature}
  <!-- Header — row 1 (title | chips + actions) + row 2 (status + tags + time + desc) -->
  <div class="detail-header">
    <!-- Row 1: title | chips + actions -->
    <div class="detail-row1">
      {#if editingTitle}
        <input type="text" class="form-input detail-title-input"
          bind:value={titleInput} onblur={saveTitle} onkeydown={handleTitleKeydown} />
      {:else}
        <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
        <h1 class="detail-header-title"
          ondblclick={toggleEditTitle} role="button" tabindex="0"
          onkeydown={(e) => e.key === 'Enter' && toggleEditTitle()} title="Double-click to edit">
          {#if titleLastSpace >= 0}{feature.title.slice(0, titleLastSpace + 1)}<span class="detail-title-last-word">{feature.title.slice(titleLastSpace + 1)}</span>{:else}<span class="detail-title-last-word">{feature.title}</span>{/if}
        </h1>
      {/if}

      <div class="detail-header-actions">
        {#if tasksDone > 0}
        <div class="detail-chip detail-chip--green">
          <svg width="10" height="10" viewBox="0 0 16 16" fill="var(--green)"><path d="M13.3 4.3L6 11.6 2.7 8.3l1.4-1.4L6 8.8l5.9-5.9z"/></svg>
          <div class="detail-chip-num">{tasksDone}</div>
          <div class="detail-chip-lbl">Done</div>
        </div>
        {/if}

        <div class="detail-chip" class:detail-chip--accent={activeSessionCount > 0}>
          <div class="detail-chip-dot" class:detail-chip-dot--on={activeSessionCount > 0}></div>
          <div class="detail-chip-num">{activeSessionCount}</div>
          <div class="detail-chip-lbl">Agents</div>
        </div>

        {#if pendingPlanCount > 0}
          <div class="detail-chip detail-chip--amber">
            <svg width="10" height="10" viewBox="0 0 16 16" fill="var(--amber)"><path d="M8 1L10 6H15L11 9.5L12.5 14.5L8 11.5L3.5 14.5L5 9.5L1 6H6Z"/></svg>
            <div class="detail-chip-num">{pendingPlanCount}</div>
            <div class="detail-chip-lbl">Plans</div>
          </div>
        {/if}

        <div class="detail-chip-sep"></div>

        <button class="btn-ghost" onclick={() => { if (feature) { navigator.clipboard.writeText(feature.id); copiedId = true; setTimeout(() => { copiedId = false; }, 1500); } }}
          title="Copy Feature ID">
          {#if copiedId}
            <svg width="13" height="13" viewBox="0 0 16 16" fill="var(--green)"><path d="M13.3 4.3L6 11.6 2.7 8.3l1.4-1.4L6 8.8l5.9-5.9z"/></svg>
          {:else}
            <svg width="13" height="13" viewBox="0 0 16 16" fill="currentColor"><path d="M4 2a2 2 0 00-2 2v6h2V4h6V2H4zm3 3a2 2 0 00-2 2v6a2 2 0 002 2h5a2 2 0 002-2V7a2 2 0 00-2-2H7z"/></svg>
          {/if}
        </button>
        <button class="btn-ghost" onclick={async () => { if (feature) { try { const p = await getFilesDirectory(feature.id); await navigator.clipboard.writeText(p); copiedPath = true; setTimeout(() => { copiedPath = false; }, 1500); } catch {} } }}
          title="Copy feature directory path">
          {#if copiedPath}
            <svg width="13" height="13" viewBox="0 0 16 16" fill="var(--green)"><path d="M13.3 4.3L6 11.6 2.7 8.3l1.4-1.4L6 8.8l5.9-5.9z"/></svg>
          {:else}
            <svg width="13" height="13" viewBox="0 0 16 16" fill="currentColor"><path d="M1.75 1A1.75 1.75 0 000 2.75v10.5C0 14.216.784 15 1.75 15h12.5A1.75 1.75 0 0016 13.25v-8.5A1.75 1.75 0 0014.25 3H7.5a.25.25 0 01-.2-.1l-.9-1.2c-.33-.44-.85-.7-1.4-.7H1.75z"/></svg>
          {/if}
        </button>
        <button class="btn-ghost" onclick={toggleEditTitle} title="Edit">
          <svg width="13" height="13" viewBox="0 0 16 16" fill="currentColor"><path d="M12.1 1.3a1 1 0 011.4 0l1.2 1.2a1 1 0 010 1.4L5.8 12.8l-3.5.9.9-3.5z"/></svg>
        </button>
        <button class="btn-ghost btn-ghost--danger" onclick={() => (showDeleteConfirm = true)}
          title="Delete">
          <svg width="13" height="13" viewBox="0 0 16 16" fill="currentColor"><path d="M5 2V1h6v1h4v1H1V2h4zm1 3v8h1V5H6zm3 0v8h1V5H9zM2 4l1 11h10l1-11H2z"/></svg>
        </button>
      </div>
    </div>

    <!-- Row 2: tags + time + desc -->
      <div class="detail-row2">
        {#each featureTags as tag (tag.id)}
          <TagBadge {tag} removable onRemove={handleRemoveTag} />
        {/each}
        <div class="tag-picker-wrapper" style="position: relative;">
          <button class="btn-add" onclick={async () => { showTagPicker = !showTagPicker; if (showTagPicker) { await tick(); tagInputEl?.focus(); } }}>+ Tag</button>
          {#if showTagPicker}
            <div class="dropdown">
              <div style="padding: 4px 6px; border-bottom: 1px solid var(--border);">
                <input
                  type="text"
                  class="form-input"
                  style="font-size: 12px; padding: 4px 8px;"
                  placeholder="New tag name..."
                  bind:this={tagInputEl}
                  bind:value={newTagName}
                  onkeydown={(e) => { if (e.key === 'Enter') handleCreateTag(); if (e.key === 'Escape') { showTagPicker = false; newTagName = ''; } }}
                />
              </div>
              {#each availableTags as tag (tag.id)}
                <button class="dropdown-item"
                  onclick={() => { handleToggleTag(tag.id); showTagPicker = false; }}>
                  <TagBadge {tag} />
                </button>
              {/each}
            </div>
          {/if}
        </div>

        {#if feature.ticket_id}
          <span class="detail-row2-sep">·</span>
          <button class="detail-ticket-id" onclick={() => { navigator.clipboard.writeText(feature.ticket_id!); }} title="Copy ticket ID">{feature.ticket_id}</button>
        {/if}

        <span class="detail-row2-sep">·</span>
        <span class="detail-row2-time">{formatRelativeTime(feature.updated_at)}</span>

        {#if feature.description}
          <span class="detail-row2-sep">·</span>
          <button class="detail-row2-desc" onclick={startEditDescription} title="Click to edit">{feature.description}</button>
        {/if}
      </div>
  </div>

  <!-- Tab bar — driven by module registry -->
  {@const toolbarActions = getToolbarActions(activeTab)}
  <div class="tab-bar">
    <div class="tab-bar-tabs">
      {#each tabs as tab (tab.id)}
        <button class="tab-btn tab {activeTab === tab.id ? 'tab-btn--active tab--active' : ''}" onclick={() => { switchTab(tab.id); }}
          title="{tab.label} ({tab.shortcutKey})">
          {showEmojis ? `${tab.emoji} ${tab.label}` : tab.label}
          {#each tab.getBadges(tabContext) as badge}
            <span class="tab-count tab__badge {badge.style === 'active' ? 'tab-count--active' : ''}"
              style="{badge.style === 'warning' ? 'background: var(--amber); color: #000;' : ''}"
              title={badge.title ?? ''}>{badge.text}</span>
          {/each}
          <span class="sk">{tab.shortcutKey}</span>
        </button>
      {/each}
    </div>
    {#if toolbarActions.length > 0}
      <div class="tab-toolbar">
        {#each toolbarActions as action (action.id)}
          <button
            class="tab-toolbar-btn {action.variant === 'primary' ? 'tab-toolbar-btn--primary' : ''}"
            onclick={action.onClick}
            disabled={action.disabled}
            title={action.title ?? ''}
          >
            {#if action.icon}
              {@html DOMPurify.sanitize(action.icon, { USE_PROFILES: { svg: true } })}
            {/if}
            {action.label}
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Tab panels wrapper — keep-alive: mount on first visit, hide with display:none -->
  <div style="flex: 1; min-height: 0; position: relative; overflow: hidden;">
    {#each tabs as tab (tab.id)}
      {#if visitedTabs.has(tab.id)}
        <div class="tab-panel" style="display: {activeTab === tab.id ? (tab.panelStyle ? '' : 'block') : 'none'}; {activeTab === tab.id && tab.panelStyle ? tab.panelStyle : ''}">
          <tab.component {...tabContext} isTabActive={isActive && activeTab === tab.id} {...(tab.extraProps ?? {})} />
        </div>
      {/if}
    {/each}
  </div>
{/if}

<Modal open={descriptionModalOpen} onClose={saveDescription} width="800px">
  <div style="padding: 20px; display: flex; flex-direction: column; gap: 12px; min-height: 400px;">
    <div style="display:flex;align-items:center;justify-content:space-between;">
      <span style="font-size:14px;font-weight:600;color:var(--text-primary)">Description</span>
      <button onclick={saveDescription} style="background:none;border:none;color:var(--text-muted);cursor:pointer;padding:4px;font-size:13px;">✕</button>
    </div>
    <textarea
      class="notes-textarea"
      style="flex: 1; min-height: 340px; resize: vertical;"
      bind:value={descriptionInput}
      onkeydown={(e) => { if (e.key === 'Escape') saveDescription(); }}
      placeholder="Short feature description..."
    ></textarea>
  </div>
</Modal>

{#if showDeleteConfirm && feature}
  {@const clonedRepos = (feature.directories ?? []).filter((d) => d.repo_url)}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal-backdrop" onclick={(e) => { if (e.target === e.currentTarget) showDeleteConfirm = false; }} onkeydown={(e) => { if (e.key === 'Escape') showDeleteConfirm = false; }}>
    <div class="modal-content" style="width: 420px;">
      <h2 class="modal-title">Delete Feature</h2>
      <p class="modal-body-text">
        Are you sure you want to delete <strong style="color: var(--text-primary);">{feature.title}</strong>? This will remove all associated tasks, notes, links, and AI sessions. This action cannot be undone.
      </p>
      {#if clonedRepos.length > 0}
        <p class="modal-hint-text">
          This feature has {clonedRepos.length} cloned repositor{clonedRepos.length === 1 ? "y" : "ies"} on disk.
        </p>
      {/if}
      <div class="modal-actions">
        <button class="btn-subtle" style="padding: 7px 16px;" onclick={() => (showDeleteConfirm = false)}>Cancel</button>
        {#if clonedRepos.length > 0}
          <button class="btn-subtle" style="padding: 7px 16px;" onclick={() => { showDeleteConfirm = false; handleDelete(false); }}>Keep Repos</button>
          <button class="btn-danger" onclick={() => { showDeleteConfirm = false; handleDelete(true); }}>Delete + Repos</button>
        {:else}
          <button class="btn-danger" onclick={() => { showDeleteConfirm = false; handleDelete(); }}>Delete</button>
        {/if}
      </div>
    </div>
  </div>
{/if}
