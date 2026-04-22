<script lang="ts">
  import type { Tag } from "../api/tauri";

  let {
    tag,
    removable = false,
    onRemove,
  }: {
    tag: Tag;
    removable?: boolean;
    onRemove?: (tag: Tag) => void;
  } = $props();
</script>

<span class="tag-badge"
  style="background: {tag.color}18; color: {tag.color};"
>
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
