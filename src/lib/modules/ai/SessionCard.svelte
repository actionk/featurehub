<script lang="ts">
  import type { Session } from "../../api/tauri";
  import { unlinkSession, renameSession, ptyKill, finishEmbeddedSession, getFhCliPath } from "../../api/tauri";
  import { formatRelativeTime, formatDuration, getTimeAge } from "../../utils/format";
  import { getActiveTerminals, removeTerminal } from "../../stores/terminals.svelte";
  import { emit } from "../../stores/events.svelte";
  import { isSessionActive as checkSessionActiveFromStore } from "../../stores/sessionActivity.svelte";

  let {
    session,
    onUnlinked,
    onRenamed,
    onResume,
  }: {
    session: Session;
    onUnlinked?: () => void;
    onRenamed?: () => void;
    onResume?: (session: Session) => void;
  } = $props();

  let isActive = $derived(session.claude_session_id ? checkSessionActiveFromStore(session.claude_session_id) : false);
  let editing = $state(false);
  let editValue = $state("");
  let copied = $state(false);
  let confirmingUnlink = $state(false);

  let hasSessionId = $derived(!!session.claude_session_id && session.claude_session_id.length > 0);
  let isInProgress = $derived(!hasSessionId && session.started_at && !session.ended_at);
  let hasEmbeddedTerminal = $derived(getActiveTerminals().some(t => t.sessionDbId === session.id));
  let isRunningExternally = $derived(isActive && !hasEmbeddedTerminal);

  let displayTitle = $derived(session.title ?? "Untitled Session");

  function handleClick() {
    if (editing || !hasSessionId || isRunningExternally) return;
    onResume?.(session);
  }

  async function handleCopyCommand(e: MouseEvent) {
    e.stopPropagation();
    if (!session.claude_session_id) return;
    try {
      const fhPath = await getFhCliPath();
      const quoted = fhPath.includes(" ") ? `"${fhPath}"` : fhPath;
      const cmd = `${quoted} resume ${session.claude_session_id}`;
      await navigator.clipboard.writeText(cmd);
      copied = true;
      setTimeout(() => { copied = false; }, 2000);
    } catch (err) {
      console.error("Failed to copy command:", err);
    }
  }

  function startEditing(e?: MouseEvent | KeyboardEvent) {
    e?.stopPropagation();
    e?.preventDefault();
    editing = true;
    editValue = session.title ?? "";
  }

  async function commitRename() {
    const trimmed = editValue.trim();
    editing = false;
    if (!trimmed || trimmed === (session.title ?? "")) return;
    try {
      await renameSession(session.id, trimmed);
      onRenamed?.();
    } catch (e) {
      console.error("Failed to rename session:", e);
    }
  }

  function handleEditKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      commitRename();
    } else if (e.key === "Escape") {
      editing = false;
    }
  }

  function handleUnlinkClick(e: MouseEvent) {
    e.stopPropagation();
    confirmingUnlink = true;
  }

  async function handleUnlinkConfirm() {
    confirmingUnlink = false;
    try {
      // Kill embedded terminal if one is running for this session
      const term = getActiveTerminals().find(t => t.sessionDbId === session.id);
      if (term) {
        await ptyKill(term.terminalId).catch(() => {});
        await finishEmbeddedSession(term.sessionDbId).catch(() => {});
        removeTerminal(term.terminalId);
      }
      await unlinkSession(session.id);
      emit("sessions:changed");
      onUnlinked?.();
    } catch (e) {
      console.error("Failed to unlink session:", e);
    }
  }

  async function handleFinish(e: MouseEvent) {
    e.stopPropagation();
    try {
      // Kill embedded terminal if one is running for this session
      const term = getActiveTerminals().find(t => t.sessionDbId === session.id);
      if (term) {
        await ptyKill(term.terminalId).catch(() => {});
        removeTerminal(term.terminalId);
      }
      await finishEmbeddedSession(session.id);
      emit("sessions:changed");
      onUnlinked?.();
    } catch (e) {
      console.error("Failed to finish session:", e);
    }
  }
</script>

