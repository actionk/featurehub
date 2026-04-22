// Reactive store for per-tab toolbar actions.
// Each tab component registers its actions; FeatureDetail renders them for the active tab.

export interface ToolbarAction {
  id: string;
  label: string;
  icon?: string; // raw SVG string
  onClick: () => void;
  disabled?: boolean;
  variant?: "default" | "primary";
  title?: string;
}

let registry = $state<Record<string, ToolbarAction[]>>({});

export function setToolbarActions(tabId: string, actions: ToolbarAction[]) {
  registry[tabId] = actions;
}

export function clearToolbarActions(tabId: string) {
  delete registry[tabId];
}

export function getToolbarActions(tabId: string): ToolbarAction[] {
  return registry[tabId] ?? [];
}
