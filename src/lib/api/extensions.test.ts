import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { getExtensions, getExtensionBadge } from "./extensions";

describe("extensions API", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("getExtensions calls get_extensions command", async () => {
    (invoke as any).mockResolvedValue([]);
    const result = await getExtensions();
    expect(invoke).toHaveBeenCalledWith("get_extensions");
    expect(result).toEqual([]);
  });

  it("getExtensionBadge calls get_extension_badge with correct params", async () => {
    (invoke as any).mockResolvedValue(3);
    const count = await getExtensionBadge("my-ext", "prs", "feature-123");
    expect(invoke).toHaveBeenCalledWith("get_extension_badge", {
      extensionId: "my-ext",
      tabId: "prs",
      featureId: "feature-123",
    });
    expect(count).toBe(3);
  });
});
