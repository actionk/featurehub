import { registerTab } from "../registry";
import type { TabContext, TabBadge } from "../registry";
import AiPanel from "./AiPanel.svelte";

registerTab({
  id: "ai",
  label: "Agents",
  emoji: "🤖",
  shortcutKey: "1",
  sortOrder: 100,
  preload: true,
  component: AiPanel,
  panelStyle: "display: flex; flex-direction: column; gap: 0; overflow: hidden; padding: 0;",
  getBadges: (ctx: TabContext) => {
    const badges: TabBadge[] = [];
    const pending = ctx.plans.filter((p) => p.status === "pending").length;
    if (pending > 0) {
      badges.push({
        text: String(pending),
        style: "warning",
        title: `${pending} pending plan${pending > 1 ? "s" : ""}`,
      });
    }
    if (ctx.activeSessionCount > 0) {
      badges.push({
        text: String(ctx.activeSessionCount),
        style: "active",
      });
    }
    return badges;
  },
});
