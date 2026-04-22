import { describe, it, expect } from "vitest";
import { getLinkTypeFromUrl, getLinkTypeInfo, isTicketLink } from "./linkTypes";

describe("getLinkTypeFromUrl", () => {
  it("detects GitHub URLs", () => {
    expect(getLinkTypeFromUrl("https://github.com/org/repo")).toBe("github");
  });

  it("detects Jira URLs", () => {
    expect(getLinkTypeFromUrl("https://myteam.atlassian.net/browse/PROJ-123")).toBe("jira");
  });

  it("detects Linear URLs", () => {
    expect(getLinkTypeFromUrl("https://linear.app/team/issue/ENG-456")).toBe("linear");
  });

  it("detects Figma URLs", () => {
    expect(getLinkTypeFromUrl("https://figma.com/file/abc123")).toBe("figma");
  });

  it("detects Slack URLs", () => {
    expect(getLinkTypeFromUrl("https://myteam.slack.com/archives/C123")).toBe("slack");
  });

  it('returns "other" for unknown URLs', () => {
    expect(getLinkTypeFromUrl("https://example.com")).toBe("other");
  });

  it('returns "other" for invalid URLs', () => {
    expect(getLinkTypeFromUrl("not-a-url")).toBe("other");
  });
});

describe("getLinkTypeInfo", () => {
  it("returns info for known type", () => {
    const info = getLinkTypeInfo("github");
    expect(info.label).toBe("GitHub");
    expect(info.color).toBeTruthy();
  });

  it("returns fallback for unknown type", () => {
    const info = getLinkTypeInfo("unknown-type");
    expect(info.label).toBe("Link");
  });
});

describe("isTicketLink", () => {
  it("returns true for ticket types", () => {
    expect(isTicketLink("jira")).toBe(true);
    expect(isTicketLink("linear")).toBe(true);
    expect(isTicketLink("github-issue")).toBe(true);
  });

  it("returns false for non-ticket types", () => {
    expect(isTicketLink("github")).toBe(false);
    expect(isTicketLink("figma")).toBe(false);
    expect(isTicketLink("other")).toBe(false);
  });
});
