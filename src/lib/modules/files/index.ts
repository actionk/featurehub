import { registerTab } from "../registry";
import FileBrowser from "./FileBrowser.svelte";

registerTab({
  id: "files",
  label: "Files",
  emoji: "📁",
  shortcutKey: "5",
  sortOrder: 500,
  component: FileBrowser,
  getBadges: () => [],
});
