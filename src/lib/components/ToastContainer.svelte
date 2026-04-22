<script lang="ts">
  interface Toast {
    id: number;
    message: string;
    featureId: string | null;
    fading: boolean;
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
        class="toast toast--clickable"
        class:toast--fading={toast.fading}
        onclick={() => onClickToast!(toast.featureId!)}
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="flex-shrink:0; color: var(--accent);">
          <circle cx="12" cy="12" r="10"/>
          <path d="M12 16v-4m0-4h.01"/>
        </svg>
        <span class="toast-message">{toast.message}</span>
      </button>
    {:else}
      <div class="toast" class:toast--fading={toast.fading}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="flex-shrink:0; color: var(--accent);">
          <circle cx="12" cy="12" r="10"/>
          <path d="M12 16v-4m0-4h.01"/>
        </svg>
        <span class="toast-message">{toast.message}</span>
      </div>
    {/if}
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    bottom: var(--space-4);
    right: var(--space-4);
    z-index: 200;
    display: flex;
    flex-direction: column-reverse;
    gap: var(--space-2);
    pointer-events: none;
  }

  .toast {
    pointer-events: auto;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: var(--shadow-lg);
    font-size: var(--text-sm);
    color: var(--text-primary);
    max-width: 360px;
    animation: toastIn 0.3s cubic-bezier(0.16, 1, 0.3, 1);
    transition: opacity 0.3s, transform 0.3s;
    backdrop-filter: blur(8px);
  }

  .toast--fading {
    opacity: 0;
    transform: translateX(20px);
  }

  .toast--clickable {
    cursor: pointer;
  }

  .toast--clickable:hover {
    border-color: var(--accent-border);
    background: var(--bg-hover);
  }

  .toast-message {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  @keyframes toastIn {
    from { opacity: 0; transform: translateX(30px) scale(0.95); }
    to { opacity: 1; transform: translateX(0) scale(1); }
  }
</style>
