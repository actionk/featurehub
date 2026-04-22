import { invoke } from "@tauri-apps/api/core";

export interface ExportOptions {
  includeDone: boolean;
  includeArchived: boolean;
  includeFiles: boolean;
  includeSessions: boolean;
  includeTasks: boolean;
  includeNotes: boolean;
  includeContext: boolean;
  includePatches: boolean;
}

export interface RepoDirectory {
  directory_id: string;
  feature_id: string;
  feature_title: string;
  repo_url: string;
  label: string | null;
  has_patch: boolean;
}

export interface ImportResult {
  feature_count: number;
  file_count: number;
  storage_path: string;
  directories_with_repos: RepoDirectory[];
}

export interface ImportCheckResult {
  zip_path: string;
  total_features: number;
  duplicate_count: number;
  duplicate_titles: string[];
}

export async function exportStorage(
  options: ExportOptions
): Promise<string> {
  return invoke<string>("export_storage", {
    options: {
      include_done: options.includeDone,
      include_archived: options.includeArchived,
      include_files: options.includeFiles,
      include_sessions: options.includeSessions,
      include_tasks: options.includeTasks,
      include_notes: options.includeNotes,
      include_context: options.includeContext,
      include_patches: options.includePatches,
    },
  });
}

export async function cancelExport(): Promise<void> {
  return invoke<void>("cancel_export");
}

export async function checkImportZip(): Promise<ImportCheckResult> {
  return invoke<ImportCheckResult>("check_import_zip");
}

export async function importStorage(
  zipPath: string,
  strategy: "replace" | "ignore" | "merge"
): Promise<ImportResult> {
  return invoke<ImportResult>("import_storage", { zipPath, strategy });
}

export async function restoreRepoFromExport(
  zipPath: string,
  directoryId: string,
  targetPath: string
): Promise<void> {
  return invoke<void>("restore_repo_from_export", {
    zipPath,
    directoryId,
    targetPath,
  });
}
