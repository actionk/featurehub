<script lang="ts">
  import type { McpServer, Repository, Extension, Skill } from "../api/tauri";
  import InstalledExtensionsPanel from "./InstalledExtensionsPanel.svelte";
  import { getSettings, saveSettings, getFhCliPath, rebuildSearchIndex, installCliToPath, checkCliInstalled, detectIdes } from "../api/tauri";
  import { setMermaidEnabled, setOpenFgaEnabled, setShowTabEmojis, setUiFont, setMonoFont, setUiFontSize, setTerminalFontSize, invalidateSettingsCache } from "../stores/settings.svelte";

  let {
    onClose,
    storageName = "",
    initialTab: initialTabProp,
  }: {
    onClose: () => void;
    storageName?: string;
    initialTab?: string;
  } = $props();

  let fhCliPath = $state("");
  let detectedPath = $state("");
  let mcpServers = $state<McpServer[]>([]);
  let defaultRepositories = $state<Repository[]>([]);
  let extensions = $state<Extension[]>([]);
  let skills = $state<Skill[]>([]);
  let mermaidDiagrams = $state(false);
  let openfgaHighlighting = $state(false);
  let showTabEmojis = $state(false);
  let preferredIdes = $state<string[]>([]);
  let allDetectedIdes = $state<import("../api/tauri").DetectedIde[]>([]);
  let uiFont = $state("");
  let monoFont = $state("");
  let uiFontSize = $state(0);
  let terminalFontSize = $state(0);
  let saving = $state(false);

  const uiFontOptions = [
    { value: "", label: "Default (Plus Jakarta Sans)" },
    { value: "Inter", label: "Inter" },
    { value: "Segoe UI", label: "Segoe UI" },
    { value: "Roboto", label: "Roboto" },
    { value: "Open Sans", label: "Open Sans" },
    { value: "Nunito", label: "Nunito" },
    { value: "Lato", label: "Lato" },
    { value: "Source Sans 3", label: "Source Sans 3" },
    { value: "IBM Plex Sans", label: "IBM Plex Sans" },
    { value: "Geist", label: "Geist" },
  ];

  const monoFontOptions = [
    { value: "", label: "Default (JetBrains Mono)" },
    { value: "Fira Code", label: "Fira Code" },
    { value: "Cascadia Code", label: "Cascadia Code" },
    { value: "Source Code Pro", label: "Source Code Pro" },
    { value: "IBM Plex Mono", label: "IBM Plex Mono" },
    { value: "Inconsolata", label: "Inconsolata" },
    { value: "Roboto Mono", label: "Roboto Mono" },
    { value: "Ubuntu Mono", label: "Ubuntu Mono" },
    { value: "Geist Mono", label: "Geist Mono" },
    { value: "Consolas", label: "Consolas" },
  ];

  const fontSizeOptions = [
    { value: 0, label: "Default (13px)" },
    { value: 11, label: "11px" },
    { value: 12, label: "12px" },
    { value: 13, label: "13px" },
    { value: 14, label: "14px" },
    { value: 15, label: "15px" },
    { value: 16, label: "16px" },
  ];

  const terminalFontSizeOptions = [
    { value: 0, label: "Default (13px)" },
    { value: 10, label: "10px" },
    { value: 11, label: "11px" },
    { value: 12, label: "12px" },
    { value: 13, label: "13px" },
    { value: 14, label: "14px" },
    { value: 15, label: "15px" },
    { value: 16, label: "16px" },
    { value: 18, label: "18px" },
    { value: 20, label: "20px" },
  ];
  let saved = $state(false);
  let loading = $state(true);
  let loaded = $state(false);
  let reindexing = $state(false);
  let reindexed = $state(false);
  let installing = $state(false);
  let installResult = $state<string | null>(null);
  let installError = $state<string | null>(null);
  let cliInPath = $state<string | null>(null);
  let activeTab = $state(initialTabProp ?? "general");
  let saveTimer: ReturnType<typeof setTimeout> | null = null;

  const STORAGE_TABS = new Set(["mcp", "directories", "extensions", "skills"]);

  const tabs = [
    { id: "general", label: "General", icon: "settings" },
    { id: "appearance", label: "Appearance", icon: "appearance" },
    { id: "mcp", label: "MCP Servers", icon: "mcp" },
    { id: "editors", label: "Editors", icon: "editors" },
    { id: "directories", label: "Default Repos", icon: "folder" },
    { id: "extensions", label: "Extensions", icon: "extensions" },
    { id: "skills", label: "Skills", icon: "skills" },
  ];

  // Load current settings
  $effect(() => {
    (async () => {
      try {
        const [settings, detected, installedPath, ides] = await Promise.all([
          getSettings(),
          getFhCliPath().catch(() => ""),
          checkCliInstalled().catch(() => null),
          detectIdes().catch(() => []),
        ]);
        cliInPath = installedPath;
        fhCliPath = settings.fh_cli_path ?? "";
        mcpServers = (settings.mcp_servers ?? []).map((s) => ({ ...s, env: s.env ?? {}, default_enabled: s.default_enabled ?? true }));
        defaultRepositories = settings.default_repositories ?? [];
        extensions = (settings.extensions ?? []).map((e) => ({ ...e, mcp_server: e.mcp_server ? { ...e.mcp_server, env: e.mcp_server.env ?? {}, default_enabled: e.mcp_server.default_enabled ?? true } : null }));
        skills = (settings.skills ?? []).map((s) => ({ ...s }));
        mermaidDiagrams = settings.mermaid_diagrams ?? false;
        openfgaHighlighting = settings.openfga_highlighting ?? false;
        showTabEmojis = settings.show_tab_emojis ?? false;
        uiFont = settings.ui_font ?? "";
        monoFont = settings.mono_font ?? "";
        uiFontSize = settings.ui_font_size ?? 0;
        terminalFontSize = settings.terminal_font_size ?? 0;
        preferredIdes = settings.preferred_ides ?? [];
        allDetectedIdes = ides;
        detectedPath = detected;
      } catch (e) {
        console.error("Failed to load settings:", e);
      } finally {
        loading = false;
        // Defer setting loaded so the auto-save effect doesn't fire on initial values
        setTimeout(() => { loaded = true; }, 0);
      }
    })();
  });

  // Auto-save on any change (debounced)
  $effect(() => {
    // Read all reactive settings to track them
    void fhCliPath;
    void mcpServers;
    void defaultRepositories;
    void extensions;
    void mermaidDiagrams;
    void openfgaHighlighting;
    void showTabEmojis;
    void uiFont;
    void monoFont;
    void uiFontSize;
    void terminalFontSize;
    void JSON.stringify(preferredIdes);
    // Deep-read mcpServers/extensions to track nested changes
    void JSON.stringify(mcpServers);
    void JSON.stringify(defaultRepositories);
    void JSON.stringify(extensions);
    void JSON.stringify(skills);

    if (!loaded) return;

    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => doSave(), 400);
  });

  async function doSave() {
    saving = true;
    try {
      const cleaned = mcpServers.filter((s) => s.name.trim() && (s.command.trim() || s.url));
      const cleanedRepos = defaultRepositories.filter((r) => r.url.trim());
      await saveSettings({
        fh_cli_path: fhCliPath.trim() || null,
        mcp_servers: cleaned,
        default_repositories: cleanedRepos,
        mermaid_diagrams: mermaidDiagrams,
        openfga_highlighting: openfgaHighlighting,
        show_tab_emojis: showTabEmojis,
        ui_font: uiFont.trim() || null,
        mono_font: monoFont.trim() || null,
        ui_font_size: uiFontSize || null,
        terminal_font_size: terminalFontSize || null,
        extensions,
        preferred_ides: preferredIdes,
        skills: skills.filter((s) => s.id.trim() && s.name.trim()),
      });
      invalidateSettingsCache();
      setMermaidEnabled(mermaidDiagrams);
      setOpenFgaEnabled(openfgaHighlighting);
      setShowTabEmojis(showTabEmojis);
      setUiFont(uiFont.trim() || null);
      setMonoFont(monoFont.trim() || null);
      setUiFontSize(uiFontSize || null);
      setTerminalFontSize(terminalFontSize || null);
      saved = true;
      setTimeout(() => { saved = false; }, 1500);
    } catch (e) {
      console.error("Failed to save settings:", e);
    } finally {
      saving = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }

  function useDetected() {
    fhCliPath = detectedPath;
  }

  function addMcpServer() {
    mcpServers = [...mcpServers, { name: "", command: "", args: [], env: {}, default_enabled: true }];
  }

  function removeMcpServer(index: number) {
    mcpServers = mcpServers.filter((_, i) => i !== index);
  }

  function updateMcpArgs(index: number, value: string) {
    mcpServers[index].args = value.split(/\s+/).filter(Boolean);
  }

  function addEnvVar(index: number) {
    mcpServers[index].env = { ...mcpServers[index].env, "": "" };
  }

  function updateEnvKey(serverIdx: number, oldKey: string, newKey: string) {
    const env = { ...mcpServers[serverIdx].env };
    const val = env[oldKey] ?? "";
    delete env[oldKey];
    env[newKey] = val;
    mcpServers[serverIdx].env = env;
  }

  function updateEnvVal(serverIdx: number, key: string, val: string) {
    mcpServers[serverIdx].env = { ...mcpServers[serverIdx].env, [key]: val };
  }

  function removeEnvVar(serverIdx: number, key: string) {
    const env = { ...mcpServers[serverIdx].env };
    delete env[key];
    mcpServers[serverIdx].env = env;
  }

  async function handleReindex() {
    reindexing = true;
    try {
      await rebuildSearchIndex();
      reindexed = true;
      setTimeout(() => { reindexed = false; }, 2000);
    } catch (e) {
      console.error("Failed to rebuild search index:", e);
    } finally {
      reindexing = false;
    }
  }

  async function handleInstallCli() {
    installing = true;
    installResult = null;
    installError = null;
    try {
      const result = await installCliToPath();
      const bins = result.binaries.join(", ");
      if (result.path_updated) {
        installResult = `Installed ${bins} to ${result.install_dir}`;
      } else {
        installResult = `Installed ${bins} to ${result.install_dir}. You may need to add this directory to your PATH or restart your terminal.`;
      }
      // Update the fh CLI path field and detected path
      fhCliPath = result.install_dir + (navigator.platform.startsWith("Win") ? "\\fh.exe" : "/fh");
      cliInPath = fhCliPath;
    } catch (e: any) {
      installError = typeof e === "string" ? e : e?.message ?? "Installation failed";
    } finally {
      installing = false;
    }
  }

  function addDefaultRepository() {
    defaultRepositories = [...defaultRepositories, { url: "", name: null, description: null }];
  }

  function removeDefaultRepository(index: number) {
    defaultRepositories = defaultRepositories.filter((_, i) => i !== index);
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="settings-fullscreen" onkeydown={handleKeydown}>
  <!-- Left sidebar with tabs -->
  <div class="settings-sidebar">
    <button class="settings-sidebar-header" onclick={onClose}>
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M10 12L6 8l4-4"/>
      </svg>
      <span class="settings-sidebar-title">Settings</span>
    </button>

    <nav class="settings-nav">
      {#each tabs as tab}
        <button
          class="settings-nav-item"
          class:active={activeTab === tab.id}
          onclick={() => activeTab = tab.id}
        >
          {#if tab.icon === "settings"}
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="8" cy="8" r="2.5"/>
              <path d="M13.5 8a5.5 5.5 0 01-.3 1.3l1.3 1-1.2 2-1.5-.5a5.5 5.5 0 01-1.1.7L10.3 14h-2.4l-.4-1.5a5.5 5.5 0 01-1.1-.7l-1.5.5-1.2-2 1.3-1A5.5 5.5 0 014.5 8c0-.5 0-.9.2-1.3l-1.3-1 1.2-2 1.5.5a5.5 5.5 0 011.1-.7L7.7 2h2.4l.4 1.5c.4.2.8.4 1.1.7l1.5-.5 1.2 2-1.3 1c.1.4.2.8.2 1.3z"/>
            </svg>
          {:else if tab.icon === "mcp"}
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="2" y="3" width="12" height="10" rx="1.5"/>
              <path d="M5 7h6M5 9.5h4"/>
            </svg>
          {:else if tab.icon === "folder"}
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M2 4.5V12a1.5 1.5 0 001.5 1.5h9A1.5 1.5 0 0014 12V6a1.5 1.5 0 00-1.5-1.5H8L6.5 3H3.5A1.5 1.5 0 002 4.5z"/>
            </svg>
          {:else if tab.icon === "appearance"}
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="8" cy="8" r="6"/>
              <path d="M8 2a6 6 0 000 12V2z"/>
            </svg>
          {:else if tab.icon === "editors"}
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="2" y="2" width="12" height="12" rx="2"/>
              <path d="M5 5l2.5 3L5 11M8.5 11H11"/>
            </svg>
          {:else if tab.icon === "extensions"}
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M9.5 2v1.5a1.5 1.5 0 003 0V2H14a1 1 0 011 1v2.5h-1.5a1.5 1.5 0 000 3H15V11a1 1 0 01-1 1h-2.5v-1.5a1.5 1.5 0 00-3 0V12H6a1 1 0 01-1-1V8.5h1.5a1.5 1.5 0 000-3H5V3a1 1 0 011-1h3.5z"/>
            </svg>
          {:else if tab.icon === "skills"}
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M8 1.5l1.5 3 3.5.5-2.5 2.5.5 3.5L8 9.5 4.5 11l.5-3.5L2.5 5l3.5-.5z"/>
              <path d="M5 13.5h6" opacity="0.5"/>
            </svg>
          {/if}
          <span>{tab.label}</span>
          {#if STORAGE_TABS.has(tab.id)}
            <span class="settings-storage-dot" title="Per-storage setting"></span>
          {/if}
        </button>
      {/each}
    </nav>

    {#if saving || saved}
      <div class="settings-sidebar-footer">
        <div class="settings-save-status">
          {#if saved}
            <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="var(--green)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 8.5l3.5 3.5 6.5-7"/></svg>
            <span style="color: var(--green);">Saved</span>
          {:else}
            <span>Saving...</span>
          {/if}
        </div>
      </div>
    {/if}
  </div>

  <!-- Main content area -->
  <div class="settings-main">
    {#if loading}
      <div class="settings-loading">Loading...</div>
    {:else}
      <!-- General tab -->
      {#if activeTab === "general"}
        <div class="settings-panel">
          <div class="settings-panel-header">
            <h2 class="settings-panel-title">General</h2>
            <p class="settings-panel-desc">Configure general application settings.</p>
          </div>

          <div class="settings-section">
            <label class="form-label" for="fh-cli-path">fh CLI Path</label>
            <div class="settings-field-desc">
              Full path to the <code>fh</code> binary. Used when copying the CLI command from sessions.
            </div>
            <div style="display: flex; gap: 6px; max-width: 560px;">
              <input
                id="fh-cli-path"
                type="text"
                placeholder="e.g. D:\path\to\fh.exe"
                class="form-input"
                style="flex: 1; font-family: var(--font-mono); font-size: 12px;"
                bind:value={fhCliPath}
              />
            </div>
            {#if detectedPath && detectedPath !== fhCliPath}
              <button class="btn-add" style="margin-top: 6px; font-size: 11px;" onclick={useDetected}>
                Use auto-detected: <code style="font-family: var(--font-mono);">{detectedPath}</code>
              </button>
            {/if}
            {#if !detectedPath && !fhCliPath}
              <div class="settings-field-hint">
                Build the CLI with <code>cd src-tauri && cargo build --bin fh</code> then set the path here.
              </div>
            {/if}
          </div>

          <div class="settings-section" style="margin-top: 20px;">
            <h3 class="settings-section-title">Install CLI to PATH</h3>
            <div class="settings-field-desc">
              Install <code>fh</code> and <code>fh-mcp</code> binaries so they're available from any terminal.
              {#if cliInPath}
                Currently found at: <code style="font-family: var(--font-mono);">{cliInPath}</code>
              {/if}
            </div>
            <button class="btn-add" onclick={handleInstallCli} disabled={installing} style="margin-top: 6px;">
              {#if installing}
                Installing...
              {:else if cliInPath}
                Reinstall to PATH
              {:else}
                Install to PATH
              {/if}
            </button>
            {#if installResult}
              <div class="settings-field-hint" style="color: var(--green); margin-top: 6px;">
                {installResult}
              </div>
            {/if}
            {#if installError}
              <div class="settings-field-hint" style="color: var(--red); margin-top: 6px;">
                {installError}
              </div>
            {/if}
          </div>

          <div class="settings-section" style="margin-top: 20px;">
            <h3 class="settings-section-title">Maintenance</h3>
            <div class="settings-field-desc" style="margin-bottom: 8px;">
              Rebuild the full-text search index. Use this if search results seem incomplete or outdated.
            </div>
            <button class="btn-add" onclick={handleReindex} disabled={reindexing}>
              {#if reindexed}
                Rebuilt!
              {:else if reindexing}
                Rebuilding...
              {:else}
                Rebuild Search Index
              {/if}
            </button>
          </div>
        </div>
      {/if}

      <!-- Appearance tab -->
      {#if activeTab === "appearance"}
        <div class="settings-panel">
          <div class="settings-panel-header">
            <h2 class="settings-panel-title">Appearance</h2>
            <p class="settings-panel-desc">Customize how the app looks.</p>
          </div>

          <div class="settings-section">
            <h3 class="settings-section-title">Fonts</h3>
            <div style="display: flex; flex-direction: column; gap: 16px; max-width: 460px;">
              <div>
                <label class="form-label" for="ui-font">UI Font</label>
                <select
                  id="ui-font"
                  class="form-input"
                  style="font-size: 12px;"
                  bind:value={uiFont}
                >
                  {#each uiFontOptions as opt}
                    <option value={opt.value}>{opt.label}</option>
                  {/each}
                </select>
                <div class="settings-field-desc" style="margin-top: 4px;">
                  Font for the interface. The font must be installed on your system.
                </div>
                <p class="settings-font-preview" style="font-family: {uiFont || 'var(--font-ui)'}; font-size: {uiFontSize ? uiFontSize + 'px' : 'var(--text-base)'};">
                  The quick brown fox jumps over the lazy dog — 0123456789
                </p>
              </div>
              <div>
                <label class="form-label" for="mono-font">Monospace Font</label>
                <select
                  id="mono-font"
                  class="form-input"
                  style="font-size: 12px;"
                  bind:value={monoFont}
                >
                  {#each monoFontOptions as opt}
                    <option value={opt.value}>{opt.label}</option>
                  {/each}
                </select>
                <div class="settings-field-desc" style="margin-top: 4px;">
                  Font for code, file sizes, and monospaced text.
                </div>
                <p class="settings-font-preview" style="font-family: {monoFont || 'var(--font-mono)'};">
                  const feature = await getFeature(id); // 0123456789
                </p>
              </div>
              <div class="settings-size-row">
                <div style="flex: 1;">
                  <label class="form-label" for="ui-font-size">Font Size</label>
                  <select
                    id="ui-font-size"
                    class="form-input"
                    style="font-size: 12px;"
                    bind:value={uiFontSize}
                  >
                    {#each fontSizeOptions as opt}
                      <option value={opt.value}>{opt.label}</option>
                    {/each}
                  </select>
                  <div class="settings-field-desc" style="margin-top: 4px;">
                    Base font size for the interface.
                  </div>
                </div>
                <div style="flex: 1;">
                  <label class="form-label" for="terminal-font-size">Terminal Font Size</label>
                  <select
                    id="terminal-font-size"
                    class="form-input"
                    style="font-size: 12px;"
                    bind:value={terminalFontSize}
                  >
                    {#each terminalFontSizeOptions as opt}
                      <option value={opt.value}>{opt.label}</option>
                    {/each}
                  </select>
                  <div class="settings-field-desc" style="margin-top: 4px;">
                    Font size for the embedded terminal.
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div class="settings-section" style="margin-top: 20px;">
            <h3 class="settings-section-title">Tabs</h3>
            <label class="settings-toggle">
              <input type="checkbox" bind:checked={showTabEmojis} />
              <span class="settings-toggle-label">Show emoji icons in tabs</span>
            </label>
            <div class="settings-field-desc">
              Display emoji icons next to tab labels in the feature detail view.
            </div>
          </div>
        </div>
      {/if}

      <!-- Editors tab -->
      {#if activeTab === "editors"}
        <div class="settings-panel">
          <div class="settings-panel-header">
            <h2 class="settings-panel-title">Editors</h2>
            <p class="settings-panel-desc">Choose which IDEs and editors appear in the terminal header and repository panels for quick "Open in" access.</p>
          </div>

          {#if allDetectedIdes.length === 0}
            <div style="font-size: 12px; color: var(--text-muted); padding: 16px 0;">No IDEs detected on this system.</div>
          {:else}
            <div style="display: flex; flex-direction: column; gap: 2px;">
              {#each allDetectedIdes as ide (ide.id)}
                <label class="settings-ide-item">
                  <input
                    type="checkbox"
                    checked={preferredIdes.includes(ide.id)}
                    onchange={() => {
                      if (preferredIdes.includes(ide.id)) {
                        preferredIdes = preferredIdes.filter(id => id !== ide.id);
                      } else {
                        preferredIdes = [...preferredIdes, ide.id];
                      }
                    }}
                  />
                  <span style="font-size: 13px; color: var(--text-primary); font-weight: 500;">{ide.name}</span>
                  <span style="font-size: 10.5px; color: var(--text-muted); font-family: var(--font-mono); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; min-width: 0;">{ide.command}</span>
                </label>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <!-- MCP Servers tab -->
      {#if activeTab === "mcp"}
        <div class="settings-panel">
          <div class="settings-panel-header">
            <div style="display: flex; align-items: center; justify-content: space-between;">
              <div>
                <h2 class="settings-panel-title">MCP Servers</h2>
                <p class="settings-panel-desc">MCP servers available to Claude Code sessions. Servers marked "Default" are enabled for all features; others can be toggled on per-feature in the AI tab.</p>
                {#if storageName}<p class="settings-storage-hint">Configured per storage. Currently editing: <strong>{storageName}</strong></p>{/if}
              </div>
              <button class="btn-add" style="font-size: 12px; flex-shrink: 0;" onclick={addMcpServer}>+ Add Server</button>
            </div>
          </div>

          {#if mcpServers.length === 0}
            <div class="settings-empty">
              No MCP servers configured
            </div>
          {/if}

          {#if mcpServers.length > 0}
            <div style="display: flex; flex-direction: column; gap: 12px;">
              {#each mcpServers as server, i}
                <div class="mcp-server-entry">
                  <div style="display: flex; align-items: center; gap: 6px; margin-bottom: 8px;">
                    <div style="width: 6px; height: 6px; border-radius: 50%; background: {server.default_enabled ? 'var(--purple)' : 'var(--text-muted)'}; flex-shrink: 0;"></div>
                    <input
                      type="text"
                      class="form-input"
                      style="flex: 1; font-size: 12px; font-weight: 600; padding: 5px 8px;"
                      placeholder="Server name (e.g. my-mcp)"
                      bind:value={server.name}
                    />
                    <label class="settings-toggle" style="flex-shrink: 0;" title="Enable by default for all features">
                      <input type="checkbox" bind:checked={server.default_enabled} />
                      <span style="font-size: 10.5px; color: var(--text-muted);">Default</span>
                    </label>
                    <button class="btn-ghost" style="color: var(--red); flex-shrink: 0; padding: 4px;"
                      onclick={() => removeMcpServer(i)} aria-label="Remove server">
                      <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor"><path d="M4.5 3.1L8 6.6l3.5-3.5 1.4 1.4L9.4 8l3.5 3.5-1.4 1.4L8 9.4l-3.5 3.5-1.4-1.4L6.6 8 3.1 4.5z"/></svg>
                    </button>
                  </div>
                  <div style="display: flex; flex-direction: column; gap: 4px; padding-left: 12px;">
                    <input
                      type="text"
                      class="form-input"
                      style="font-family: var(--font-mono); font-size: 11.5px; padding: 5px 8px;"
                      placeholder="Command (e.g. npx, node, python)"
                      bind:value={server.command}
                    />
                    <textarea
                      class="form-input"
                      style="font-family: var(--font-mono); font-size: 11.5px; padding: 6px 8px; resize: vertical; min-height: 52px;"
                      placeholder="Arguments (space-separated)"
                      value={server.args.join(" ")}
                      oninput={(e) => updateMcpArgs(i, (e.target as HTMLTextAreaElement).value)}
                    ></textarea>
                    <div style="margin-top: 6px;">
                      <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 4px;">
                        <span style="font-size: 10.5px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.04em;">Environment</span>
                        <button class="btn-add" style="font-size: 10px; padding: 1px 6px;" onclick={() => addEnvVar(i)}>+ Env</button>
                      </div>
                      {#each Object.entries(server.env) as [key, val], ei}
                        <div style="display: flex; gap: 4px; margin-bottom: 3px; align-items: center;">
                          <input
                            type="text"
                            class="form-input"
                            style="font-family: var(--font-mono); font-size: 11px; padding: 3px 6px; width: 40%;"
                            placeholder="KEY"
                            value={key}
                            onchange={(e) => updateEnvKey(i, key, (e.target as HTMLInputElement).value)}
                          />
                          <input
                            type="text"
                            class="form-input"
                            style="font-family: var(--font-mono); font-size: 11px; padding: 3px 6px; flex: 1;"
                            placeholder="value"
                            value={val}
                            oninput={(e) => updateEnvVal(i, key, (e.target as HTMLInputElement).value)}
                          />
                          <button class="btn-ghost" style="color: var(--red); padding: 2px; flex-shrink: 0;"
                            onclick={() => removeEnvVar(i, key)} aria-label="Remove env var">
                            <svg width="9" height="9" viewBox="0 0 16 16" fill="currentColor"><path d="M4.5 3.1L8 6.6l3.5-3.5 1.4 1.4L9.4 8l3.5 3.5-1.4 1.4L8 9.4l-3.5 3.5-1.4-1.4L6.6 8 3.1 4.5z"/></svg>
                          </button>
                        </div>
                      {/each}
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <!-- Directories tab -->
      {#if activeTab === "directories"}
        <div class="settings-panel">
          <div class="settings-panel-header">
            <div style="display: flex; align-items: center; justify-content: space-between;">
              <div>
                <h2 class="settings-panel-title">Default Repositories</h2>
                <p class="settings-panel-desc">Repositories listed here appear as quick-clone suggestions in every feature's Repositories tab. When cloned, they are shallow-copied into the feature's workspace.</p>
                {#if storageName}<p class="settings-storage-hint">Configured per storage. Currently editing: <strong>{storageName}</strong></p>{/if}
              </div>
              <button class="btn-add" style="font-size: 12px; flex-shrink: 0;" onclick={addDefaultRepository}>+ Add Repository</button>
            </div>
          </div>

          {#if defaultRepositories.length === 0}
            <div class="settings-empty">
              <div>No default repositories configured</div>
              <div style="font-size: 11px; color: var(--text-muted); margin-top: 4px; opacity: 0.7;">
                Repositories added here appear as quick-clone suggestions in every feature's Repositories tab.
              </div>
            </div>
          {:else}
            <div style="display: flex; flex-direction: column; gap: 6px;">
              {#each defaultRepositories as repo, i}
                <div class="dir-entry" style="flex-direction: column; align-items: stretch; gap: 4px;">
                  <div style="display: flex; align-items: center; gap: 8px;">
                    <span style="font-size: 10px; font-weight: 600; color: var(--text-muted); opacity: 0.5; min-width: 14px; text-align: right;">{i + 1}</span>
                    <input
                      type="text"
                      class="settings-input"
                      style="flex: 1; font-size: 12px;"
                      placeholder="https://github.com/org/repo.git"
                      value={repo.url}
                      oninput={(e) => { defaultRepositories[i].url = e.currentTarget.value; }}
                    />
                    <button class="dir-chip-remove" style="opacity: 0.6;" title="Remove" onclick={() => removeDefaultRepository(i)}>
                      <svg width="10" height="10" viewBox="0 0 16 16"><path d="M4.5 3.5l7 7m0-7l-7 7" stroke="currentColor" stroke-width="2" stroke-linecap="round" fill="none"/></svg>
                    </button>
                  </div>
                  <div style="display: flex; gap: 6px; margin-left: 22px;">
                    <input
                      type="text"
                      class="settings-input"
                      style="flex: 1; font-size: 11px;"
                      placeholder="Display name (auto-derived from URL)"
                      value={repo.name ?? ""}
                      oninput={(e) => { defaultRepositories[i].name = e.currentTarget.value || null; }}
                    />
                    <input
                      type="text"
                      class="settings-input"
                      style="flex: 2; font-size: 11px;"
                      placeholder="Description (shown in Repositories tab and to Claude)"
                      value={repo.description ?? ""}
                      oninput={(e) => { defaultRepositories[i].description = e.currentTarget.value || null; }}
                    />
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <!-- Extensions tab -->
      {#if activeTab === "extensions"}
        <div class="settings-panel settings-panel--wide">
          <div class="settings-panel-header">
            <h2 class="settings-panel-title">Extensions</h2>
            <p class="settings-panel-desc">Enable integrations and rendering extensions.</p>
            {#if storageName}<p class="settings-storage-hint">Configured per storage. Currently editing: <strong>{storageName}</strong></p>{/if}
          </div>

          <div class="extensions-grid">
          <div class="extension-card">
            <div class="extension-card__header">
              <div class="extension-card__info">
                <span class="extension-card__name">Mermaid Diagrams</span>
                <span class="extension-card__badge">Rendering</span>
              </div>
              <label class="settings-toggle">
                <input type="checkbox" bind:checked={mermaidDiagrams} />
              </label>
            </div>
            <div class="extension-card__desc">
              Render Mermaid diagram code blocks as interactive SVG diagrams in markdown previews — notes, context, and file previews. Supports flowcharts, sequence diagrams, class diagrams, and more.
            </div>
            <div class="extension-card__hint">
              Use <code>```mermaid</code> fenced code blocks in any markdown content.
            </div>
          </div>

          <div class="extension-card">
            <div class="extension-card__header">
              <div class="extension-card__info">
                <span class="extension-card__name">OpenFGA Highlighting</span>
                <span class="extension-card__badge">Rendering</span>
              </div>
              <label class="settings-toggle">
                <input type="checkbox" bind:checked={openfgaHighlighting} />
              </label>
            </div>
            <div class="extension-card__desc">
              Syntax highlight OpenFGA authorization model files (.fga, .openfga) in file previews, and render <code>```openfga</code> / <code>```fga</code> fenced code blocks with highlighting in markdown content.
            </div>
            <div class="extension-card__hint">
              Keywords, types, relations, operators, and comments are color-coded for readability.
            </div>
          </div>
          </div>
          <InstalledExtensionsPanel />
        </div>
      {/if}

      <!-- Skills tab -->
      {#if activeTab === "skills"}
        <div class="settings-panel settings-panel--wide">
          <div class="settings-panel-header">
            <h2 class="settings-panel-title">Skills</h2>
            <p class="settings-panel-desc">Reusable instruction sets that can be enabled per-feature. When enabled, skill content is injected into Claude sessions as guidelines.</p>
            {#if storageName}<p class="settings-storage-hint">Configured per storage. Currently editing: <strong>{storageName}</strong></p>{/if}
          </div>

          <div style="display: flex; flex-direction: column; gap: 12px;">
            {#each skills as skill, si (skill.id || si)}
              <div class="skill-card">
                <div class="skill-card__header">
                  <div style="display: flex; flex-direction: column; gap: 6px; flex: 1; min-width: 0;">
                    <div style="display: flex; gap: 8px; align-items: center;">
                      <input
                        class="form-input"
                        style="flex: 0 0 160px; font-size: 12px; font-family: var(--font-mono); padding: 4px 8px;"
                        placeholder="skill-id"
                        value={skill.id}
                        oninput={(e) => { skills[si].id = (e.target as HTMLInputElement).value; }}
                      />
                      <input
                        class="form-input"
                        style="flex: 1; font-size: 13px; padding: 4px 8px;"
                        placeholder="Skill Name"
                        value={skill.name}
                        oninput={(e) => { skills[si].name = (e.target as HTMLInputElement).value; }}
                      />
                      <label class="settings-toggle" title="Enabled by default for new features">
                        <input type="checkbox" bind:checked={skills[si].default_enabled} />
                      </label>
                      <button
                        class="skill-remove-btn"
                        onclick={() => { skills = skills.filter((_, i) => i !== si); }}
                        title="Remove skill"
                      >
                        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.749.749 0 0 1 1.275.326.749.749 0 0 1-.215.734L9.06 8l3.22 3.22a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215L8 9.06l-3.22 3.22a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
                      </button>
                    </div>
                  </div>
                </div>
                <textarea
                  class="form-input skill-content-editor"
                  placeholder="Skill instructions in Markdown...&#10;&#10;These instructions are injected into Claude sessions when this skill is enabled for a feature."
                  value={skill.content}
                  oninput={(e) => { skills[si].content = (e.target as HTMLTextAreaElement).value; }}
                  rows="6"
                ></textarea>
              </div>
            {/each}

            <button
              class="add-skill-btn"
              onclick={() => {
                const id = `skill-${Date.now().toString(36)}`;
                skills = [...skills, { id, name: "", content: "", default_enabled: true }];
              }}
            >
              <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M7.75 2a.75.75 0 01.75.75V7h4.25a.75.75 0 010 1.5H8.5v4.25a.75.75 0 01-1.5 0V8.5H2.75a.75.75 0 010-1.5H7V2.75A.75.75 0 017.75 2z"/></svg>
              Add Skill
            </button>

            {#if skills.length === 0}
              <div class="skill-empty">
                <p>No skills configured yet.</p>
                <p style="font-size: 11px; margin-top: 4px;">Skills are reusable instruction sets (coding standards, review guidelines, workflow rules) that get injected into Claude sessions. You can also ask Claude to create skills via the <code>create_skill</code> MCP tool.</p>
              </div>
            {/if}
          </div>
        </div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .settings-fullscreen {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: flex;
    background: var(--bg-primary);
    animation: fadeIn 0.15s ease;
  }

  .settings-sidebar {
    width: 230px;
    flex-shrink: 0;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
  }

  .settings-sidebar-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-4) var(--space-4) var(--space-3);
    border-bottom: 1px solid var(--border);
    background: none;
    border-left: none;
    border-right: none;
    border-top: none;
    color: var(--text-secondary);
    cursor: pointer;
    width: 100%;
    transition: all var(--transition-fast);
  }

  .settings-sidebar-header:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .settings-sidebar-title {
    font-size: var(--text-md);
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.02em;
  }

  .settings-nav {
    flex: 1;
    padding: var(--space-2);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .settings-nav-item {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: 9px var(--space-3);
    border: none;
    background: none;
    color: var(--text-secondary);
    font-size: var(--text-base);
    font-weight: 500;
    border-radius: var(--radius);
    cursor: pointer;
    transition: all var(--transition-fast);
    text-align: left;
    width: 100%;
  }

  .settings-nav-item:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .settings-nav-item.active {
    background: var(--accent-dim);
    color: var(--accent);
    font-weight: 600;
  }

  .settings-sidebar-footer {
    padding: var(--space-3);
    border-top: 1px solid var(--border);
  }

  .settings-save-status {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    font-size: 11px;
    color: var(--text-muted);
    padding: var(--space-1) 0;
  }

  .settings-main {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-8) var(--space-12);
  }

  .settings-loading {
    font-size: var(--text-base);
    color: var(--text-muted);
    padding: var(--space-5) 0;
  }

  .settings-panel {
    max-width: 640px;
  }

  .settings-panel--wide {
    max-width: none;
  }

  .extensions-grid {
    column-count: 2;
    column-gap: var(--space-3);
  }

  .settings-panel-header {
    margin-bottom: var(--space-6);
  }

  .settings-panel-title {
    font-size: var(--text-xl);
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.02em;
    margin: 0 0 var(--space-1);
  }

  .settings-panel-desc {
    font-size: var(--text-sm);
    color: var(--text-muted);
    margin: 0;
    line-height: 1.5;
  }

  .settings-storage-hint {
    font-size: 11px;
    color: var(--text-muted);
    margin: 6px 0 0;
    padding: 4px 8px;
    background: var(--bg-hover);
    border-radius: 4px;
    display: inline-block;
  }

  .settings-storage-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--purple);
    display: inline-block;
    margin-left: 4px;
    flex-shrink: 0;
  }

  .settings-section {
    margin-bottom: var(--space-6);
  }

  .settings-field-desc {
    font-size: 11.5px;
    color: var(--text-muted);
    margin-bottom: var(--space-2);
  }

  .settings-field-desc code {
    font-family: var(--font-mono);
    background: var(--bg-hover);
    padding: 1px 4px;
    border-radius: var(--radius-sm);
  }

  .settings-field-hint {
    font-size: 11px;
    color: var(--text-muted);
    margin-top: var(--space-2);
  }

  .settings-field-hint code {
    font-family: var(--font-mono);
    background: var(--bg-hover);
    padding: 1px 4px;
    border-radius: var(--radius-sm);
  }

  .settings-empty {
    font-size: var(--text-sm);
    color: var(--text-muted);
    padding: var(--space-4);
    border: 1px dashed var(--border);
    border-radius: var(--radius);
    text-align: center;
  }

  .settings-section-title {
    font-size: var(--text-base);
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 var(--space-2) 0;
  }

  .settings-font-preview {
    margin-top: var(--space-2);
    padding: var(--space-3) var(--space-4);
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    font-size: var(--text-base);
    color: var(--text-secondary);
    line-height: 1.6;
  }

  .settings-size-row {
    display: flex;
    gap: 16px;
  }

  .settings-toggle {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    cursor: pointer;
  }

  .settings-toggle input[type="checkbox"] {
    width: 16px;
    height: 16px;
    accent-color: var(--accent);
    cursor: pointer;
  }

  .settings-toggle-label {
    font-size: var(--text-base);
    color: var(--text-primary);
  }

  .extension-card {
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
    background: var(--bg-secondary);
    transition: all var(--transition-fast);
    break-inside: avoid;
    margin-bottom: var(--space-3);
  }

  .extension-card:hover {
    border-color: var(--border-strong);
  }

  .extension-card__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }

  .extension-card__info {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .extension-card__name {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .extension-card__badge {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--purple);
    background: color-mix(in srgb, var(--purple) 15%, transparent);
    padding: 2px 7px;
    border-radius: 4px;
  }

  .ext-field-copy-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    flex-shrink: 0;
    border: 1px solid var(--border);
    border-radius: 5px;
    background: none;
    color: var(--text-muted);
    cursor: pointer;
    transition: border-color 0.15s, color 0.15s, background 0.15s;
  }

  .ext-field-copy-btn:hover {
    border-color: var(--purple);
    color: var(--purple);
    background: color-mix(in srgb, var(--purple) 8%, transparent);
  }

  .extension-card__desc {
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.5;
    margin-bottom: 8px;
  }

  .extension-card__hint {
    font-size: 11px;
    color: var(--text-muted);
  }

  .extension-auth-hint {
    display: flex;
    gap: var(--space-2);
    font-size: 11.5px;
    line-height: 1.55;
    color: var(--text-secondary);
    background: color-mix(in srgb, var(--amber) 8%, transparent);
    border: 1px solid color-mix(in srgb, var(--amber) 25%, transparent);
    border-radius: var(--radius-md);
    padding: var(--space-3);
  }

  .extension-card__hint code {
    font-family: var(--font-mono);
    background: var(--bg-hover);
    padding: 1px 4px;
    border-radius: var(--radius-sm);
  }

  .mcp-server-managed {
    border-color: color-mix(in srgb, var(--purple) 30%, transparent);
    background: color-mix(in srgb, var(--purple) 4%, var(--bg-secondary));
  }

  .mcp-managed-badge {
    font-size: 10px;
    font-weight: 600;
    color: var(--purple);
    background: color-mix(in srgb, var(--purple) 15%, transparent);
    padding: 2px 7px;
    border-radius: 4px;
    white-space: nowrap;
  }

  .extension-card--enabled {
    border-color: color-mix(in srgb, var(--purple) 40%, transparent);
  }

  .extension-card__body {
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .settings-ide-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 8px;
    border-radius: 5px;
    cursor: pointer;
    transition: background 0.08s;
  }
  .settings-ide-item:hover {
    background: var(--bg-hover);
  }
  .settings-ide-item input[type="checkbox"] {
    width: 15px;
    height: 15px;
    accent-color: var(--accent);
    cursor: pointer;
    flex-shrink: 0;
  }

  /* Skills */
  .skill-card {
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg-card);
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .skill-card__header {
    display: flex;
    align-items: flex-start;
    gap: 8px;
  }
  .skill-content-editor {
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.5;
    resize: vertical;
    min-height: 80px;
    padding: 8px;
    border-radius: 6px;
  }
  .skill-remove-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 6px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    flex-shrink: 0;
  }
  .skill-remove-btn:hover {
    background: var(--bg-hover);
    color: var(--red);
    border-color: var(--red);
  }
  .add-skill-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 16px;
    border-radius: 6px;
    border: 1px dashed var(--border);
    background: transparent;
    color: var(--text-muted);
    font-family: inherit;
    font-size: 13px;
    cursor: pointer;
    transition: all 0.15s;
    align-self: flex-start;
  }
  .add-skill-btn:hover {
    background: var(--bg-hover);
    border-color: var(--purple);
    color: var(--purple);
  }
  .skill-empty {
    font-size: 12px;
    color: var(--text-muted);
    padding: 16px;
    text-align: center;
  }
  .skill-empty code {
    font-family: var(--font-mono);
    background: var(--bg-hover);
    padding: 1px 5px;
    border-radius: 3px;
    font-size: 11px;
  }
</style>
