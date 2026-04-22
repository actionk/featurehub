import { getSettings, type AppSettings } from "../api/tauri";

let mermaidDiagrams = $state(false);
let openfgaHighlighting = $state(false);
let showTabEmojis = $state(false);
let uiFont = $state<string | null>(null);
let monoFont = $state<string | null>(null);
let uiFontSize = $state<number | null>(null);
let terminalFontSize = $state<number | null>(null);
let loaded = $state(false);

// Cached full settings object — avoids repeated IPC calls from child components
let cachedSettings = $state<AppSettings | null>(null);

const DEFAULT_UI_FONT = "'Plus Jakarta Sans', -apple-system, BlinkMacSystemFont, sans-serif";
const DEFAULT_MONO_FONT = "'JetBrains Mono', monospace";

function applyAppearance() {
  const root = document.documentElement;
  root.style.setProperty("--font-ui", uiFont ? `'${uiFont}', -apple-system, BlinkMacSystemFont, sans-serif` : DEFAULT_UI_FONT);
  root.style.setProperty("--font-mono", monoFont ? `'${monoFont}', monospace` : DEFAULT_MONO_FONT);
  root.style.setProperty("--font-size", uiFontSize ? `${uiFontSize}px` : "13px");
}

export function getMermaidEnabled() {
  return mermaidDiagrams;
}

export function getOpenFgaEnabled() {
  return openfgaHighlighting;
}

export function getShowTabEmojis() {
  return showTabEmojis;
}

export function getSettingsLoaded() {
  return loaded;
}

export function getUiFont() {
  return uiFont;
}

export function getMonoFont() {
  return monoFont;
}

export function getUiFontSize() {
  return uiFontSize;
}

export function getTerminalFontSize() {
  return terminalFontSize;
}

export async function loadAppSettings() {
  try {
    const settings = await getSettings();
    cachedSettings = settings;
    mermaidDiagrams = settings.mermaid_diagrams ?? false;
    openfgaHighlighting = settings.openfga_highlighting ?? false;
    showTabEmojis = settings.show_tab_emojis ?? false;
    uiFont = settings.ui_font ?? null;
    monoFont = settings.mono_font ?? null;
    uiFontSize = settings.ui_font_size ?? null;
    terminalFontSize = settings.terminal_font_size ?? null;
    applyAppearance();
    loaded = true;
  } catch (e) {
    console.error("Failed to load app settings:", e);
    loaded = true;
  }
}

/** Returns cached settings synchronously, or fetches if not yet loaded. */
export async function getCachedSettings(): Promise<AppSettings> {
  if (cachedSettings) return cachedSettings;
  const settings = await getSettings();
  cachedSettings = settings;
  return settings;
}

/** Invalidate the settings cache (call after saving settings). */
export function invalidateSettingsCache() {
  cachedSettings = null;
}

export function setMermaidEnabled(value: boolean) {
  mermaidDiagrams = value;
}

export function setOpenFgaEnabled(value: boolean) {
  openfgaHighlighting = value;
}

export function setShowTabEmojis(value: boolean) {
  showTabEmojis = value;
}

export function setUiFont(value: string | null) {
  uiFont = value;
  applyAppearance();
}

export function setMonoFont(value: string | null) {
  monoFont = value;
  applyAppearance();
}

export function setUiFontSize(value: number | null) {
  uiFontSize = value;
  applyAppearance();
}

export function setTerminalFontSize(value: number | null) {
  terminalFontSize = value;
}
