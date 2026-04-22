import { invoke } from "@tauri-apps/api/core";
import type { Link } from "./types";

// ── Links ───────────────────────────────────────────────────────────────

export async function addLink(
  featureId: string,
  title: string,
  url: string,
  linkType: string,
  description?: string | null
): Promise<Link> {
  return invoke<Link>("add_link", { featureId, title, url, linkType, description: description ?? null });
}

export async function updateLink(
  id: string,
  title: string,
  url: string,
  linkType: string,
  description?: string | null
): Promise<Link> {
  return invoke<Link>("update_link", { id, title, url, linkType, description: description ?? null });
}

export async function deleteLink(id: string): Promise<void> {
  return invoke<void>("delete_link", { id });
}
