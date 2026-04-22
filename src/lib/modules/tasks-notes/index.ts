import { registerTab } from "../registry";
import type { TabContext } from "../registry";
import TasksNotesPanel from "./TasksNotesPanel.svelte";

registerTab({
  id: "notes",
  label: "Tasks & Notes",
  emoji: "✅",
  shortcutKey: "4",
  sortOrder: 400,
  preload: true,
  component: TasksNotesPanel,
  getBadges: (ctx: TabContext) => {
    const total = ctx.tasks.length;
    if (total > 0) {
      const done = ctx.tasks.filter((t) => t.done).length;
      return [{ text: `${done}/${total}` }];
    }
    return [];
  },
});
