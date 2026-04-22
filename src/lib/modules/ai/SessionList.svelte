<script lang="ts">
  import type { Session } from "../../api/tauri";
  import { scanSessions, getSessions, getFhCliPath } from "../../api/tauri";
  import SessionCard from "./SessionCard.svelte";
  import { subscribe } from "../../stores/events.svelte";

  let {
    featureId,
    featureTitle = "",
    sessions,
    onSessionsChanged,
    onStartSession,
    onResumeSession,
    launching = false,
  }: {
    featureId: string;
    featureTitle?: string;
    sessions: Session[];
    onSessionsChanged?: () => void;
    onStartSession?: () => void;
    onResumeSession?: (session: Session) => void;
    launching?: boolean;
  } = $props();

  let scanning = $state(false);
  let copiedStart = $state(false);

  async function handleCopyStartCommand() {
    try {
      const fhPath = await getFhCliPath();
      const quoted = fhPath.includes(" ") ? `"${fhPath}"` : fhPath;
      const cmd = `${quoted} start "${featureTitle}"`;
      await navigator.clipboard.writeText(cmd);
      copiedStart = true;
      setTimeout(() => { copiedStart = false; }, 2000);
    } catch (err) {
      console.error("Failed to copy start command:", err);
    }
  }

  // Auto-poll for new sessions (picks up CLI-created sessions)
  $effect(() => {
    const interval = setInterval(async () => {
      try {
        const fresh = await getSessions(featureId);
        // Only trigger refresh if count changed
        if (fresh.length !== sessions.length) {
          onSessionsChanged?.();
        } else {
          // Check if any session was updated (e.g. ended_at filled in)
          const changed = fresh.some((s, i) => {
            const old = sessions[i];
            if (!old) return true;
            return s.claude_session_id !== old.claude_session_id
              || s.title !== old.title
              || s.ended_at !== old.ended_at;
          });
          if (changed) onSessionsChanged?.();
        }
      } catch {}
    }, 15_000);
    return () => clearInterval(interval);
  });

  async function handleScan() {
    scanning = true;
    try {
      await scanSessions(featureId);
      onSessionsChanged?.();
    } catch (e) {
      console.error("Failed to scan sessions:", e);
    } finally {
      scanning = false;
    }
  }

  let activeSessions = $derived(sessions.filter(s => !s.ended_at));
  let pastSessions = $derived(sessions.filter(s => s.ended_at));
</script>

