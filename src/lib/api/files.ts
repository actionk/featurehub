import { invoke } from "@tauri-apps/api/core";
import type { FileEntry, Folder, FilePreview } from "./types";

// ── Files ───────────────────────────────────────────────────────────────

export async function getFiles(featureId: string): Promise<FileEntry[]> {
  return invoke<FileEntry[]>("get_files", { featureId });
}

export async function addFiles(featureId: string, paths: string[], folderId?: string | null): Promise<FileEntry[]> {
  return invoke<FileEntry[]>("add_files", { featureId, paths, folderId: folderId ?? null });
}

export async function deleteFile(id: string): Promise<void> {
  return invoke<void>("delete_file", { id });
}

export async function renameFile(id: string, newName: string): Promise<FileEntry> {
  return invoke<FileEntry>("rename_file", { id, newName });
}

export async function openFile(id: string): Promise<void> {
  return invoke<void>("open_file", { id });
}

export async function revealFile(id: string): Promise<void> {
  return invoke<void>("reveal_file", { id });
}

export async function getFilePath(id: string): Promise<string> {
  return invoke<string>("get_file_path", { id });
}

export async function getFilesDirectory(featureId: string): Promise<string> {
  return invoke<string>("get_files_directory", { featureId });
}

export async function openFilesDirectory(featureId: string): Promise<void> {
  return invoke<void>("open_files_directory", { featureId });
}

export async function syncWorkspaceFiles(featureId: string): Promise<FileEntry[]> {
  return invoke<FileEntry[]>("sync_workspace_files", { featureId });
}

// ── Folders ─────────────────────────────────────────────────────────────

export async function getFolders(featureId: string): Promise<Folder[]> {
  return invoke<Folder[]>("get_folders", { featureId });
}

export async function createFolder(featureId: string, parentId: string | null, name: string): Promise<Folder> {
  return invoke<Folder>("create_folder", { featureId, parentId, name });
}

export async function renameFolder(id: string, newName: string): Promise<Folder> {
  return invoke<Folder>("rename_folder", { id, newName });
}

export async function deleteFolder(id: string): Promise<void> {
  return invoke<void>("delete_folder", { id });
}

export async function moveFolder(id: string, newParentId: string | null): Promise<Folder> {
  return invoke<Folder>("move_folder", { id, newParentId });
}

export async function moveFile(id: string, folderId: string | null): Promise<FileEntry> {
  return invoke<FileEntry>("move_file", { id, folderId });
}

export async function previewFile(id: string): Promise<FilePreview> {
  return invoke<FilePreview>("preview_file", { id });
}

export async function saveFileContent(id: string, content: string): Promise<void> {
  return invoke<void>("save_file_content", { id, content });
}
