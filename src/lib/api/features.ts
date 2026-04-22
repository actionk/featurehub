import { invoke } from "@tauri-apps/api/core";
import type { Feature, FeatureData, FeatureGroup } from "./types";

// ── Features ────────────────────────────────────────────────────────────

export async function getFeatures(): Promise<Feature[]> {
  return invoke<Feature[]>("get_features");
}

export async function getFeature(id: string): Promise<Feature> {
  return invoke<Feature>("get_feature", { id });
}

export async function getFeatureData(id: string): Promise<FeatureData> {
  return invoke<FeatureData>("get_feature_data", { id });
}

export async function createFeature(title: string, ticketId?: string | null, status?: string, description?: string | null, parentId?: string | null): Promise<Feature> {
  return invoke<Feature>("create_feature", {
    title,
    ticketId: ticketId ?? null,
    status: status ?? "todo",
    description: description ?? null,
    parentId: parentId ?? null,
  });
}

export async function updateFeature(
  id: string,
  updates: {
    title?: string;
    description?: string | null;
    ticket_id?: string | null;
    status?: string;
  }
): Promise<Feature> {
  return invoke<Feature>("update_feature", { id, ...updates });
}

export async function deleteFeature(id: string, cleanupRepos?: boolean): Promise<void> {
  return invoke<void>("delete_feature", { id, cleanupRepos: cleanupRepos ?? false });
}

export async function reorderFeatures(orderedIds: string[]): Promise<void> {
  return invoke<void>("reorder_features", { orderedIds });
}

export async function duplicateFeature(id: string): Promise<Feature> {
  return invoke<Feature>("duplicate_feature", { id });
}

export async function togglePinFeature(id: string): Promise<Feature> {
  return invoke<Feature>("toggle_pin_feature", { id });
}

export async function setFeatureArchived(id: string, archived: boolean): Promise<Feature> {
  return invoke<Feature>("set_feature_archived", { id, archived });
}

export async function setFeatureParent(id: string, parentId: string | null): Promise<Feature> {
  return invoke<Feature>("set_feature_parent", { id, parentId });
}

// ── Feature Groups ──────────────────────────────────────────────────────

export async function getFeatureGroups(): Promise<FeatureGroup[]> {
  return invoke<FeatureGroup[]>("get_feature_groups");
}

export async function createFeatureGroup(name: string, color?: string | null): Promise<FeatureGroup> {
  return invoke<FeatureGroup>("create_feature_group", { name, color: color ?? null });
}

export async function updateFeatureGroup(id: string, name?: string, color?: string | null): Promise<FeatureGroup> {
  return invoke<FeatureGroup>("update_feature_group", { id, name: name ?? null, color: color ?? null });
}

export async function deleteFeatureGroup(id: string): Promise<void> {
  return invoke<void>("delete_feature_group", { id });
}

export async function reorderFeatureGroups(ids: string[]): Promise<void> {
  return invoke<void>("reorder_feature_groups", { ids });
}

export async function setFeatureGroup(featureId: string, groupId: string | null): Promise<void> {
  return invoke<void>("set_feature_group", { featureId, groupId });
}
