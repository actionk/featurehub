<script lang="ts">
  import type { TimelineEvent } from "../../api/tauri";
  import { getTimeline } from "../../api/tauri";
  import { formatRelativeTime, formatDate, eventColor } from "../../utils/format";
  import type { TabContext } from "../registry";

  let { featureId }: TabContext = $props();

  let events = $state<TimelineEvent[]>([]);
  let loading = $state(true);

  async function loadTimeline() {
    loading = true;
    try {
      events = await getTimeline(featureId);
    } catch (e) {
      console.error("Failed to load timeline:", e);
      events = [];
    } finally {
      loading = false;
    }
  }

  // Load + poll, reactive to featureId changes
  $effect(() => {
    featureId; // track dependency
    loadTimeline();
    const interval = setInterval(loadTimeline, 30_000);
    return () => clearInterval(interval);
  });

  const eventMeta: Record<string, { color: string; verb: string }> = {
    feature_created: { color: "var(--green)", verb: "Created" },
    link_added: { color: "var(--blue)", verb: "Link" },
    session_linked: { color: "var(--purple)", verb: "Session" },
    task_added: { color: "var(--amber)", verb: "Task" },
    task_completed: { color: "var(--green)", verb: "Done" },
    note_updated: { color: "var(--text-muted)", verb: "Note" },
    file_added: { color: "var(--cyan)", verb: "File" },
    directory_linked: { color: "var(--amber)", verb: "Dir" },
  };

  function getMeta(type: string) {
    return eventMeta[type] ?? { color: "var(--text-muted)", verb: type };
  }

  function formatTime(iso: string): string {
    const d = new Date(iso);
    return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  }

  function isToday(dateStr: string): boolean {
    const d = new Date(dateStr + "T00:00:00");
    const now = new Date();
    return d.getFullYear() === now.getFullYear() && d.getMonth() === now.getMonth() && d.getDate() === now.getDate();
  }

  function isYesterday(dateStr: string): boolean {
    const d = new Date(dateStr + "T00:00:00");
    const y = new Date();
    y.setDate(y.getDate() - 1);
    return d.getFullYear() === y.getFullYear() && d.getMonth() === y.getMonth() && d.getDate() === y.getDate();
  }

  function dateLabel(dateStr: string): string {
    if (isToday(dateStr)) return "Today";
    if (isYesterday(dateStr)) return "Yesterday";
    return formatDate(dateStr + "T00:00:00");
  }

  // Group events by date
  function groupByDate(items: TimelineEvent[]): { date: string; events: TimelineEvent[] }[] {
    const groups: Map<string, TimelineEvent[]> = new Map();
    for (const ev of items) {
      const d = new Date(ev.timestamp);
      const key = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
      if (!groups.has(key)) groups.set(key, []);
      groups.get(key)!.push(ev);
    }
    return Array.from(groups.entries()).map(([date, events]) => ({ date, events }));
  }

  let grouped = $derived(groupByDate(events));
</script>

{#if loading && events.length === 0}
  <div style="font-size: 12px; color: var(--text-muted); padding: 20px 0; text-align: center;">Loading timeline...</div>
{:else if events.length === 0}
  <div style="font-size: 12px; color: var(--text-muted); padding: 20px 0; text-align: center;">No activity yet</div>
{:else}
  <div class="tl timeline">
    {#each grouped as group, gi}
      <div class="tl-day" class:tl-day--first={gi === 0}>
        <div class="tl-day-header">
          <span class="tl-day-label">{dateLabel(group.date)}</span>
          <span class="tl-day-line"></span>
          <span class="tl-day-count">{group.events.length}</span>
        </div>
        {#each group.events as event}
          {@const meta = getMeta(event.event_type)}
          {@const evColor = eventColor(event.event_type)}
          <div class="tl-row timeline__event glass-panel--soft">
            <span class="tl-time timeline__meta">{formatTime(event.timestamp)}</span>
            <span class="tl-dot timeline__dot live-dot live-dot--static" style="background: {evColor}; color: {evColor};"></span>
            <span class="tl-verb" style="color: {meta.color};">{meta.verb}</span>
            <span class="tl-title timeline__title">{event.title}</span>
            {#if event.detail && event.event_type !== "note_updated" && event.detail !== event.title}
              <span class="tl-detail timeline__meta">{event.detail}</span>
            {/if}
          </div>
        {/each}
      </div>
    {/each}
  </div>
{/if}

<style>
  .tl {
    padding: 0;
  }

  .tl-day {
    margin-top: var(--space-4);
  }

  .tl-day--first {
    margin-top: 0;
  }

  .tl-day-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-1);
  }

  .tl-day-label {
    font-size: 11px;
    font-weight: 700;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    white-space: nowrap;
  }

  .tl-day-line {
    flex: 1;
    height: 1px;
    background: var(--border);
  }

  .tl-day-count {
    font-size: 10px;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  .tl-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 3px 0;
    min-height: 24px;
  }

  .tl-time {
    font-size: 10px;
    color: var(--text-muted);
    font-family: var(--font-mono);
    width: 42px;
    flex-shrink: 0;
    text-align: right;
  }

  .tl-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .tl-verb {
    font-size: 11px;
    font-weight: 600;
    flex-shrink: 0;
    min-width: 52px;
  }

  .tl-title {
    font-size: var(--text-sm);
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }

  .tl-detail {
    font-size: 10.5px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-shrink: 1;
    min-width: 0;
  }
</style>
