export interface ActiveTerminal {
  terminalId: string;
  sessionDbId: string;
  featureId: string;
  featureTitle: string;
  label: string;
  exited: boolean;
  statusLine: string;
  needsInput: boolean;
}

let terminals = $state<ActiveTerminal[]>([]);

export function getActiveTerminals(): ActiveTerminal[] {
  return terminals;
}

export function restoreTerminals(restored: ActiveTerminal[]) {
  // Merge restored terminals with any that already exist (avoid duplicates)
  const existingIds = new Set(terminals.map(t => t.terminalId));
  const toAdd = restored.filter(t => !existingIds.has(t.terminalId));
  if (toAdd.length > 0) {
    terminals = [...terminals, ...toAdd];
  }
}

export function getTerminalsForFeature(featureId: string): ActiveTerminal[] {
  return terminals.filter(t => t.featureId === featureId);
}

export function addTerminal(terminal: Omit<ActiveTerminal, "statusLine" | "needsInput">) {
  terminals = [...terminals, { ...terminal, statusLine: "", needsInput: false }];
}

export function removeTerminal(terminalId: string) {
  terminals = terminals.filter(t => t.terminalId !== terminalId);
}

export function removeTerminalsForFeature(featureId: string) {
  terminals = terminals.filter(t => t.featureId !== featureId);
}

export function markExited(terminalId: string) {
  terminals = terminals.map(t =>
    t.terminalId === terminalId ? { ...t, exited: true, statusLine: "Process exited", needsInput: false } : t
  );
}

export function updateLabel(terminalId: string, label: string) {
  terminals = terminals.map(t =>
    t.terminalId === terminalId ? { ...t, label } : t
  );
}

export function updateStatus(terminalId: string, statusLine: string, needsInput: boolean) {
  terminals = terminals.map(t =>
    t.terminalId === terminalId ? { ...t, statusLine, needsInput } : t
  );
}

// Pending session to resume in an embedded terminal (set by sessions panel, consumed by AiPanel)
let pendingResumeSessionDbId = $state<string | null>(null);
let pendingResumeVersion = $state(0);

export function requestResumeSession(sessionDbId: string) {
  pendingResumeSessionDbId = sessionDbId;
  pendingResumeVersion++;
}

export function getPendingResumeRequest(): { sessionDbId: string | null; version: number } {
  return { sessionDbId: pendingResumeSessionDbId, version: pendingResumeVersion };
}

// Pending terminal to view (set by sidebar, consumed by AiPanel).
// terminalId === null means "show overview" (close any open terminal).
let pendingViewTerminalId = $state<string | null>(null);
let pendingViewVersion = $state(0);
let pendingViewIsClear = $state(false);

export function requestViewTerminal(terminalId: string) {
  pendingViewTerminalId = terminalId;
  pendingViewIsClear = false;
  pendingViewVersion++;
}

export function requestShowOverview() {
  pendingViewTerminalId = null;
  pendingViewIsClear = true;
  pendingViewVersion++;
}

export function getPendingViewRequest(): { terminalId: string | null; isClear: boolean; version: number } {
  return { terminalId: pendingViewTerminalId, isClear: pendingViewIsClear, version: pendingViewVersion };
}

export function consumePendingView(): string | null {
  const id = pendingViewTerminalId;
  pendingViewTerminalId = null;
  return id;
}

// Which terminal is currently being viewed (set by AiPanel, read by Sidebar)
let viewingTerminalId = $state<string | null>(null);

export function setViewingTerminal(terminalId: string | null) {
  viewingTerminalId = terminalId;
}

export function getViewingTerminal(): string | null {
  return viewingTerminalId;
}
