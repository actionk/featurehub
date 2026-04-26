<script lang="ts">
  import type { StorageInfo } from "../api/tauri";
  import {
    getStorages,
    switchStorage,
    pickStorageFolder,
    createStorage,
    removeStorage,
    renameStorage,
  } from "../api/tauri";
  import ExportImportModal from "./ExportImportModal.svelte";

  let {
    activeStorage,
    onSwitch,
  }: {
    activeStorage: StorageInfo;
    onSwitch: () => void;
  } = $props();

  let exportImportMode = $state<"export" | "import" | null>(null);
  let renamingId = $state<string | null>(null);
  let renameValue = $state("");
  let busyMessage = $state<string | null>(null);

  let open = $state(false);
  let storages = $state<StorageInfo[]>([]);

  const gitTooltip: Record<string, string> = {
    clean: "Git: clean, up to date",
    dirty: "Git: uncommitted or unpushed changes",
    none: "Not a git repository",
  };

  async function loadStorages() {
    try {
      storages = await getStorages();
    } catch (e) {
      console.error("Failed to load storages:", e);
    }
  }

  function toggle() {
    if (!open) loadStorages();
    open = !open;
  }

  function handleClickOutside(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest(".storage-selector")) {
      open = false;
    }
  }

  $effect(() => {
    if (open) {
      document.addEventListener("click", handleClickOutside, true);
      return () => document.removeEventListener("click", handleClickOutside, true);
    }
  });

  async function handleSwitch(e: MouseEvent, id: string) {
    const target = e.target as HTMLElement;
    if (target.closest(".storage-item-actions")) return;

    if (id === activeStorage.id) {
      open = false;
      return;
    }
    try {
      await switchStorage(id);
      open = false;
      onSwitch();
    } catch (e) {
      console.error("Failed to switch storage:", e);
      alert("Failed to switch storage: " + e);
    }
  }

  async function handleAdd() {
    try {
      const folder = await pickStorageFolder();
      if (!folder) return;
      await createStorage(folder);
      open = false;
      onSwitch();
    } catch (e) {
      console.error("Failed to add storage:", e);
    }
  }

  function startRename(e: Event, s: StorageInfo) {
    e.stopPropagation();
    renamingId = s.id;
    const parts = s.path.replace(/\\/g, "/").split("/");
    renameValue = parts[parts.length - 1] || s.name;
  }

  async function commitRename(s: StorageInfo) {
    if (!renameValue.trim()) {
      renamingId = null;
      return;
    }
    const oldParts = s.path.replace(/\\/g, "/").split("/");
    const oldName = oldParts[oldParts.length - 1];
    if (renameValue.trim() === oldName) {
      renamingId = null;
      return;
    }
    const parentParts = oldParts.slice(0, -1);
    const sep = s.path.includes("\\") ? "\\" : "/";
    const newPath = [...parentParts, renameValue.trim()].join(sep);
    try {
      renamingId = null;
      open = false;
      busyMessage = `Renaming storage to "${renameValue.trim()}"...`;
      await renameStorage(s.id, newPath);
      busyMessage = null;
      onSwitch();
    } catch (err) {
      busyMessage = null;
      console.error("Failed to rename storage:", err);
      alert("Failed to rename storage: " + err);
    }
  }

  async function handleMove(e: Event, s: StorageInfo) {
    e.stopPropagation();
    try {
      const folder = await pickStorageFolder();
      if (!folder) return;
      const parts = s.path.replace(/\\/g, "/").split("/");
      const folderName = parts[parts.length - 1];
      const sep = folder.includes("\\") ? "\\" : "/";
      const newPath = folder + sep + folderName;
      open = false;
      busyMessage = "Moving storage...";
      await renameStorage(s.id, newPath);
      busyMessage = null;
      onSwitch();
    } catch (err) {
      busyMessage = null;
      console.error("Failed to move storage:", err);
      alert("Failed to move storage: " + err);
    }
  }

  async function handleRemove(e: Event, id: string) {
    e.stopPropagation();
    if (!confirm("Remove this storage from the list? (Files won't be deleted)")) return;
    try {
      await removeStorage(id);
      open = false;
      onSwitch();
    } catch (e) {
      console.error("Failed to remove storage:", e);
    }
  }
</script>

