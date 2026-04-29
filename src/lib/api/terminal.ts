import { invoke } from "@tauri-apps/api/core";
import type { PtySessionResult } from "./types";

// ── Terminal / PTY ──────────────────────────────────────────────────────

export async function ptySpawn(
  featureId: string,
  cols: number,
  rows: number,
  cwd?: string | null,
  shell?: string | null,
  args?: string[] | null,
): Promise<string> {
  return invoke<string>("pty_spawn", {
    featureId,
    cols,
    rows,
    cwd: cwd ?? null,
    shell: shell ?? null,
    args: args ?? null,
  });
}

export async function ptyWrite(id: string, data: string): Promise<void> {
  return invoke<void>("pty_write", { id, data });
}

export async function ptyResize(id: string, cols: number, rows: number): Promise<void> {
  return invoke<void>("pty_resize", { id, cols, rows });
}

export async function ptyKill(id: string): Promise<void> {
  return invoke<void>("pty_kill", { id });
}

export async function ptyKillFeature(featureId: string): Promise<void> {
  return invoke<void>("pty_kill_feature", { featureId });
}

export async function ptySpawnSession(
  featureId: string,
  directories: string[],
  featureTitle: string,
  cols: number,
  rows: number,
  context?: string | null,
  dangerouslySkipPermissions?: boolean,
): Promise<PtySessionResult> {
  return invoke<PtySessionResult>("pty_spawn_session", {
    featureId,
    directories,
    featureTitle,
    cols,
    rows,
    context: context ?? null,
    dangerouslySkipPermissions: dangerouslySkipPermissions ?? false,
  });
}

export async function ptyResumeSession(
  sessionDbId: string,
  cols: number,
  rows: number,
): Promise<PtySessionResult> {
  return invoke<PtySessionResult>("pty_resume_session", {
    sessionDbId,
    cols,
    rows,
  });
}

export interface ActiveTerminalInfo {
  terminal_id: string;
  feature_id: string;
  session_db_id: string | null;
  label: string | null;
}

export async function ptyListActive(): Promise<ActiveTerminalInfo[]> {
  return invoke<ActiveTerminalInfo[]>("pty_list_active");
}

/** Returns the rolling PTY output buffer as a base64 string (up to 256 KB). */
export async function ptyGetScrollback(id: string): Promise<string> {
  return invoke<string>("pty_get_scrollback", { id });
}

export async function finishEmbeddedSession(sessionDbId: string): Promise<void> {
  return invoke<void>("finish_embedded_session", { sessionDbId });
}

export async function cleanupOrphanedSessions(): Promise<number> {
  return invoke<number>("cleanup_orphaned_sessions");
}
