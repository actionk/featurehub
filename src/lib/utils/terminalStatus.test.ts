import { describe, expect, test } from "vitest";
import { getTerminalSidebarStatus, getTerminalSidebarTitle } from "./terminalStatus";

describe("getTerminalSidebarStatus", () => {
  test("shows waiting when the terminal needs input", () => {
    expect(getTerminalSidebarStatus({ exited: false, needsInput: true })).toBe("Waiting");
  });

  test("shows exited when the terminal has exited", () => {
    expect(getTerminalSidebarStatus({ exited: true, needsInput: false })).toBe("Exited");
  });

  test("hides raw PTY status text instead of showing fake activity", () => {
    expect(
      getTerminalSidebarStatus({
        exited: false,
        needsInput: false,
        statusLine: "----------------------------------------",
      }),
    ).toBeNull();
  });

  test("hides generic running when there is no concrete action", () => {
    expect(
      getTerminalSidebarStatus({
        exited: false,
        needsInput: false,
        statusLine: "Running",
      }),
    ).toBeNull();
  });

  test("shows meaningful Claude action status lines", () => {
    expect(
      getTerminalSidebarStatus({
        exited: false,
        needsInput: false,
        statusLine: "Reading files",
      }),
    ).toBe("Reading files");
  });

  test("prefers transcript last action over generic running", () => {
    expect(
      getTerminalSidebarStatus({
        exited: false,
        needsInput: false,
        statusLine: "Claude Code v2.1.123",
        lastAction: "Running command",
      }),
    ).toBe("Running command");
  });

  test("shows waiting from parsed transcript status", () => {
    expect(
      getTerminalSidebarStatus({
        exited: false,
        needsInput: false,
        status: "WaitingForInput",
      }),
    ).toBe("Waiting");
  });

  test("does not show Claude startup banner lines as status", () => {
    expect(
      getTerminalSidebarStatus({
        exited: false,
        needsInput: false,
        statusLine: "Claude Code v2.1.123",
      }),
    ).toBeNull();
  });
});

describe("getTerminalSidebarTitle", () => {
  test("prefers parsed transcript title over terminal OSC label", () => {
    expect(
      getTerminalSidebarTitle({
        label: "Claude Code - Status",
        parsedTitle: "Fix sidebar session status",
      }),
    ).toBe("Fix sidebar session status");
  });

  test("hides generic Claude Code terminal titles", () => {
    expect(
      getTerminalSidebarTitle({
        label: "Claude Code - Status",
        featureTitle: "Move permissions",
      }),
    ).toBe("Move permissions");
  });

  test("keeps explicit resumed session labels as fallback", () => {
    expect(
      getTerminalSidebarTitle({
        label: "Investigate transcript parser",
      }),
    ).toBe("Investigate transcript parser");
  });
});
