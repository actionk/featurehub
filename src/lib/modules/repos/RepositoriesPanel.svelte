<script lang="ts">
  import type { TabContext } from "../registry";
  import type { Directory, Repository, GitStatusSummary, DetectedIde } from "../../api/tauri";
  import {
    getFeature, cloneRepository, retryClone, removeDirectory, openPath,
    getGitStatus, listGitBranches, checkoutGitBranch, gitFetch, createGitBranch,
    detectIdes, openInIde,
  } from "../../api/tauri";
  import { getCachedSettings } from "../../stores/settings.svelte";
  import { setToolbarActions, clearToolbarActions } from "../../stores/tabToolbar.svelte";
  import { onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { createLatestGuard, createPollingGate } from "../../utils/asyncGuards";

  let {
    featureId,
    onOpenSettings,
    isTabActive = true,
  }: TabContext = $props();

  let directories = $state<Directory[]>([]);
  let defaultRepos = $state<Repository[]>([]);
  let installedIdes = $state<DetectedIde[]>([]);
  let idesDetected = false;
  let loading = $state(true);

  // Git status per directory
  let statusByDir = $state<Record<string, GitStatusSummary>>({});
  let loadingGitStatus = $state<Set<string>>(new Set());
  const dataGuard = createLatestGuard();
  const gitStatusGate = createPollingGate();

  // Add repo form
  let showAddForm = $state(false);
  let addRepoUrl = $state("");
  let addRepoName = $state("");
  let addError = $state<string | null>(null);

  // Branch switcher
  let branchDropdownDir = $state<string | null>(null);
  let branchList = $state<string[]>([]);
  let branchFilter = $state("");
  let branchLoading = $state(false);
  let branchError = $state<string | null>(null);
  let switchingBranch = $state(false);
  let dropdownPos = $state<{ top: number; left: number } | null>(null);
  let newBranchName = $state("");
  let creatingBranch = $state(false);

  // Copy feedback
  let copiedPathDir = $state<string | null>(null);
  let copiedUrlDir = $state<string | null>(null);

  // Fetch state
  let fetchingDir = $state<string | null>(null);

  // Search
  let repoSearch = $state("");

  let suggestedRepos = $derived(
    defaultRepos.filter((r) => !directories.some((dir) => dir.repo_url === r.url))
  );

  let filteredDirectories = $derived.by(() => {
    if (!repoSearch.trim()) return directories;
    const q = repoSearch.trim().toLowerCase();
    return directories.filter(dir => getDirName(dir).toLowerCase().includes(q));
  });

  let filteredSuggested = $derived.by(() => {
    if (!repoSearch.trim()) return suggestedRepos;
    const q = repoSearch.trim().toLowerCase();
    return suggestedRepos.filter(repo => {
      const name = repo.name ?? repo.url.split("/").pop()?.replace(".git", "") ?? repo.url;
      return name.toLowerCase().includes(q);
    });
  });

  $effect(() => {
    const id = featureId;
    if (!isTabActive) {
      dataGuard.invalidate();
      return;
    }
    loadData(id, true);
  });

  $effect(() => {
    if (!isTabActive) return;
    const unlistenComplete = listen<string>("clone-complete", () => refresh());
    const unlistenFailed = listen<any>("clone-failed", () => refresh());
    return () => {
      unlistenComplete.then((f) => f());
      unlistenFailed.then((f) => f());
    };
  });

  // Poll git status every 10s to catch external branch/file changes
  $effect(() => {
    if (!isTabActive) return;
    const interval = setInterval(() => {
      if (directories.length > 0) refreshGitStatuses();
    }, 10_000);
    return () => clearInterval(interval);
  });

  async function loadData(fid: string, showLoading = false) {
    const token = dataGuard.next();
    if (showLoading) loading = true;
    try {
      // Phase 1: fast — get feature + settings (show UI immediately)
      const [feat, settings] = await Promise.all([
        getFeature(fid),
        getCachedSettings(),
      ]);
      if (!dataGuard.isCurrent(token) || fid !== featureId || !isTabActive) return;
      directories = feat.directories ?? [];
      defaultRepos = settings.default_repositories ?? [];
      loading = false;

      // Phase 2: async — detect IDEs (only once) and git statuses (can be slow)
      if (!idesDetected) {
        detectIdes().then((ides) => {
          if (!dataGuard.isCurrent(token) || fid !== featureId) return;
          const preferred: string[] = settings.preferred_ides ?? [];
          installedIdes = preferred.length > 0
            ? ides.filter((ide: DetectedIde) => preferred.includes(ide.id))
            : ides;
          idesDetected = true;
        }).catch(() => {});
      }

      refreshGitStatuses(true);
    } catch (e) {
      console.error("Failed to load config:", e);
      if (dataGuard.isCurrent(token) && fid === featureId) {
        loading = false;
      }
    }
  }

  function refreshGitStatuses(showLoading = false) {
    if (!isTabActive) return;
    const fid = featureId;
    const readyDirs = directories.filter(d => d.clone_status === "ready" || !d.clone_status);
    void gitStatusGate.run(async () => {
      if (showLoading) {
        statusByDir = {};
        loadingGitStatus = new Set(readyDirs.map(d => d.id));
      }
      await Promise.all(readyDirs.map(async (dir) => {
        try {
          const status = await getGitStatus(dir.path);
          if (fid !== featureId || !isTabActive) return;
          statusByDir = { ...statusByDir, [dir.id]: status };
        } catch {
          // not a git repo
        } finally {
          if (fid !== featureId || !isTabActive) return;
          const next = new Set(loadingGitStatus);
          next.delete(dir.id);
          loadingGitStatus = next;
        }
      }));
    });
  }

  async function refresh() {
    await loadData(featureId, false);
  }

  function getDirName(dir: Directory): string {
    if (dir.label) return dir.label;
    if (dir.repo_url) {
      return dir.repo_url.split("/").pop()?.replace(".git", "") ?? dir.repo_url;
    }
    return dir.path.split(/[\\/]/).pop() ?? dir.path;
  }

  function getChangeSummary(s: GitStatusSummary): string | null {
    const parts: string[] = [];
    if (s.staged > 0) parts.push(`${s.staged} staged`);
    if (s.modified > 0) parts.push(`${s.modified} modified`);
    if (s.untracked > 0) parts.push(`${s.untracked} new`);
    return parts.length > 0 ? parts.join(", ") : null;
  }

  function getTotalChanges(s: GitStatusSummary): number {
    return s.staged + s.modified + s.untracked;
  }

  function getAheadBehindText(s: GitStatusSummary): string | null {
    if (s.ahead === null && s.behind === null) return null;
    const parts: string[] = [];
    if (s.ahead && s.ahead > 0) parts.push(`↑${s.ahead}`);
    if (s.behind && s.behind > 0) parts.push(`↓${s.behind}`);
    return parts.length > 0 ? parts.join(" ") : null;
  }

  async function handleCloneRepo(url: string, name?: string | null) {
    addError = null;
    try {
      await cloneRepository(featureId, url, name);
      showAddForm = false;
      addRepoUrl = "";
      addRepoName = "";
      await refresh();
    } catch (e) {
      addError = String(e);
    }
  }

  async function handleAddFromForm() {
    const url = addRepoUrl.trim();
    if (!url) { addError = "Repository URL is required"; return; }
    await handleCloneRepo(url, addRepoName.trim() || null);
  }

  async function handleRetryClone(dirId: string) {
    try {
      await retryClone(dirId);
      await refresh();
    } catch (e) {
      console.error("Failed to retry clone:", e);
    }
  }

  async function handleFetch(dir: Directory) {
    if (fetchingDir) return;
    fetchingDir = dir.id;
    try {
      await gitFetch(dir.path);
      // Refresh git status to update ahead/behind
      const status = await getGitStatus(dir.path);
      statusByDir = { ...statusByDir, [dir.id]: status };
    } catch (e) {
      console.error("Failed to fetch:", e);
    } finally {
      fetchingDir = null;
    }
  }

  // Remove confirmation
  let removeConfirmDir = $state<Directory | null>(null);
  let removeConfirmDirty = $state(false);
  let removeConfirmChanges = $state(0);

  async function promptRemoveDirectory(dir: Directory) {
    // Check for uncommitted changes
    const status = statusByDir[dir.id];
    if (status) {
      const total = status.modified + status.untracked + status.staged;
      removeConfirmDirty = total > 0;
      removeConfirmChanges = total;
    } else {
      removeConfirmDirty = false;
      removeConfirmChanges = 0;
    }
    removeConfirmDir = dir;
  }

  async function confirmRemoveDirectory() {
    if (!removeConfirmDir) return;
    const id = removeConfirmDir.id;
    removeConfirmDir = null;
    try {
      await removeDirectory(id);
      await refresh();
    } catch (e) {
      console.error("Failed to remove directory:", e);
    }
  }

  async function openBranchSwitcher(dir: Directory, event: MouseEvent) {
    if (branchDropdownDir === dir.id) {
      branchDropdownDir = null;
      return;
    }
    // Position dropdown below the clicked button
    const btn = (event.currentTarget as HTMLElement);
    const rect = btn.getBoundingClientRect();
    dropdownPos = { top: rect.bottom + 4, left: rect.left };

    branchDropdownDir = dir.id;
    branchFilter = "";
    branchError = null;
    branchLoading = true;
    branchList = [];
    newBranchName = "";
    try {
      branchList = await listGitBranches(dir.path);
    } catch (e) {
      branchError = String(e);
    } finally {
      branchLoading = false;
    }
  }

  function filteredBranches(currentBranch: string): string[] {
    let filtered = branchList.filter(b => b !== currentBranch);
    if (branchFilter.trim()) {
      const q = branchFilter.trim().toLowerCase();
      filtered = filtered.filter(b => b.toLowerCase().includes(q));
    }
    return filtered;
  }

  async function handleSwitchBranch(dir: Directory, branchName: string) {
    branchError = null;
    switchingBranch = true;
    try {
      await checkoutGitBranch(dir.path, branchName);
      branchDropdownDir = null;
      await refresh();
    } catch (e) {
      branchError = String(e);
    } finally {
      switchingBranch = false;
    }
  }

  async function handleCreateBranch(dir: Directory) {
    const name = newBranchName.trim();
    if (!name) return;
    branchError = null;
    creatingBranch = true;
    try {
      await createGitBranch(dir.path, name);
      branchDropdownDir = null;
      await refresh();
    } catch (e) {
      branchError = String(e);
    } finally {
      creatingBranch = false;
    }
  }

  function closeBranchDropdown() {
    branchDropdownDir = null;
  }

  // Close dropdown on click outside or scroll
  $effect(() => {
    if (!branchDropdownDir) return;
    function handleClick(e: MouseEvent) {
      const target = e.target as HTMLElement;
      if (!target.closest(".branch-dropdown") && !target.closest(".repo-tile-branch")) {
        closeBranchDropdown();
      }
    }
    function handleScroll() {
      closeBranchDropdown();
    }
    // Use setTimeout to skip the opening click
    const timer = setTimeout(() => document.addEventListener("click", handleClick), 0);
    window.addEventListener("scroll", handleScroll, true);
    return () => {
      clearTimeout(timer);
      document.removeEventListener("click", handleClick);
      window.removeEventListener("scroll", handleScroll, true);
    };
  });

  // Register toolbar actions
  $effect(() => {
    setToolbarActions("repos", [
      {
        id: "clone-repo",
        label: "Clone",
        icon: '<svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor"><path d="M8 1a1 1 0 011 1v5h5a1 1 0 110 2H9v5a1 1 0 11-2 0V9H2a1 1 0 010-2h5V2a1 1 0 011-1z"/></svg>',
        onClick: () => { showAddForm = !showAddForm; },
        title: "Clone a repository",
      },
    ]);
  });
  onDestroy(() => clearToolbarActions("repos"));
</script>

{#if loading}
  <div style="text-align: center; padding: 16px; font-size: 12px; color: var(--text-muted);">Loading...</div>
{:else}
  <div class="fs-section">
    <div class="fs-section-header">
      <div class="fs-section-label">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="var(--purple)"><path fill-rule="evenodd" d="M2 2.5A2.5 2.5 0 014.5 0h8.75a.75.75 0 01.75.75v12.5a.75.75 0 01-.75.75h-2.5a.75.75 0 110-1.5h1.75v-2h-8a1 1 0 00-.714 1.7.75.75 0 01-1.072 1.05A2.495 2.495 0 012 11.5v-9zm10.5-1h-8a1 1 0 00-1 1v6.708A2.486 2.486 0 014.5 9h8.5V1.5zM5 12.25v3.25a.25.25 0 00.4.2l1.45-1.087a.25.25 0 01.3 0L8.6 15.7a.25.25 0 00.4-.2v-3.25a.25.25 0 00-.25-.25h-3.5a.25.25 0 00-.25.25z"/></svg>
        Repositories
      </div>
      <button class="btn-add btn btn--primary btn--sm" onclick={() => { showAddForm = !showAddForm; addError = null; }}>+ Clone</button>
    </div>

    {#if directories.length + suggestedRepos.length > 3}
      <div class="repo-search-row">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="var(--text-muted)" opacity="0.6"><path d="M11.5 7a4.5 4.5 0 1 1-9 0 4.5 4.5 0 0 1 9 0zm-.82 4.74a6 6 0 1 1 1.06-1.06l3.04 3.04a.75.75 0 1 1-1.06 1.06l-3.04-3.04z"/></svg>
        <input
          class="repo-search-input"
          type="text"
          placeholder="Search repositories..."
          bind:value={repoSearch}
          onkeydown={(e) => { if (e.key === 'Escape') { repoSearch = ''; (e.target as HTMLElement).blur(); } }}
        />
        {#if repoSearch}
          <button class="repo-search-clear" onclick={() => repoSearch = ''} aria-label="Clear search">
            <svg width="10" height="10" viewBox="0 0 16 16"><path d="M4.5 3.5l7 7m0-7l-7 7" stroke="currentColor" stroke-width="2" stroke-linecap="round" fill="none"/></svg>
          </button>
        {/if}
      </div>
    {/if}

    {#if showAddForm}
      <div class="repo-add-form">
        <div class="repo-add-row">
          <input
            class="repo-add-input"
            type="text"
            placeholder="https://github.com/org/repo.git"
            bind:value={addRepoUrl}
            onkeydown={(e) => { if (e.key === 'Enter') handleAddFromForm(); if (e.key === 'Escape') showAddForm = false; }}
            style="flex: 1;"
          />
          <input
            class="repo-add-input"
            type="text"
            placeholder="Name (optional)"
            bind:value={addRepoName}
            onkeydown={(e) => { if (e.key === 'Enter') handleAddFromForm(); if (e.key === 'Escape') showAddForm = false; }}
            style="width: 120px;"
          />
          <button class="repo-add-btn btn btn--primary btn--sm" onclick={handleAddFromForm}>Clone</button>
          <button class="repo-add-cancel" onclick={() => showAddForm = false} title="Cancel">
            <svg width="10" height="10" viewBox="0 0 16 16"><path d="M4.5 3.5l7 7m0-7l-7 7" stroke="currentColor" stroke-width="2" stroke-linecap="round" fill="none"/></svg>
          </button>
        </div>
        {#if addError}
          <div class="repo-add-error">{addError}</div>
        {/if}
      </div>
    {/if}

    {#if directories.length === 0 && suggestedRepos.length === 0 && !showAddForm}
      <div class="fs-empty" onclick={() => showAddForm = true} role="button" tabindex="0"
        onkeydown={(e) => e.key === 'Enter' && (showAddForm = true)}>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="var(--text-muted)" opacity="0.5"><path fill-rule="evenodd" d="M2 2.5A2.5 2.5 0 014.5 0h8.75a.75.75 0 01.75.75v12.5a.75.75 0 01-.75.75h-2.5a.75.75 0 110-1.5h1.75v-2h-8a1 1 0 00-.714 1.7.75.75 0 01-1.072 1.05A2.495 2.495 0 012 11.5v-9zm10.5-1h-8a1 1 0 00-1 1v6.708A2.486 2.486 0 014.5 9h8.5V1.5zM5 12.25v3.25a.25.25 0 00.4.2l1.45-1.087a.25.25 0 01.3 0L8.6 15.7a.25.25 0 00.4-.2v-3.25a.25.25 0 00-.25-.25h-3.5a.25.25 0 00-.25.25z"/></svg>
        Click to clone a repository
        <span class="repo-settings-hint">
          Add default repositories in
          <button class="repo-settings-link" onclick={(e) => { e.stopPropagation(); onOpenSettings?.('directories'); }}>Settings</button>
          to see them as quick-clone suggestions here.
        </span>
      </div>
    {:else}
      <div class="repo-grid">
        {#each filteredDirectories as dir (dir.id)}
          {@const status = dir.clone_status ?? "ready"}
          {@const gitStatus = statusByDir[dir.id]}
          {@const changes = gitStatus ? getTotalChanges(gitStatus) : 0}
          {@const changeSummary = gitStatus ? getChangeSummary(gitStatus) : null}
          {#if status === "cloning"}
            <div class="repo-tile repo-tile--cloning repo-row glass-panel">
              <button class="repo-tile-remove" title="Cancel clone" onclick={(e) => { e.stopPropagation(); promptRemoveDirectory(dir); }}>
                <svg width="10" height="10" viewBox="0 0 16 16"><path d="M4.5 3.5l7 7m0-7l-7 7" stroke="currentColor" stroke-width="2" stroke-linecap="round" fill="none"/></svg>
              </button>
              <div class="repo-tile-icon repo-tile-icon--amber">
                <svg width="18" height="18" viewBox="0 0 16 16" fill="var(--amber)" style="animation: spin 1s linear infinite;"><path d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1zm0 2a5 5 0 1 1 0 10 5 5 0 0 1 0-10z" opacity="0.3"/><path d="M8 1a7 7 0 0 1 7 7h-2a5 5 0 0 0-5-5V1z"/></svg>
              </div>
              <span class="repo-tile-name repo-row__name">{getDirName(dir)}</span>
              <span class="repo-tile-status-text aurora-pill aurora-pill--warn aurora-pill--sm" style="color: var(--amber);">Cloning...</span>
            </div>
          {:else if status === "failed"}
            <div class="repo-tile repo-tile--failed repo-row glass-panel">
              <button class="repo-tile-remove" title="Remove repository" onclick={(e) => { e.stopPropagation(); promptRemoveDirectory(dir); }}>
                <svg width="10" height="10" viewBox="0 0 16 16"><path d="M4.5 3.5l7 7m0-7l-7 7" stroke="currentColor" stroke-width="2" stroke-linecap="round" fill="none"/></svg>
              </button>
              <div class="repo-tile-icon repo-tile-icon--red">
                <svg width="18" height="18" viewBox="0 0 16 16" fill="var(--red)"><path d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1zm1 11H7v-2h2v2zm0-4H7V4h2v4z"/></svg>
              </div>
              <span class="repo-tile-name repo-row__name">{getDirName(dir)}</span>
              <span class="repo-tile-status-text aurora-pill aurora-pill--danger aurora-pill--sm" style="color: var(--red);">Clone failed</span>
              {#if dir.clone_error}
                <span class="repo-tile-error" title={dir.clone_error}>{dir.clone_error}</span>
              {/if}
              <button class="repo-tile-retry btn btn--sm" onclick={() => handleRetryClone(dir.id)}>Retry</button>
            </div>
          {:else}
            <!-- Ready -->
            <div class="repo-tile repo-row glass-panel">
              <button class="repo-tile-copy-path" title="Copy local path" onclick={(e) => { e.stopPropagation(); navigator.clipboard.writeText(dir.path); copiedPathDir = dir.id; setTimeout(() => { if (copiedPathDir === dir.id) copiedPathDir = null; }, 1500); }}>
                {#if copiedPathDir === dir.id}
                  <svg width="10" height="10" viewBox="0 0 16 16" fill="var(--green)"><path d="M13.3 4.3L6 11.6 2.7 8.3l1.4-1.4L6 8.8l5.9-5.9z"/></svg>
                {:else}
                  <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M4 2a2 2 0 00-2 2v6h2V4h6V2H4zm3 3a2 2 0 00-2 2v6a2 2 0 002 2h5a2 2 0 002-2V7a2 2 0 00-2-2H7z"/></svg>
                {/if}
              </button>
              {#if dir.repo_url}
                <button class="repo-tile-copy-url" title="Copy remote URL" onclick={(e) => { e.stopPropagation(); navigator.clipboard.writeText(dir.repo_url!); copiedUrlDir = dir.id; setTimeout(() => { if (copiedUrlDir === dir.id) copiedUrlDir = null; }, 1500); }}>
                  {#if copiedUrlDir === dir.id}
                    <svg width="10" height="10" viewBox="0 0 16 16" fill="var(--green)"><path d="M13.3 4.3L6 11.6 2.7 8.3l1.4-1.4L6 8.8l5.9-5.9z"/></svg>
                  {:else}
                    <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M4.715 6.542L3.343 7.914a3 3 0 104.243 4.243l1.828-1.829A3 3 0 008.586 5.5L8 6.086a1 1 0 00-.154.199 2 2 0 01.861 3.337L6.88 11.45a2 2 0 11-2.83-2.83l.793-.792a4 4 0 01-.128-1.287z"/><path d="M6.586 4.672A3 3 0 007.414 9.5l.586-.586a1 1 0 00.154-.199 2 2 0 01-.861-3.337L9.12 3.55a2 2 0 112.83 2.83l-.793.792c.112.42.155.855.128 1.287l1.372-1.372a3 3 0 10-4.243-4.243L6.586 4.672z"/></svg>
                  {/if}
                </button>
              {/if}
              <button class="repo-tile-remove" title="Remove repository" onclick={(e) => { e.stopPropagation(); promptRemoveDirectory(dir); }}>
                <svg width="10" height="10" viewBox="0 0 16 16"><path d="M4.5 3.5l7 7m0-7l-7 7" stroke="currentColor" stroke-width="2" stroke-linecap="round" fill="none"/></svg>
              </button>
              {#if dir.repo_url}
                <button class="repo-tile-fetch" title="Fetch updates from remote" disabled={fetchingDir === dir.id}
                  onclick={(e) => { e.stopPropagation(); handleFetch(dir); }}>
                  {#if fetchingDir === dir.id}
                    <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor" style="animation: spin 1s linear infinite;"><path d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1zm0 2a5 5 0 1 1 0 10 5 5 0 0 1 0-10z" opacity="0.3"/><path d="M8 1a7 7 0 0 1 7 7h-2a5 5 0 0 0-5-5V1z"/></svg>
                  {:else}
                    <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M8 2a6 6 0 1 0 6 6h-1.5A4.5 4.5 0 1 1 8 3.5V2z"/><path d="M8 0v5l3-2.5L8 0z"/></svg>
                  {/if}
                </button>
              {/if}
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div class="repo-tile-top repo-row__head" onclick={() => openPath(dir.path)} onkeydown={(e) => { if (e.key === 'Enter') openPath(dir.path); }} role="button" tabindex="0" title={dir.path}>
                <div class="repo-tile-icon">
                  <svg width="18" height="18" viewBox="0 0 16 16" fill="var(--purple)" opacity="0.7"><path fill-rule="evenodd" d="M2 2.5A2.5 2.5 0 014.5 0h8.75a.75.75 0 01.75.75v12.5a.75.75 0 01-.75.75h-2.5a.75.75 0 110-1.5h1.75v-2h-8a1 1 0 00-.714 1.7.75.75 0 01-1.072 1.05A2.495 2.495 0 012 11.5v-9zm10.5-1h-8a1 1 0 00-1 1v6.708A2.486 2.486 0 014.5 9h8.5V1.5zM5 12.25v3.25a.25.25 0 00.4.2l1.45-1.087a.25.25 0 01.3 0L8.6 15.7a.25.25 0 00.4-.2v-3.25a.25.25 0 00-.25-.25h-3.5a.25.25 0 00-.25.25z"/></svg>
                </div>
                <span class="repo-tile-name repo-row__name">{getDirName(dir)}</span>
                <span class="aurora-pill aurora-pill--success aurora-pill--sm aurora-pill--no-dot" style="display: none;">Cloned</span>
              </div>
              {#if gitStatus}
                {@const aheadBehind = getAheadBehindText(gitStatus)}
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <button class="repo-tile-branch list-row list-row--active" class:repo-tile-branch--dirty={changes > 0}
                  onclick={(e) => { e.stopPropagation(); openBranchSwitcher(dir, e); }}
                  title="Switch branch">
                  <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M5 3v5.17a3 3 0 1 0 2 0V7a1 1 0 0 1 1-1h1a3 3 0 0 0 3-3V2h-2v1a1 1 0 0 1-1 1H9a3 3 0 0 0-2.83 2H5V3H3v3h2V5z"/><circle cx="6" cy="12" r="1.5"/></svg>
                  <span>{gitStatus.branch}</span>
                  <svg width="8" height="8" viewBox="0 0 16 16" fill="currentColor" opacity="0.5" style="margin-left: auto;"><path d="M4 6l4 4 4-4z"/></svg>
                </button>
                {#if aheadBehind}
                  <span class="repo-tile-sync" title="{gitStatus.ahead ?? 0} ahead, {gitStatus.behind ?? 0} behind remote">
                    {aheadBehind}
                  </span>
                {/if}
                {#if changeSummary}
                  <span class="repo-tile-changes" title={changeSummary}>
                    {changes} change{changes !== 1 ? "s" : ""}
                  </span>
                {:else}
                  <span class="repo-tile-clean">Clean</span>
                {/if}
              {:else if loadingGitStatus.has(dir.id)}
                <span class="repo-tile-status-loading">
                  <span class="repo-tile-status-dot"></span>
                  <span class="repo-tile-status-dot"></span>
                  <span class="repo-tile-status-dot"></span>
                </span>
              {/if}
              {#if installedIdes.length > 0}
                <div class="repo-tile-ides">
                  {#each installedIdes as ide}
                    <button class="repo-tile-ide-btn btn btn--sm" title="Open in {ide.name}"
                      onclick={(e) => { e.stopPropagation(); openInIde(dir.path, ide.command); }}>
                      {#if ide.id === "vscode"}
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M17.583 2.603L12.2 7.386 7.847 4.15l-.87.507v14.71l.87.508 5.384-3.252-4.354-3.236L17.583 21.4l.87-.507V3.11l-.87-.507zM7.847 14.86V9.163l3.508 2.849-3.508 2.849z"/></svg>
                      {:else if ide.id === "vscode-insiders"}
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M17.583 2.603L12.2 7.386 7.847 4.15l-.87.507v14.71l.87.508 5.384-3.252-4.354-3.236L17.583 21.4l.87-.507V3.11l-.87-.507zM7.847 14.86V9.163l3.508 2.849-3.508 2.849z"/></svg>
                      {:else if ide.id === "cursor"}
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/></svg>
                      {:else}
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><rect x="3" y="3" width="18" height="18" rx="3" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M7 7h4v4H7zM13 7h4v4h-4zM7 13h4v4H7z" opacity="0.7"/></svg>
                      {/if}
                      <span class="repo-tile-ide-label">{ide.name}</span>
                    </button>
                  {/each}
                </div>
              {/if}
            </div>
          {/if}
        {/each}

        {#if filteredSuggested.length > 0}
          <button class="repo-grid-divider" onclick={() => onOpenSettings?.('directories')} title="Edit default repositories in Settings">
            <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor" opacity="0.4"><path d="M7.775 3.275a.75.75 0 001.06 1.06l1.25-1.25a2 2 0 112.83 2.83l-2.5 2.5a2 2 0 01-2.83 0 .75.75 0 00-1.06 1.06 3.5 3.5 0 004.95 0l2.5-2.5a3.5 3.5 0 00-4.95-4.95l-1.25 1.25zm-4.69 9.64a2 2 0 010-2.83l2.5-2.5a2 2 0 012.83 0 .75.75 0 001.06-1.06 3.5 3.5 0 00-4.95 0l-2.5 2.5a3.5 3.5 0 004.95 4.95l1.25-1.25a.75.75 0 00-1.06-1.06l-1.25 1.25a2 2 0 01-2.83 0z"/></svg>
            <span>From Settings</span>
            <svg width="8" height="8" viewBox="0 0 16 16" fill="currentColor" opacity="0.3" style="margin-left: 2px;"><path d="M3.75 2h3.5a.75.75 0 010 1.5H4.5v8h8V8.75a.75.75 0 011.5 0v3.5A1.75 1.75 0 0112.25 14h-8.5A1.75 1.75 0 012 12.25v-8.5C2 2.784 2.784 2 3.75 2zm6.72 0h2.78a.75.75 0 01.75.75v2.78a.75.75 0 11-1.5 0V4.06L8.78 7.78a.75.75 0 01-1.06-1.06L11.44 3H10.47a.75.75 0 010-1.5z" fill-rule="evenodd"/></svg>
          </button>
          {#each filteredSuggested as repo}
            <button class="repo-tile repo-tile--suggestion" onclick={() => handleCloneRepo(repo.url, repo.name)} title="Clone {repo.url}">
              <div class="repo-tile-icon repo-tile-icon--ghost">
                <svg width="18" height="18" viewBox="0 0 16 16" fill="currentColor" opacity="0.35"><path d="M8 3v10M3 8h10" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/></svg>
              </div>
              <span class="repo-tile-name">{repo.name ?? repo.url.split("/").pop()?.replace(".git", "") ?? repo.url}</span>
              {#if repo.description}
                <span class="repo-tile-desc">{repo.description}</span>
              {/if}
            </button>
          {/each}
        {/if}
      </div>
      {#if repoSearch && filteredDirectories.length === 0 && filteredSuggested.length === 0}
        <div style="text-align: center; padding: 12px; font-size: 12px; color: var(--text-muted);">
          No repositories matching "{repoSearch}"
        </div>
      {/if}
    {/if}
  </div>
{/if}

{#if branchDropdownDir && dropdownPos}
  {@const dir = directories.find(d => d.id === branchDropdownDir)}
  {@const gitStatus = dir ? statusByDir[dir.id] : null}
  {#if dir && gitStatus}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="branch-dropdown" style="top: {dropdownPos.top}px; left: {dropdownPos.left}px;"
      onclick={(e) => e.stopPropagation()} onkeydown={(e) => { if (e.key === 'Escape') closeBranchDropdown(); }}>
      <input class="branch-dropdown-filter" type="text" placeholder="Filter branches..."
        bind:value={branchFilter}
        onkeydown={(e) => { if (e.key === 'Escape') closeBranchDropdown(); }} />
      {#if branchLoading}
        <div class="branch-dropdown-empty">Loading...</div>
      {:else if branchError}
        <div class="branch-dropdown-empty" style="color: var(--red);">{branchError}</div>
      {:else}
        <div class="branch-dropdown-list">
          {#each filteredBranches(gitStatus.branch) as branch}
            <button class="branch-dropdown-item list-row" disabled={switchingBranch}
              onclick={() => handleSwitchBranch(dir, branch)}>
              {branch}
            </button>
          {:else}
            <div class="branch-dropdown-empty">No other branches</div>
          {/each}
        </div>
      {/if}
      <div class="branch-dropdown-create">
        <input
          class="branch-dropdown-create-input"
          type="text"
          placeholder="New branch name..."
          bind:value={newBranchName}
          onkeydown={(e) => { if (e.key === 'Enter' && newBranchName.trim()) handleCreateBranch(dir); if (e.key === 'Escape') closeBranchDropdown(); }}
        />
        <button
          class="branch-dropdown-create-btn btn btn--primary btn--sm"
          disabled={!newBranchName.trim() || creatingBranch}
          onclick={() => handleCreateBranch(dir)}
        >
          {creatingBranch ? "..." : "Create"}
        </button>
      </div>
    </div>
  {/if}
{/if}

{#if removeConfirmDir}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal-backdrop" onclick={(e) => { if (e.target === e.currentTarget) removeConfirmDir = null; }}
    onkeydown={(e) => { if (e.key === 'Escape') removeConfirmDir = null; }}>
    <div class="modal-content" style="width: 380px;">
      <h2 style="font-size: 16px; font-weight: 700; letter-spacing: -0.02em; color: var(--text-primary); margin-bottom: 12px;">
        Remove Repository
      </h2>
      <p style="font-size: 13px; color: var(--text-secondary); margin-bottom: 16px; line-height: 1.5;">
        Remove <strong style="color: var(--text-primary);">{getDirName(removeConfirmDir)}</strong> and delete the cloned files from disk?
      </p>
      {#if removeConfirmDirty}
        <div style="font-size: 12px; color: var(--amber); padding: 8px 10px; background: color-mix(in srgb, var(--amber) 8%, transparent); border-radius: 6px; border: 1px solid color-mix(in srgb, var(--amber) 20%, transparent); margin-bottom: 16px; line-height: 1.4;">
          This repository has {removeConfirmChanges} uncommitted change{removeConfirmChanges !== 1 ? "s" : ""}. They will be lost.
        </div>
      {/if}
      <div style="display: flex; justify-content: flex-end; gap: 8px;">
        <button class="btn-subtle" style="padding: 7px 16px;" onclick={() => removeConfirmDir = null}>Cancel</button>
        <button class="btn-new" style="width: auto; padding: 7px 18px; background: var(--red); border-color: var(--red); color: #fff;"
          onclick={confirmRemoveDirectory}>Remove</button>
      </div>
    </div>
  </div>
{/if}

<style>
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .repo-search-row {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 8px;
    padding: 4px 8px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--bg-input);
    transition: border-color 0.15s;
  }

  .repo-search-row:focus-within {
    border-color: var(--purple);
  }

  .repo-search-input {
    flex: 1;
    font-size: 12px;
    border: none;
    background: none;
    color: var(--text-primary);
    outline: none;
    padding: 2px 0;
  }

  .repo-search-input::placeholder {
    color: var(--text-muted);
    opacity: 0.6;
  }

  .repo-search-clear {
    background: none;
    border: none;
    padding: 2px;
    cursor: pointer;
    color: var(--text-muted);
    display: flex;
    border-radius: 3px;
  }

  .repo-search-clear:hover {
    color: var(--text-primary);
  }

  .repo-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 6px;
  }

  .repo-tile {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 5px;
    padding: 14px 12px 12px;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 8px;
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
    text-align: center;
  }

  .repo-tile:hover {
    border-color: var(--border-focus);
    background: var(--bg-hover);
  }

  .repo-tile-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 7px;
    background: color-mix(in srgb, var(--purple) 10%, transparent);
  }

  .repo-tile-icon--amber {
    background: color-mix(in srgb, var(--amber) 10%, transparent);
  }

  .repo-tile-icon--red {
    background: color-mix(in srgb, var(--red) 10%, transparent);
  }

  .repo-tile-icon--ghost {
    background: none;
  }

  .repo-tile-name {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
    transition: color 0.15s;
  }

  .repo-tile:hover .repo-tile-name {
    color: var(--text-primary);
  }

  .repo-tile-top {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 5px;
    cursor: pointer;
  }

  .repo-tile-branch {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    font-weight: 500;
    font-family: var(--font-mono);
    color: var(--green);
    padding: 2px 9px;
    border-radius: 10px;
    border: 1px solid transparent;
    background: color-mix(in srgb, var(--green) 10%, transparent);
    max-width: 100%;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
  }

  .repo-tile-branch:hover {
    border-color: color-mix(in srgb, var(--green) 30%, transparent);
    background: color-mix(in srgb, var(--green) 16%, transparent);
  }

  .repo-tile-branch svg {
    flex-shrink: 0;
  }

  .repo-tile-branch--dirty {
    color: var(--amber);
    background: color-mix(in srgb, var(--amber) 10%, transparent);
  }

  .repo-tile-branch--dirty:hover {
    border-color: color-mix(in srgb, var(--amber) 30%, transparent);
    background: color-mix(in srgb, var(--amber) 16%, transparent);
  }

  /* Branch dropdown */
  .branch-dropdown {
    position: fixed;
    z-index: 999;
    min-width: 260px;
    max-width: 380px;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 8px 24px rgba(0,0,0,0.3);
    overflow: hidden;
  }

  .branch-dropdown-filter {
    width: 100%;
    padding: 7px 10px;
    font-size: 12px;
    border: none;
    border-bottom: 1px solid var(--border);
    background: transparent;
    color: var(--text-primary);
    outline: none;
    box-sizing: border-box;
  }

  .branch-dropdown-list {
    max-height: 200px;
    overflow-y: auto;
  }

  .branch-dropdown-item {
    display: block;
    width: 100%;
    padding: 6px 10px;
    font-size: 12px;
    font-family: var(--font-mono);
    color: var(--text-secondary);
    background: none;
    border: none;
    text-align: left;
    cursor: pointer;
    transition: background 0.1s, color 0.1s;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .branch-dropdown-item:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .branch-dropdown-item:disabled {
    opacity: 0.5;
    cursor: wait;
  }

  .branch-dropdown-empty {
    padding: 8px 10px;
    font-size: 11px;
    color: var(--text-muted);
  }

  .branch-dropdown-create {
    display: flex;
    gap: 6px;
    padding: 8px 10px;
    border-top: 1px solid var(--border);
  }

  .branch-dropdown-create-input {
    flex: 1;
    padding: 4px 8px;
    font-size: 11px;
    font-family: var(--font-mono);
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-input);
    color: var(--text-primary);
    outline: none;
    transition: border-color 0.15s;
  }

  .branch-dropdown-create-input:focus {
    border-color: var(--purple);
  }

  .branch-dropdown-create-btn {
    padding: 4px 10px;
    font-size: 11px;
    font-weight: 500;
    border: 1px solid var(--purple);
    border-radius: 4px;
    background: var(--purple-dim);
    color: var(--purple);
    cursor: pointer;
    transition: background 0.15s;
  }

  .branch-dropdown-create-btn:hover:not(:disabled) {
    background: color-mix(in srgb, var(--purple) 20%, transparent);
  }

  .branch-dropdown-create-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .repo-tile-changes {
    font-size: 10px;
    color: var(--amber);
    font-weight: 500;
  }

  .repo-tile-clean {
    font-size: 10px;
    color: var(--text-muted);
    opacity: 0.6;
  }

  .repo-tile-sync {
    font-size: 10px;
    font-weight: 500;
    color: var(--blue);
    font-family: var(--font-mono);
  }

  .repo-tile-status-text {
    font-size: 10px;
    font-weight: 500;
  }

  .repo-tile-error {
    font-size: 10px;
    color: var(--text-muted);
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding: 0 8px;
  }

  .repo-tile-retry {
    font-size: 10px;
    padding: 2px 10px;
    border-radius: 4px;
    border: 1px solid var(--border);
    background: none;
    color: var(--text-secondary);
    cursor: pointer;
    transition: border-color 0.15s, color 0.15s;
  }

  .repo-tile-retry:hover {
    border-color: var(--text-muted);
    color: var(--text-primary);
  }

  .repo-tile-remove {
    position: absolute;
    top: 5px;
    right: 5px;
    background: none;
    border: none;
    padding: 3px;
    cursor: pointer;
    color: var(--text-muted);
    opacity: 0;
    transition: opacity 0.15s, color 0.15s;
    display: flex;
    border-radius: 3px;
  }

  .repo-tile:hover .repo-tile-remove {
    opacity: 0.4;
  }

  .repo-tile-remove:hover {
    opacity: 1 !important;
    color: var(--red);
    background: color-mix(in srgb, var(--red) 10%, transparent);
  }

  .repo-tile-copy-path {
    position: absolute;
    top: 5px;
    right: 59px;
    background: none;
    border: none;
    padding: 3px;
    cursor: pointer;
    color: var(--text-muted);
    opacity: 0;
    transition: opacity 0.15s, color 0.15s;
    display: flex;
    border-radius: 3px;
  }

  .repo-tile:hover .repo-tile-copy-path {
    opacity: 0.4;
  }

  .repo-tile-copy-path:hover {
    opacity: 1 !important;
    color: var(--text-primary);
    background: color-mix(in srgb, var(--text-muted) 10%, transparent);
  }

  .repo-tile-copy-url {
    position: absolute;
    top: 5px;
    right: 41px;
    background: none;
    border: none;
    padding: 3px;
    cursor: pointer;
    color: var(--text-muted);
    opacity: 0;
    transition: opacity 0.15s, color 0.15s;
    display: flex;
    border-radius: 3px;
  }

  .repo-tile:hover .repo-tile-copy-url {
    opacity: 0.4;
  }

  .repo-tile-copy-url:hover {
    opacity: 1 !important;
    color: var(--blue);
    background: color-mix(in srgb, var(--blue) 10%, transparent);
  }

  .repo-tile-fetch {
    position: absolute;
    top: 5px;
    right: 23px;
    background: none;
    border: none;
    padding: 3px;
    cursor: pointer;
    color: var(--text-muted);
    opacity: 0;
    transition: opacity 0.15s, color 0.15s;
    display: flex;
    border-radius: 3px;
  }

  .repo-tile:hover .repo-tile-fetch {
    opacity: 0.4;
  }

  .repo-tile-fetch:hover {
    opacity: 1 !important;
    color: var(--green);
    background: color-mix(in srgb, var(--green) 10%, transparent);
  }

  .repo-tile-fetch:disabled {
    opacity: 1 !important;
    cursor: wait;
    color: var(--amber);
  }

  /* IDE buttons */
  .repo-tile-ides {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
    justify-content: center;
    margin-top: 2px;
  }

  .repo-tile-ide-btn {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 2px 7px;
    font-size: 10px;
    border-radius: 4px;
    border: 1px solid var(--border);
    background: none;
    color: var(--text-muted);
    cursor: pointer;
    transition: border-color 0.15s, color 0.15s, background 0.15s;
    white-space: nowrap;
  }

  .repo-tile-ide-btn:hover {
    border-color: var(--blue);
    color: var(--blue);
    background: color-mix(in srgb, var(--blue) 8%, transparent);
  }

  .repo-tile-ide-btn svg {
    flex-shrink: 0;
  }

  .repo-tile-ide-label {
    font-weight: 500;
  }

  .repo-settings-hint {
    display: block;
    font-size: 11px;
    color: var(--text-muted);
    opacity: 0.6;
    margin-top: 4px;
    font-weight: 400;
  }

  .repo-settings-link {
    background: none;
    border: none;
    padding: 0;
    color: var(--purple);
    cursor: pointer;
    font-size: inherit;
    font-weight: 500;
    text-decoration: underline;
    text-decoration-style: dotted;
    text-underline-offset: 2px;
  }

  .repo-settings-link:hover {
    color: var(--text-primary);
  }

  .repo-grid-divider {
    grid-column: 1 / -1;
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
    opacity: 0.7;
    padding: 4px 2px 0;
    background: none;
    border: none;
    cursor: pointer;
    transition: opacity 0.15s;
  }

  .repo-grid-divider:hover {
    opacity: 1;
  }

  .repo-tile-desc {
    font-size: 10px;
    color: var(--text-muted);
    opacity: 0.7;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }

  /* Suggestion tiles */
  .repo-tile--suggestion {
    border-style: dashed;
    background: none;
  }

  .repo-tile--suggestion .repo-tile-name {
    color: var(--text-muted);
    font-weight: 500;
  }

  .repo-tile--suggestion:hover {
    border-color: var(--purple);
    border-style: solid;
    background: var(--purple-dim);
  }

  .repo-tile--suggestion:hover .repo-tile-name {
    color: var(--purple);
  }

  /* Add form */
  .repo-add-form {
    margin-bottom: 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .repo-add-row {
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .repo-add-input {
    font-size: 12px;
    padding: 5px 8px;
    border-radius: 5px;
    border: 1px solid var(--border);
    background: var(--bg-input);
    color: var(--text-primary);
    outline: none;
    transition: border-color 0.15s;
  }

  .repo-add-input:focus {
    border-color: var(--purple);
  }

  .repo-add-btn {
    font-size: 11px;
    padding: 5px 12px;
    border-radius: 5px;
    border: 1px solid var(--purple);
    background: var(--purple-dim);
    color: var(--purple);
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.15s;
  }

  .repo-add-btn:hover {
    background: color-mix(in srgb, var(--purple) 20%, transparent);
  }

  .repo-add-cancel {
    background: none;
    border: none;
    padding: 4px;
    cursor: pointer;
    color: var(--text-muted);
    display: flex;
    border-radius: 3px;
  }

  .repo-add-cancel:hover {
    color: var(--text-primary);
  }

  .repo-add-error {
    font-size: 11px;
    color: var(--red);
    padding: 4px 8px;
    background: var(--red-dim);
    border-radius: 4px;
    border: 1px solid rgba(229,72,77,0.15);
  }

  /* Git status loading dots */
  .repo-tile-status-loading {
    display: flex;
    gap: 3px;
    align-items: center;
    justify-content: center;
    height: 18px;
  }

  .repo-tile-status-dot {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--text-muted);
    opacity: 0.3;
    animation: statusPulse 1s ease-in-out infinite;
  }

  .repo-tile-status-dot:nth-child(2) { animation-delay: 0.15s; }
  .repo-tile-status-dot:nth-child(3) { animation-delay: 0.3s; }

  @keyframes statusPulse {
    0%, 100% { opacity: 0.15; }
    50% { opacity: 0.6; }
  }
</style>
