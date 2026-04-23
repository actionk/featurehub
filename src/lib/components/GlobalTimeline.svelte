<script lang="ts">
  import { getGlobalTimeline, type GlobalTimelineEvent } from "../api/timeline";
  import { formatDate, eventColor } from "../utils/format";

  let events = $state<GlobalTimelineEvent[]>([]);
  let loading = $state(true);

  async function load() {
    loading = true;
    try {
      events = await getGlobalTimeline();
    } catch (e) {
      console.error("Failed to load global timeline:", e);
      events = [];
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    load();
    const interval = setInterval(load, 30_000);
    return () => clearInterval(interval);
  });

  const eventMeta: Record<string, { color: string; verb: string }> = {
    feature_created: { color: "var(--green)",      verb: "Created"  },
    link_added:      { color: "var(--accent)",      verb: "Link"     },
    session_linked:  { color: "#a78bfa",            verb: "Session"  },
    task_added:      { color: "var(--amber)",       verb: "Task"     },
    task_completed:  { color: "var(--green)",       verb: "Done"     },
    note_updated:    { color: "var(--text-muted)",  verb: "Note"     },
    file_added:      { color: "#22d3ee",            verb: "File"     },
  };

  function getMeta(type: string) {
    return eventMeta[type] ?? { color: "var(--text-muted)", verb: type };
  }

  function formatTime(iso: string): string {
    return new Date(iso).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  }

  function isToday(key: string): boolean {
    const now = new Date();
    return key === `${now.getFullYear()}-${String(now.getMonth()+1).padStart(2,'0')}-${String(now.getDate()).padStart(2,'0')}`;
  }

  function isYesterday(key: string): boolean {
    const y = new Date(); y.setDate(y.getDate() - 1);
    return key === `${y.getFullYear()}-${String(y.getMonth()+1).padStart(2,'0')}-${String(y.getDate()).padStart(2,'0')}`;
  }

  function dateLabel(key: string): string {
    if (isToday(key)) return "Today";
    if (isYesterday(key)) return "Yesterday";
    return formatDate(key + "T00:00:00");
  }

  function groupByDate(items: GlobalTimelineEvent[]): { date: string; events: GlobalTimelineEvent[] }[] {
    const groups = new Map<string, GlobalTimelineEvent[]>();
    for (const ev of items) {
      const d = new Date(ev.timestamp);
      const key = `${d.getFullYear()}-${String(d.getMonth()+1).padStart(2,'0')}-${String(d.getDate()).padStart(2,'0')}`;
      if (!groups.has(key)) groups.set(key, []);
      groups.get(key)!.push(ev);
    }
    return Array.from(groups.entries()).map(([date, events]) => ({ date, events }));
  }

  let grouped = $derived(groupByDate(events));
</script>

<div class="gtl-root">
  <div class="gtl-header">
    <h2 class="gtl-title">Timeline</h2>
    <span class="gtl-subtitle">Activity across all features</span>
  </div>

  <div class="gtl-body timeline">
    {#if loading && events.length === 0}
      <div class="gtl-empty">Loading…</div>
    {:else if events.length === 0}
      <div class="gtl-empty">No activity yet</div>
    {:else}
      {#each grouped as group, gi}
        <div class="gtl-day" class:gtl-day--first={gi === 0}>
          <div class="gtl-day-header">
            <span class="gtl-day-label">{dateLabel(group.date)}</span>
            <span class="gtl-day-line"></span>
            <span class="gtl-day-count">{group.events.length}</span>
          </div>
          {#each group.events as ev}
            {@const meta = getMeta(ev.event_type)}
            {@const evColor = eventColor(ev.event_type)}
            <div class="gtl-row timeline__event glass-panel--soft">
              <span class="gtl-time timeline__meta">{formatTime(ev.timestamp)}</span>
              <span class="gtl-dot timeline__dot live-dot live-dot--static" style="background: {evColor}; color: {evColor};"></span>
              <span class="gtl-verb" style="color: {meta.color};">{meta.verb}</span>
              <span class="gtl-title-text timeline__title">{ev.title}</span>
              <span class="gtl-feature-badge aurora-pill aurora-pill--muted aurora-pill--sm aurora-pill--no-dot">{ev.feature_title}</span>
              {#if ev.detail && ev.event_type !== "note_updated" && ev.detail !== ev.title}
                <span class="gtl-detail timeline__meta">{ev.detail}</span>
              {/if}
            </div>
          {/each}
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .gtl-root {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
    overflow: hidden;
  }

  .gtl-header {
    padding: var(--space-5) var(--space-6) var(--space-4);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    display: flex;
    align-items: baseline;
    gap: var(--space-3);
  }

  .gtl-title {
    font-size: var(--text-xl);
    font-weight: 700;
    letter-spacing: -0.03em;
    color: var(--text-primary);
    margin: 0;
  }

  .gtl-subtitle {
    font-size: var(--text-sm);
    color: var(--text-muted);
  }

  .gtl-body {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-4) var(--space-6) var(--space-6);
  }

  .gtl-empty {
    font-size: var(--text-sm);
    color: var(--text-muted);
    text-align: center;
    padding: var(--space-6) 0;
  }

  .gtl-day {
    margin-top: var(--space-4);
  }

  .gtl-day--first {
    margin-top: 0;
  }

  .gtl-day-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-1);
  }

  .gtl-day-label {
    font-size: 11px;
    font-weight: 700;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    white-space: nowrap;
  }

  .gtl-day-line {
    flex: 1;
    height: 1px;
    background: var(--border);
  }

  .gtl-day-count {
    font-size: 10px;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  .gtl-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 3px 0;
    min-height: 26px;
    border-radius: var(--radius-sm);
  }

  .gtl-row:hover {
    background: var(--bg-hover);
    margin: 0 calc(-1 * var(--space-2));
    padding-left: var(--space-2);
    padding-right: var(--space-2);
  }

  .gtl-time {
    font-size: 10px;
    color: var(--text-muted);
    font-family: var(--font-mono);
    width: 42px;
    flex-shrink: 0;
    text-align: right;
  }

  .gtl-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .gtl-verb {
    font-size: 11px;
    font-weight: 600;
    flex-shrink: 0;
    min-width: 52px;
  }

  .gtl-title-text {
    font-size: var(--text-sm);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
    flex: 1;
  }

  .gtl-feature-badge {
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--accent);
    background: var(--accent-dim);
    border: 1px solid rgba(77,124,255,0.18);
    border-radius: var(--radius-full);
    padding: 1px 7px;
    white-space: nowrap;
    flex-shrink: 0;
    max-width: 180px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .gtl-detail {
    font-size: 10.5px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-shrink: 1;
    min-width: 0;
  }
</style>
