import { invoke } from "@tauri-apps/api/core";
import type { Context } from "./types";

// ── Context ─────────────────────────────────────────────────────────────

export async function getContext(featureId: string): Promise<Context | null> {
  return invoke<Context | null>("get_context", { featureId });
}

export async function saveContext(featureId: string, content: string): Promise<Context> {
  return invoke<Context>("save_context", { featureId, content });
}
