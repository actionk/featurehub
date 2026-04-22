export interface LinkTypeInfo {
  label: string;
  color: string;
  icon: string; // SVG path data for a 16x16 viewBox
}

const GITHUB_ICON = "M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0016 8c0-4.42-3.58-8-8-8z";

const linkTypes: Record<string, LinkTypeInfo> = {
  jira: {
    label: "Jira",
    color: "#2684FF",
    icon: "M14.1 7.3l-5.5-5.5a1 1 0 00-1.4 0L5.8 3.3l1.8 1.8a1.2 1.2 0 011.5 1.5l1.7 1.7a1.2 1.2 0 11-.7.7L8.5 7.3v4a1.2 1.2 0 11-1-.1V7.1a1.2 1.2 0 01-.6-1.6L5.2 3.8 1.8 7.3a1 1 0 000 1.4l5.5 5.5a1 1 0 001.4 0l5.5-5.5a1 1 0 000-1.4z",
  },
  linear: {
    label: "Linear",
    color: "#5E6AD2",
    icon: "M2.1 10.3a6.96 6.96 0 003.6 3.6l8-8A7 7 0 002.1 10.3zm1-2.6a7 7 0 005.2 5.2l5.2-5.2a7 7 0 00-5.2-5.2L3.1 7.7zM5.9 2.1l-3.6 3.6A7 7 0 0113.9 5.7l-8 8z",
  },
  trello: {
    label: "Trello",
    color: "#0079BF",
    icon: "M2 2h12a1 1 0 011 1v10a1 1 0 01-1 1H2a1 1 0 01-1-1V3a1 1 0 011-1zm1.5 1.5v7h3.5v-7H3.5zm5.5 0v4.5H12.5v-4.5H9z",
  },
  "github-pr": {
    label: "Pull Request",
    color: "#3fb950",
    icon: GITHUB_ICON,
  },
  "github-issue": {
    label: "Issue",
    color: "#a371f7",
    icon: GITHUB_ICON,
  },
  github: {
    label: "GitHub",
    color: "#e8e8e8",
    icon: GITHUB_ICON,
  },
  gitlab: {
    label: "GitLab",
    color: "#FC6D26",
    icon: "M8 14.5L1.5 7.5 3 2l2.5 5.5h5L13 2l1.5 5.5L8 14.5z",
  },
  notion: {
    label: "Notion",
    color: "#e8e8e8",
    icon: "M3 1h10a2 2 0 012 2v10a2 2 0 01-2 2H3a2 2 0 01-2-2V3a2 2 0 012-2zm1 3v8h2V7l4 5h2V4h-2v5L6 4H4z",
  },
  "google-doc": {
    label: "Google Docs",
    color: "#4285F4",
    icon: "M4 1a1 1 0 00-1 1v12a1 1 0 001 1h8a1 1 0 001-1V5l-4-4H4zm5 0v4h4M5 8h6M5 10h6M5 12h4",
  },
  gdocs: {
    label: "Google Docs",
    color: "#4285F4",
    icon: "M4 1a1 1 0 00-1 1v12a1 1 0 001 1h8a1 1 0 001-1V5l-4-4H4zm5 0v4h4M5 8h6M5 10h6M5 12h4",
  },
  figma: {
    label: "Figma",
    color: "#A259FF",
    icon: "M5.5 1A2.5 2.5 0 003 3.5 2.5 2.5 0 005.5 6 2.5 2.5 0 003 8.5 2.5 2.5 0 005.5 11a2.5 2.5 0 100-5A2.5 2.5 0 008 3.5 2.5 2.5 0 0010.5 6a2.5 2.5 0 100-5h-5zm0 0h5A2.5 2.5 0 0113 3.5 2.5 2.5 0 0110.5 6h-5",
  },
  slite: {
    label: "Slite",
    color: "#22c55e",
    icon: "M3 3h10v2H3zm0 4h7v2H3zm0 4h10v2H3z",
  },
  slack: {
    label: "Slack",
    color: "#A259FF",
    icon: "M5.5 1a1.5 1.5 0 000 3H7V2.5A1.5 1.5 0 005.5 1zM9 2.5V4h1.5a1.5 1.5 0 000-3A1.5 1.5 0 009 2.5zm4.5 3A1.5 1.5 0 0015 7H13.5V5.5a1.5 1.5 0 00-3 0V7h1.5zM2.5 9A1.5 1.5 0 001 10.5a1.5 1.5 0 003 0V9H2.5z",
  },
  discord: {
    label: "Discord",
    color: "#5865F2",
    icon: "M13.5 3.5S12 2 10 1.5l-.3.6A10 10 0 006.3 2.1L6 1.5C4 2 2.5 3.5 2.5 3.5S.5 6.5.5 10c1.5 1.7 3.7 2.5 3.7 2.5l.5-.7A7 7 0 013.5 11c.7.4 1.5.8 4.5.8s3.8-.4 4.5-.8a7 7 0 01-1.2.8l.5.7s2.2-.8 3.7-2.5c0-3.5-2-6.5-2-6.5zM5.5 9a1.25 1.25 0 110-2.5 1.25 1.25 0 010 2.5zm5 0a1.25 1.25 0 110-2.5 1.25 1.25 0 010 2.5z",
  },
  stackoverflow: {
    label: "Stack Overflow",
    color: "#F48024",
    icon: "M12 15H2v-4h1v3h8v-3h1v4zM4.5 9.4l.4-1 5.7 2.4-.4 1L4.5 9.4zm1.2-2.5l.8-.8 4.5 4.2-.8.8-4.5-4.2zm2.3-2.2l1-.5 3 5.5-1 .5-3-5.5zM11 2l1.2.1-1 6-1.2-.1 1-6zM4 12.5v-1h6v1H4z",
  },
  other: {
    label: "Link",
    color: "#666",
    icon: "M6.5 3.5a1 1 0 00-1.41 0L3.5 5.09a3 3 0 000 4.24l1.59 1.59a1 1 0 001.41-1.41L4.91 7.91a1 1 0 010-1.41L6.5 4.91a1 1 0 000-1.41zm3 9a1 1 0 001.41 0l1.59-1.59a3 3 0 000-4.24L10.91 5.09a1 1 0 00-1.41 1.41l1.59 1.59a1 1 0 010 1.41L9.5 11.09a1 1 0 000 1.41zM5 8h6",
  },
};

/** Link types that represent tickets/stories/issues */
const TICKET_TYPES = new Set([
  "jira", "linear", "trello", "github-issue", "github-pr", "gitlab",
]);

export function isTicketLink(linkType: string): boolean {
  return TICKET_TYPES.has(linkType);
}

const domainMap: Record<string, string> = {
  "atlassian.net": "jira",
  "jira.": "jira",
  "linear.app": "linear",
  "trello.com": "trello",
  "notion.so": "notion",
  "notion.site": "notion",
  "docs.google.com": "gdocs",
  "drive.google.com": "gdocs",
  "figma.com": "figma",
  "slite.com": "slite",
  "slack.com": "slack",
  "discord.com": "discord",
  "discord.gg": "discord",
  "github.com": "github",
  "github.dev": "github",
  "gitlab.com": "gitlab",
  "stackoverflow.com": "stackoverflow",
};

export function getLinkTypeFromUrl(url: string): string {
  try {
    const hostname = new URL(url).hostname.toLowerCase();
    for (const [domain, type] of Object.entries(domainMap)) {
      if (hostname.includes(domain)) return type;
    }
  } catch {
    // invalid URL
  }
  return "other";
}

export function getLinkTypeInfo(type: string): LinkTypeInfo {
  return linkTypes[type] ?? linkTypes.other;
}
