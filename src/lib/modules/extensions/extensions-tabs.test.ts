import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("../registry", () => {
  const tabs: any[] = [];
  return {
    registerTab: vi.fn((mod: any) => tabs.push(mod)),
    getRegisteredTabs: vi.fn(() => tabs),
  };
});

import { invoke } from "@tauri-apps/api/core";
import { registerTab } from "../registry";
import { registerExtensionTabs } from "./index";
import type { ExtensionInfo } from "../../api/types";

const mockExt: ExtensionInfo = {
  manifest: {
    id: "my-ext",
    name: "My Extension",
    version: "1.0.0",
    description: "",
    author: "",
    requires: [],
    tools: [],
    tabs: [
      {
        id: "prs",
        label: "PRs",
        emoji: "🔀",
        sort_order: 350,
        component: "/path/to/tab.html",
        badge_query: null,
      },
    ],
    instructions: "",
  },
  enabled: true,
  dir: "/path/to/ext",
  requires_status: [],
};

describe("registerExtensionTabs", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (invoke as any).mockResolvedValue([mockExt]);
  });

  it("registers a tab for each enabled extension tab", async () => {
    await registerExtensionTabs();
    expect(registerTab).toHaveBeenCalledTimes(1);
    const call = (registerTab as any).mock.calls[0][0];
    expect(call.id).toBe("my-ext:prs");
    expect(call.label).toBe("PRs");
    expect(call.emoji).toBe("🔀");
    expect(call.sortOrder).toBe(350);
  });

  it("does not register tabs for disabled extensions", async () => {
    (invoke as any).mockResolvedValue([{ ...mockExt, enabled: false }]);
    await registerExtensionTabs();
    expect(registerTab).not.toHaveBeenCalled();
  });
});
