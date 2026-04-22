export interface WorkspaceTab {
  id: string;
  featureId: string;
}

export const BOARD_TAB_FEATURE_ID = "__board__";

export function isBoardTab(tab: WorkspaceTab): boolean {
  return tab.featureId === BOARD_TAB_FEATURE_ID;
}

interface WorkspaceTabState {
  tabs: WorkspaceTab[];
  activeTabId: string | null;
}

const STORAGE_KEY = "featurehub:workspaceTabs";

function generateId(): string {
  return crypto.randomUUID();
}

function loadState(): WorkspaceTabState {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (Array.isArray(parsed.tabs)) {
        return { tabs: parsed.tabs, activeTabId: parsed.activeTabId ?? null };
      }
    }
  } catch {}
  return { tabs: [], activeTabId: null };
}

function saveState(tabs: WorkspaceTab[], activeTabId: string | null) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify({ tabs, activeTabId }));
}

const initial = loadState();
let tabs = $state<WorkspaceTab[]>(initial.tabs);
let activeTabId = $state<string | null>(initial.activeTabId);

function persist() {
  saveState(tabs, activeTabId);
}

/** Get all open workspace tabs */
export function getWorkspaceTabs(): WorkspaceTab[] {
  return tabs;
}

/** Get the active tab ID */
export function getActiveTabId(): string | null {
  return activeTabId;
}

/** Get the feature ID of the active tab */
export function getActiveFeatureId(): string | null {
  if (!activeTabId) return null;
  const tab = tabs.find(t => t.id === activeTabId);
  return tab?.featureId ?? null;
}

/** Get all open feature IDs */
export function getOpenFeatureIds(): string[] {
  return tabs.filter(t => t.featureId !== BOARD_TAB_FEATURE_ID).map(t => t.featureId);
}

/** Check if a feature already has an open tab */
export function hasTabForFeature(featureId: string): boolean {
  return tabs.some(t => t.featureId === featureId);
}

/** Find tab by feature ID */
export function findTabByFeature(featureId: string): WorkspaceTab | undefined {
  return tabs.find(t => t.featureId === featureId);
}

/**
 * Open a new tab for a feature, or switch to existing tab if already open.
 * Returns the tab ID.
 */
export function openTab(featureId: string): string {
  const existing = tabs.find(t => t.featureId === featureId);
  if (existing) {
    activeTabId = existing.id;
    persist();
    return existing.id;
  }
  const tab: WorkspaceTab = { id: generateId(), featureId };
  tabs = [...tabs, tab];
  activeTabId = tab.id;
  persist();
  return tab.id;
}

/** Open the board tab, or switch to it if already open. Returns the tab ID. */
export function openBoardTab(): string {
  const existing = tabs.find(t => t.featureId === BOARD_TAB_FEATURE_ID);
  if (existing) {
    activeTabId = existing.id;
    persist();
    return existing.id;
  }
  const tab: WorkspaceTab = { id: generateId(), featureId: BOARD_TAB_FEATURE_ID };
  tabs = [...tabs, tab];
  activeTabId = tab.id;
  persist();
  return tab.id;
}

/**
 * Normal click behavior: if feature has a tab, switch to it.
 * Otherwise replace the active tab's feature (or create a new tab if none exist).
 */
export function switchToFeature(featureId: string): string {
  const existing = tabs.find(t => t.featureId === featureId);
  if (existing) {
    activeTabId = existing.id;
    persist();
    return existing.id;
  }
  // Replace active tab's feature, or create new if no tabs
  if (activeTabId && tabs.length > 0) {
    const idx = tabs.findIndex(t => t.id === activeTabId);
    if (idx >= 0) {
      const newTab: WorkspaceTab = { id: generateId(), featureId };
      tabs = [...tabs.slice(0, idx), newTab, ...tabs.slice(idx + 1)];
      activeTabId = newTab.id;
      persist();
      return newTab.id;
    }
  }
  // No tabs exist — create first tab
  return openTab(featureId);
}

/** Switch to a specific tab by ID */
export function switchToTab(tabId: string) {
  if (tabs.some(t => t.id === tabId)) {
    activeTabId = tabId;
    persist();
  }
}

/** Close a tab. Returns the new active feature ID or null. */
export function closeTab(tabId: string): string | null {
  const idx = tabs.findIndex(t => t.id === tabId);
  if (idx < 0) return getActiveFeatureId();

  const wasActive = tabId === activeTabId;
  tabs = tabs.filter(t => t.id !== tabId);

  if (wasActive) {
    if (tabs.length === 0) {
      activeTabId = null;
    } else {
      // Prefer right neighbor, then left
      const newIdx = Math.min(idx, tabs.length - 1);
      activeTabId = tabs[newIdx].id;
    }
  }
  persist();
  return getActiveFeatureId();
}

/** Close all tabs except the specified one */
export function closeOtherTabs(tabId: string) {
  const keep = tabs.find(t => t.id === tabId);
  if (!keep) return;
  tabs = [keep];
  activeTabId = keep.id;
  persist();
}

/** Close all tabs */
export function closeAllTabs() {
  tabs = [];
  activeTabId = null;
  persist();
}

/** Close tab for a specific feature */
export function closeTabForFeature(featureId: string): string | null {
  const tab = tabs.find(t => t.featureId === featureId);
  if (tab) return closeTab(tab.id);
  return getActiveFeatureId();
}

/** Reorder tabs by moving from one index to another */
export function reorderTabs(fromIndex: number, toIndex: number) {
  if (fromIndex === toIndex) return;
  if (fromIndex < 0 || fromIndex >= tabs.length) return;
  if (toIndex < 0 || toIndex >= tabs.length) return;
  const newTabs = [...tabs];
  const [moved] = newTabs.splice(fromIndex, 1);
  newTabs.splice(toIndex, 0, moved);
  tabs = newTabs;
  persist();
}

/** Switch to the next tab (wraps around) */
export function nextTab() {
  if (tabs.length <= 1) return;
  const idx = tabs.findIndex(t => t.id === activeTabId);
  const next = (idx + 1) % tabs.length;
  activeTabId = tabs[next].id;
  persist();
}

/** Switch to the previous tab (wraps around) */
export function prevTab() {
  if (tabs.length <= 1) return;
  const idx = tabs.findIndex(t => t.id === activeTabId);
  const prev = (idx - 1 + tabs.length) % tabs.length;
  activeTabId = tabs[prev].id;
  persist();
}

/** Clear all tabs (used on storage switch) */
export function clearWorkspaceTabs() {
  tabs = [];
  activeTabId = null;
  localStorage.removeItem(STORAGE_KEY);
}

/** Remove tabs for features that no longer exist */
export function pruneInvalidTabs(validFeatureIds: Set<string>) {
  const before = tabs.length;
  tabs = tabs.filter(t => t.featureId === BOARD_TAB_FEATURE_ID || validFeatureIds.has(t.featureId));
  if (tabs.length !== before) {
    if (activeTabId && !tabs.some(t => t.id === activeTabId)) {
      activeTabId = tabs.length > 0 ? tabs[0].id : null;
    }
    persist();
  }
}
