<script lang="ts">
  import { fly } from "svelte/transition";

  interface Toast {
    id: number;
    message: string;
    featureId: string | null;
    fading: boolean;
    kind?: "info" | "success" | "warn" | "error";
  }

  let { toasts, onClickToast }: {
    toasts: Toast[];
    onClickToast?: (featureId: string) => void;
  } = $props();
</script>

<div class="toast-container">
  {#each toasts as toast (toast.id)}
    {#if toast.featureId && onClickToast}
      <button
        class="toast glass-panel toast--{toast.kind ?? 'info'} toast--clickable"
        class:toast--fading={toast.fading}
        onclick={() => onClickToast!(toast.featureId!)}
        transition:fly={{ x: 30, duration: 220 }}
      >
        <div class="toast__rail"></div>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="flex-shrink:0; color: var(--accent);">
          <circle cx="12" cy="12" r="10"/>
          <path d="M12 16v-4m0-4h.01"/>
        </svg>
        <span class="toast__body toast-message">{toast.message}</span>
      </button>
    {:else}
      <div
        class="toast glass-panel toast--{toast.kind ?? 'info'}"
        class:toast--fading={toast.fading}
        transition:fly={{ x: 30, duration: 220 }}
      >
        <div class="toast__rail"></div>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="flex-shrink:0; color: var(--accent);">
          <circle cx="12" cy="12" r="10"/>
          <path d="M12 16v-4m0-4h.01"/>
        </svg>
        <span class="toast__body toast-message">{toast.message}</span>
      </div>
    {/if}
  {/each}
</div>

<style>
  .toast--fading {
    opacity: 0;
    transform: translateX(20px);
    transition: opacity 0.3s, transform 0.3s;
  }

  .toast--clickable {
    cursor: pointer;
    text-align: left;
    font: inherit;
  }

  .toast--clickable:hover {
    border-color: var(--accent-border);
    background: var(--bg-hover);
  }

  .toast-message {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
