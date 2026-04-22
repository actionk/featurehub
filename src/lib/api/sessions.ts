import { invoke } from "@tauri-apps/api/core";
import type { Session, SessionActivity, SessionsPanelData } from "./types";
import { getFeature } from "./features";

// ── Sessions ────────────────────────────────────────────────────────────

export async function getSessions(featureId: string): Promise<Session[]> {
  return invoke<Session[]>("get_sessions", { featureId });
}

export async function scanSessions(featureId?: string): Promise<Session[]> {
  return invoke<Session[]>("scan_sessions", { featureId: featureId ?? null });
}

export async function checkSessionActive(sessionId: string): Promise<boolean> {
  return invoke<boolean>("check_session_active", { sessionId });
}

export async function getActiveSessionCounts(): Promise<Record<string, number>> {
  return invoke<Record<string, number>>("get_active_session_counts");
}

export async function getActiveSessionActivity(): Promise<SessionActivity> {
  return invoke<SessionActivity>("get_active_session_activity");
}

export async function getSessionsPanelData(): Promise<SessionsPanelData> {
  return invoke<SessionsPanelData>("get_sessions_panel_data");
}

export async function linkSession(featureId: string, sessionId: string): Promise<Session> {
  return invoke<Session>("link_session", { featureId, sessionId });
}

export async function renameSession(id: string, title: string): Promise<void> {
  return invoke<void>("rename_session", { id, title });
}

export async function unlinkSession(id: string): Promise<void> {
  return invoke<void>("unlink_session", { id });
}

export async function resumeSession(id: string): Promise<void> {
  return invoke<void>("resume_session", { id });
}

export async function ensureMcpConfig(projectPath?: string | null, claudeSessionId?: string | null): Promise<void> {
  return invoke<void>("ensure_mcp_config", {
    projectPath: projectPath ?? null,
    claudeSessionId: claudeSessionId ?? null,
  });
}

export async function startNewSession(featureId: string): Promise<void> {
  const feature = await getFeature(featureId);
  const directories = (feature.directories ?? []).filter((d) => d.clone_status === "ready" || !d.clone_status).map((d) => d.path);
  const featureTitle = feature.title;
  return invoke<void>("start_new_session", { featureId, directories, featureTitle });
}
