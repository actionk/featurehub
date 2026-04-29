import type { Component } from "svelte";
import type { Feature, Session, Task, Plan, Note, Tag } from "../api/tauri";

export interface TabContext {
  featureId: string;
  feature: Feature;
  sessions: Session[];
  tasks: Task[];
  plans: Plan[];
  note: Note | null;
  allTags: Tag[];
  activeSessionCount: number;
  pendingPlanId?: string | null;
  onPendingPlanHandled?: () => void;
  onRefresh: () => void;
  onSessionsChanged: () => void;
  onOpenSettings?: (tab?: string) => void;
  isTabActive?: boolean;
}

export interface TabBadge {
  text: string;
  style?: "default" | "active" | "warning";
  title?: string;
}

export interface TabModule {
  id: string;
  label: string;
  emoji: string;
  shortcutKey: string;
  sortOrder: number;
  component: Component;
  getBadges: (ctx: TabContext) => TabBadge[];
  preload?: boolean;
  panelStyle?: string;
  /** Extra props spread onto the component alongside TabContext (used by extension tabs for componentPath). */
  extraProps?: Record<string, unknown>;
}

const tabs: TabModule[] = [];

export function registerTab(mod: TabModule): void {
  // Replace if already registered (hot reload)
  const idx = tabs.findIndex((t) => t.id === mod.id);
  if (idx >= 0) {
    tabs[idx] = mod;
  } else {
    tabs.push(mod);
  }
  // Keep sorted
  tabs.sort((a, b) => a.sortOrder - b.sortOrder);
}

export function getRegisteredTabs(): readonly TabModule[] {
  return tabs;
}
