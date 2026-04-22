import { invoke } from "@tauri-apps/api/core";
import type { Directory, GitStatusSummary } from "./types";

// ── Repositories ─────────────────────────────────────────────────────────

export async function cloneRepository(featureId: string, repoUrl: string, name?: string | null): Promise<Directory> {
  return invoke<Directory>("clone_repository", { featureId, repoUrl, name: name ?? null });
}

export async function retryClone(directoryId: string): Promise<Directory> {
  return invoke<Directory>("retry_clone", { directoryId });
}

export async function removeDirectory(id: string): Promise<void> {
  return invoke<void>("remove_directory", { id });
}

export async function openPath(path: string): Promise<void> {
  return invoke<void>("open_path", { path });
}

// ── Git Info ────────────────────────────────────────────────────────────

export async function getGitCurrentBranch(directoryPath: string): Promise<string> {
  return invoke<string>("get_git_current_branch", { directoryPath });
}

export async function getGitStatus(directoryPath: string): Promise<GitStatusSummary> {
  return invoke<GitStatusSummary>("get_git_status", { directoryPath });
}

export async function listGitBranches(directoryPath: string): Promise<string[]> {
  return invoke<string[]>("list_git_branches", { directoryPath });
}

export async function checkoutGitBranch(directoryPath: string, branchName: string): Promise<void> {
  return invoke<void>("checkout_git_branch", { directoryPath, branchName });
}

export async function gitFetch(directoryPath: string): Promise<void> {
  return invoke<void>("git_fetch", { directoryPath });
}

export async function createGitBranch(directoryPath: string, branchName: string): Promise<void> {
  return invoke<void>("create_git_branch", { directoryPath, branchName });
}

export async function cleanupFeatureRepos(featureId: string): Promise<void> {
  return invoke<void>("cleanup_feature_repos", { featureId });
}
