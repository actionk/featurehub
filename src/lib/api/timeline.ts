import { invoke } from "@tauri-apps/api/core";
import type { TimelineEvent } from "./types";

// ── Timeline ────────────────────────────────────────────────────────────

export async function getTimeline(featureId: string): Promise<TimelineEvent[]> {
  return invoke<TimelineEvent[]>("get_timeline", { featureId });
}

export interface GlobalTimelineEvent {
  event_type: string;
  title: string;
  detail: string | null;
  timestamp: string;
  feature_id: string;
  feature_title: string;
}

export async function getGlobalTimeline(): Promise<GlobalTimelineEvent[]> {
  return invoke<GlobalTimelineEvent[]>("get_global_timeline");
}
