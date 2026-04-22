import { registerTab } from "../registry";
import type { TabContext } from "../registry";
import LinksGrid from "./LinksGrid.svelte";

registerTab({
  id: "links",
  label: "Links",
  emoji: "🔗",
  shortcutKey: "2",
  sortOrder: 200,
  preload: true,
  component: LinksGrid,
  getBadges: (ctx: TabContext) => {
    const links = ctx.feature.links ?? [];
    if (links.length > 0) {
      return [{ text: String(links.length) }];
    }
    return [];
  },
});
