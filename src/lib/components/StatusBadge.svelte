<script lang="ts">
  let { status }: { status: string } = $props();

  const variantMap: Record<string, string> = {
    done: 'success',
    in_progress: 'warn',
    in_review: 'warn',
    blocked: 'danger',
    paused: 'muted',
    todo: 'muted',
    active: '',
  };

  const variant = $derived(variantMap[status] ?? '');
  const label = $derived(status.replace(/_/g, ' '));
</script>

<span
  class="aurora-pill"
  class:aurora-pill--success={variant === 'success'}
  class:aurora-pill--warn={variant === 'warn'}
  class:aurora-pill--danger={variant === 'danger'}
  class:aurora-pill--muted={variant === 'muted'}
>{label}</span>
