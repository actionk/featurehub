<script lang="ts">
  import type { Feature } from "../api/types";
  import { Chart, registerables } from "chart.js";
  import { onMount } from "svelte";
  Chart.register(...registerables);

  let { features }: { features: Feature[] } = $props();

  // Status groups
  const STATUS_ORDER = ['active', 'in_progress', 'in_review', 'todo', 'blocked', 'paused', 'done'] as const;
  const STATUS_LABELS: Record<string, string> = {
    active: 'Active', in_progress: 'In Progress', in_review: 'In Review',
    todo: 'Todo', blocked: 'Blocked', paused: 'Paused', done: 'Done',
  };
  const STATUS_COLORS: Record<string, string> = {
    active:      '#4d7cff',
    in_progress: '#a78bfa',
    in_review:   '#22d3ee',
    todo:        '#8b93a8',
    blocked:     '#f87171',
    paused:      '#fb923c',
    done:        '#34d399',
  };

  let nonArchived = $derived(features.filter(f => !('archived' in f && f.archived)));

  let statusCounts = $derived(
    STATUS_ORDER.reduce((acc, s) => {
      acc[s] = nonArchived.filter(f => f.status === s).length;
      return acc;
    }, {} as Record<string, number>)
  );

  let totalTasks = $derived(nonArchived.reduce((s, f) => s + (f.task_count_total ?? 0), 0));
  let doneTasks  = $derived(nonArchived.reduce((s, f) => s + (f.task_count_done  ?? 0), 0));

  let recentFeatures = $derived(
    [...nonArchived]
      .sort((a, b) => new Date(b.updated_at ?? 0).getTime() - new Date(a.updated_at ?? 0).getTime())
      .slice(0, 8)
  );

  // Chart
  let chartCanvas = $state<HTMLCanvasElement | undefined>(undefined);
  let chart: Chart | null = null;

  $effect(() => {
    const canvas = chartCanvas;
    if (!canvas) return;
    const counts = STATUS_ORDER.map(s => statusCounts[s]);
    const labels = STATUS_ORDER.map(s => STATUS_LABELS[s]);
    const colors = STATUS_ORDER.map(s => STATUS_COLORS[s]);
    chart = new Chart(canvas, {
      type: 'doughnut',
      data: {
        labels,
        datasets: [{
          data: counts,
          backgroundColor: colors.map(c => c + '99'),
          borderColor: colors,
          borderWidth: 1.5,
          hoverOffset: 4,
        }]
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        cutout: '68%',
        plugins: {
          legend: { display: false },
          tooltip: {
            backgroundColor: 'rgba(22,24,38,0.96)',
            borderColor: 'rgba(255,255,255,0.1)',
            borderWidth: 1,
            titleColor: '#8b93a8',
            bodyColor: '#e2e6f0',
            padding: { x: 10, y: 8 },
            displayColors: false,
            callbacks: {
              label: (item) => `${item.parsed} feature${item.parsed !== 1 ? 's' : ''}`,
            }
          }
        }
      }
    });
    return () => { chart?.destroy(); chart = null; };
  });

  $effect(() => {
    if (!chart) return;
    chart.data.datasets[0].data = STATUS_ORDER.map(s => statusCounts[s]);
    chart.update('active');
  });

  function progressPct(f: Feature): number {
    const total = f.task_count_total ?? 0;
    const done  = f.task_count_done  ?? 0;
    return total > 0 ? Math.round((done / total) * 100) : 0;
  }

  function relativeTime(iso: string | undefined): string {
    if (!iso) return '';
    const ms = Date.now() - new Date(iso).getTime();
    const m = Math.floor(ms / 60000);
    if (m < 1)  return 'just now';
    if (m < 60) return `${m}m ago`;
    const h = Math.floor(m / 60);
    if (h < 24) return `${h}h ago`;
    return `${Math.floor(h / 24)}d ago`;
  }
</script>

