<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    open = false,
    onClose,
    width = "420px",
    children,
  }: {
    open: boolean;
    onClose: () => void;
    width?: string;
    children: Snippet;
  } = $props();

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) onClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal-backdrop" onclick={handleBackdropClick} onkeydown={handleKeydown}>
    <div class="modal-content" style="width: {width};">
      {@render children()}
    </div>
  </div>
{/if}
