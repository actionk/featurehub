<script lang="ts">
  import type { McpServer, FeatureMcpServer } from "../../api/tauri";
  import { getFeatureMcpServers, setFeatureMcpServer } from "../../api/tauri";
  import { getCachedSettings } from "../../stores/settings.svelte";

  let {
    featureId,
  }: {
    featureId: string;
  } = $props();

  let allServers = $state<McpServer[]>([]);
  let featureOverrides = $state<FeatureMcpServer[]>([]);
  let loading = $state(true);

  let enabledCount = $derived(allServers.filter(isServerEnabled).length);

  function isServerEnabled(server: McpServer): boolean {
    const override = featureOverrides.find((o) => o.server_name === server.name);
    if (override) return override.enabled;
    return server.default_enabled;
  }

  async function toggleServer(server: McpServer) {
    const current = isServerEnabled(server);
    await setFeatureMcpServer(featureId, server.name, !current);
    const existing = featureOverrides.findIndex((o) => o.server_name === server.name);
    if (existing >= 0) {
      featureOverrides[existing] = { server_name: server.name, enabled: !current };
    } else {
      featureOverrides = [...featureOverrides, { server_name: server.name, enabled: !current }];
    }
  }

  $effect(() => {
    loadData();
  });

  async function loadData() {
    loading = true;
    try {
      const [settings, overrides] = await Promise.all([
        getCachedSettings(),
        getFeatureMcpServers(featureId),
      ]);
      const extServers: McpServer[] = (settings.extensions ?? [])
        .filter((e) => e.enabled && e.mcp_server)
        .map((e) => ({ ...e.mcp_server!, name: e.id }));
      allServers = [...(settings.mcp_servers ?? []), ...extServers];
      featureOverrides = overrides;
    } catch (e) {
      console.error("Failed to load MCP servers:", e);
    } finally {
      loading = false;
    }
  }
</script>

{#if !loading && allServers.length > 0}
  <div class="mcp-section">
    <div class="mcp-header">
      <svg class="mcp-icon" width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M1.75 1h12.5c.966 0 1.75.784 1.75 1.75v4.5A1.75 1.75 0 0114.25 9H1.75A1.75 1.75 0 010 7.25v-4.5C0 1.784.784 1 1.75 1zm0 1.5a.25.25 0 00-.25.25v4.5c0 .138.112.25.25.25h12.5a.25.25 0 00.25-.25v-4.5a.25.25 0 00-.25-.25zM3.5 4.5a1 1 0 112 0 1 1 0 01-2 0zm7.75-.75a.75.75 0 000 1.5h1a.75.75 0 000-1.5z"/></svg>
      <span class="mcp-title">MCP Servers</span>
      <span class="mcp-count">{enabledCount}/{allServers.length}</span>
    </div>
    <div class="mcp-blocks">
      {#each allServers as server (server.name)}
        {@const enabled = isServerEnabled(server)}
        <button
          class="mcp-block"
          class:mcp-block--on={enabled}
          onclick={() => toggleServer(server)}
        >
          <span class="mcp-dot" class:mcp-dot--on={enabled}></span>
          <span class="mcp-name">{server.name}</span>
        </button>
      {/each}
    </div>
  </div>
{/if}

<style>
  .mcp-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .mcp-header {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .mcp-icon {
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .mcp-title {
    font-size: 11.5px;
    font-weight: 650;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted);
  }

  .mcp-count {
    font-size: 10.5px;
    font-weight: 500;
    color: var(--text-muted);
    font-family: var(--font-mono);
    opacity: 0.7;
  }

  .mcp-blocks {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .mcp-block {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 5px 12px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--bg-card);
    color: var(--text-muted);
    font-family: inherit;
    font-size: 12px;
    cursor: pointer;
    transition: all 0.15s;
    white-space: nowrap;
  }
  .mcp-block:hover {
    background: var(--bg-hover);
    border-color: color-mix(in srgb, var(--text-muted) 30%, var(--border));
  }
  .mcp-block--on {
    color: var(--text-primary);
    border-color: color-mix(in srgb, var(--green) 30%, var(--border));
    background: color-mix(in srgb, var(--green) 5%, var(--bg-card));
  }
  .mcp-block--on:hover {
    border-color: color-mix(in srgb, var(--green) 50%, var(--border));
    background: color-mix(in srgb, var(--green) 8%, var(--bg-card));
  }

  .mcp-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
    background: var(--text-muted);
    opacity: 0.3;
    transition: all 0.15s;
  }
  .mcp-dot--on {
    background: var(--green);
    opacity: 1;
    box-shadow: 0 0 4px color-mix(in srgb, var(--green) 50%, transparent);
  }

  .mcp-name {
    font-size: 12px;
    font-weight: 500;
  }
</style>