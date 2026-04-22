import { invoke } from "@tauri-apps/api/core";
import type { StorageInfo } from "./types";

// ── Storage ─────────────────────────────────────────────────────────────

export async function getStorages(): Promise<StorageInfo[]> {
  return invoke<StorageInfo[]>("get_storages");
}

export async function getActiveStorage(): Promise<StorageInfo | null> {
  return invoke<StorageInfo | null>("get_active_storage");
}

export async function createStorage(path: string): Promise<StorageInfo> {
  return invoke<StorageInfo>("create_storage", { path });
}

export async function switchStorage(id: string): Promise<void> {
  return invoke<void>("switch_storage", { id });
}

export async function removeStorage(id: string): Promise<void> {
  return invoke<void>("remove_storage", { id });
}

export async function renameStorage(id: string, newPath: string): Promise<StorageInfo> {
  return invoke<StorageInfo>("rename_storage", { id, newPath });
}

export async function updateStorageIcon(id: string, icon: string | null): Promise<StorageInfo> {
  return invoke<StorageInfo>("update_storage_icon", { id, icon });
}

export async function getStorageGitStatus(path: string): Promise<string> {
  return invoke<string>("get_storage_git_status", { path });
}

export async function pickStorageFolder(): Promise<string | null> {
  return invoke<string | null>("pick_storage_folder");
}
