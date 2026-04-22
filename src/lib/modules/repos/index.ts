import { registerTab } from "../registry";
import RepositoriesPanel from "./RepositoriesPanel.svelte";

registerTab({
  id: "repos",
  label: "Repositories",
  emoji: "📦",
  shortcutKey: "3",
  sortOrder: 300,
  component: RepositoriesPanel,
  getBadges: () => [],
});
