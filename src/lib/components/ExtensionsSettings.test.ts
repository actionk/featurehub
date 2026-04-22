import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/svelte";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockImplementation((cmd: string) => {
    if (cmd === "get_extensions") {
      return Promise.resolve([]);
    }
    return Promise.resolve({});
  }),
}));

import InstalledExtensionsPanel from "./InstalledExtensionsPanel.svelte";

describe("InstalledExtensionsPanel", () => {
  it("renders nothing when no extensions installed", async () => {
    const { container } = render(InstalledExtensionsPanel);
    await vi.waitFor(() => {
      expect(container.querySelector(".settings-section")).toBeNull();
    });
  });
});
