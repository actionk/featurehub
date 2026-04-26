<script lang="ts">
  import type { PanelSession } from "../api/types";
  import { getPanelSessions, getPanelActiveCount } from "../stores/sessionActivity.svelte";
  import { getActiveTerminals, removeTerminal, getViewingTerminal } from "../stores/terminals.svelte";
  import { emit } from "../stores/events.svelte";
  import { getFhCliPath, unlinkSession, finishEmbeddedSession, ptyKill } from "../api/tauri";

  function isExternal(sessionId: string): boolean {
    return !getActiveTerminals().some(t => t.sessionDbId === sessionId);
  }

  interface Props {
    onSessionClick: (featureId: string, sessionDbId: string, isActive: boolean) => void;
    width: number;
  }

  let { onSessionClick, width }: Props = $props();

  let allSessions = $derived(getPanelSessions());
  let active = $derived(allSessions.filter(s => s.is_active));
  let recent = $derived(allSessions.filter(s => !s.is_active));
  let viewingSessionDbId = $derived(
    (() => {
      const id = getViewingTerminal();
      if (!id) return null;
      return getActiveTerminals().find(t => t.terminalId === id)?.sessionDbId ?? null;
    })()
  );
  let copiedId = $state<string | null>(null);
  let contextMenu = $state<{ x: number; y: number; session: PanelSession } | null>(null);
  let unlinkConfirmId = $state<string | null>(null);

  function formatRelativeTime(iso: string): string {
    const diff = Date.now() - new Date(iso).getTime();
    const mins = Math.floor(diff / 60_000);
    if (mins < 1) return "just now";
    if (mins < 60) return `${mins}m ago`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `${hours}h ago`;
    return `${Math.floor(hours / 24)}d ago`;
  }

  function formatTokens(n: number | null): string | null {
    if (!n) return null;
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
    if (n >= 1_000) return `${Math.round(n / 1_000)}k`;
    return `${n}`;
  }

  function shortModel(model: string | null): string | null {
    if (!model || model === '<synthetic>') return null;
    return model.replace(/^claude-/, "");
  }

  async function handleCopyResume(e: MouseEvent, claudeSessionId: string, sessionId: string) {
    e.stopPropagation();
    try {
      const fhPath = await getFhCliPath();
      const quoted = fhPath.includes(" ") ? `"${fhPath}"` : fhPath;
      await navigator.clipboard.writeText(`${quoted} resume ${claudeSessionId}`);
      copiedId = sessionId;
      setTimeout(() => { copiedId = null; }, 2000);
    } catch {}
  }

  function handleContextMenu(e: MouseEvent, session: PanelSession) {
    e.preventDefault();
    e.stopPropagation();
    contextMenu = { x: e.clientX, y: e.clientY, session };
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  async function handleMenuCopyResume() {
    if (!contextMenu) return;
    const { session } = contextMenu;
    closeContextMenu();
    try {
      const fhPath = await getFhCliPath();
      const quoted = fhPath.includes(" ") ? `"${fhPath}"` : fhPath;
      await navigator.clipboard.writeText(`${quoted} resume ${session.claude_session_id}`);
      copiedId = session.id;
      setTimeout(() => { copiedId = null; }, 2000);
    } catch {}
  }

  async function handleMenuFinish() {
    if (!contextMenu) return;
    const { session } = contextMenu;
    closeContextMenu();
    try {
      const term = getActiveTerminals().find(t => t.sessionDbId === session.id);
      if (term) {
        await ptyKill(term.terminalId).catch(() => {});
        removeTerminal(term.terminalId);
      }
      await finishEmbeddedSession(session.id);
      emit("sessions:changed");
    } catch {}
  }

  function handleMenuUnlink() {
    if (!contextMenu) return;
    const { session } = contextMenu;
    closeContextMenu();
    unlinkConfirmId = session.id;
  }

  async function confirmUnlink() {
    if (!unlinkConfirmId) return;
    const sessionId = unlinkConfirmId;
    unlinkConfirmId = null;
    try {
      const term = getActiveTerminals().find(t => t.sessionDbId === sessionId);
      if (term) {
        await ptyKill(term.terminalId).catch(() => {});
        await finishEmbeddedSession(term.sessionDbId).catch(() => {});
        removeTerminal(term.terminalId);
      }
      await unlinkSession(sessionId);
      emit("sessions:changed");
    } catch {}
  }

  $effect(() => {
    if (!contextMenu) return;
    function handleClick() { contextMenu = null; }
    window.addEventListener("click", handleClick);
    return () => window.removeEventListener("click", handleClick);
  });
</script>

<div class="sessions-panel" style="width: {width}px; min-width: {width}px;">
  <div class="sessions-panel-header">
    <span class="sessions-panel-title">🤖 AGENTS</span>
    {#if getPanelActiveCount() > 0}
      <span class="sessions-panel-active-badge">
        <span class="sessions-panel-active-dot"></span>
        {getPanelActiveCount()} active
      </span>
    {/if}
  </div>

  <div class="sessions-panel-scroll">
    {#if active.length > 0}
      <div class="sessions-panel-section-label">ACTIVE</div>
      <div class="sessions-panel-list">
        {#each active as session (session.id)}
          <div class="session-row session-row--active glass-panel--soft session-list-row {viewingSessionDbId === session.id ? 'session-row--viewing' : ''}" oncontextmenu={(e) => handleContextMenu(e, session)}>
            {#if unlinkConfirmId === session.id}
              <div class="session-unlink-confirm" role="dialog">
                <span class="session-unlink-confirm-text">Unlink?</span>
                <button class="session-unlink-confirm-yes" onclick={confirmUnlink}>Yes</button>
                <button class="session-unlink-confirm-no" onclick={() => unlinkConfirmId = null}>No</button>
              </div>
            {/if}
            <button class="session-row-main session-list-row__body" onclick={() => onSessionClick(session.feature_id, session.id, true)}>
              <div class="session-row-top">
                <span class="panel-session-dot panel-session-dot--active live-dot" class:live-dot--warn={session.status === 'WaitingForInput'}></span>
                <span class="session-feature-name">{session.feature_name}</span>
                {#if session.branch}
                  <span class="session-branch-pill">{session.branch}</span>
                {/if}
              </div>
              {#if session.title}
                <div class="session-title session-list-row__title">{session.title}</div>
              {/if}
              <div class="session-stats-row session-list-row__meta">
                {#if shortModel(session.model)}
                  <span class="session-stat session-stat--model session-list-row__model">{shortModel(session.model)}</span>
                {/if}
                <span class="session-stat session-stat--time session-list-row__ago">{formatRelativeTime(session.last_activity)}</span>
                {#if session.status === 'WaitingForInput'}
                  <span class="aurora-pill aurora-pill--warn aurora-pill--sm aurora-pill--no-dot">Waiting</span>
                {/if}
                {#if isExternal(session.id)}
                  <span class="aurora-pill aurora-pill--warn aurora-pill--sm aurora-pill--no-dot">External</span>
                {/if}
              </div>
              <div class="session-stats-row">
                {#if formatTokens(session.context_tokens)}
                  <span class="session-stat session-stat--ctx">{formatTokens(session.context_tokens)} ctx</span>
                {/if}
                {#if formatTokens(session.total_tokens)}
                  <span class="session-stat session-stat--total">{formatTokens(session.total_tokens)} total</span>
                {/if}
              </div>
            </button>
            <button
              class="session-copy-btn"
              class:session-copy-btn--copied={copiedId === session.id}
              title="Copy resume command"
              onclick={(e) => handleCopyResume(e, session.claude_session_id, session.id)}
            >{copiedId === session.id ? '✓' : '⧉'}</button>
          </div>
        {/each}
      </div>
    {/if}

    {#if recent.length > 0}
      <div class="sessions-panel-section-label">RECENT</div>
      <div class="sessions-panel-list">
        {#each recent as session (session.id)}
          <div class="session-row session-row--idle glass-panel--soft session-list-row {viewingSessionDbId === session.id ? 'session-row--viewing' : ''}" oncontextmenu={(e) => handleContextMenu(e, session)}>
            {#if unlinkConfirmId === session.id}
              <div class="session-unlink-confirm" role="dialog">
                <span class="session-unlink-confirm-text">Unlink?</span>
                <button class="session-unlink-confirm-yes" onclick={confirmUnlink}>Yes</button>
                <button class="session-unlink-confirm-no" onclick={() => unlinkConfirmId = null}>No</button>
              </div>
            {/if}
            <button class="session-row-main session-list-row__body" onclick={() => onSessionClick(session.feature_id, session.id, false)}>
              <div class="session-row-top">
                <span class="panel-session-dot panel-session-dot--idle live-dot live-dot--static"></span>
                <span class="session-feature-name">{session.feature_name}</span>
                {#if session.branch}
                  <span class="session-branch-pill">{session.branch}</span>
                {/if}
              </div>
              {#if session.title}
                <div class="session-title session-list-row__title">{session.title}</div>
              {/if}
              <div class="session-stats-row session-list-row__meta">
                {#if shortModel(session.model)}
                  <span class="session-stat session-stat--model session-list-row__model">{shortModel(session.model)}</span>
                {/if}
                <span class="session-stat session-stat--time session-list-row__ago">{formatRelativeTime(session.last_activity)}</span>
              </div>
              <div class="session-stats-row">
                {#if formatTokens(session.total_tokens)}
                  <span class="session-stat session-stat--total">{formatTokens(session.total_tokens)} total</span>
                {/if}
              </div>
            </button>
            <button
              class="session-copy-btn"
              class:session-copy-btn--copied={copiedId === session.id}
              title="Copy resume command"
              onclick={(e) => handleCopyResume(e, session.claude_session_id, session.id)}
            >{copiedId === session.id ? '✓' : '⧉'}</button>
          </div>
        {/each}
      </div>
    {/if}

    {#if allSessions.length === 0}
      <div class="sessions-panel-empty">No sessions yet</div>
    {/if}
  </div>
</div>

{#if contextMenu}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="context-menu"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
    oncontextmenu={(e) => e.preventDefault()}
  >
    {#if contextMenu.session.claude_session_id}
      <button class="context-menu-item" onclick={handleMenuCopyResume}>
        <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M0 6.75C0 5.784.784 5 1.75 5h1.5a.75.75 0 0 1 0 1.5h-1.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-1.5a.75.75 0 0 1 1.5 0v1.5A1.75 1.75 0 0 1 9.25 16h-7.5A1.75 1.75 0 0 1 0 14.25ZM5 1.75C5 .784 5.784 0 6.75 0h7.5C15.216 0 16 .784 16 1.75v7.5A1.75 1.75 0 0 1 14.25 11h-7.5A1.75 1.75 0 0 1 5 9.25Zm1.75-.25a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25Z"/></svg>
        Copy resume command
      </button>
    {/if}
    {#if !isExternal(contextMenu.session.id)}
      <button class="context-menu-item" onclick={handleMenuFinish}>
        <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 0 1 1.06-1.06L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/></svg>
        Finish
      </button>
    {/if}
    <div class="context-menu-separator"></div>
    <button class="context-menu-item context-menu-item--danger" onclick={handleMenuUnlink}>
      <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.749.749 0 0 1 1.275.326.749.749 0 0 1-.215.734L9.06 8l3.22 3.22a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215L8 9.06l-3.22 3.22a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
      Unlink
    </button>
  </div>
{/if}
