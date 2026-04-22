import { invoke } from "@tauri-apps/api/core";
import type { ExtensionInfo } from "./types";

export async function getExtensions(): Promise<ExtensionInfo[]> {
  return invoke<ExtensionInfo[]>("get_extensions");
}

export async function getExtensionBadge(
  extensionId: string,
  tabId: string,
  featureId: string
): Promise<number> {
  return invoke<number>("get_extension_badge", { extensionId, tabId, featureId });
}
