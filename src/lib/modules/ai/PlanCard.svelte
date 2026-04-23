<script lang="ts">
  import type { Plan } from "../../api/tauri";
  import { formatRelativeTime } from "../../utils/format";

  let {
    plan,
    selected = false,
    onSelect,
  }: {
    plan: Plan;
    selected?: boolean;
    onSelect?: (plan: Plan) => void;
  } = $props();

  function statusVariant(status: string): string {
    if (status === "approved") return "aurora-pill--success";
    if (status === "rejected") return "aurora-pill--danger";
    return "aurora-pill--warn";
  }

  function statusLabel(status: string): string {
    if (status === "approved") return "Approved";
    if (status === "rejected") return "Rejected";
    return "Pending Review";
  }
</script>

<button
  class="plan-card glass-panel glass-panel--hover {selected ? 'plan-card--selected' : ''} {plan.status === 'pending' ? 'plan-card--pending' : ''}"
  onclick={() => onSelect?.(plan)}
>
  <div class="plan-card-header">
    <span class="plan-card-status aurora-pill aurora-pill--sm {statusVariant(plan.status)}">
      {statusLabel(plan.status)}
    </span>
    <span class="plan-card-time">{formatRelativeTime(plan.created_at)}</span>
  </div>
  <div class="plan-card-title">{plan.title}</div>
</button>

<style>
  .plan-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-card);
    cursor: pointer;
    text-align: left;
    width: 100%;
    transition: border-color var(--transition-fast), background var(--transition-fast);
  }
  .plan-card:hover {
    border-color: var(--border-strong);
    background: var(--bg-hover);
  }
  .plan-card--selected {
    border-color: var(--accent);
  }
  .plan-card--pending {
    border-left: 3px solid var(--amber);
  }
  .plan-card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
  }
  .plan-card-status {
    font-size: 10px;
    font-weight: 600;
    padding: 1px 6px;
    border-radius: var(--radius-sm);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }
  .plan-card-time {
    font-size: 10px;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }
  .plan-card-title {
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--text-primary);
    line-height: 1.4;
  }
</style>