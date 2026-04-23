<script lang="ts">
  import { pickStorageFolder, createStorage } from "../api/tauri";
  import type { StorageInfo } from "../api/tauri";

  let { onCreated }: { onCreated: (storage: StorageInfo) => void } = $props();

  let loading = $state(false);
  let error = $state<string | null>(null);

  async function handleSelect() {
    error = null;
    try {
      const folder = await pickStorageFolder();
      if (!folder) return;

      loading = true;
      const storage = await createStorage(folder);
      onCreated(storage);
    } catch (e: any) {
      error = e?.toString() ?? "Failed to create storage";
    } finally {
      loading = false;
    }
  }
</script>

<div class="storage-setup">
  <div class="storage-setup-card">
    <div style="width: 48px; height: 48px; background: linear-gradient(135deg, #7c7cff 0%, #a78bfa 100%); border-radius: 12px; display: grid; place-items: center; font-size: 18px; font-weight: 800; color: #fff; letter-spacing: -0.5px;">
      FH
    </div>
    <h1 style="font-size: 20px; font-weight: 700; letter-spacing: -0.03em; margin-top: 16px;">Welcome to Feature Hub</h1>
    <p style="font-size: 13px; color: var(--text-secondary); margin-top: 8px; max-width: 340px; text-align: center; line-height: 1.6;">
      Choose a folder to store your features, files, and database. You can add more storage locations later.
    </p>

    {#if error}
      <div style="margin-top: 16px; padding: 8px 12px; border-radius: 6px; background: var(--red-dim); border: 1px solid rgba(229,72,77,0.2); color: var(--red); font-size: 12px; max-width: 340px;">
        {error}
      </div>
    {/if}

    <button class="btn btn--primary btn-new" style="width: auto; margin-top: 24px; padding: 10px 28px; font-size: 14px;" onclick={handleSelect} disabled={loading}>
      {loading ? "Creating..." : "Select Folder"}
    </button>
  </div>
</div>