<div class="sc" class:sc--active={isActive || isInProgress} role="group">
  {#if hasSessionId}
    <button class="sc__main" class:sc__main--disabled={isRunningExternally} onclick={handleClick} title={isRunningExternally ? "Session is running externally" : "Open session in embedded terminal"}>
      <span class="sc__dot" class:sc__dot--live={isActive || isInProgress}></span>
      {#if editing}
        <!-- svelte-ignore a11y_autofocus -->
        <input
          class="sc__edit"
          type="text"
          bind:value={editValue}
          onblur={commitRename}
          onkeydown={handleEditKeydown}
          onclick={(e) => e.stopPropagation()}
          autofocus
        />
      {:else}
        <span class="sc__title" role="button" tabindex="0" ondblclick={startEditing} onkeydown={(e) => { if (e.key === 'Enter') startEditing(); }} title="Double-click to rename">{displayTitle}</span>
      {/if}
      <span class="sc__meta">
        {#if session.started_at}
          <span class="sc__time sc__time--{getTimeAge(session.started_at)}">{formatRelativeTime(session.started_at)}</span>
        {/if}
        {#if session.duration_mins != null}
          <span>{formatDuration(session.duration_mins)}</span>
        {/if}
        {#if session.turns != null}
          <span>{session.turns}t</span>
        {/if}
      </span>
      {#if isRunningExternally}
        <span class="sc__external-badge">External</span>
      {:else}
        <span class="sc__action-icon">
          <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M0 2.75C0 1.784.784 1 1.75 1h12.5c.966 0 1.75.784 1.75 1.75v10.5A1.75 1.75 0 0114.25 15H1.75A1.75 1.75 0 010 13.25zm1.75-.25a.25.25 0 00-.25.25v10.5c0 .138.112.25.25.25h12.5a.25.25 0 00.25-.25V2.75a.25.25 0 00-.25-.25zM3.5 6a.75.75 0 01.22-.53l2-2a.75.75 0 011.06 1.06L5.56 5.75l1.22 1.22a.75.75 0 01-1.06 1.06l-2-2A.75.75 0 013.5 6zm4.75.75a.75.75 0 000 1.5h2.5a.75.75 0 000-1.5z"/></svg>
        </span>
      {/if}
    </button>
    <button class="sc__copy" onclick={handleCopyCommand} title="Copy resume command to clipboard">
      {#if copied}
        <svg width="10" height="10" viewBox="0 0 16 16" fill="var(--green)"><path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 0 1 1.06-1.06L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/></svg>
      {:else}
        <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M0 6.75C0 5.784.784 5 1.75 5h1.5a.75.75 0 0 1 0 1.5h-1.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-1.5a.75.75 0 0 1 1.5 0v1.5A1.75 1.75 0 0 1 9.25 16h-7.5A1.75 1.75 0 0 1 0 14.25ZM5 1.75C5 .784 5.784 0 6.75 0h7.5C15.216 0 16 .784 16 1.75v7.5A1.75 1.75 0 0 1 14.25 11h-7.5A1.75 1.75 0 0 1 5 9.25Zm1.75-.25a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25Z"/></svg>
      {/if}
    </button>
  {:else}
    <div class="sc__main sc__main--static">
      <span class="sc__dot" class:sc__dot--live={isInProgress}></span>
      {#if editing}
        <!-- svelte-ignore a11y_autofocus -->
        <input
          class="sc__edit"
          type="text"
          bind:value={editValue}
          onblur={commitRename}
          onkeydown={handleEditKeydown}
          onclick={(e) => e.stopPropagation()}
          autofocus
        />
      {:else}
        <span class="sc__title" role="button" tabindex="0" ondblclick={startEditing} onkeydown={(e) => { if (e.key === 'Enter') startEditing(); }} title="Double-click to rename">{displayTitle}</span>
      {/if}
      <span class="sc__meta">
        {#if session.started_at}
          <span class="sc__time sc__time--{getTimeAge(session.started_at)}">{formatRelativeTime(session.started_at)}</span>
        {/if}
      </span>
      <span class="sc__badge" class:sc__badge--amber={isInProgress} class:sc__badge--red={!isInProgress}>
        {isInProgress ? "In progress" : "Session lost"}
      </span>
    </div>
  {/if}
  {#if hasEmbeddedTerminal}
    <button class="sc__finish" onclick={handleFinish} title="Mark session as finished">
      <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor"><path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 0 1 1.06-1.06L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/></svg>
    </button>
  {/if}
  <button class="sc__unlink" onclick={handleUnlinkClick} title="Unlink session">
    <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.749.749 0 0 1 1.275.326.749.749 0 0 1-.215.734L9.06 8l3.22 3.22a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215L8 9.06l-3.22 3.22a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
  </button>
  {#if confirmingUnlink}
    <div class="sc__confirm-overlay" role="dialog">
      <span class="sc__confirm-text">Unlink?</span>
      <button class="sc__confirm-yes" onclick={handleUnlinkConfirm}>Yes</button>
      <button class="sc__confirm-no" onclick={() => confirmingUnlink = false}>No</button>
    </div>
  {/if}
</div>

<style>
  .sc {
    position: relative;
    display: flex;
    align-items: center;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-card);
    overflow: hidden;
    transition: border-color var(--transition-fast);
  }
  .sc:hover {
    border-color: var(--accent-border);
  }
  .sc--active {
    border-color: color-mix(in srgb, var(--green) 30%, var(--border));
    background: color-mix(in srgb, var(--green) 5%, var(--bg-card));
  }
  .sc--active:hover {
    border-color: color-mix(in srgb, var(--green) 45%, var(--border));
  }
  .sc--active .sc__main:hover {
    background: color-mix(in srgb, var(--green) 8%, var(--bg-card));
  }

  .sc__main {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 6px 10px;
    border: none;
    background: transparent;
    color: inherit;
    font-family: inherit;
    text-align: left;
    cursor: pointer;
    transition: background var(--transition-fast);
  }
  .sc__main:hover {
    background: var(--bg-hover);
  }
  .sc__main--static {
    cursor: default;
  }
  .sc__main--static:hover {
    background: transparent;
  }
  .sc__main--disabled {
    cursor: default;
  }
  .sc__main--disabled:hover {
    background: transparent;
  }

  .sc__dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    flex-shrink: 0;
    background: var(--text-muted);
    opacity: 0.4;
  }
  .sc__dot--live {
    background: var(--green);
    opacity: 1;
    box-shadow: 0 0 5px var(--green);
  }

  .sc__title {
    font-size: var(--text-sm);
    font-weight: 550;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
    flex-shrink: 1;
  }

  .sc__edit {
    font-size: var(--text-sm);
    font-weight: 550;
    color: var(--text-primary);
    background: var(--bg-input);
    border: 1px solid var(--accent);
    border-radius: var(--radius-sm);
    padding: 1px 5px;
    flex: 1;
    min-width: 0;
    font-family: inherit;
    outline: none;
  }

  .sc__meta {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-shrink: 0;
    font-size: 10.5px;
    color: var(--text-muted);
    font-family: var(--font-mono);
    margin-left: auto;
  }

  .sc__time--fresh { color: color-mix(in srgb, var(--green) 70%, var(--text-muted)); }
  .sc__time--recent { color: var(--text-muted); }
  .sc__time--old { color: color-mix(in srgb, var(--text-muted) 70%, transparent); }
  .sc__time--stale { color: color-mix(in srgb, var(--text-muted) 50%, transparent); }

  .sc__action-icon {
    flex-shrink: 0;
    color: var(--text-muted);
    opacity: 0;
    transition: opacity var(--transition-fast);
    display: flex;
    align-items: center;
  }
  .sc:hover .sc__action-icon {
    opacity: 0.6;
  }
  .sc__main:hover .sc__action-icon {
    opacity: 1;
    color: var(--accent);
  }

  .sc__external-badge {
    font-size: 9.5px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--amber);
    flex-shrink: 0;
    white-space: nowrap;
  }

  .sc__badge {
    font-size: 10px;
    font-weight: 550;
    white-space: nowrap;
    flex-shrink: 0;
  }
  .sc__badge--amber { color: var(--amber); }
  .sc__badge--red { color: var(--red); }

  .sc__copy {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    align-self: stretch;
    flex-shrink: 0;
    border: none;
    border-left: 1px solid var(--border);
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    opacity: 0;
    transition: opacity var(--transition-fast), background var(--transition-fast), color var(--transition-fast);
  }
  .sc:hover .sc__copy {
    opacity: 1;
  }
  .sc__copy:hover {
    background: var(--bg-hover);
    color: var(--accent);
  }

  .sc__finish {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    align-self: stretch;
    flex-shrink: 0;
    border: none;
    border-left: 1px solid var(--border);
    background: transparent;
    color: var(--green);
    cursor: pointer;
    opacity: 0;
    transition: opacity var(--transition-fast), background var(--transition-fast), color var(--transition-fast);
  }
  .sc:hover .sc__finish {
    opacity: 1;
  }
  .sc__finish:hover {
    background: color-mix(in srgb, var(--green) 15%, transparent);
    color: var(--green);
  }

  .sc__unlink {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    align-self: stretch;
    flex-shrink: 0;
    border: none;
    border-left: 1px solid var(--border);
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    opacity: 0;
    transition: opacity var(--transition-fast), background var(--transition-fast), color var(--transition-fast);
  }
  .sc:hover .sc__unlink {
    opacity: 1;
  }
  .sc__unlink:hover {
    background: var(--red-dim);
    color: var(--red);
  }

  .sc__confirm-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    background: color-mix(in srgb, var(--bg-card) 95%, transparent);
    border-radius: var(--radius-md);
    z-index: 1;
  }

  .sc__confirm-text {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .sc__confirm-yes,
  .sc__confirm-no {
    font-size: 10.5px;
    font-weight: 600;
    font-family: inherit;
    padding: 2px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .sc__confirm-yes {
    background: color-mix(in srgb, var(--red) 15%, transparent);
    color: var(--red);
    border-color: color-mix(in srgb, var(--red) 30%, transparent);
  }
  .sc__confirm-yes:hover {
    background: color-mix(in srgb, var(--red) 25%, transparent);
  }

  .sc__confirm-no {
    background: transparent;
    color: var(--text-muted);
  }
  .sc__confirm-no:hover {
    background: var(--bg-hover);
    color: var(--text-secondary);
  }
</style>