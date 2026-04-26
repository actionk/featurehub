<script lang="ts">
  import type { SearchResult, Feature } from "../api/tauri";
  import { globalSearch, getFeatures } from "../api/tauri";

  let {
    onClose,
    onSelect,
  }: {
    onClose: () => void;
    onSelect: (featureId: string, entityType: string) => void;
  } = $props();

  let query = $state("");
  let results = $state<SearchResult[]>([]);
  let recentResults = $state<SearchResult[]>([]);
  let selectedIndex = $state(0);
  let searching = $state(false);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let inputEl: HTMLInputElement | undefined = $state();

  $effect(() => {
    inputEl?.focus();
    loadRecent();
  });

  async function loadRecent() {
    try {
      const features = await getFeatures();
      recentResults = features
        .sort((a, b) => b.updated_at.localeCompare(a.updated_at))
        .slice(0, 10)
        .map((f): SearchResult => ({
          entity_type: "feature",
          entity_id: f.id,
          feature_id: f.id,
          title: f.title,
          snippet: f.status.replace("_", " "),
        }));
    } catch (e) {
      console.error("Failed to load recent features:", e);
    }
  }

  let displayResults = $derived(query.trim() ? results : recentResults);

  let grouped = $derived.by(() => {
    const groups: Record<string, SearchResult[]> = {};
    for (const r of displayResults) {
      const key = r.entity_type;
      if (!groups[key]) groups[key] = [];
      groups[key].push(r);
    }
    return groups;
  });

  let flatResults = $derived(displayResults);

  const groupLabels: Record<string, string> = {
    feature: "Features",
    link: "Links",
    session: "Sessions",
    file: "Files",
    note: "Notes",
  };

  const typeIcons: Record<string, { bg: string; color: string; label: string }> = {
    feature: { bg: "var(--amber-dim)", color: "var(--amber)", label: "F" },
    link: { bg: "var(--blue-dim)", color: "var(--blue)", label: "L" },
    session: { bg: "var(--purple-dim)", color: "var(--purple)", label: "S" },
    file: { bg: "var(--green-dim)", color: "var(--green)", label: "D" },
    note: { bg: "var(--cyan-dim)", color: "var(--cyan)", label: "N" },
  };

  function handleInput() {
    if (debounceTimer) clearTimeout(debounceTimer);
    if (!query.trim()) {
      results = [];
      return;
    }
    debounceTimer = setTimeout(async () => {
      try {
        const r = await globalSearch(query.trim());
        results = r;
        selectedIndex = 0;
      } catch (e) {
        console.error("Search failed:", e);
      }
    }, 200);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
      return;
    }
    if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, flatResults.length - 1);
    }
    if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
    }
    if (e.key === "Enter" && flatResults[selectedIndex]) {
      onSelect(flatResults[selectedIndex].feature_id, flatResults[selectedIndex].entity_type);
      onClose();
    }
  }

  function selectResult(result: SearchResult) {
    onSelect(result.feature_id, result.entity_type);
    onClose();
  }

  let mouseDownOnOverlay = false;
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="search-overlay"
  onmousedown={(e) => { mouseDownOnOverlay = e.target === e.currentTarget; }}
  onclick={(e) => { if (e.target === e.currentTarget && mouseDownOnOverlay) onClose(); }}
  onkeydown={handleKeydown}
>
  <div class="search-box">
    <div class="search-bar input input--search" style="padding: 14px 16px; border-bottom: 1px solid var(--border); border-radius: 0;">
      <span class="search-bar__icon" style="flex-shrink: 0; display: inline-flex;">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M6.5 1a5.5 5.5 0 014.38 8.82l3.65 3.65a.75.75 0 01-1.06 1.06l-3.65-3.65A5.5 5.5 0 116.5 1zm0 1.5a4 4 0 100 8 4 4 0 000-8z"/>
        </svg>
      </span>
      <input
        bind:this={inputEl}
        type="text"
        placeholder="Search features, links, sessions, files..."
        class="search-input"
        bind:value={query}
        oninput={handleInput}
      />
      <span class="kbd search-trigger-key">ESC</span>
    </div>

    <div class="search-results glass-panel">
      {#if query.trim() && displayResults.length === 0}
        <div class="search-empty-state">No results found</div>
      {:else if displayResults.length > 0}
        {#if !query.trim()}
          <div class="search-group-label">
            Recent
          </div>
        {/if}
        {#each Object.entries(grouped) as [type, items]}
          {#if query.trim()}
          <div class="search-group-label">
            {groupLabels[type] ?? type}
          </div>
          {/if}
          {#each items as result}
            {@const globalIdx = displayResults.indexOf(result)}
            {@const icon = typeIcons[type] ?? typeIcons.feature}
            <button
              class="list-row search-result-row search-result-item {globalIdx === selectedIndex ? 'list-row--active search-result-item--selected' : ''}"
              onclick={() => selectResult(result)}
              onmouseenter={() => (selectedIndex = globalIdx)}
            >
              <div class="search-result-icon" style="background: {icon.bg}; color: {icon.color};">
                {icon.label}
              </div>
              <div style="flex: 1; min-width: 0;">
                <div class="search-result-row__title search-result-title">
                  {result.title}
                </div>
                {#if result.snippet}
                  <div class="search-result-row__snippet search-result-snippet">
                    {result.snippet}
                  </div>
                {/if}
              </div>
              <span class="aurora-pill aurora-pill--muted aurora-pill--sm aurora-pill--no-dot search-result-type">{type}</span>
            </button>
          {/each}
        {/each}
      {:else}
        <div class="search-empty-state">
          No features yet
        </div>
      {/if}
    </div>

    <div class="search-footer">
      <span class="search-footer-hint">
        <kbd class="kbd search-footer-key">↑↓</kbd> navigate
      </span>
      <span class="search-footer-hint">
        <kbd class="kbd search-footer-key">↵</kbd> open
      </span>
      <span class="search-footer-hint">
        <kbd class="kbd search-footer-key">Esc</kbd> close
      </span>
    </div>
  </div>
</div>