<div class="sl">
  <div class="sl__header">
    <svg class="sl__icon" width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M0 2.75C0 1.784.784 1 1.75 1h12.5c.966 0 1.75.784 1.75 1.75v10.5A1.75 1.75 0 0114.25 15H1.75A1.75 1.75 0 010 13.25zm1.75-.25a.25.25 0 00-.25.25v10.5c0 .138.112.25.25.25h12.5a.25.25 0 00.25-.25V2.75a.25.25 0 00-.25-.25zM3.5 6a.75.75 0 01.22-.53l2-2a.75.75 0 011.06 1.06L5.56 5.75l1.22 1.22a.75.75 0 01-1.06 1.06l-2-2A.75.75 0 013.5 6zm4.75.75a.75.75 0 000 1.5h2.5a.75.75 0 000-1.5z"/></svg>
    <span class="sl__title">Sessions</span>
    {#if sessions.length > 0}
      <span class="sl__count">{sessions.length}</span>
    {/if}
    <button class="sl__new-btn" onclick={onStartSession} disabled={launching}>
      {#if launching}
        Starting...
      {:else}
        + New Terminal
      {/if}
    </button>
    <button class="sl__copy-btn" onclick={handleCopyStartCommand} title="Copy start command to clipboard">
      {#if copiedStart}
        <svg width="10" height="10" viewBox="0 0 16 16" fill="var(--green)"><path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 0 1 1.06-1.06L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/></svg>
      {:else}
        <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M0 6.75C0 5.784.784 5 1.75 5h1.5a.75.75 0 0 1 0 1.5h-1.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-1.5a.75.75 0 0 1 1.5 0v1.5A1.75 1.75 0 0 1 9.25 16h-7.5A1.75 1.75 0 0 1 0 14.25ZM5 1.75C5 .784 5.784 0 6.75 0h7.5C15.216 0 16 .784 16 1.75v7.5A1.75 1.75 0 0 1 14.25 11h-7.5A1.75 1.75 0 0 1 5 9.25Zm1.75-.25a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25Z"/></svg>
      {/if}
    </button>
    <div style="flex: 1;"></div>
    {#if sessions.length > 0}
      <button class="sl__action" onclick={handleScan} disabled={scanning}>
        <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor" style="opacity: 0.7;"><path d="M8 2.5a5.487 5.487 0 00-4.131 1.869l1.204 1.204A.25.25 0 014.896 6H1.25A.25.25 0 011 5.75V2.104a.25.25 0 01.427-.177l1.38 1.38A7.001 7.001 0 0115 8a.75.75 0 01-1.5 0A5.5 5.5 0 008 2.5zM1.75 8a.75.75 0 01.75.75 5.5 5.5 0 009.131 4.131l-1.204-1.204A.25.25 0 0110.604 11h3.646a.25.25 0 01.25.25v3.646a.25.25 0 01-.427.177l-1.38-1.38A7.001 7.001 0 011 8.75.75.75 0 011.75 8z"/></svg>
        {scanning ? "Scanning..." : "Scan"}
      </button>
    {/if}
  </div>

  {#if sessions.length === 0}
    <div class="sl__empty">
      No sessions yet — click <strong>+ New Terminal</strong> to start a Claude Code session
    </div>
  {:else}
    <div class="sl__list">
      {#if activeSessions.length > 0 && pastSessions.length > 0}
        <div class="sl__group-label">
          <span class="sl__group-dot"></span>
          Active
        </div>
      {/if}
      {#each activeSessions as session (session.id)}
        <SessionCard {session} onUnlinked={onSessionsChanged} onRenamed={onSessionsChanged} onResume={onResumeSession} />
      {/each}
      {#if activeSessions.length > 0 && pastSessions.length > 0}
        <div class="sl__group-label">Past</div>
      {/if}
      {#each pastSessions as session (session.id)}
        <SessionCard {session} onUnlinked={onSessionsChanged} onRenamed={onSessionsChanged} onResume={onResumeSession} />
      {/each}
    </div>
  {/if}
</div>

<style>
  .sl {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .sl__header {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .sl__icon {
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .sl__title {
    font-size: 11.5px;
    font-weight: 650;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted);
  }

  .sl__count {
    font-size: 10.5px;
    font-weight: 500;
    color: var(--text-muted);
    font-family: var(--font-mono);
    opacity: 0.7;
  }

  .sl__action {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--text-muted);
    background: none;
    border: none;
    cursor: pointer;
    padding: 3px 8px;
    border-radius: 4px;
    font-family: inherit;
    transition: all 0.1s;
  }
  .sl__action:hover {
    color: var(--text-secondary);
    background: var(--bg-hover);
  }
  .sl__action:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .sl__new-btn {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
    font-weight: 600;
    font-family: inherit;
    color: var(--accent);
    background: color-mix(in srgb, var(--accent) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--accent) 25%, transparent);
    border-radius: 4px;
    padding: 2px 10px;
    cursor: pointer;
    transition: all 0.1s;
  }
  .sl__new-btn:hover {
    background: color-mix(in srgb, var(--accent) 18%, transparent);
    border-color: color-mix(in srgb, var(--accent) 40%, transparent);
  }
  .sl__new-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .sl__copy-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 22px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    transition: all 0.1s;
  }
  .sl__copy-btn:hover {
    color: var(--accent);
    border-color: color-mix(in srgb, var(--accent) 40%, transparent);
    background: color-mix(in srgb, var(--accent) 8%, transparent);
  }

  .sl__empty {
    font-size: 12px;
    color: var(--text-muted);
    padding: 12px 0 4px;
    line-height: 1.5;
  }
  .sl__list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .sl__group-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 10px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding: 4px 2px 0;
  }

  .sl__group-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--green);
    box-shadow: 0 0 5px var(--green);
  }
</style>