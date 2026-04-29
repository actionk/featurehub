<script lang="ts">
  import type { Session, Plan, Directory, DetectedIde, McpServer, FeatureMcpServer, Skill, FeatureSkill } from "../../api/tauri";
  import { ptySpawnSession, ptyResumeSession, ptyKill, ptyWrite, finishEmbeddedSession, detectIdes, openInIde, getFhCliPath, scanSessions, getFeatureMcpServers, setFeatureMcpServer, getFeatureSkills, setFeatureSkill, unlinkSession, getContext } from "../../api/tauri";
  import { getCachedSettings } from "../../stores/settings.svelte";
  import { getPanelSessions, isSessionActive as checkSessionActive } from "../../stores/sessionActivity.svelte";
  import SessionList from "./SessionList.svelte";
  import PlanCard from "./PlanCard.svelte";
  import PlanDetail from "./PlanDetail.svelte";
  import ContextEditor from "./ContextEditor.svelte";
  import Modal from "../../components/ui/Modal.svelte";
  import Terminal from "./Terminal.svelte";
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { Chart, registerables } from "chart.js";
  Chart.register(...registerables);
  import {
    getTerminalsForFeature,
    getActiveTerminals,
    addTerminal,
    removeTerminal,
    markExited,
    getPendingViewRequest,
    getPendingResumeRequest,
    setViewingTerminal,
  } from "../../stores/terminals.svelte";
  import type { TabContext } from "../registry";
  import { setToolbarActions, clearToolbarActions } from "../../stores/tabToolbar.svelte";
  import { onDestroy } from "svelte";
  import { formatElapsed, formatDuration, formatRelativeTime, getTimeAge } from '../../utils/format';

  let { featureId, feature, sessions, plans, tasks, pendingPlanId, onPendingPlanHandled, onSessionsChanged, onRefresh: onPlansChanged }: TabContext = $props();
  let featureTitle = $derived(feature.title);
  let directories = $derived(feature.directories ?? []);

  let selectedPlan = $state<Plan | null>(null);
  let sidePlan = $state<Plan | null>(null); // Plan shown alongside terminal in split view
  let pendingPlans = $derived(plans.filter(p => p.status === "pending"));
  let pendingPlanCount = $derived(pendingPlans.length);

  let activeTerminalId = $state<string | null>(null);
  let suppressTerminalAutoFocus = $state(false);
  let launching = $state(false);
  let preferredIdeList = $state<DetectedIde[]>([]);
  // Map claude_session_id → last_activity from JSONL mtime (more accurate than DB started_at)
  let panelSessionMap = $derived(
    new Map(getPanelSessions().map(s => [s.claude_session_id, s.last_activity]))
  );

  function sessionLastActive(session: Session): string | null {
    if (session.claude_session_id) {
      const fromPanel = panelSessionMap.get(session.claude_session_id);
      if (fromPanel) return fromPanel;
    }
    return session.started_at ?? null;
  }

  function isExternalSession(session: Session): boolean {
    if (!session.claude_session_id) return false;
    const isActive = checkSessionActive(session.claude_session_id);
    const hasEmbeddedTerminal = getActiveTerminals().some(t => t.sessionDbId === session.id);
    return isActive && !hasEmbeddedTerminal;
  }

  let copiedStart = $state(false);
  let copiedResumeId = $state<string | null>(null);
  let scanning = $state(false);

  async function handleCopyStartCommand() {
    try {
      const fhPath = await getFhCliPath();
      const quoted = fhPath.includes(" ") ? `"${fhPath}"` : fhPath;
      await navigator.clipboard.writeText(`${quoted} start "${featureTitle}"`);
      copiedStart = true;
      setTimeout(() => { copiedStart = false; }, 2000);
    } catch {}
  }

  async function handleCopyResumeCommand(session: Session) {
    if (!session.claude_session_id) return;
    try {
      const fhPath = await getFhCliPath();
      const quoted = fhPath.includes(" ") ? `"${fhPath}"` : fhPath;
      await navigator.clipboard.writeText(`${quoted} resume ${session.claude_session_id}`);
      copiedResumeId = session.id;
      setTimeout(() => { copiedResumeId = null; }, 2000);
    } catch {}
  }

  let contextMenu = $state<{ sessionId: string; x: number; y: number } | null>(null);

  function handleSessionContextMenu(e: MouseEvent, session: Session) {
    e.preventDefault();
    e.stopPropagation();
    contextMenu = { sessionId: session.id, x: e.clientX, y: e.clientY };
  }

  async function handleUnlinkSession(sessionId: string) {
    contextMenu = null;
    try {
      const term = getActiveTerminals().find(t => t.sessionDbId === sessionId);
      if (term) {
        await ptyKill(term.terminalId).catch(() => {});
        await finishEmbeddedSession(term.sessionDbId).catch(() => {});
        removeTerminal(term.terminalId);
        if (activeTerminalId === term.terminalId) activeTerminalId = null;
      }
      await unlinkSession(sessionId);
      onSessionsChanged();
    } catch (e) {
      console.error("Failed to unlink session:", e);
    }
  }

  $effect(() => {
    if (!contextMenu) return;
    const close = () => { contextMenu = null; };
    window.addEventListener("click", close);
    window.addEventListener("contextmenu", close, true);
    window.addEventListener("scroll", close, true);
    return () => {
      window.removeEventListener("click", close);
      window.removeEventListener("contextmenu", close, true);
      window.removeEventListener("scroll", close, true);
    };
  });

  async function handleScan() {
    scanning = true;
    try {
      await scanSessions(featureId);
      onSessionsChanged();
    } catch {} finally {
      scanning = false;
    }
  }

  let featureTerminals = $derived(getTerminalsForFeature(featureId));
  let readyDirs = $derived(
    directories.filter(d => d.clone_status === "ready" || !d.clone_status)
  );

  // Active = no ended_at OR externally running (per session activity store)
  let activeSessions = $derived(sessions.filter(s =>
    !s.ended_at || (s.claude_session_id && checkSessionActive(s.claude_session_id))
  ));
  // Split active sessions: embedded (can open in app) vs external (running outside)
  let embeddedActiveSessions = $derived(activeSessions.filter(s => !isExternalSession(s)));
  let externalActiveSessions = $derived(activeSessions.filter(s => isExternalSession(s)));
  let pastSessions = $derived(
    [...sessions]
      .filter(s => !!s.ended_at && !(s.claude_session_id && checkSessionActive(s.claude_session_id)))
      .sort((a, b) => (sessionLastActive(b) ?? '').localeCompare(sessionLastActive(a) ?? ''))
      .slice(0, 8)
  );

  // MCP + Skills data for config rows
  let mcpAllServers = $state<McpServer[]>([]);
  let mcpOverrides = $state<FeatureMcpServer[]>([]);
  let skillsAll = $state<Skill[]>([]);
  let skillsOverrides = $state<FeatureSkill[]>([]);

  let enabledMcpServers = $derived(
    mcpAllServers.filter(srv => {
      const override = mcpOverrides.find(o => o.server_name === srv.name);
      return override ? override.enabled : srv.default_enabled;
    })
  );

  let enabledSkills = $derived(
    skillsAll.filter(sk => {
      const override = skillsOverrides.find(o => o.skill_id === sk.id);
      return override ? override.enabled : sk.default_enabled;
    })
  );

  // Config row open/close state
  let mcpDropdownOpen = $state(false);
  let skillsDropdownOpen = $state(false);
  let contextModalOpen = $state(false);
  let contextEmpty = $state(false);

  async function loadContextEmpty() {
    try {
      const ctx = await getContext(featureId);
      contextEmpty = !ctx || !ctx.content || ctx.content.trim() === "";
    } catch {
      contextEmpty = false;
    }
  }

  async function toggleMcpServer(server: McpServer) {
    const enabled = !!enabledMcpServers.find(s => s.name === server.name);
    try {
      await setFeatureMcpServer(featureId, server.name, !enabled);
      const idx = mcpOverrides.findIndex(o => o.server_name === server.name);
      if (idx >= 0) {
        mcpOverrides[idx] = { server_name: server.name, enabled: !enabled };
      } else {
        mcpOverrides = [...mcpOverrides, { server_name: server.name, enabled: !enabled }];
      }
    } catch (e) {
      console.error('Failed to toggle MCP server:', e);
    }
  }

  async function toggleSkill(skill: Skill) {
    const enabled = !!enabledSkills.find(s => s.id === skill.id);
    try {
      await setFeatureSkill(featureId, skill.id, !enabled);
      const idx = skillsOverrides.findIndex(o => o.skill_id === skill.id);
      if (idx >= 0) {
        skillsOverrides[idx] = { skill_id: skill.id, enabled: !enabled };
      } else {
        skillsOverrides = [...skillsOverrides, { skill_id: skill.id, enabled: !enabled }];
      }
    } catch (e) {
      console.error('Failed to toggle skill:', e);
    }
  }

  function handlePanelClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest('.sc-tile')) {
      mcpDropdownOpen = false;
      skillsDropdownOpen = false;
    }
  }

  let links = $derived(feature.links ?? []);
  let tasksDone = $derived(tasks.filter(t => t.done).length);
  function sessionDurationMins(s: Session): number | null {
    if (s.duration_mins != null && s.duration_mins > 0) return s.duration_mins;
    if (s.started_at && s.ended_at) {
      const diff = Math.round((new Date(s.ended_at).getTime() - new Date(s.started_at).getTime()) / 60000);
      return diff > 0 ? diff : null;
    }
    return null;
  }
  let totalTimeMins = $derived(
    sessions.reduce((sum, s) => sum + (sessionDurationMins(s) ?? 0), 0)
  );
  let completedWithDuration = $derived(sessions.filter(s => sessionDurationMins(s) != null));
  let avgDurationMins = $derived(
    completedWithDuration.length > 0
      ? Math.round(totalTimeMins / completedWithDuration.length)
      : null
  );
  let activeDays = $derived(
    new Set(sessions.map(s => s.started_at ? s.started_at.slice(0, 10) : null).filter(Boolean)).size
  );

  function formatMins(mins: number): string {
    if (mins < 60) return `${mins}m`;
    const h = Math.floor(mins / 60);
    const m = mins % 60;
    return m > 0 ? `${h}h ${m}m` : `${h}h`;
  }
  let insightsRange = $state<'7d' | '14d' | '30d'>('7d');
  let sparkDays = $derived(insightsRange === '7d' ? 7 : insightsRange === '14d' ? 14 : 30);
  let sparkBuckets = $derived(getSessionBuckets(sessions, sparkDays));

  let sparkCanvas = $state<HTMLCanvasElement | undefined>(undefined);
  let sparkChart: Chart | null = null;

  function getDayLabels(days: number): string[] {
    const labels: string[] = [];
    for (let i = days - 1; i >= 0; i--) {
      const d = new Date();
      d.setDate(d.getDate() - i);
      labels.push(d.toLocaleDateString('en', { month: 'short', day: 'numeric' }));
    }
    return labels;
  }

  // Create chart once canvas is bound
  $effect(() => {
    const canvas = sparkCanvas;
    if (!canvas) return;
    const ctx = canvas.getContext('2d')!;
    const grad = ctx.createLinearGradient(0, 0, 0, 100);
    grad.addColorStop(0, 'rgba(77,124,255,0.3)');
    grad.addColorStop(1, 'rgba(77,124,255,0)');
    sparkChart = new Chart(ctx, {
      type: 'line',
      data: {
        labels: getDayLabels(sparkDays),
        datasets: [{
          data: sparkBuckets,
          borderColor: '#4d7cff',
          borderWidth: 1.5,
          tension: 0.4,
          fill: true,
          backgroundColor: grad,
          pointRadius: 0,
          pointHoverRadius: 4,
          pointHoverBackgroundColor: '#4d7cff',
          pointHoverBorderColor: 'rgba(255,255,255,0.9)',
          pointHoverBorderWidth: 2,
        }]
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        animation: { duration: 400 },
        plugins: {
          legend: { display: false },
          tooltip: {
            mode: 'index',
            intersect: false,
            backgroundColor: 'rgba(22,24,38,0.96)',
            borderColor: 'rgba(255,255,255,0.1)',
            borderWidth: 1,
            titleColor: '#8b93a8',
            bodyColor: '#e2e6f0',
            padding: { x: 10, y: 8 },
            displayColors: false,
            callbacks: {
              title: (items) => items[0]?.label ?? '',
              label: (item) => `${item.parsed.y} session${item.parsed.y !== 1 ? 's' : ''}`,
            }
          }
        },
        scales: {
          x: { display: false },
          y: { display: false, min: 0 }
        },
        interaction: { mode: 'nearest', axis: 'x', intersect: false }
      }
    });
    return () => { sparkChart?.destroy(); sparkChart = null; };
  });

  // Update chart when data changes
  $effect(() => {
    const buckets = sparkBuckets;
    const days = sparkDays;
    if (!sparkChart) return;
    sparkChart.data.labels = getDayLabels(days);
    sparkChart.data.datasets[0].data = buckets;
    sparkChart.update('active');
  });

  let now = $state(Date.now());
  $effect(() => {
    const id = setInterval(() => { now = Date.now(); }, 30000);
    return () => clearInterval(id);
  });
  let sessionElapsed = $derived(
    embeddedActiveSessions.length > 0 && embeddedActiveSessions[0].started_at
      ? formatElapsed(embeddedActiveSessions[0].started_at, now)
      : null
  );

  function getSessionBuckets(sessions: Session[], days: number): number[] {
    const buckets = new Array(days).fill(0);
    const now = Date.now();
    for (const s of sessions) {
      if (!s.started_at) continue;
      const age = (now - new Date(s.started_at).getTime()) / 86400000;
      const idx = Math.floor(age);
      if (idx >= 0 && idx < days) {
        buckets[days - 1 - idx]++;
      }
    }
    return buckets;
  }

  // Load preferred IDEs
  onMount(() => {
    (async () => {
      try {
        const [ides, settings] = await Promise.all([
          detectIdes(),
          getCachedSettings(),
        ]);
        const preferred = settings.preferred_ides ?? [];
        if (preferred.length > 0) {
          preferredIdeList = ides.filter(ide => preferred.includes(ide.id));
        } else {
          // If no preference set, show first detected IDE
          preferredIdeList = ides.slice(0, 1);
        }
      } catch {}
    })();
  });

  async function loadConfigData() {
    try {
      const [settings, mcpOvr, skOvr] = await Promise.all([
        getCachedSettings(),
        getFeatureMcpServers(featureId),
        getFeatureSkills(featureId),
      ]);
      const extServers: McpServer[] = (settings.extensions ?? [])
        .filter(e => e.enabled && e.mcp_server)
        .map(e => ({ ...e.mcp_server!, name: e.id }));
      mcpAllServers = [...(settings.mcp_servers ?? []), ...extServers];
      mcpOverrides = mcpOvr;
      skillsAll = settings.skills ?? [];
      skillsOverrides = skOvr;
    } catch (e) {
      console.error('Failed to load config data:', e);
    }
  }

  $effect(() => {
    featureId; // reactive dep — reload when feature changes
    loadConfigData();
    loadContextEmpty();
  });

  // Reset plan view when feature changes
  $effect(() => {
    featureId;
    selectedPlan = null;
    sidePlan = null;
  });

  // Keep sidePlan in sync with refreshed plans data
  $effect(() => {
    if (sidePlan) {
      const fresh = plans.find(p => p.id === sidePlan!.id);
      if (fresh) {
        sidePlan = fresh;
      }
    }
  });

  // When a plan notification arrives, show it alongside the terminal if the session matches
  $effect(() => {
    if (!pendingPlanId) return;
    const plan = plans.find(p => p.id === pendingPlanId);
    onPendingPlanHandled?.();
    if (!plan) return;

    if (activeTerminalId && plan.session_id) {
      // Check if the plan's session matches the active terminal's session
      const activeTerm = featureTerminals.find(t => t.terminalId === activeTerminalId);
      if (activeTerm) {
        const matchingSession = sessions.find(s => s.id === activeTerm.sessionDbId);
        if (matchingSession?.claude_session_id === plan.session_id) {
          sidePlan = plan;
          return;
        }
      }
    }
    // If no terminal match, show in the normal plan detail view
    if (!activeTerminalId) {
      selectedPlan = plan;
    }
  });

  // Check if sidebar requested viewing a specific terminal or showing the overview
  let lastViewVersion = -1;
  $effect(() => {
    const req = getPendingViewRequest();
    // Also read featureTerminals so this effect re-runs when terminals change
    const terms = featureTerminals;
    if (req.version === lastViewVersion) return;
    if (req.isClear) {
      lastViewVersion = req.version;
      suppressTerminalAutoFocus = true;
      activeTerminalId = null;
    } else if (req.terminalId) {
      if (terms.some(t => t.terminalId === req.terminalId)) {
        lastViewVersion = req.version;
        suppressTerminalAutoFocus = false;
        activeTerminalId = req.terminalId;
      }
      // Don't update lastViewVersion if terminal not found yet —
      // the effect will re-run when featureTerminals updates
    }
  });

  // Fall back to overview if active terminal was removed externally (e.g. sidebar finish)
  $effect(() => {
    if (activeTerminalId && !featureTerminals.some(t => t.terminalId === activeTerminalId)) {
      activeTerminalId = null;
    }
  });

  // If a live embedded terminal is restored while this panel is reopened, make it active.
  $effect(() => {
    if (activeTerminalId || suppressTerminalAutoFocus) return;
    const liveTerminal = featureTerminals.findLast(t => !t.exited);
    if (liveTerminal) {
      activeTerminalId = liveTerminal.terminalId;
    }
  });

  // Track which terminal is being viewed (for sidebar highlighting)
  $effect(() => {
    setViewingTerminal(activeTerminalId);
  });

  // Auto-resume a session when requested from the sessions panel.
  // Re-runs when sessions load (async), so it handles the case where
  // sessions weren't loaded yet when the request came in.
  let lastResumeVersion = -1;
  $effect(() => {
    const req = getPendingResumeRequest();
    const s = sessions; // reactive dependency — re-runs when sessions load
    if (req.version !== lastResumeVersion && req.sessionDbId) {
      const session = s.find(sess => sess.id === req.sessionDbId);
      if (session) {
        lastResumeVersion = req.version;
        handleResumeSession(session);
      }
      // If session not found yet (still loading), don't consume the version —
      // the effect will re-run when sessions updates.
    }
  });

  // Find if a session already has an embedded terminal open
  function findTerminalForSession(sessionDbId: string) {
    return featureTerminals.find(t => t.sessionDbId === sessionDbId);
  }

  async function handleStartSession(typedPromptArg?: string, dangerouslySkipPermissions?: boolean) {
    const typedPrompt = typeof typedPromptArg === "string" ? typedPromptArg : undefined;
    if (launching) return;
    launching = true;
    try {
      const readyDirs = directories
        .filter(d => d.clone_status === "ready" || !d.clone_status)
        .map(d => d.path);
      const result = await ptySpawnSession(
        featureId,
        readyDirs,
        featureTitle,
        80,
        24,
        null,
        dangerouslySkipPermissions,
      );
      const idx = featureTerminals.length + 1;
      addTerminal({
        terminalId: result.terminalId,
        sessionDbId: result.sessionDbId,
        featureId,
        featureTitle,
        label: `Session ${idx}`,
        exited: false,
      });
      suppressTerminalAutoFocus = false;
      activeTerminalId = result.terminalId;
      onSessionsChanged();

      if (typedPrompt) {
        sendWhenClaudeReady(result.terminalId, typedPrompt);
      }
    } catch (e) {
      console.error("Failed to start embedded session:", e);
    } finally {
      launching = false;
    }
  }

  // Watch terminal output and drive a state machine:
  //   waitReady → send paste → waitPasteEcho → send Enter → done.
  // Detect each step from the PTY output instead of using fixed delays.
  function sendWhenClaudeReady(terminalId: string, prompt: string) {
    type Phase = "waitReady" | "waitPasteEcho" | "done";
    let phase: Phase = "waitReady";
    let buffer = "";
    let unlisten: (() => void) | null = null;
    let quietTimer: ReturnType<typeof setTimeout> | null = null;
    let safetyTimer: ReturnType<typeof setTimeout> | null = null;
    const decoder = new TextDecoder("utf-8", { fatal: false });


    function cleanup() {
      phase = "done";
      if (unlisten) unlisten();
      if (quietTimer) clearTimeout(quietTimer);
      if (safetyTimer) clearTimeout(safetyTimer);
    }

    function bumpQuiet(ms: number, fn: () => void) {
      if (quietTimer) clearTimeout(quietTimer);
      quietTimer = setTimeout(fn, ms);
    }

    // Strip ANSI/VT escape sequences so pattern matching works regardless of
    // how Claude renders colours or moves the cursor between box characters.
    function stripAnsi(s: string): string {
      return s
        .replace(/\x1b\[[0-9;?]*[A-Za-z]/g, "")   // CSI sequences
        .replace(/\x1b[()][A-Z0-9]/g, "")           // character-set designators
        .replace(/\x1b[>=]/g, "");                   // keypad mode switches
    }

    function utf8ToBase64(s: string): string {
      const bytes = new TextEncoder().encode(s);
      let bin = "";
      for (const b of bytes) bin += String.fromCharCode(b);
      return btoa(bin);
    }

    function sendPaste(reason: string) {
      // paste init
      const PASTE_START = "\x1b[200~";
      const PASTE_END = "\x1b[201~";
      buffer = "";
      phase = "waitPasteEcho";
      ptyWrite(terminalId, utf8ToBase64(PASTE_START + prompt + PASTE_END)).catch(() => {});
      bumpQuiet(400, () => sendEnter("paste echo settled"));
    }

    function sendEnter(reason: string) {
      // enter init
      ptyWrite(terminalId, btoa("\r")).catch(() => {});
      cleanup();
    }

    // Start a fallback quiet timer immediately in case early PTY events are
    // missed due to the async listener registration.
    bumpQuiet(2500, () => sendPaste("initial quiet"));

    listen<string>(`pty-data-${terminalId}`, (event) => {
      if (phase === "done") return;
      try {
        const raw = atob(event.payload);
        const bytes = Uint8Array.from(raw, (c) => c.charCodeAt(0));
        const text = decoder.decode(bytes, { stream: true });
        buffer += text;
        if (buffer.length > 16384) buffer = buffer.slice(-16384);

        if (phase === "waitReady") {
          const clean = stripAnsi(buffer);
          if (/[│┃]\s*>/.test(clean) || /╭[─━]/.test(clean)) {
            sendPaste("input prompt detected");
            return;
          }
          bumpQuiet(1200, () => sendPaste("output settled"));
        } else if (phase === "waitPasteEcho") {
          bumpQuiet(350, () => sendEnter("paste echo settled"));
        }
      } catch {}
    }).then((fn) => {
      if (phase === "done") { fn(); return; }
      unlisten = fn;
    });

    safetyTimer = setTimeout(() => {
      if (phase === "waitReady") sendPaste("safety timeout");
      else if (phase === "waitPasteEcho") sendEnter("safety timeout");
    }, 15000);
  }

  async function handleStartAndInitialize() {
    const initPrompt = "Please initialize this feature by following the Feature Initialization instructions from the FeatureHub MCP server. Read my feature's description and any inputs, then: (1) populate context via save_context and set an appropriate status; (2) search for all related links - use the gh CLI to find GitHub PRs and issues - if a PR URL is already in the feature, run gh pr view on it first to extract the ticket ID from the title or branch name, then run gh search prs \"TICKET-ID\" (without --repo) to find all related PRs across all repos in one shot; do NOT use gh pr list or web search for GitHub lookups. Also add any Jira tickets, Confluence pages, Slack threads, Figma files, or other relevant URLs via add_link; (3) check which repositories are needed for this feature and, for any that are not yet cloned locally, offer to clone them via clone_repository. Do NOT create tasks or notes.";
    // Copy while we still have user-gesture context (before any awaits).
    navigator.clipboard.writeText(initPrompt).catch(() => {});
    await handleStartSession(initPrompt);
  }

  async function handleResumeSession(session: Session, dangerouslySkipPermissions = false) {
    // If terminal already open for this session, just switch to it
    const existing = findTerminalForSession(session.id);
    if (existing) {
      suppressTerminalAutoFocus = false;
      activeTerminalId = existing.terminalId;
      return;
    }

    try {
      const result = await ptyResumeSession(session.id, 80, 24, dangerouslySkipPermissions);
      const title = session.title ?? "Resumed Session";
      addTerminal({
        terminalId: result.terminalId,
        sessionDbId: result.sessionDbId,
        featureId,
        featureTitle,
        label: title,
        exited: false,
      });
      suppressTerminalAutoFocus = false;
      activeTerminalId = result.terminalId;
    } catch (e) {
      console.error("Failed to resume session:", e);
      if (String(e).includes("Session was empty and has been removed")) {
        onSessionsChanged();
      }
    }
  }

  async function finishTerminal(terminalId: string) {
    // Compute next terminal before removing
    const term = featureTerminals.find(t => t.terminalId === terminalId);
    const remaining = featureTerminals.filter(t => t.terminalId !== terminalId);
    const nextId = remaining.length > 0 ? remaining[remaining.length - 1].terminalId : null;

    await ptyKill(terminalId).catch(() => {});
    // Mark the session as ended in the DB
    if (term) {
      await finishEmbeddedSession(term.sessionDbId).catch(() => {});
    }
    removeTerminal(terminalId);
    if (activeTerminalId === terminalId) {
      activeTerminalId = nextId;
    }
    onSessionsChanged();
  }

  async function handleTerminalExited(terminalId: string) {
    // Use global store — terminal may belong to another feature if it exited while
    // the user was viewing a different feature (terminals are kept alive globally).
    const term = getActiveTerminals().find(t => t.terminalId === terminalId);
    markExited(terminalId);
    if (term) {
      await finishEmbeddedSession(term.sessionDbId).catch(() => {});
    }
    onSessionsChanged();
  }

  function showOverview() {
    suppressTerminalAutoFocus = true;
    activeTerminalId = null;
    sidePlan = null;
  }

  function viewTerminal(terminalId: string) {
    suppressTerminalAutoFocus = false;
    activeTerminalId = terminalId;
  }

  // Register toolbar actions for the AI tab
  $effect(() => {
    setToolbarActions("ai", [
      {
        id: "new-session",
        label: "New Session",
        icon: '<svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor"><path d="M8 1a1 1 0 011 1v5h5a1 1 0 110 2H9v5a1 1 0 11-2 0V9H2a1 1 0 010-2h5V2a1 1 0 011-1z"/></svg>',
        onClick: () => handleStartSession(),
        disabled: launching,
        variant: "primary",
        title: "Start a new Claude session",
      },
    ]);
  });

  onDestroy(() => clearToolbarActions("ai"));
</script>

<!-- Always render terminals so they stay alive in background -->
<div style="position: absolute; inset: 0; display: flex; flex-direction: column; z-index: {activeTerminalId ? 1 : -1}; visibility: {activeTerminalId ? 'visible' : 'hidden'};">
  <!-- Terminal tab bar -->
  <div class="terminal-tabs">
    <button
      class="terminal-tab"
      onclick={showOverview}
      title="Back to overview"
    >
      <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M7.78 12.53a.75.75 0 01-1.06 0L2.47 8.28a.75.75 0 010-1.06l4.25-4.25a.75.75 0 011.06 1.06L4.81 7h8.44a.75.75 0 010 1.5H4.81l2.97 2.97a.75.75 0 010 1.06z"/></svg>
      <span>Overview</span>
    </button>
    <div style="width: 1px; height: 16px; background: var(--border); margin: 0 4px;"></div>
    {#each featureTerminals as term (term.terminalId)}
      <div
        class="terminal-tab {activeTerminalId === term.terminalId ? 'terminal-tab--active' : ''} {term.exited ? 'terminal-tab--exited' : ''}"
        onclick={() => viewTerminal(term.terminalId)}
        role="tab"
        tabindex="0"
        onkeydown={(e) => { if (e.key === 'Enter') viewTerminal(term.terminalId); }}
      >
        <span class="terminal-tab-dot {term.exited ? '' : 'terminal-tab-dot--live'}"></span>
        <span>{term.label}</span>
      </div>
    {/each}
    <div style="flex: 1;"></div>
    <!-- Repo quick-open buttons -->
    {#if readyDirs.length > 0 && preferredIdeList.length > 0}
      <div class="terminal-repos">
        {#each readyDirs as dir (dir.id)}
          {@const dirName = dir.label || dir.path.split(/[/\\]/).pop() || dir.path}
          {#each preferredIdeList as ide (ide.id)}
            <button
              class="terminal-repo-btn"
              onclick={() => openInIde(dir.path, ide.command)}
              title="Open {dirName} in {ide.name}"
            >
              {#if ide.id === "vscode" || ide.id === "vscode-insiders"}
                <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor"><path d="M17.583 2.603L12.2 7.386 7.847 4.15l-.87.507v14.71l.87.508 5.384-3.252-4.354-3.236L17.583 21.4l.87-.507V3.11l-.87-.507zM7.847 14.86V9.163l3.508 2.849-3.508 2.849z"/></svg>
              {:else if ide.id === "cursor"}
                <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/></svg>
              {:else}
                <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><rect x="2" y="2" width="12" height="12" rx="2" fill="none" stroke="currentColor" stroke-width="1.2"/><path d="M5 5h2.5v2.5H5zM8.5 5H11v2.5H8.5zM5 8.5h2.5V11H5z" opacity="0.7"/></svg>
              {/if}
              <span>{dirName}</span>
            </button>
          {/each}
        {/each}
      </div>
      <div style="width: 1px; height: 16px; background: var(--border); margin: 0 2px;"></div>
    {/if}
    {#if activeTerminalId}
      {@const activeTerm = featureTerminals.find(t => t.terminalId === activeTerminalId)}
      {#if activeTerm}
        <button
          class="terminal-finish-btn"
          onclick={() => finishTerminal(activeTerm.terminalId)}
          title="Finish session and close terminal"
        >
          <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor"><path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.749.749 0 0 1 1.275.326.749.749 0 0 1-.215.734L9.06 8l3.22 3.22a.749.749 0 0 1-.326 1.275.749.749 0 0 1-.734-.215L8 9.06l-3.22 3.22a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06Z"/></svg>
          Finish
        </button>
      {/if}
    {/if}
  </div>

  <!-- Terminal instances + optional side plan -->
  <!-- All active terminals (across all features) are rendered here so xterm
       instances survive feature switches. Non-current terminals are kept hidden
       with visibility:hidden; only the active tab terminal is shown. -->
  <div style="flex: 1; min-height: 0; position: relative; display: flex;">
    <div style="flex: {sidePlan ? '0 0 40%' : '1 1 100%'}; min-width: 0; position: relative; transition: flex 0.15s ease;">
      {#each getActiveTerminals() as term (term.terminalId)}
        <div
          class="terminal-wrapper"
          style="visibility: {activeTerminalId === term.terminalId ? 'visible' : 'hidden'}; position: absolute; inset: 0;"
        >
          <Terminal terminalId={term.terminalId} active={activeTerminalId === term.terminalId} onExited={() => handleTerminalExited(term.terminalId)} />
        </div>
      {/each}
    </div>
    {#if sidePlan}
      <div class="side-plan-panel">
        <PlanDetail plan={sidePlan} onClose={() => { sidePlan = null; }} onResolved={() => { sidePlan = null; onPlansChanged(); }} />
      </div>
    {/if}
  </div>
</div>

<!-- Overview (shown when no terminal is active) -->
{#if !activeTerminalId}
  {#if selectedPlan}
    <div style="flex: 1; min-height: 0; display: flex; flex-direction: column;">
      <PlanDetail plan={selectedPlan} onClose={() => { selectedPlan = null; }} onResolved={() => { selectedPlan = null; onPlansChanged(); }} />
    </div>
  {:else}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="sc-panel" onclick={handlePanelClick} role="region" aria-label="AI Sessions">

      <!-- Sessions card -->
      <div class="sessions-card">

        <!-- Header -->
        <div class="sc-header">
          <span class="sc-header-title">Sessions</span>
          {#if sessions.length > 0}
            <span class="sc-header-count">{sessions.length}</span>
          {/if}
          {#if activeSessions.length >= 2}
            <span class="sc-active-badge">
              <span class="sc-active-badge-dot"></span>
              {activeSessions.length} active
            </span>
          {/if}
          <div class="sc-header-actions">
            <button class="sc-icon-btn" onclick={() => handleStartSession()} disabled={launching} data-tooltip="New session">
              <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M8 1a1 1 0 011 1v5h5a1 1 0 110 2H9v5a1 1 0 11-2 0V9H2a1 1 0 010-2h5V2a1 1 0 011-1z"/></svg>
            </button>
            <button class="sc-icon-btn sc-icon-btn--danger" onclick={() => handleStartSession(undefined, true)} disabled={launching} data-tooltip="New session (bypass permissions)">
              <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M8 1a1 1 0 011 1v5h5a1 1 0 110 2H9v5a1 1 0 11-2 0V9H2a1 1 0 010-2h5V2a1 1 0 011-1z"/></svg>
            </button>
            <button class="sc-icon-btn {copiedStart ? 'sc-icon-btn--copied' : ''}" onclick={handleCopyStartCommand} data-tooltip={copiedStart ? 'Copied!' : 'Copy fh start command'}>
              {#if copiedStart}
                <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 0 1 1.06-1.06L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/></svg>
              {:else}
                <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M0 6.75C0 5.784.784 5 1.75 5h1.5a.75.75 0 0 1 0 1.5h-1.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-1.5a.75.75 0 0 1 1.5 0v1.5A1.75 1.75 0 0 1 9.25 16h-7.5A1.75 1.75 0 0 1 0 14.25ZM5 1.75C5 .784 5.784 0 6.75 0h7.5C15.216 0 16 .784 16 1.75v7.5A1.75 1.75 0 0 1 14.25 11h-7.5A1.75 1.75 0 0 1 5 9.25Zm1.75-.25a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25Z"/></svg>
              {/if}
            </button>
            {#if sessions.length > 0}
              <button class="sc-icon-btn {scanning ? 'sc-icon-btn--spinning' : ''}" onclick={handleScan} disabled={scanning} data-tooltip={scanning ? 'Scanning…' : 'Scan for sessions'}>
                <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M8 2.5a5.487 5.487 0 00-4.131 1.869l1.204 1.204A.25.25 0 014.896 6H1.25A.25.25 0 011 5.75V2.104a.25.25 0 01.427-.177l1.38 1.38A7.001 7.001 0 0115 8a.75.75 0 01-1.5 0A5.5 5.5 0 008 2.5zM1.75 8a.75.75 0 01.75.75 5.5 5.5 0 009.131 4.131l-1.204-1.204A.25.25 0 0110.604 11h3.646a.25.25 0 01.25.25v3.646a.25.25 0 01-.427.177l-1.38-1.38A7.001 7.001 0 011 8.75.75.75 0 011.75 8z"/></svg>
              </button>
            {/if}
          </div>
        </div>

        <!-- Active session slot (embedded only) -->
        {#if embeddedActiveSessions.length === 0}
          <div class="sc-start-cta">
            <span class="sc-start-hint">No active session</span>
            <div class="sc-start-actions">
              <button class="sc-start-btn" onclick={() => handleStartSession()} disabled={launching}>
                <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true"><path d="M4 2.5v11l10-5.5z"/></svg>
                <span>{launching ? 'Starting…' : 'Start Session'}</span>
              </button>
              <button
                class="sc-start-btn sc-start-btn--danger"
                onclick={() => handleStartSession(undefined, true)}
                disabled={launching}
                title="Start with --dangerously-skip-permissions (no tool prompts)"
              >
                <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true"><path d="M8 1a7 7 0 100 14A7 7 0 008 1zm-.75 4a.75.75 0 011.5 0v4a.75.75 0 01-1.5 0V5zM8 12a1 1 0 110-2 1 1 0 010 2z"/></svg>
                <span>Full-Access</span>
              </button>
            </div>
            {#if contextEmpty}
              <button
                class="sc-start-btn sc-start-btn--init"
                onclick={handleStartAndInitialize}
                disabled={launching}
                title="Start a Claude session and ask it to initialize this feature's context from description and links"
              >
                ✨ Start & Initialize
              </button>
            {/if}
          </div>
        {:else}
          {@const primarySession = embeddedActiveSessions[0]}
          <div
            class="sc-active"
            onclick={() => handleResumeSession(primarySession)}
            onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleResumeSession(primarySession); } }}
            oncontextmenu={(e) => handleSessionContextMenu(e, primarySession)}
            role="button"
            tabindex="0"
            title="Open active terminal"
          >
            <div class="sc-active-content">
              <div class="sc-active-row1">
                <span class="sc-live-pill">
                  <span class="sc-live-dot"></span>
                  Active Now
                </span>
                {#if sessionElapsed}
                  <span class="sc-active-timer">
                    <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M8 1a7 7 0 100 14A7 7 0 008 1zm0 1.5a5.5 5.5 0 110 11 5.5 5.5 0 010-11zM8 4v4.25l2.75 1.75-.75 1.25L7 9V4h1z"/></svg>
                    {sessionElapsed}
                  </span>
                {/if}
              </div>
              <div class="sc-active-title">{primarySession.title ?? 'Running session'}</div>
              {#if primarySession.summary}
                <div class="sc-active-summary">{primarySession.summary}</div>
              {/if}
            </div>
            <button class="sc-open-btn" onclick={(e) => { e.stopPropagation(); handleResumeSession(primarySession); }}>
              Open Terminal →
            </button>
          </div>
          {#each embeddedActiveSessions.slice(1) as session (session.id)}
            <div
              class="sc-active-compact"
              onclick={() => handleResumeSession(session)}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleResumeSession(session); } }}
              oncontextmenu={(e) => handleSessionContextMenu(e, session)}
              role="button"
              tabindex="0"
              title="Open active terminal"
            >
              <span class="sc-compact-dot"></span>
              <span class="sc-compact-title">{session.title ?? 'Running session'}</span>
              {#if session.started_at}
                <span class="sc-compact-timer">{formatElapsed(session.started_at, now)}</span>
              {/if}
              <button class="sc-compact-open" onclick={(e) => { e.stopPropagation(); handleResumeSession(session); }}>Open →</button>
            </div>
          {/each}
        {/if}

        <!-- External sessions (running outside app) -->
        {#each externalActiveSessions as session (session.id)}
          <div class="sc-session-row" oncontextmenu={(e) => handleSessionContextMenu(e, session)} role="listitem">
            <span class="sc-session-dot" style="background: var(--green);"></span>
            <span class="sc-session-name">{session.title ?? 'Session'}</span>
            <span class="sc-external-badge">External</span>
            {#if session.claude_session_id}
              <button
                class="sc-session-copy"
                onclick={(e) => { e.stopPropagation(); handleCopyResumeCommand(session); }}
                title="Copy resume command"
              >
                {#if copiedResumeId === session.id}
                  <svg width="9" height="9" viewBox="0 0 16 16" fill="var(--green)"><path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 0 1 1.06-1.06L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/></svg>
                {:else}
                  <svg width="9" height="9" viewBox="0 0 16 16" fill="currentColor"><path d="M0 6.75C0 5.784.784 5 1.75 5h1.5a.75.75 0 0 1 0 1.5h-1.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-1.5a.75.75 0 0 1 1.5 0v1.5A1.75 1.75 0 0 1 9.25 16h-7.5A1.75 1.75 0 0 1 0 14.25ZM5 1.75C5 .784 5.784 0 6.75 0h7.5C15.216 0 16 .784 16 1.75v7.5A1.75 1.75 0 0 1 14.25 11h-7.5A1.75 1.75 0 0 1 5 9.25Zm1.75-.25a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25Z"/></svg>
                {/if}
              </button>
            {/if}
            <button
              class="sc-session-open"
              onclick={(e) => { e.stopPropagation(); handleResumeSession(session); }}
              title="Open session"
            >
              Open
            </button>
            <button
              class="sc-session-open sc-session-danger-open"
              onclick={(e) => { e.stopPropagation(); handleResumeSession(session, true); }}
              title="Open with --dangerously-skip-permissions"
            >
              Full Access
            </button>
          </div>
        {/each}

        <!-- Past sessions -->
        {#each pastSessions as session (session.id)}
          <div class="sc-session-row" onclick={() => handleResumeSession(session)} onkeydown={(e) => { if (e.key === 'Enter') handleResumeSession(session); }} oncontextmenu={(e) => handleSessionContextMenu(e, session)} role="button" tabindex="0">
            <span class="sc-session-dot"></span>
            <span class="sc-session-name">{session.title ?? 'Session'}</span>
            <span class="sc-session-when {sessionLastActive(session) ? 'sc-session-when--' + getTimeAge(sessionLastActive(session)!) : ''}">
              {#if sessionLastActive(session)}
                {formatRelativeTime(sessionLastActive(session)!)}
              {/if}
            </span>
            {#if session.claude_session_id}
              <button
                class="sc-session-copy"
                onclick={(e) => { e.stopPropagation(); handleCopyResumeCommand(session); }}
                title="Copy resume command"
              >
                {#if copiedResumeId === session.id}
                  <svg width="9" height="9" viewBox="0 0 16 16" fill="var(--green)"><path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 0 1 1.06-1.06L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/></svg>
                {:else}
                  <svg width="9" height="9" viewBox="0 0 16 16" fill="currentColor"><path d="M0 6.75C0 5.784.784 5 1.75 5h1.5a.75.75 0 0 1 0 1.5h-1.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-1.5a.75.75 0 0 1 1.5 0v1.5A1.75 1.75 0 0 1 9.25 16h-7.5A1.75 1.75 0 0 1 0 14.25ZM5 1.75C5 .784 5.784 0 6.75 0h7.5C15.216 0 16 .784 16 1.75v7.5A1.75 1.75 0 0 1 14.25 11h-7.5A1.75 1.75 0 0 1 5 9.25Zm1.75-.25a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25Z"/></svg>
                {/if}
              </button>
            {/if}
            <button
              class="sc-session-open"
              onclick={(e) => { e.stopPropagation(); handleResumeSession(session); }}
              title="Open session"
            >
              Open
            </button>
            <button
              class="sc-session-open sc-session-danger-open"
              onclick={(e) => { e.stopPropagation(); handleResumeSession(session, true); }}
              title="Open with --dangerously-skip-permissions"
            >
              Full Access
            </button>
          </div>
        {/each}


      </div>

      <!-- Config tiles (3-column) -->
      <div class="sc-tiles">

        <!-- MCP tile -->
        <div class="sc-tile" onclick={() => { mcpDropdownOpen = !mcpDropdownOpen; skillsDropdownOpen = false; }} onkeydown={(e) => { if (e.key === 'Enter') { mcpDropdownOpen = !mcpDropdownOpen; skillsDropdownOpen = false; } }} role="button" tabindex="0">
          <div class="sc-tile-header">
            <span class="sc-tile-label">MCP</span>
            {#if mcpAllServers.length > 0}
              <span class="sc-tile-caret {mcpDropdownOpen ? 'sc-tile-caret--open' : ''}">▾</span>
            {/if}
          </div>
          <div class="sc-tile-body">
            {#each enabledMcpServers as srv (srv.name)}
              <span class="sc-pill"><span class="sc-pill-dot"></span>{srv.name}</span>
            {/each}
            {#if enabledMcpServers.length === 0}
              <span style="font-size:11px;color:var(--text-muted)">None enabled</span>
            {/if}
          </div>
          {#if mcpDropdownOpen}
            <div class="sc-dropdown" onclick={(e) => e.stopPropagation()} onkeydown={(e) => { if (e.key === 'Escape') mcpDropdownOpen = false; }} role="menu" tabindex="-1">
              {#each mcpAllServers as srv (srv.name)}
                {@const on = !!enabledMcpServers.find(s => s.name === srv.name)}
                <button class="sc-dropdown-item" onclick={() => toggleMcpServer(srv)}>
                  <span class="sc-dropdown-dot {on ? 'sc-dropdown-dot--on' : ''}"></span>
                  {srv.name}
                </button>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Skills tile -->
        <div class="sc-tile" onclick={() => { skillsDropdownOpen = !skillsDropdownOpen; mcpDropdownOpen = false; }} onkeydown={(e) => { if (e.key === 'Enter') { skillsDropdownOpen = !skillsDropdownOpen; mcpDropdownOpen = false; } }} role="button" tabindex="0">
          <div class="sc-tile-header">
            <span class="sc-tile-label">Skills</span>
            {#if skillsAll.length > 0}
              <span class="sc-tile-caret {skillsDropdownOpen ? 'sc-tile-caret--open' : ''}">▾</span>
            {/if}
          </div>
          <div class="sc-tile-body">
            {#each enabledSkills.slice(0, 3) as sk (sk.id)}
              <span class="sc-pill">{sk.name}</span>
            {/each}
            {#if enabledSkills.length > 3}
              <span class="sc-pill">+{enabledSkills.length - 3}</span>
            {/if}
            {#if enabledSkills.length === 0}
              <span style="font-size:11px;color:var(--text-muted)">None enabled</span>
            {/if}
          </div>
          {#if skillsDropdownOpen}
            <div class="sc-dropdown" onclick={(e) => e.stopPropagation()} onkeydown={(e) => { if (e.key === 'Escape') skillsDropdownOpen = false; }} role="menu" tabindex="-1">
              {#each skillsAll as sk (sk.id)}
                {@const on = !!enabledSkills.find(s => s.id === sk.id)}
                <button class="sc-dropdown-item" onclick={() => toggleSkill(sk)}>
                  <span class="sc-dropdown-dot {on ? 'sc-dropdown-dot--on' : ''}"></span>
                  {sk.name}
                </button>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Context tile -->
        <div class="sc-tile" onclick={() => { contextModalOpen = true; mcpDropdownOpen = false; skillsDropdownOpen = false; }} onkeydown={(e) => { if (e.key === 'Enter') { contextModalOpen = true; mcpDropdownOpen = false; skillsDropdownOpen = false; } }} role="button" tabindex="0">
          <div class="sc-tile-header">
            <span class="sc-tile-label">Context</span>
          </div>
          <div class="sc-tile-body">
            <button class="sc-ctx-btn" onclick={(e) => { e.stopPropagation(); contextModalOpen = true; }}>
              <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor"><path d="M0 2.75A2.75 2.75 0 012.75 0h10.5A2.75 2.75 0 0116 2.75v10.5A2.75 2.75 0 0113.25 16H2.75A2.75 2.75 0 010 13.25zm2.75-1.25a1.25 1.25 0 00-1.25 1.25v10.5c0 .69.56 1.25 1.25 1.25h10.5c.69 0 1.25-.56 1.25-1.25V2.75c0-.69-.56-1.25-1.25-1.25zM4 5.75A.75.75 0 014.75 5h6.5a.75.75 0 010 1.5h-6.5A.75.75 0 014 5.75zm0 3A.75.75 0 014.75 8h4a.75.75 0 010 1.5h-4A.75.75 0 014 8.75z"/></svg>
              View / Edit Context
            </button>
          </div>
        </div>

      </div>

      <!-- Context modal -->
      <Modal open={contextModalOpen} onClose={() => { contextModalOpen = false; loadContextEmpty(); }} width="1100px">
        <div style="padding: 20px; display: flex; flex-direction: column; gap: 12px; min-height: 400px;">
          <div style="display:flex;align-items:center;justify-content:space-between;">
            <span style="font-size:14px;font-weight:600;color:var(--text-primary)">Context</span>
            <button onclick={() => { contextModalOpen = false; }} style="background:none;border:none;color:var(--text-muted);cursor:pointer;padding:4px;font-size:13px;">✕</button>
          </div>
          <ContextEditor {featureId} hideHeader />
        </div>
      </Modal>

      <!-- Insights card -->
      <div class="sc-insights-card">
        <div class="sc-insights-card-header">
          <span class="sc-insights-card-title">Insights</span>
          <div class="sc-range-btns">
            {#each (['7d', '14d', '30d'] as const) as range}
              <button
                class="sc-range-btn {insightsRange === range ? 'sc-range-btn--active' : ''}"
                onclick={() => { insightsRange = range; }}
              >{range}</button>
            {/each}
          </div>
        </div>
        <div class="sc-insights-chart">
          <canvas bind:this={sparkCanvas}></canvas>
        </div>
        <div class="sc-insights-stats">
          <div class="sc-insights-stat">
            <span class="sc-insights-stat-val">{sessions.length}</span>
            <span class="sc-insights-stat-label">Sessions</span>
          </div>
          <div class="sc-insights-stat">
            <span class="sc-insights-stat-val">{totalTimeMins > 0 ? formatMins(totalTimeMins) : '—'}</span>
            <span class="sc-insights-stat-label">Total time</span>
          </div>
          <div class="sc-insights-stat">
            <span class="sc-insights-stat-val">{avgDurationMins != null ? formatMins(avgDurationMins) : '—'}</span>
            <span class="sc-insights-stat-label">Avg session</span>
          </div>
          <div class="sc-insights-stat">
            <span class="sc-insights-stat-val">{activeDays > 0 ? activeDays : '—'}</span>
            <span class="sc-insights-stat-label">Active days</span>
          </div>
        </div>
      </div>

    </div>
  {/if}
{/if}

{#if contextMenu}
  <div
    class="sc-ctx-menu"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => { if (e.key === 'Escape') contextMenu = null; }}
    oncontextmenu={(e) => e.preventDefault()}
    role="menu"
    tabindex="-1"
  >
    <button class="sc-ctx-item sc-ctx-item--danger" onclick={() => handleUnlinkSession(contextMenu!.sessionId)}>
      Unlink session
    </button>
  </div>
{/if}
