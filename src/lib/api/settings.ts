import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, CliInstallResult, FeatureMcpServer, FeatureSkill, DetectedIde } from "./types";

// ── CLI / Settings ───────────────────────────────────────────────────

export async function getFhCliPath(): Promise<string> {
  return invoke<string>("get_fh_cli_path");
}

export async function installCliToPath(): Promise<CliInstallResult> {
  return invoke<CliInstallResult>("install_cli_to_path");
}

export async function checkCliInstalled(): Promise<string | null> {
  return invoke<string | null>("check_cli_installed");
}

export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

export async function saveSettings(settings: Partial<AppSettings>): Promise<AppSettings> {
  return invoke<AppSettings>("save_settings", {
    fhCliPath: settings.fh_cli_path ?? null,
    mcpServers: settings.mcp_servers ?? null,
    defaultRepositories: settings.default_repositories ?? null,
    mermaidDiagrams: settings.mermaid_diagrams ?? null,
    openfgaHighlighting: settings.openfga_highlighting ?? null,
    showTabEmojis: settings.show_tab_emojis ?? null,
    uiFont: settings.ui_font ?? null,
    monoFont: settings.mono_font ?? null,
    uiFontSize: settings.ui_font_size ?? null,
    terminalFontSize: settings.terminal_font_size ?? null,
    extensions: settings.extensions ?? null,
    preferredIdes: settings.preferred_ides ?? null,
    skills: settings.skills ?? null,
  });
}

// ── Feature MCP Servers ──────────────────────────────────────────────

export async function getFeatureMcpServers(featureId: string): Promise<FeatureMcpServer[]> {
  return invoke<FeatureMcpServer[]>("get_feature_mcp_servers", { featureId });
}

export async function setFeatureMcpServer(featureId: string, serverName: string, enabled: boolean): Promise<void> {
  return invoke<void>("set_feature_mcp_server", { featureId, serverName, enabled });
}

// ── Feature Skills ───────────────────────────────────────────────────

export async function getFeatureSkills(featureId: string): Promise<FeatureSkill[]> {
  return invoke<FeatureSkill[]>("get_feature_skills", { featureId });
}

export async function setFeatureSkill(featureId: string, skillId: string, enabled: boolean): Promise<void> {
  return invoke<void>("set_feature_skill", { featureId, skillId, enabled });
}

// ── IDE Detection ───────────────────────────────────────────────────────

export async function detectIdes(): Promise<DetectedIde[]> {
  return invoke<DetectedIde[]>("detect_ides");
}

export async function openInIde(path: string, ideCommand: string): Promise<void> {
  return invoke<void>("open_in_ide", { path, ideCommand });
}
