<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getExtensions } from "../api/extensions";
  import type { ExtensionInfo } from "../api/types";

  let installedExtensions = $state<ExtensionInfo[]>([]);
  let loading = $state(true);

  async function toggleEnabled(extensionId: string, enabled: boolean) {
    const key = installedExtensions.find((e) => e.manifest.id === extensionId)?.manifest
      .storage_settings_key ?? extensionId;
    const current = await invoke<Record<string, unknown>>("get_extension_settings", { key });
    await invoke("set_extension_settings", {
      key,
      value: { ...current, enabled },
    });
    const ext = installedExtensions.find((e) => e.manifest.id === extensionId);
    if (ext) ext.enabled = enabled;
    alert("Restart Feature Hub for the change to take effect.");
  }

  onMount(async () => {
    try {
      installedExtensions = await getExtensions();
    } catch (e) {
      console.error("[extensions] Failed to load installed extensions:", e);
    } finally {
      loading = false;
    }
  });
</script>

{#if !loading && installedExtensions.length > 0}
  <div class="settings-section">
    <div class="settings-section-title">Installed Extensions</div>
    {#each installedExtensions as ext}
      <div class="extension-card" class:extension-card--enabled={ext.enabled}>
        <div class="extension-card__header">
          <div class="extension-card__info">
            <span class="extension-card__name">{ext.manifest.name}</span>
            <span class="extension-card__badge" style="background: var(--bg3); color: var(--fg2); font-size: 10px; padding: 2px 6px; border-radius: 4px;">v{ext.manifest.version}</span>
            <span class="extension-card__badge">Installed</span>
          </div>
          <label style="display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--fg2);">
            <input
              type="checkbox"
              checked={ext.enabled}
              onchange={(e) => toggleEnabled(ext.manifest.id, e.currentTarget.checked)}
            />
            Enabled
          </label>
        </div>
        {#if ext.manifest.description}
          <div class="extension-card__desc">{ext.manifest.description}</div>
        {/if}
        {#if ext.requires_status.length > 0}
          <div class="extension-card__body">
            <div class="extension-card__section" style="font-size: 12px; color: var(--fg2);">
              <strong>Requires:</strong>
              {#each ext.requires_status as req}
                <span style="margin-left: 8px;">
                  {#if req.found}
                    <span style="color: var(--green);">✓</span>
                  {:else}
                    <span style="color: var(--red);">✗</span>
                  {/if}
                  <code>{req.name}</code>
                </span>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/each}
  </div>
{/if}
