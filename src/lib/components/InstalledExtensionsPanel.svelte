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
  <div class="settings-section extensions-panel">
    <div class="settings-section-title">Installed Extensions</div>
    {#each installedExtensions as ext}
      <div class="extension-card glass-panel glass-panel--hover extension-row" class:extension-card--enabled={ext.enabled}>
        <div class="extension-card__header extension-row__head">
          <div class="extension-card__info">
            <span class="extension-card__name extension-row__name">{ext.manifest.name}</span>
            <span class="aurora-pill aurora-pill--muted aurora-pill--sm aurora-pill--no-dot">v{ext.manifest.version}</span>
            <span class="extension-card__badge">Installed</span>
          </div>
          <label class="extension-row__actions" style="font-size: 12px; color: var(--fg2);">
            <button
              type="button"
              class="toggle"
              class:toggle--on={ext.enabled}
              aria-pressed={ext.enabled}
              aria-label="Toggle extension enabled"
              onclick={() => toggleEnabled(ext.manifest.id, !ext.enabled)}
            ></button>
            Enabled
          </label>
        </div>
        {#if ext.manifest.description}
          <div class="extension-card__desc extension-row__desc">{ext.manifest.description}</div>
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
