<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    open = $bindable(false),
    align = "left",
    children,
    trigger,
  }: {
    open: boolean;
    align?: "left" | "right";
    children: Snippet;
    trigger: Snippet;
  } = $props();

  let wrapperEl: HTMLDivElement | undefined = $state();

  $effect(() => {
    if (!open) return;
    function handleClick(e: MouseEvent) {
      if (wrapperEl && !wrapperEl.contains(e.target as Node)) {
        open = false;
      }
    }
    // Delay to skip the opening click
    const timer = setTimeout(() => {
      document.addEventListener("click", handleClick, true);
    }, 0);
    return () => {
      clearTimeout(timer);
      document.removeEventListener("click", handleClick, true);
    };
  });
</script>

<div class="dropdown-wrapper" bind:this={wrapperEl}>
  {@render trigger()}
  {#if open}
    <div class="dropdown-panel glass-panel dropdown__panel" style="{align === 'right' ? 'right: 0; left: auto;' : ''}">
      {@render children()}
    </div>
  {/if}
</div>

<style>
  .dropdown-wrapper {
    position: relative;
  }

  .dropdown-panel {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: var(--space-1);
    padding: var(--space-1);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border-strong);
    background: var(--bg-card);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    box-shadow: var(--shadow-lg);
    z-index: 20;
    min-width: 140px;
    animation: dropdown-in var(--transition-fast) ease;
  }

  @keyframes dropdown-in {
    from { opacity: 0; transform: translateY(-4px) scale(0.97); }
    to   { opacity: 1; transform: translateY(0) scale(1); }
  }
</style>
