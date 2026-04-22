import { invoke } from "@tauri-apps/api/core";
import type { Note } from "./types";

// ── Notes ───────────────────────────────────────────────────────────────

export async function getNote(featureId: string): Promise<Note | null> {
  return invoke<Note | null>("get_note", { featureId });
}

export async function saveNote(featureId: string, content: string): Promise<Note> {
  return invoke<Note>("save_note", { featureId, content });
}
