import { invoke } from "@tauri-apps/api/core";
import type { Plan } from "./types";

// ── Plans ────────────────────────────────────────────────────────────────

export async function getPlans(featureId: string): Promise<Plan[]> {
  return invoke<Plan[]>("get_plans", { featureId });
}

export async function getPlan(id: string): Promise<Plan> {
  return invoke<Plan>("get_plan", { id });
}

export async function resolvePlan(id: string, status: string, feedback?: string | null): Promise<Plan> {
  return invoke<Plan>("resolve_plan", { id, status, feedback: feedback ?? null });
}

export async function deletePlan(id: string): Promise<void> {
  return invoke<void>("delete_plan", { id });
}
