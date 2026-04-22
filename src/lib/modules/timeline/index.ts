import { registerTab } from "../registry";
import Timeline from "./Timeline.svelte";

registerTab({
  id: "timeline",
  label: "Timeline",
  emoji: "🕐",
  shortcutKey: "6",
  sortOrder: 600,
  component: Timeline,
  getBadges: () => [],
});
