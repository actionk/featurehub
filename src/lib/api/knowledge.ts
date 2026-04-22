import { invoke } from "@tauri-apps/api/core";
import type { KnowledgeEntry, KnowledgeFolder } from "./types";

// ─── Folders ────────────────────────────────────────────────────────────────

export async function getKnowledgeFolders(): Promise<KnowledgeFolder[]> {
  return invoke<KnowledgeFolder[]>("get_knowledge_folders");
}

export async function createKnowledgeFolder(
  name: string,
  parentId?: string | null,
): Promise<KnowledgeFolder> {
  return invoke<KnowledgeFolder>("create_knowledge_folder", {
    name,
    parentId: parentId ?? null,
  });
}

export async function renameKnowledgeFolder(
  id: string,
  name: string,
): Promise<KnowledgeFolder> {
  return invoke<KnowledgeFolder>("rename_knowledge_folder", { id, name });
}

export async function deleteKnowledgeFolder(id: string): Promise<void> {
  return invoke<void>("delete_knowledge_folder", { id });
}

// ─── Entries ────────────────────────────────────────────────────────────────

export async function getKnowledgeEntries(
  folderId?: string | null,
): Promise<KnowledgeEntry[]> {
  return invoke<KnowledgeEntry[]>("get_knowledge_entries", {
    folderId: folderId ?? null,
  });
}

export async function getAllKnowledgeEntries(): Promise<KnowledgeEntry[]> {
  return invoke<KnowledgeEntry[]>("get_all_knowledge_entries");
}

export async function getKnowledgeEntry(id: string): Promise<KnowledgeEntry> {
  return invoke<KnowledgeEntry>("get_knowledge_entry", { id });
}

export async function createKnowledgeEntry(params: {
  title: string;
  content: string;
  description?: string;
  folderId?: string | null;
  featureId?: string | null;
}): Promise<KnowledgeEntry> {
  return invoke<KnowledgeEntry>("create_knowledge_entry", {
    title: params.title,
    content: params.content,
    description: params.description ?? null,
    folderId: params.folderId ?? null,
    featureId: params.featureId ?? null,
  });
}

export async function updateKnowledgeEntry(params: {
  id: string;
  title?: string;
  content?: string;
  description?: string;
  folderId?: string | null;
  featureId?: string | null;
}): Promise<KnowledgeEntry> {
  return invoke<KnowledgeEntry>("update_knowledge_entry", {
    id: params.id,
    title: params.title ?? null,
    content: params.content ?? null,
    description: params.description ?? null,
    folderId: params.folderId !== undefined ? params.folderId : null,
    featureId: params.featureId !== undefined ? params.featureId : null,
  });
}

export async function deleteKnowledgeEntry(id: string): Promise<void> {
  return invoke<void>("delete_knowledge_entry", { id });
}
