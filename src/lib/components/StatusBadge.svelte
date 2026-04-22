<script lang="ts">
  let { status }: { status: string } = $props();

  const statusConfig: Record<string, { label: string; color: string; bg: string }> = {
    todo: { label: "Todo", color: "var(--text-muted)", bg: "rgba(85, 85, 106, 0.15)" },
    in_progress: { label: "In Progress", color: "var(--amber)", bg: "var(--amber-dim)" },
    in_review: { label: "In Review", color: "var(--blue)", bg: "var(--blue-dim)" },
    done: { label: "Done", color: "var(--green)", bg: "var(--green-dim)" },
  };

  let config = $derived(statusConfig[status] ?? { label: status, color: "var(--text-muted)", bg: "rgba(85, 85, 106, 0.15)" });
</script>

<span class="status-badge" style="background: {config.bg}; color: {config.color};">
  {config.label}
</span>
