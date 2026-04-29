import { describe, expect, test } from "vitest";
import { getTerminalSidebarStatus } from "./terminalStatus";

describe("getTerminalSidebarStatus", () => {
  test("shows waiting when the terminal needs input", () => {
    expect(getTerminalSidebarStatus({ exited: false, needsInput: true })).toBe("Waiting");
  });

  test("shows exited when the terminal has exited", () => {
    expect(getTerminalSidebarStatus({ exited: true, needsInput: false })).toBe("Exited");
  });

  test("shows running instead of raw PTY status text", () => {
    expect(
      getTerminalSidebarStatus({
        exited: false,
        needsInput: false,
        statusLine: "----------------------------------------",
      }),
    ).toBe("Running");
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
    ).toBe("Running");
  });
});
