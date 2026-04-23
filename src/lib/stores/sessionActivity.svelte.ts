import { getSessionsPanelData } from "../api/sessions";
import type { PanelSession } from "../api/types";
import { subscribe } from "./events.svelte";

let counts = $state<Record<string, number>>({});
let activeSessionIds = $state<Set<string>>(new Set());
let panelSessions = $state<PanelSession[]>([]);
let panelActiveCount = $state(0);

async function refresh() {
  try {
    const panel = await getSessionsPanelData();

    panelSessions = panel.sessions;
    panelActiveCount = panel.active_count;

    // Derive counts and active IDs from panel data (single source of truth)
    const newCounts: Record<string, number> = {};
    const newActiveIds = new Set<string>();

    for (const s of panel.sessions) {
      if (s.is_active || s.status === 'Active' || s.status === 'WaitingForInput') {
        newCounts[s.feature_id] = (newCounts[s.feature_id] ?? 0) + 1;
        newActiveIds.add(s.claude_session_id);
      }
    }

    counts = newCounts;
    activeSessionIds = newActiveIds;
  } catch {
    // ignore errors (e.g. no storage set yet)
  }
}

export function getActiveCountForFeature(featureId: string): number {
  return counts[featureId] ?? 0;
}

export function isAnySessionWaitingForFeature(featureId: string): boolean {
  return panelSessions.some(
    (s) => s.feature_id === featureId && s.status === "WaitingForInput",
  );
}

export function isSessionActive(claudeSessionId: string): boolean {
  return activeSessionIds.has(claudeSessionId);
}

export function getAllActiveSessionCounts(): Record<string, number> {
  return counts;
}

export function getPanelSessions(): PanelSession[] {
  return panelSessions;
}

export function getPanelActiveCount(): number {
  return panelActiveCount;
}

export function refreshSessionActivity(): Promise<void> {
  return refresh();
}

export function startSessionActivityPolling(intervalMs = 10_000): () => void {
  refresh();
  const interval = setInterval(refresh, intervalMs);
  const unsubscribe = subscribe("sessions:changed", () => {
    refresh();
  });
  return () => {
    clearInterval(interval);
    unsubscribe();
  };
}
