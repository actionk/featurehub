export interface TerminalSidebarStatusInput {
  exited: boolean;
  needsInput: boolean;
  statusLine?: string;
  status?: string | null;
  lastAction?: string | null;
}

const ACTION_STATUS_PATTERNS = [
  /^(Reading|Writing|Editing|Searching|Running|Listing|Creating|Deleting|Moving|Copying|Updating|Opening|Fetching|Checking|Installing|Building|Testing)\b/i,
  /^Using\b/i,
  /^Calling\b/i,
  /^Thinking\b/i,
  /^Asking\b/i,
];

function normalizeStatusLine(statusLine?: string): string | null {
  const line = statusLine?.trim();
  if (!line) return null;
  if (/^[-\s|+]+$/.test(line)) return null;
  if (/^Claude Code\b/i.test(line)) return null;
  if (/^[A-Z]:[\\/]/i.test(line) || /^[/~]/.test(line)) return null;
  if (!ACTION_STATUS_PATTERNS.some((pattern) => pattern.test(line))) return null;
  return line.length > 48 ? `${line.slice(0, 47)}...` : line;
}

export function getTerminalSidebarStatus(term: TerminalSidebarStatusInput): string {
  if (term.needsInput || term.status === "WaitingForInput") return "Waiting";
  if (term.exited) return "Exited";
  const lastAction = normalizeStatusLine(term.lastAction ?? undefined);
  if (lastAction) return lastAction;
  const status = normalizeStatusLine(term.statusLine);
  if (status) return status;
  return "Running";
}
