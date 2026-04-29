export interface TerminalSidebarStatusInput {
  exited: boolean;
  needsInput: boolean;
  statusLine?: string;
}

export function getTerminalSidebarStatus(term: TerminalSidebarStatusInput): string {
  if (term.needsInput) return "Waiting";
  if (term.exited) return "Exited";
  return "Running";
}
