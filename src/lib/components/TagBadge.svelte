<script lang="ts">
  import type { Tag } from "../api/tauri";
  import { hexToAlpha } from "../utils/format";

  let {
    tag,
    removable = false,
    onRemove,
  }: {
    tag: Tag;
    removable?: boolean;
    onRemove?: (tag: Tag) => void;
  } = $props();

  const style = $derived(
    tag.color
      ? `background: ${hexToAlpha(tag.color, 0.15)}; color: ${tag.color}; border-color: ${hexToAlpha(tag.color, 0.4)}; box-shadow: 0 0 12px ${hexToAlpha(tag.color, 0.3)};`
      : ''
  );
</script>

<span class="aurora-pill aurora-pill--no-dot" {style}>
  {tag.name}
  {#if removable && onRemove}
    <button
      style="margin-left: 3px; cursor: pointer; background: none; border: none; color: inherit; padding: 0; line-height: 1; opacity: 0.6; font-size: 10px;"
      onclick={() => onRemove?.(tag)}
      aria-label="Remove tag {tag.name}"
    >
      ✕
    </button>
  {/if}
</span>
