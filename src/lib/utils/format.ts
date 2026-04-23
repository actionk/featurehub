const months = [
  "Jan", "Feb", "Mar", "Apr", "May", "Jun",
  "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

export function formatDate(iso: string): string {
  const d = new Date(iso);
  return `${months[d.getMonth()]} ${d.getDate()}, ${d.getFullYear()}`;
}

export function formatRelativeTime(iso: string): string {
  const now = Date.now();
  const then = new Date(iso).getTime();
  const diffSec = Math.floor((now - then) / 1000);

  if (diffSec < 60) return "just now";
  const diffMin = Math.floor(diffSec / 60);
  if (diffMin < 60) return `${diffMin}m ago`;
  const diffHr = Math.floor(diffMin / 60);
  if (diffHr < 24) return `${diffHr}h ago`;
  const diffDay = Math.floor(diffHr / 24);
  if (diffDay < 30) return `${diffDay}d ago`;
  const diffMo = Math.floor(diffDay / 30);
  if (diffMo < 12) return `${diffMo}mo ago`;
  const diffYr = Math.floor(diffMo / 12);
  return `${diffYr}y ago`;
}

export type TimeAge = 'fresh' | 'recent' | 'old' | 'stale';

export function getTimeAge(iso: string): TimeAge {
  const now = Date.now();
  const then = new Date(iso).getTime();
  const diffMs = now - then;
  const diffHr = diffMs / (1000 * 60 * 60);

  if (diffHr < 1) return 'fresh';      // < 1 hour
  if (diffHr < 24) return 'recent';    // < 1 day
  if (diffHr < 168) return 'old';      // < 1 week
  return 'stale';                       // >= 1 week
}

export function formatDuration(mins: number): string {
  if (mins < 60) return `${mins}m`;
  const h = Math.floor(mins / 60);
  const m = mins % 60;
  return m > 0 ? `${h}h ${m}m` : `${h}h`;
}

export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const kb = bytes / 1024;
  if (kb < 1024) return `${kb.toFixed(1)} KB`;
  const mb = kb / 1024;
  if (mb < 1024) return `${mb.toFixed(1)} MB`;
  const gb = mb / 1024;
  return `${gb.toFixed(2)} GB`;
}

export function formatElapsed(isoStart: string, now: number = Date.now()): string {
  const diffMs = now - new Date(isoStart).getTime();
  if (diffMs < 0) return '0s';
  const diffSec = Math.floor(diffMs / 1000);
  if (diffSec < 60) return `${diffSec}s`;
  const diffMin = Math.floor(diffSec / 60);
  if (diffMin < 60) return `${diffMin}m`;
  const h = Math.floor(diffMin / 60);
  const m = diffMin % 60;
  return m > 0 ? `${h}h ${m}m` : `${h}h`;
}

export function fileTypeColor(ext: string | undefined | null): string {
  if (!ext) return 'var(--text-muted)';
  const map: Record<string, string> = {
    md: 'var(--amber)', ts: 'var(--cyan)', tsx: 'var(--cyan)',
    js: 'var(--amber)', jsx: 'var(--amber)',
    svelte: 'var(--pink)', rs: 'var(--accent)',
    toml: 'var(--text-muted)', json: 'var(--green)',
    css: 'var(--blue)', html: 'var(--red)',
    sql: 'var(--violet)', py: 'var(--cyan)',
    png: 'var(--pink)', jpg: 'var(--pink)', jpeg: 'var(--pink)',
    svg: 'var(--violet)', gif: 'var(--pink)', webp: 'var(--pink)',
    pdf: 'var(--red)', txt: 'var(--text-secondary)',
    sh: 'var(--green)', bash: 'var(--green)', zsh: 'var(--green)',
  };
  return map[ext.toLowerCase()] ?? 'var(--text-muted)';
}

export function linkTypeColor(type: string | undefined | null): string {
  if (!type) return 'var(--text-muted)';
  const map: Record<string, string> = {
    github: 'var(--text-secondary)',
    gitlab: 'var(--amber)',
    jira: 'var(--blue)',
    linear: 'var(--violet)',
    figma: 'var(--pink)',
    confluence: 'var(--blue)',
    slack: 'var(--violet)',
    notion: 'var(--text-secondary)',
    discord: 'var(--violet)',
    google: 'var(--cyan)',
    youtube: 'var(--red)',
    loom: 'var(--violet)',
    arena: 'var(--green)',
    drive: 'var(--green)',
    docs: 'var(--blue)',
    sheets: 'var(--green)',
    miro: 'var(--amber)',
  };
  return map[type.toLowerCase()] ?? 'var(--accent)';
}

export function eventColor(kind: string | undefined | null): string {
  if (!kind) return 'var(--text-muted)';
  const k = kind.toLowerCase();
  if (k.includes('error') || k.includes('fail') || k.includes('reject')) return 'var(--red)';
  if (k.includes('warn') || k.includes('paus')) return 'var(--amber)';
  if (k.includes('done') || k.includes('approved') || k.includes('complete') || k.includes('success')) return 'var(--cyan)';
  if (k.includes('plan')) return 'var(--violet)';
  if (k.includes('session') || k.includes('agent')) return 'var(--accent)';
  if (k.includes('task')) return 'var(--blue)';
  if (k.includes('note') || k.includes('context')) return 'var(--pink)';
  return 'var(--accent)';
}
