<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    onclick,
    title = "",
    variant = "ghost",
    size = "sm",
    disabled = false,
    children,
  }: {
    onclick?: (e: MouseEvent) => void;
    title?: string;
    variant?: "ghost" | "subtle" | "accent";
    size?: "xs" | "sm" | "md";
    disabled?: boolean;
    children: Snippet;
  } = $props();

  const sizeClass = {
    xs: "icon-btn--xs",
    sm: "icon-btn--sm",
    md: "icon-btn--md",
  };

  const variantClass = {
    ghost: "icon-btn--ghost",
    subtle: "icon-btn--subtle",
    accent: "icon-btn--accent",
  };
</script>

<button
  class="icon-btn {sizeClass[size]} {variantClass[variant]}"
  {onclick}
  {title}
  {disabled}
>
  {@render children()}
</button>

<style>
  .icon-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border: none;
    cursor: pointer;
    transition: all var(--transition-fast);
    flex-shrink: 0;
    border-radius: var(--radius-md);
  }

  .icon-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  /* Sizes */
  .icon-btn--xs { width: 24px; height: 24px; border-radius: var(--radius-sm); }
  .icon-btn--sm { width: 28px; height: 28px; }
  .icon-btn--md { width: 32px; height: 32px; }

  /* Variants */
  .icon-btn--ghost {
    background: transparent;
    color: var(--text-muted);
  }
  .icon-btn--ghost:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-secondary);
  }

  .icon-btn--subtle {
    background: var(--bg-card);
    border: 1px solid var(--border);
    color: var(--text-secondary);
  }
  .icon-btn--subtle:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-primary);
    border-color: var(--border-strong);
  }

  .icon-btn--accent {
    background: var(--accent-dim);
    color: var(--accent);
  }
  .icon-btn--accent:hover:not(:disabled) {
    background: rgba(124,124,255,0.22);
    color: var(--accent-hover);
  }
</style>
