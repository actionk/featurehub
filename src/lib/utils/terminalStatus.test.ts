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
});