<div class="dash-root dashboard">
  <div class="dash-header">
    <h2 class="dash-title">Dashboard</h2>
    <span class="dash-subtitle">{nonArchived.length} features</span>
  </div>

  <div class="dash-body">
    <!-- Summary stat row -->
    <div class="dash-stats dashboard__grid">
      <div class="dash-stat glass-panel dashboard__stat">
        <span class="dash-stat-num dashboard__stat-value">{nonArchived.length}</span>
        <span class="dash-stat-lbl dashboard__stat-label">Features</span>
      </div>
      <div class="dash-stat glass-panel dashboard__stat">
        <span class="dash-stat-num dashboard__stat-value gt gt-s">{statusCounts.active + statusCounts.in_progress + statusCounts.in_review}</span>
        <span class="dash-stat-lbl dashboard__stat-label">In flight</span>
      </div>
      <div class="dash-stat glass-panel dashboard__stat">
        <span class="dash-stat-num dashboard__stat-value" style="color: var(--green)">{statusCounts.done}</span>
        <span class="dash-stat-lbl dashboard__stat-label">Done</span>
      </div>
      <div class="dash-stat glass-panel dashboard__stat">
        <span class="dash-stat-num dashboard__stat-value">{totalTasks > 0 ? `${doneTasks}/${totalTasks}` : '—'}</span>
        <span class="dash-stat-lbl dashboard__stat-label">Tasks done</span>
      </div>
    </div>

    <div class="dash-row">
      <!-- Doughnut chart -->
      <div class="dash-card dash-card--chart glass-panel dashboard__section">
        <div class="dash-card-title dashboard__section-title">Status breakdown</div>
        <div class="dash-chart-wrap">
          <canvas bind:this={chartCanvas} class="dash-chart-canvas"></canvas>
        </div>
        <div class="dash-legend">
          {#each STATUS_ORDER as s}
            {#if statusCounts[s] > 0}
              <div class="dash-legend-item">
                <span class="dash-legend-dot" style="background: {STATUS_COLORS[s]};"></span>
                <span class="dash-legend-label">{STATUS_LABELS[s]}</span>
                <span class="dash-legend-count">{statusCounts[s]}</span>
              </div>
            {/if}
          {/each}
        </div>
      </div>

      <!-- Recent activity -->
      <div class="dash-card dash-card--recent glass-panel dashboard__section">
        <div class="dash-card-title dashboard__section-title">Recently updated</div>
        <div class="dash-recent-list">
          {#each recentFeatures as f}
            <div class="dash-recent-row">
              <span class="dash-recent-dot" style="background: {STATUS_COLORS[f.status] ?? 'var(--text-muted)'};"></span>
              <span class="dash-recent-title">{f.title}</span>
              {#if (f.task_count_total ?? 0) > 0}
                <div class="dash-recent-pmb">
                  <div class="dash-recent-pmb-track">
                    <div class="dash-recent-pmb-fill" style="width: {progressPct(f)}%;"></div>
                  </div>
                </div>
              {/if}
              <span class="dash-recent-time">{relativeTime(f.updated_at)}</span>
            </div>
          {/each}
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .dash-root {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
    overflow: hidden;
  }

  .dash-header {
    padding: var(--space-5) var(--space-6) var(--space-4);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    display: flex;
    align-items: baseline;
    gap: var(--space-3);
  }

  .dash-title {
    font-size: var(--text-xl);
    font-weight: 700;
    letter-spacing: -0.03em;
    color: var(--text-primary);
    margin: 0;
  }

  .dash-subtitle {
    font-size: var(--text-sm);
    color: var(--text-muted);
  }

  .dash-body {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-5) var(--space-6) var(--space-6);
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  /* Stat row */
  .dash-stats {
    display: flex;
    gap: var(--space-3);
  }

  .dash-stat {
    flex: 1;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: var(--space-3) var(--space-4);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .dash-stat-num {
    font-size: var(--text-xl);
    font-weight: 700;
    letter-spacing: -0.04em;
    color: var(--text-primary);
    line-height: 1;
  }

  .dash-stat-lbl {
    font-size: 10px;
    color: var(--text-muted);
  }

  /* Cards row */
  .dash-row {
    display: grid;
    grid-template-columns: 260px 1fr;
    gap: var(--space-4);
    flex: 1;
    min-height: 0;
  }

  .dash-card {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    overflow: hidden;
  }

  .dash-card-title {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--text-secondary);
  }

  /* Chart */
  .dash-chart-wrap {
    height: 160px;
    position: relative;
  }

  .dash-chart-canvas {
    width: 100% !important;
    height: 100% !important;
    display: block;
  }

  .dash-legend {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .dash-legend-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .dash-legend-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .dash-legend-label {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    flex: 1;
  }

  .dash-legend-count {
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--text-muted);
  }

  /* Recent list */
  .dash-recent-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow-y: auto;
  }

  .dash-recent-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 4px var(--space-2);
    border-radius: var(--radius-sm);
    cursor: default;
  }

  .dash-recent-row:hover {
    background: var(--bg-hover);
  }

  .dash-recent-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .dash-recent-title {
    font-size: var(--text-sm);
    color: var(--text-primary);
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }

  .dash-recent-pmb {
    flex-shrink: 0;
  }

  .dash-recent-pmb-track {
    width: 40px;
    height: 3px;
    background: rgba(255,255,255,0.07);
    border-radius: 2px;
    overflow: hidden;
  }

  .dash-recent-pmb-fill {
    height: 100%;
    background: var(--grad-primary);
    border-radius: 2px;
    min-width: 2px;
  }

  .dash-recent-time {
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--text-muted);
    white-space: nowrap;
    flex-shrink: 0;
  }
</style>
