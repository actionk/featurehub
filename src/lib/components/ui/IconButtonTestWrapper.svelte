<script lang="ts">
  import IconButton from "./IconButton.svelte";

  let { onclick, title, variant, size, disabled, label = "X" }: {
    onclick?: (e: MouseEvent) => void;
    title?: string;
    variant?: "ghost" | "subtle" | "accent";
    size?: "xs" | "sm" | "md";
    disabled?: boolean;
    label?: string;
  } = $props();
</script>

<IconButton {onclick} {title} {variant} {size} {disabled}>
  {label}
</IconButton>
