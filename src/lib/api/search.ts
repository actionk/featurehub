import { invoke } from "@tauri-apps/api/core";
import type { SearchResult } from "./types";

// ── Search ──────────────────────────────────────────────────────────────

export async function globalSearch(query: string): Promise<SearchResult[]> {
  return invoke<SearchResult[]>("global_search", { query });
}

export async function rebuildSearchIndex(): Promise<void> {
  return invoke<void>("rebuild_search_index");
}
