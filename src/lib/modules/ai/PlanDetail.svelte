<script lang="ts">
  import type { Plan } from "../../api/tauri";
  import { resolvePlan, deletePlan } from "../../api/tauri";
  import { formatRelativeTime } from "../../utils/format";
  import MarkdownPreview from "../../components/MarkdownPreview.svelte";

  let {
    plan,
    onResolved,
    onClose,
  }: {
    plan: Plan;
    onResolved?: () => void;
    onClose?: () => void;
  } = $props();

  let feedbackText = $state("");
  let resolving = $state(false);

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

  async function handleApprove() {
    resolving = true;
    try {
      await resolvePlan(plan.id, "approved");
      onResolved?.();
    } catch (e) {
      console.error("Failed to approve plan:", e);
    } finally {
      resolving = false;
    }
  }

  async function handleReject() {
    resolving = true;
    try {
      await resolvePlan(plan.id, "rejected", feedbackText.trim() || null);
      onResolved?.();
    } catch (e) {
      console.error("Failed to reject plan:", e);
    } finally {
      resolving = false;
    }
  }

  async function handleDelete() {
    try {
      await deletePlan(plan.id);
      onResolved?.();
    } catch (e) {
      console.error("Failed to delete plan:", e);
    }
  }
</script>

<div class="plan-detail glass-panel">
  <div class="plan-detail-header">
    <button class="btn btn--ghost btn--icon btn--sm" onclick={onClose} title="Back">
      <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M10.3 2.3L4.6 8l5.7 5.7 1.4-1.4L7.4 8l4.3-4.3z"/></svg>
    </button>
    <h2 class="plan-detail-title">{plan.title}</h2>
    <span class="plan-detail-status aurora-pill {statusVariant(plan.status)}">
      {statusLabel(plan.status)}
    </span>
  </div>

  <div class="plan-detail-meta">
    <span>Submitted {formatRelativeTime(plan.created_at)}</span>
    {#if plan.session_id}
      <span>Session: {plan.session_id.slice(0, 8)}...</span>
    {/if}
    {#if plan.resolved_at}
      <span>Resolved {formatRelativeTime(plan.resolved_at)}</span>
    {/if}
  </div>

  <div class="plan-detail-body">
    <MarkdownPreview content={plan.body} />
  </div>

  {#if plan.status === "pending"}
    <div class="plan-detail-feedback">
      <textarea
        class="form-input"
        style="resize: vertical; font-size: 12px; min-height: 40px;"
        rows="2"
        placeholder="Optional feedback for Claude..."
        bind:value={feedbackText}
      ></textarea>
    </div>
    <div class="plan-detail-actions">
      <button class="btn btn--ghost btn--icon btn--sm" style="color: var(--text-muted);" onclick={handleDelete} title="Delete plan">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M5 2V1h6v1h4v1H1V2h4zm1 3v8h1V5H6zm3 0v8h1V5H9zM2 4l1 11h10l1-11H2z"/></svg>
      </button>
      <div style="flex: 1;"></div>
      <button class="btn plan-btn-reject" onclick={handleReject} disabled={resolving}>
        Reject
      </button>
      <button class="btn btn--primary plan-btn-approve" onclick={handleApprove} disabled={resolving}>
        Approve
      </button>
    </div>
  {:else if plan.feedback}
    <div class="plan-detail-resolved-feedback">
      <div style="font-size: 10px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 4px;">
        Feedback
      </div>
      <div style="font-size: 12.5px; color: var(--text-secondary); white-space: pre-wrap;">{plan.feedback}</div>
    </div>
  {/if}
</div>

<style>
  .plan-detail {
    display: flex;
    flex-direction: column;
    gap: 12px;
    height: 100%;
  }
  .plan-detail-header {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }
  .plan-detail-title {
    flex: 1;
    font-size: 16px;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text-primary);
    margin: 0;
  }
  .plan-detail-status {
    font-size: 10px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 4px;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    flex-shrink: 0;
  }
  .plan-detail-meta {
    display: flex;
    gap: 12px;
    font-size: 11px;
    color: var(--text-muted);
    flex-shrink: 0;
  }
  .plan-detail-body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 12px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 6px;
  }
  .plan-detail-feedback {
    flex-shrink: 0;
  }
  .plan-detail-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
    padding-top: 4px;
  }
  .plan-btn-approve {
    width: auto;
    padding: 7px 20px;
    background: var(--green) !important;
    border-color: var(--green) !important;
    color: #fff !important;
  }
  .plan-btn-approve:hover {
    filter: brightness(1.1);
  }
  .plan-btn-reject {
    width: auto;
    padding: 7px 20px;
    background: var(--red) !important;
    border-color: var(--red) !important;
    color: #fff !important;
  }
  .plan-btn-reject:hover {
    filter: brightness(1.1);
  }
  .plan-detail-resolved-feedback {
    flex-shrink: 0;
    padding: 10px 12px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 6px;
  }
</style>