<div class="storage-selector">
  <button class="storage-selector-btn" onclick={toggle} title={activeStorage.path}>
    {#if activeStorage.icon?.startsWith("data:")}
      <img src={activeStorage.icon} alt="" class="storage-selector-icon-img" />
    {:else if activeStorage.icon}
      <span class="storage-selector-icon">{activeStorage.icon}</span>
    {:else}
      <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor" opacity="0.5">
        <path d="M2 2h5l1 1h6v10H2V2zm1 1v9h10V4H7.5L6.5 3H3z"/>
      </svg>
    {/if}
    <span class="storage-selector-name">{activeStorage.name}</span>
    <span
      class="git-dot git-dot--{activeStorage.git_status}"
      title={gitTooltip[activeStorage.git_status]}
    ></span>
    <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor" opacity="0.4" style="margin-left: auto;">
      <path d="M4 6l4 4 4-4"/>
    </svg>
  </button>

  {#if open}
    <div class="storage-dropdown">
      {#each storages as s, i (s.id)}
        <button
          class="storage-dropdown-item glass-panel glass-panel--hover storage-row {s.is_active ? 'storage-dropdown-item--active list-row--active' : ''}"
          onclick={(e) => handleSwitch(e, s.id)}
          title={s.path}
        >
          <span class="storage-dropdown-icon">
            {#if s.icon?.startsWith("data:")}
              <img src={s.icon} alt="" class="storage-dropdown-icon-img" />
            {:else if s.icon}
              {s.icon}
            {:else}
              <span class="git-dot git-dot--{s.git_status}" title={gitTooltip[s.git_status]}></span>
            {/if}
          </span>
          {#if renamingId === s.id}
            <!-- svelte-ignore a11y_autofocus -->
            <input
              class="storage-rename-input input"
              type="text"
              bind:value={renameValue}
              autofocus
              onclick={(e) => e.stopPropagation()}
              onkeydown={(e) => {
                if (e.key === 'Enter') commitRename(s);
                if (e.key === 'Escape') { renamingId = null; }
              }}
              onblur={() => commitRename(s)}
            />
          {:else}
            <span class="storage-dropdown-label">
              <span class="storage-dropdown-name storage-row__name">{s.name}</span>
              <span class="storage-dropdown-path storage-row__path">{s.path}</span>
            </span>
          {/if}
          {#if i < 9}
            <kbd class="storage-shortcut">Shift+{i + 1}</kbd>
          {/if}
          <span class="storage-item-actions">
            <span
              class="storage-action-btn"
              role="button"
              tabindex="0"
              onclick={(e) => startRename(e, s)}
              onkeydown={(e) => { if (e.key === 'Enter') startRename(e, s); }}
              title="Rename storage"
            >
              <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor">
                <path d="M11.013 1.427a1.75 1.75 0 012.474 0l1.086 1.086a1.75 1.75 0 010 2.474l-8.61 8.61c-.21.21-.47.364-.756.445l-3.251.93a.75.75 0 01-.927-.928l.929-3.25a1.75 1.75 0 01.445-.758l8.61-8.61zm1.414 1.06a.25.25 0 00-.354 0L3.463 11.1l-.584 2.042 2.042-.584L14.427 3.9a.25.25 0 000-.354l-1.086-1.086-.914.027z"/>
              </svg>
            </span>
            <span
              class="storage-action-btn"
              role="button"
              tabindex="0"
              onclick={(e) => handleMove(e, s)}
              onkeydown={(e) => { if (e.key === 'Enter') handleMove(e, s); }}
              title="Move storage to another location"
            >
              <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor">
                <path d="M2 2h5l1 1h6v10H2V2zm1 1v9h10V4H7.5L6.5 3H3z"/>
              </svg>
            </span>
            {#if !s.is_active}
              <span
                class="storage-action-btn storage-remove-btn"
                role="button"
                tabindex="0"
                onclick={(e) => handleRemove(e, s.id)}
                onkeydown={(e) => { if (e.key === 'Enter') handleRemove(e, s.id); }}
                title="Remove storage"
              >
                <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor">
                  <path d="M3.72 3.72a.75.75 0 011.06 0L8 6.94l3.22-3.22a.75.75 0 111.06 1.06L9.06 8l3.22 3.22a.75.75 0 11-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 01-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 010-1.06z"/>
                </svg>
              </span>
            {/if}
          </span>
        </button>
      {/each}

      <div style="border-top: 1px solid var(--border-color); margin-top: 4px; padding-top: 4px;">
        <button class="storage-dropdown-item" onclick={handleAdd}>
          <svg width="12" height="12" viewBox="0 0 16 16" fill="none" style="flex-shrink: 0;">
            <path d="M8 2v12M2 8h12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
          <span>Add Storage</span>
        </button>
        <button class="storage-dropdown-item" onclick={() => { open = false; exportImportMode = "export"; }}>
          <svg width="12" height="12" viewBox="0 0 16 16" fill="none" style="flex-shrink: 0;">
            <path d="M8 2v8M4 7l4 4 4-4M2 13h12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          <span>Export Storage</span>
        </button>
        <button class="storage-dropdown-item" onclick={() => { open = false; exportImportMode = "import"; }}>
          <svg width="12" height="12" viewBox="0 0 16 16" fill="none" style="flex-shrink: 0;">
            <path d="M8 14V6M4 9l4-4 4 4M2 3h12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          <span>Import Storage</span>
        </button>
      </div>
    </div>
  {/if}
</div>

{#if busyMessage}
  <div class="storage-busy-overlay">
    <div class="storage-busy-card">
      <svg class="storage-busy-spinner" width="20" height="20" viewBox="0 0 20 20">
        <circle cx="10" cy="10" r="8" fill="none" stroke="var(--text-muted)" stroke-width="2" stroke-dasharray="40 60" stroke-linecap="round"/>
      </svg>
      <span>{busyMessage}</span>
    </div>
  </div>
{/if}

{#if exportImportMode}
  <ExportImportModal
    mode={exportImportMode}
    onClose={() => exportImportMode = null}
    onImported={() => { exportImportMode = null; onSwitch(); }}
  />
{/if}
