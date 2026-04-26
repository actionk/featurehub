<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import {
    exportStorage,
    cancelExport,
    checkImportZip,
    importStorage,
    restoreRepoFromExport,
    pickStorageFolder,
  } from "../api/tauri";
  import type { ImportResult, ImportCheckResult, RepoDirectory, ExportOptions } from "../api/tauri";

  let {
    mode,
    onClose,
    onImported,
  }: {
    mode: "export" | "import";
    onClose: () => void;
    onImported?: () => void;
  } = $props();

  let stage = $state("Preparing...");
  let percent = $state(0);
  let running = $state(false);
  let done = $state(false);
  let error = $state<string | null>(null);
  let resultPath = $state<string | null>(null);
  let importResult = $state<ImportResult | null>(null);
  let restoringRepo = $state<string | null>(null);

  // Import: duplicate-resolution step
  let checkResult = $state<ImportCheckResult | null>(null);
  let duplicateStrategy = $state<"replace" | "ignore" | "merge">("ignore");

  // Export options
  let includeDone = $state(true);
  let includeArchived = $state(false);
  let includeFiles = $state(true);
  let includeSessions = $state(true);
  let includeTasks = $state(true);
  let includeNotes = $state(true);
  let includeContext = $state(true);
  let includePatches = $state(true);

  let unlistenExport: (() => void) | null = null;
  let unlistenImport: (() => void) | null = null;

  async function setupListeners() {
    unlistenExport = await listen<{ stage: string; percent: number }>(
      "export-progress",
      (event) => {
        stage = event.payload.stage;
        percent = event.payload.percent;
      }
    );
    unlistenImport = await listen<{ stage: string; percent: number }>(
      "import-progress",
      (event) => {
        stage = event.payload.stage;
        percent = event.payload.percent;
      }
    );
  }

  function cleanup() {
    unlistenExport?.();
    unlistenImport?.();
  }

  async function handleExport() {
    running = true;
    error = null;
    await setupListeners();
    try {
      const opts: ExportOptions = {
        includeDone,
        includeArchived,
        includeFiles,
        includeSessions,
        includeTasks,
        includeNotes,
        includeContext,
        includePatches,
      };
      const path = await exportStorage(opts);
      resultPath = path;
      done = true;
    } catch (e: any) {
      if (typeof e === "string" && e.includes("cancelled")) {
        onClose();
        return;
      }
      error = typeof e === "string" ? e : e.message || "Export failed";
    } finally {
      running = false;
      cleanup();
    }
  }

  async function handleCheckImport() {
    running = true;
    error = null;
    try {
      const result = await checkImportZip();
      checkResult = result;
      if (result.duplicate_count === 0) {
        // No duplicates — go straight to import.
        await handleImport(result.zip_path, "ignore");
      }
      // Otherwise the UI shows the strategy picker.
    } catch (e: any) {
      if (typeof e === "string" && e.includes("cancelled")) {
        onClose();
        return;
      }
      error = typeof e === "string" ? e : e.message || "Failed to read archive";
    } finally {
      running = false;
    }
  }

  async function handleImport(zipPath: string, strategy: "replace" | "ignore" | "merge") {
    running = true;
    error = null;
    await setupListeners();
    try {
      const result = await importStorage(zipPath, strategy);
      importResult = result;
      done = true;
      checkResult = null;
      onImported?.();
    } catch (e: any) {
      if (typeof e === "string" && e.includes("cancelled")) {
        onClose();
        return;
      }
      error = typeof e === "string" ? e : e.message || "Import failed";
    } finally {
      running = false;
      cleanup();
    }
  }

  async function handleRestoreRepo(repo: RepoDirectory) {
    try {
      const folder = await pickStorageFolder();
      if (!folder) return;
      restoringRepo = repo.directory_id;
      await restoreRepoFromExport(
        importResult?.storage_path
          ? checkResult?.zip_path ?? ""
          : "",
        repo.directory_id,
        folder
      );
      if (importResult) {
        importResult = {
          ...importResult,
          directories_with_repos: importResult.directories_with_repos.filter(
            (d) => d.directory_id !== repo.directory_id
          ),
        };
      }
    } catch (e: any) {
      error = typeof e === "string" ? e : e.message || "Restore failed";
    } finally {
      restoringRepo = null;
    }
  }

  function handleStart() {
    if (mode === "export") handleExport();
    else handleCheckImport();
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="modal-backdrop"
  onmousedown={(e) => { if (e.target === e.currentTarget && !running) onClose(); }}
>
  <div class="modal-content" style="width: 520px;">
    <h2 style="font-size: 16px; font-weight: 700; letter-spacing: -0.02em; color: var(--text-primary); margin-bottom: 20px;">
      {mode === "export" ? "Export Storage" : "Import Storage"}
    </h2>

    {#if !running && !done && !error}
      {#if mode === "export"}
        <p class="ei-desc">
          Choose what to include in the export archive.
        </p>

        <div class="ei-options">
          <div class="ei-option-group">
            <h3>Features</h3>
            <label class="ei-checkbox">
              <input type="checkbox" bind:checked={includeDone} />
              <span>Done features</span>
            </label>
            <label class="ei-checkbox">
              <input type="checkbox" bind:checked={includeArchived} />
              <span>Archived features</span>
            </label>
          </div>

          <div class="ei-option-group">
            <h3>Data</h3>
            <label class="ei-checkbox">
              <input type="checkbox" bind:checked={includeTasks} />
              <span>Tasks</span>
            </label>
            <label class="ei-checkbox">
              <input type="checkbox" bind:checked={includeNotes} />
              <span>Notes</span>
            </label>
            <label class="ei-checkbox">
              <input type="checkbox" bind:checked={includeContext} />
              <span>Context</span>
            </label>
            <label class="ei-checkbox">
              <input type="checkbox" bind:checked={includeSessions} />
              <span>Sessions</span>
            </label>
          </div>

          <div class="ei-option-group">
            <h3>Files & Repos</h3>
            <label class="ei-checkbox">
              <input type="checkbox" bind:checked={includeFiles} />
              <span>Workspace files</span>
            </label>
            <label class="ei-checkbox">
              <input type="checkbox" bind:checked={includePatches} />
              <span>Git patches from dirty repos</span>
            </label>
          </div>
        </div>

        <p class="ei-hint">
          Links, directories, tags, and feature metadata are always included.
        </p>
      {:else if !checkResult}
        <p class="ei-desc">
          Import a previously exported archive into the current storage.
          You'll choose a ZIP file to import from.
        </p>
      {:else}
        <div class="ei-check-result">
          <p class="ei-desc">
            Archive contains <strong>{checkResult.total_features}</strong> feature{checkResult.total_features === 1 ? "" : "s"}.
            {checkResult.duplicate_count > 0
              ? `${checkResult.duplicate_count} already exist in this storage — how should they be handled?`
              : ""}
          </p>

          {#if checkResult.duplicate_count > 0}
            <div class="ei-strategy">
              <label class="ei-radio">
                <input type="radio" bind:group={duplicateStrategy} value="ignore" />
                <span>
                  <strong>Ignore duplicates</strong>
                  <small>Skip features that already exist; only add new ones.</small>
                </span>
              </label>
              <label class="ei-radio">
                <input type="radio" bind:group={duplicateStrategy} value="replace" />
                <span>
                  <strong>Replace duplicates</strong>
                  <small>Overwrite existing features and all their data with the imported version.</small>
                </span>
              </label>
              <label class="ei-radio">
                <input type="radio" bind:group={duplicateStrategy} value="merge" />
                <span>
                  <strong>Merge</strong>
                  <small>Update feature metadata and notes; add new tasks, links, and sessions without removing existing ones.</small>
                </span>
              </label>
            </div>

            {#if checkResult.duplicate_titles.length > 0}
              <details class="ei-dup-list">
                <summary>{checkResult.duplicate_count} duplicate{checkResult.duplicate_count === 1 ? "" : "s"}</summary>
                <ul>
                  {#each checkResult.duplicate_titles as title}
                    <li>{title}</li>
                  {/each}
                </ul>
              </details>
            {/if}
          {/if}
        </div>
      {/if}

      <div class="modal-actions" style="margin-top: 20px;">
        <button class="btn btn-subtle" style="padding: 7px 16px;" onclick={onClose}>Cancel</button>
        {#if mode === "export"}
          <button class="btn btn--primary btn-new" style="width: auto; padding: 7px 18px;" onclick={handleStart}>
            Export
          </button>
        {:else if !checkResult}
          <button class="btn btn--primary btn-new" style="width: auto; padding: 7px 18px;" onclick={handleStart}>
            Choose File…
          </button>
        {:else}
          <button
            class="btn btn--primary btn-new"
            style="width: auto; padding: 7px 18px;"
            onclick={() => handleImport(checkResult!.zip_path, duplicateStrategy)}
          >
            Import
          </button>
        {/if}
      </div>
    {:else if running}
      <div class="ei-progress">
        <div class="ei-progress-track">
          <div class="ei-progress-fill" style="width: {percent}%"></div>
        </div>
        <p class="ei-progress-stage">{stage}</p>
        {#if mode === "export"}
          <div class="modal-actions" style="margin-top: 12px;">
            <button class="btn btn-subtle" style="padding: 5px 14px;" onclick={() => cancelExport()}>Cancel</button>
          </div>
        {/if}
      </div>
    {:else if error}
      <div class="ei-error">
        <p>{error}</p>
      </div>
      <div class="modal-actions" style="margin-top: 16px;">
        <button class="btn btn-subtle" style="padding: 7px 16px;" onclick={onClose}>Close</button>
      </div>
    {:else if done}
      {#if mode === "export" && resultPath}
        <div class="ei-success">
          <p>Storage exported successfully!</p>
          <p class="ei-path">{resultPath}</p>
        </div>
      {:else if importResult}
        <div class="ei-success">
          <p>Import complete!</p>
          <div class="ei-summary">
            <span>{importResult.feature_count} features</span>
            <span>{importResult.file_count} files</span>
          </div>

          {#if importResult.directories_with_repos.length > 0}
            <div class="ei-repos">
              <h3>Linked Repositories</h3>
              <p class="ei-desc">
                These repositories can be re-cloned to restore your project directories.
              </p>
              {#each importResult.directories_with_repos as repo (repo.directory_id)}
                <div class="ei-repo-item">
                  <div class="ei-repo-info">
                    <span class="ei-repo-feature">{repo.feature_title}</span>
                    <span class="ei-repo-url">{repo.repo_url}</span>
                    {#if repo.has_patch}
                      <span class="ei-repo-patch">Has uncommitted changes patch</span>
                    {/if}
                  </div>
                  <button
                    class="btn btn-accent"
                    disabled={restoringRepo === repo.directory_id}
                    onclick={() => handleRestoreRepo(repo)}
                  >
                    {restoringRepo === repo.directory_id ? "Cloning..." : "Re-clone"}
                  </button>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <div class="modal-actions" style="margin-top: 20px;">
        <button class="btn btn--primary btn-new" style="width: auto; padding: 7px 18px;" onclick={onClose}>Done</button>
      </div>
    {/if}
  </div>
</div>

<style>
  .ei-desc {
    color: var(--text-secondary);
    font-size: var(--text-sm);
    margin-bottom: var(--space-3);
    line-height: 1.5;
  }

  .ei-hint {
    color: var(--text-muted);
    font-size: var(--text-xs);
    margin-top: var(--space-3);
  }

  .ei-options {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .ei-option-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .ei-option-group h3 {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
    margin-bottom: 2px;
  }

  .ei-checkbox {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    color: var(--text-secondary);
    cursor: pointer;
    padding: 3px 0;
  }

  .ei-checkbox:hover {
    color: var(--text-primary);
  }

  .ei-checkbox input {
    accent-color: var(--accent);
  }

  .ei-progress {
    padding: var(--space-4) 0;
  }

  .ei-progress-track {
    width: 100%;
    height: 6px;
    background: var(--bg-input);
    border-radius: 3px;
    overflow: hidden;
  }

  .ei-progress-fill {
    height: 100%;
    background: var(--accent);
    border-radius: 3px;
    transition: width 0.3s ease;
  }

  .ei-progress-stage {
    margin-top: var(--space-3);
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  .ei-error {
    color: var(--red);
    font-size: var(--text-sm);
  }

  .ei-success p {
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  .ei-path {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--text-muted) !important;
    word-break: break-all;
    margin-top: var(--space-2);
  }

  .ei-summary {
    display: flex;
    gap: var(--space-3);
    margin: var(--space-3) 0;
  }

  .ei-summary span {
    padding: var(--space-1) var(--space-3);
    background: var(--accent-dim);
    border-radius: var(--radius);
    color: var(--accent);
    font-size: var(--text-sm);
    font-weight: 500;
  }

  .ei-repos {
    margin-top: var(--space-4);
    border-top: 1px solid var(--border);
    padding-top: var(--space-3);
  }

  .ei-repos h3 {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: var(--space-2);
  }

  .ei-repo-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-2) 0;
    border-bottom: 1px solid var(--border-subtle);
  }

  .ei-repo-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    flex: 1;
  }

  .ei-repo-feature {
    font-size: var(--text-sm);
    color: var(--text-primary);
  }

  .ei-repo-url {
    font-size: var(--text-xs);
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .ei-repo-patch {
    font-size: var(--text-xs);
    color: var(--amber);
  }

  .ei-check-result {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .ei-strategy {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .ei-radio {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius);
    cursor: pointer;
  }

  .ei-radio:has(input:checked) {
    border-color: var(--accent);
    background: var(--accent-dim);
  }

  .ei-radio input {
    margin-top: 2px;
    accent-color: var(--accent);
    flex-shrink: 0;
  }

  .ei-radio span {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .ei-radio strong {
    font-size: var(--text-sm);
    color: var(--text-primary);
  }

  .ei-radio small {
    font-size: var(--text-xs);
    color: var(--text-muted);
    line-height: 1.4;
  }

  .ei-dup-list {
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  .ei-dup-list summary {
    cursor: pointer;
    color: var(--text-secondary);
  }

  .ei-dup-list ul {
    margin: var(--space-2) 0 0 var(--space-4);
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: 120px;
    overflow-y: auto;
  }
</style>
