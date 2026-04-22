import { invoke } from "@tauri-apps/api/core";
import type { AppNotification } from "./types";

// ── Notifications ────────────────────────────────────────────────────

export async function pollNotifications(): Promise<AppNotification[]> {
  return invoke<AppNotification[]>("poll_notifications");
}
