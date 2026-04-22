// Side-effect imports — each module registers itself on import
import "./ai";
import "./links";
import "./repos";
import "./tasks-notes";
import "./files";
import "./timeline";

// Dynamically register extension tabs at startup
import { registerExtensionTabs } from "./extensions";
export const extensionTabsReady: Promise<void> = registerExtensionTabs();
