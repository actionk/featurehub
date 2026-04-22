import { invoke } from "@tauri-apps/api/core";
import type { Tag } from "./types";

// ── Tags ────────────────────────────────────────────────────────────────

export async function getTags(): Promise<Tag[]> {
  return invoke<Tag[]>("get_tags");
}

export async function createTag(name: string, color: string): Promise<Tag> {
  return invoke<Tag>("create_tag", { name, color });
}

export async function deleteTag(id: string): Promise<void> {
  return invoke<void>("delete_tag", { id });
}

export async function toggleTag(featureId: string, tagId: string): Promise<void> {
  return invoke<void>("toggle_tag", { featureId, tagId });
}
