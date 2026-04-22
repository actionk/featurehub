import { registerTab } from "../registry";
import { getExtensions } from "../../api/extensions";
import ExtensionTabFrame from "./ExtensionTabFrame.svelte";

export async function registerExtensionTabs(): Promise<void> {
  let extensions;
  try {
    extensions = await getExtensions();
  } catch (e) {
    console.error("[extensions] Failed to load extensions:", e);
    return;
  }

  for (const ext of extensions) {
    if (!ext.enabled) continue;
    for (const tabDecl of ext.manifest.tabs) {
      const componentPath = `${ext.dir}/${tabDecl.component}`.replace(/\\/g, "/");
      registerTab({
        id: `${ext.manifest.id}:${tabDecl.id}`,
        label: tabDecl.label,
        emoji: tabDecl.emoji,
        shortcutKey: "",
        sortOrder: tabDecl.sort_order,
        component: ExtensionTabFrame as any,
        getBadges: () => [],
        panelStyle: "padding: 0;",
        extraProps: { componentPath },
      });
    }
  }
}
