import { describe, it, expect, vi, afterEach } from "vitest";
import { formatDate, formatRelativeTime, formatDuration, formatFileSize, formatElapsed } from "./format";

describe("formatDate", () => {
  it("formats an ISO date string", () => {
    expect(formatDate("2026-01-15T10:30:00Z")).toBe("Jan 15, 2026");
  });

  it("formats a date in a different month", () => {
    expect(formatDate("2025-12-03T12:00:00Z")).toBe("Dec 3, 2025");
  });
});

describe("formatRelativeTime", () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it('returns "just now" for recent timestamps', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-04-02T12:00:30Z"));
    expect(formatRelativeTime("2026-04-02T12:00:00Z")).toBe("just now");
  });

  it("returns minutes ago", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-04-02T12:05:00Z"));
    expect(formatRelativeTime("2026-04-02T12:00:00Z")).toBe("5m ago");
  });

  it("returns hours ago", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-04-02T15:00:00Z"));
    expect(formatRelativeTime("2026-04-02T12:00:00Z")).toBe("3h ago");
  });

  it("returns days ago", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-04-05T12:00:00Z"));
    expect(formatRelativeTime("2026-04-02T12:00:00Z")).toBe("3d ago");
  });
});

describe("formatDuration", () => {
  it("formats minutes only", () => {
    expect(formatDuration(45)).toBe("45m");
  });

  it("formats hours only", () => {
    expect(formatDuration(120)).toBe("2h");
  });

  it("formats hours and minutes", () => {
    expect(formatDuration(90)).toBe("1h 30m");
  });
});

describe("formatFileSize", () => {
  it("formats bytes", () => {
    expect(formatFileSize(500)).toBe("500 B");
  });

  it("formats kilobytes", () => {
    expect(formatFileSize(1536)).toBe("1.5 KB");
  });

  it("formats megabytes", () => {
    expect(formatFileSize(2.5 * 1024 * 1024)).toBe("2.5 MB");
  });

  it("formats gigabytes", () => {
    expect(formatFileSize(1.5 * 1024 * 1024 * 1024)).toBe("1.50 GB");
  });
});

describe('formatElapsed', () => {
  it('shows seconds for < 60s', () => {
    const now = Date.now();
    expect(formatElapsed(new Date(now - 45000).toISOString(), now)).toBe('45s');
  });

  it('shows minutes for < 60m', () => {
    const now = Date.now();
    expect(formatElapsed(new Date(now - 5 * 60 * 1000).toISOString(), now)).toBe('5m');
  });

  it('shows hours and minutes', () => {
    const now = Date.now();
    expect(formatElapsed(new Date(now - (2 * 3600 + 34 * 60) * 1000).toISOString(), now)).toBe('2h 34m');
  });

  it('shows hours only when no remainder', () => {
    const now = Date.now();
    expect(formatElapsed(new Date(now - 3 * 3600 * 1000).toISOString(), now)).toBe('3h');
  });
});
