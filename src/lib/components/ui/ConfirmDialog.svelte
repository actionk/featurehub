<script lang="ts">
  import type { Snippet } from "svelte";
  import Modal from "./Modal.svelte";

  let {
    open = false,
    title,
    onClose,
    onConfirm,
    confirmLabel = "Confirm",
    cancelLabel = "Cancel",
    variant = "danger",
    children,
    actions,
  }: {
    open: boolean;
    title: string;
    onClose: () => void;
    onConfirm?: () => void;
    confirmLabel?: string;
    cancelLabel?: string;
    variant?: "danger" | "primary";
    children: Snippet;
    actions?: Snippet;
  } = $props();
</script>

<Modal {open} {onClose} width="420px">
  <h2 class="confirm-title">{title}</h2>
  <div class="confirm-body">
    {@render children()}
  </div>
  <div class="confirm-actions">
    {#if actions}
      {@render actions()}
    {:else}
      <button class="btn btn-subtle" style="padding: 7px 16px;" onclick={onClose}>{cancelLabel}</button>
      <button
        class="btn {variant === 'danger' ? 'btn--danger confirm-btn confirm-btn--danger' : 'btn--primary confirm-btn confirm-btn--primary'}"
        onclick={onConfirm}
      >{confirmLabel}</button>
    {/if}
  </div>
</Modal>

<style>
  .confirm-title {
    font-size: var(--text-lg);
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text-primary);
    margin-bottom: var(--space-3);
  }

  .confirm-body {
    font-size: var(--text-base);
    color: var(--text-secondary);
    line-height: 1.5;
    margin-bottom: var(--space-4);
  }

  .confirm-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
  }

  .confirm-btn {
    padding: 7px 18px;
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
    font-weight: 600;
    font-family: inherit;
    cursor: pointer;
    border: none;
    transition: all var(--transition-fast);
  }

  .confirm-btn--danger {
    background: var(--red);
    color: white;
  }
  .confirm-btn--danger:hover {
    background: color-mix(in srgb, var(--red) 80%, white);
  }

  .confirm-btn--primary {
    background: var(--accent);
    color: white;
  }
  .confirm-btn--primary:hover {
    background: var(--accent-hover);
  }
</style>
