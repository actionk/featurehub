import { invoke } from "@tauri-apps/api/core";
import type { Task } from "./types";

// ── Tasks ───────────────────────────────────────────────────────────────

export async function getTasks(featureId: string): Promise<Task[]> {
  return invoke<Task[]>("get_tasks", { featureId });
}

export async function createTask(
  featureId: string,
  title: string,
  opts?: {
    source?: string;
    externalKey?: string;
    externalUrl?: string;
    externalStatus?: string;
    description?: string;
  }
): Promise<Task> {
  return invoke<Task>("create_task", {
    featureId,
    title,
    source: opts?.source ?? null,
    externalKey: opts?.externalKey ?? null,
    externalUrl: opts?.externalUrl ?? null,
    externalStatus: opts?.externalStatus ?? null,
    description: opts?.description ?? null,
  });
}

export async function updateTask(
  id: string,
  title?: string,
  done?: boolean,
  externalStatus?: string,
  description?: string
): Promise<Task> {
  return invoke<Task>("update_task", {
    id,
    title: title ?? null,
    done: done ?? null,
    externalStatus: externalStatus ?? null,
    description: description ?? null,
  });
}

export async function deleteTask(id: string): Promise<void> {
  return invoke<void>("delete_task", { id });
}
