<script lang="ts">
  import "./app.css";
  import type { Feature, FeatureGroup, StorageInfo } from "./lib/api/tauri";
  import { getFeatures, getFeatureGroups, getActiveStorage, getStorages, switchStorage, pollNotifications, cleanupOrphanedSessions, ptyKill, finishEmbeddedSession, ptyListActive } from "./lib/api/tauri";
  import { removeTerminal, restoreTerminals, getActiveTerminals, requestResumeSession } from "./lib/stores/terminals.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import FeatureDetail from "./lib/components/FeatureDetail.svelte";
  import WorkspaceTabBar from "./lib/components/WorkspaceTabBar.svelte";
  import SearchBar from "./lib/components/SearchBar.svelte";
  import CreateFeatureModal from "./lib/components/CreateFeatureModal.svelte";
  import StorageSetup from "./lib/components/StorageSetup.svelte";
  import SettingsModal from "./lib/components/SettingsModal.svelte";
  import KnowledgePanel from "./lib/modules/knowledge/KnowledgePanel.svelte";
  import BoardPanel from "./lib/modules/board/BoardPanel.svelte";
  import GlobalTimeline from "./lib/components/GlobalTimeline.svelte";
  import SessionsPanel from "./lib/components/SessionsPanel.svelte";
  import ToastContainer from "./lib/components/ToastContainer.svelte";
  import { loadAppSettings } from "./lib/stores/settings.svelte";
  import { requestViewTerminal } from "./lib/stores/terminals.svelte";
  import { subscribe, emit } from "./lib/stores/events.svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { listen } from "@tauri-apps/api/event";
  import { startSessionActivityPolling, refreshSessionActivity } from "./lib/stores/sessionActivity.svelte";
  import {
    getWorkspaceTabs,
    getActiveTabId,
    getActiveFeatureId,
    getOpenFeatureIds,
    openTab,
    switchToFeature,
    switchToTab,
    closeTab,
    closeOtherTabs,
    closeAllTabs,
    closeTabForFeature,
    clearWorkspaceTabs,
    pruneInvalidTabs,
    nextTab,
    prevTab,
    isBoardTab,
    BOARD_TAB_FEATURE_ID,
    openBoardTab,
  } from "./lib/stores/workspaceTabs.svelte";

  let features = $state<Feature[]>([]);
  let featureGroups = $state<FeatureGroup[]>([]);
  let showSearch = $state(false);
  let showCreateModal = $state(false);
  let showSettings = $state(false);
  let settingsInitialTab = $state<string | undefined>(undefined);
  let activeStorage = $state<StorageInfo | null>(null);
  let storageChecked = $state(false);
  let storageSwitching = $state(false);
  type MainView = 'features' | 'knowledge' | 'dashboard' | 'timeline';
  let activeView = $state<MainView>('features');
  // Sessions panel — always visible, resizable
  const SESSIONS_PANEL_MIN = 180;
  const SESSIONS_PANEL_MAX = 450;
  const SESSIONS_PANEL_DEFAULT = 260;
  let sessionsPanelWidth = $state(
    parseInt(localStorage.getItem("featurehub:sessionsPanelWidth") || "") || SESSIONS_PANEL_DEFAULT
  );
  let showSessionsPanel = $state(localStorage.getItem("featurehub:sessionsPanelVisible") !== "0");

  function setSessionsPanelVisible(value: boolean) {
    showSessionsPanel = value;
    localStorage.setItem("featurehub:sessionsPanelVisible", value ? "1" : "0");
  }

  function onResizePanelStart(e: MouseEvent) {
    e.preventDefault();
    isResizing = true;
    const startX = e.clientX;
    const startWidth = sessionsPanelWidth;
    const onMove = (ev: MouseEvent) => {
      const delta = startX - ev.clientX;
      const w = Math.min(SESSIONS_PANEL_MAX, Math.max(SESSIONS_PANEL_MIN, startWidth + delta));
      sessionsPanelWidth = w;
    };
    const onUp = () => {
      isResizing = false;
      localStorage.setItem("featurehub:sessionsPanelWidth", String(sessionsPanelWidth));
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }

  // Workspace tabs — derived state
  let workspaceTabs = $derived(getWorkspaceTabs());
  let activeTabId = $derived(getActiveTabId());
  let selectedId = $derived(getActiveFeatureId());

  // Toast notifications
  interface Toast {
    id: number;
    message: string;
    featureId: string | null;
    fading: boolean;
  }
  let toasts = $state<Toast[]>([]);
  let nextToastId = 0;
  let refreshFeatureId = $state<string | null>(null);
  let pendingPlanId = $state<string | null>(null);
  let pendingPlanFeatureId = $state<string | null>(null);
  let initialTab = $state<string | null>(null);
  // Track which tab the initialTab is for
  let initialTabTargetId = $state<string | null>(null);

  // Resizable sidebar
  const SIDEBAR_MIN = 200;
  const SIDEBAR_MAX = 500;
  const SIDEBAR_DEFAULT = 272;
  const ICON_RAIL_WIDTH = 60;
  const APP_FRAME_PADDING = 10;
  let sidebarWidth = $state(
    parseInt(localStorage.getItem("featurehub:sidebarWidth") || "") || SIDEBAR_DEFAULT
  );
  let isResizing = $state(false);

  function onResizeStart(e: MouseEvent) {
    e.preventDefault();
    isResizing = true;
    const onMove = (ev: MouseEvent) => {
      const w = Math.min(SIDEBAR_MAX, Math.max(SIDEBAR_MIN, ev.clientX - ICON_RAIL_WIDTH - APP_FRAME_PADDING));
      sidebarWidth = w;
    };
    const onUp = () => {
      isResizing = false;
      localStorage.setItem("featurehub:sidebarWidth", String(sidebarWidth));
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }

  const appWindow = getCurrentWindow();
  function minimizeWindow() { appWindow.minimize(); }
  function closeWindow() { appWindow.close(); }

  let isMaximized = $state(false);
  $effect(() => {
    appWindow.isMaximized().then(v => { isMaximized = v; });
    const unlisten = listen<boolean>('window-maximized', e => { isMaximized = e.payload; });
    return () => { unlisten.then(fn => fn()); };
  });

  $effect(() => {
    checkStorage();
    loadAppSettings();
  });

  $effect(() => {
    if (activeStorage) {
      loadFeatures();
      // Restore embedded terminal state from backend (survives webview reload)
      restoreActiveTerminals();
      // Clean up sessions orphaned by app reload (only affects sessions whose process died)
      cleanupOrphanedSessions().catch(() => {});
    }
  });

  async function restoreActiveTerminals() {
    try {
      const active = await ptyListActive();
      if (active.length === 0) return;
      const restored = active
        .filter(t => t.session_db_id)
        .map(t => ({
          terminalId: t.terminal_id,
          sessionDbId: t.session_db_id!,
          featureId: t.feature_id,
          featureTitle: "",
          label: t.label ?? "Session",
          exited: false,
          statusLine: "",
          needsInput: false,
        }));
      if (restored.length > 0) {
        restoreTerminals(restored);
      }
    } catch (e) {
      console.error("Failed to restore terminals:", e);
    }
  }

  // Poll for notifications from MCP
  $effect(() => {
    if (!activeStorage) return;
    const openFeatureIds = getOpenFeatureIds();
    const interval = setInterval(async () => {
      try {
        const notifs = await pollNotifications();
        let needsFeatureReload = false;
        for (const n of notifs) {
          addToast(n.message, n.feature_id);
          // Refresh any open tab whose feature matches
          if (n.feature_id && openFeatureIds.includes(n.feature_id)) {
            refreshFeatureId = n.feature_id;
            if (n.plan_id) {
              pendingPlanId = n.plan_id;
              pendingPlanFeatureId = n.feature_id;
            }
          }
          if (n.feature_id) {
            needsFeatureReload = true;
          }
        }
        // Reload sidebar once per poll cycle, not per notification
        if (needsFeatureReload) {
          loadFeatures();
        }
      } catch {
        // ignore polling errors
      }
    }, 2000);
    return () => clearInterval(interval);
  });

  // Centralized session activity polling (counts + active IDs)
  $effect(() => {
    if (!activeStorage) return;
    return startSessionActivityPolling(10_000);
  });

  const notificationSound = new Audio("/notification.mp3");

  function addToast(message: string, featureId: string | null) {
    const id = nextToastId++;
    toasts = [...toasts, { id, message, featureId, fading: false }];
    notificationSound.currentTime = 0;
    notificationSound.play().catch(() => {});
    // Start fade after 3.5s, remove after 4s
    setTimeout(() => {
      toasts = toasts.map(t => t.id === id ? { ...t, fading: true } : t);
    }, 3500);
    setTimeout(() => {
      toasts = toasts.filter(t => t.id !== id);
    }, 4000);
  }

  function handleToastClick(featureId: string) {
    switchToFeature(featureId);
  }

  $effect(() => {
    function handleKeydown(e: KeyboardEvent) {
      if ((e.ctrlKey || e.metaKey) && e.code === "KeyT") {
        e.preventDefault();
        showSearch = !showSearch;
      }
      // Tab to toggle dashboard/features, F3/F4 for knowledge/timeline
      if (e.code === "Tab" && !e.ctrlKey && !e.metaKey && !e.altKey && !e.shiftKey) {
        const tag = (e.target as HTMLElement)?.tagName;
        if (tag !== "INPUT" && tag !== "TEXTAREA" && !(e.target as HTMLElement)?.isContentEditable) {
          e.preventDefault();
          activeView = activeView === 'dashboard' ? 'features' : 'dashboard';
        }
      }
      if (e.code === "F3") { e.preventDefault(); activeView = 'knowledge'; }
      if (e.code === "F4") { e.preventDefault(); activeView = 'timeline'; }
      if ((e.ctrlKey || e.metaKey) && e.code === "KeyN") {
        e.preventDefault();
        showCreateModal = true;
      }
      if ((e.ctrlKey || e.metaKey) && e.altKey && e.code === "KeyS") {
        e.preventDefault();
        showSettings = !showSettings;
      }
      // Ctrl+W to close active workspace tab
      if ((e.ctrlKey || e.metaKey) && e.code === "KeyW") {
        if (activeTabId && workspaceTabs.length > 0) {
          e.preventDefault();
          closeTab(activeTabId);
        }
      }
      // Ctrl+Tab / Ctrl+Shift+Tab to cycle workspace tabs
      if ((e.ctrlKey || e.metaKey) && e.code === "Tab") {
        if (workspaceTabs.length > 1) {
          e.preventDefault();
          if (e.shiftKey) {
            prevTab();
          } else {
            nextTab();
          }
        }
      }
      // Shift+1 through Shift+9 to switch storages
      if (e.shiftKey && !e.ctrlKey && !e.metaKey && !e.altKey && e.code >= "Digit1" && e.code <= "Digit9") {
        // Don't intercept if user is typing in an input/textarea
        const tag = (e.target as HTMLElement)?.tagName;
        if (tag === "INPUT" || tag === "TEXTAREA" || (e.target as HTMLElement)?.isContentEditable) return;
        e.preventDefault();
        const index = parseInt(e.code.replace("Digit", "")) - 1;
        switchToStorageByIndex(index);
      }
      if (e.code === "Escape") {
        if (showSearch) {
          e.preventDefault();
          showSearch = false;
        } else if (showCreateModal) {
          e.preventDefault();
          showCreateModal = false;
        } else if (showSettings) {
          e.preventDefault();
          showSettings = false;
        }
      }
    }
    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  });

  async function checkStorage() {
    try {
      activeStorage = await getActiveStorage();
    } catch (e) {
      console.error("Failed to check storage:", e);
    }
    storageChecked = true;
  }

  async function loadFeatures() {
    try {
      const [f, g] = await Promise.all([getFeatures(), getFeatureGroups()]);
      features = f;
      featureGroups = g;
      // Prune workspace tabs for features that no longer exist
      const validIds = new Set(features.map(f => f.id));
      pruneInvalidTabs(validIds);
      // Restore last active feature on initial load (if no workspace tabs)
      if (workspaceTabs.length === 0) {
        const lastId = localStorage.getItem("featurehub:lastFeatureId");
        if (lastId && features.some((f) => f.id === lastId)) {
          switchToFeature(lastId);
        }
      }
    } catch (e) {
      console.error("Failed to load features:", e);
    }
  }

  function handleSelect(id: string) {
    if (!id) {
      if (activeTabId) closeTab(activeTabId);
      localStorage.removeItem("featurehub:lastFeatureId");
      return;
    }
    switchToFeature(id);
    localStorage.setItem("featurehub:lastFeatureId", id);
  }

  function handleSelectNewTab(id: string) {
    openTab(id);
    localStorage.setItem("featurehub:lastFeatureId", id);
  }

  function handleCreateNew() {
    showCreateModal = true;
  }

  function handleCreated(feature: Feature) {
    showCreateModal = false;
    loadFeatures();
    openTab(feature.id);
    localStorage.setItem("featurehub:lastFeatureId", feature.id);
  }

  async function handleDeleted() {
    // Close the tab for the deleted feature
    if (selectedId) {
      closeTabForFeature(selectedId);
    }
    localStorage.removeItem("featurehub:lastFeatureId");
    await loadFeatures();
    // Update lastFeatureId to current active
    const currentId = getActiveFeatureId();
    if (currentId) {
      localStorage.setItem("featurehub:lastFeatureId", currentId);
    }
  }

  const entityTypeToTab: Record<string, string> = {
    note: "notes",
    link: "links",
    session: "sessions",
    file: "files",
  };

  function handleSearchSelect(featureId: string, entityType: string) {
    initialTab = entityTypeToTab[entityType] || null;
    initialTabTargetId = featureId;
    openTab(featureId);
    showSearch = false;
  }

  function handleStorageCreated(storage: StorageInfo) {
    activeStorage = storage;
    features = [];
    clearWorkspaceTabs();
  }

  async function handleStorageSwitch() {
    storageSwitching = true;
    await new Promise(r => setTimeout(r, 150));
    activeStorage = await getActiveStorage();
    features = [];
    clearWorkspaceTabs();
    if (activeStorage) {
      await loadFeatures();
    }
    storageSwitching = false;
  }

  async function switchToStorageByIndex(index: number) {
    try {
      const storages = await getStorages();
      if (index >= storages.length) return;
      const target = storages[index];
      if (target.is_active) return;
      storageSwitching = true;
      await new Promise(r => setTimeout(r, 150));
      await switchStorage(target.id);
      activeStorage = await getActiveStorage();
      features = [];
      clearWorkspaceTabs();
      if (activeStorage) {
        await loadFeatures();
      }
      storageSwitching = false;
    } catch (e) {
      storageSwitching = false;
      console.error("Failed to switch storage:", e);
    }
  }

  function handleSessionPanelClick(featureId: string, sessionDbId: string, isActive: boolean) {
    openTab(featureId);
    initialTab = "ai";
    initialTabTargetId = featureId;
    const isRunningExternally = isActive && !getActiveTerminals().some(t => t.sessionDbId === sessionDbId);
    if (!isRunningExternally) {
      requestResumeSession(sessionDbId);
    }
  }

  function handleCloseTab(tabId: string) {
    closeTab(tabId);
    const currentId = getActiveFeatureId();
    if (currentId) {
      localStorage.setItem("featurehub:lastFeatureId", currentId);
    } else {
      localStorage.removeItem("featurehub:lastFeatureId");
    }
  }
</script>

{#if !storageChecked}
  <!-- loading -->
{:else if !activeStorage}
  <StorageSetup onCreated={handleStorageCreated} />
{:else if showSettings}
  <SettingsModal onClose={() => { showSettings = false; settingsInitialTab = undefined; }} storageName={activeStorage?.name ?? ""} initialTab={settingsInitialTab} />
{:else}
  <div class="app-frame aurora-bg" class:maximized={isMaximized}>
  <div class="app-shell" class:resizing={isResizing} class:storage-switching={storageSwitching}>
    <!-- Icon rail -->
    <div class="icon-rail">
      <div class="icon-rail-logo" data-tauri-drag-region>
        <img src="/icon.png" alt="FeatureHub" width="22" height="22" style="border-radius: 4px;" />
      </div>

      <button class="icon-rail-btn icon-rail-btn--keyed" class:icon-rail-btn--on={activeView === 'dashboard'} data-tip="Dashboard  Tab" aria-label="Dashboard"
        onclick={() => { activeView = 'dashboard'; }}>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <rect x="0" y="0" width="4" height="3" rx="1"/>
          <rect x="0" y="4" width="4" height="3" rx="1"/>
          <rect x="0" y="8" width="4" height="3" rx="1"/>
          <rect x="0" y="12" width="4" height="3" rx="1"/>
          <rect x="6" y="0" width="4" height="3" rx="1"/>
          <rect x="6" y="4" width="4" height="3" rx="1"/>
          <rect x="6" y="8" width="4" height="3" rx="1"/>
          <rect x="12" y="0" width="4" height="3" rx="1"/>
          <rect x="12" y="4" width="4" height="3" rx="1"/>
        </svg>
        <span class="icon-rail-key">⇥</span>
      </button>

      <button class="icon-rail-btn icon-rail-btn--keyed" class:icon-rail-btn--on={activeView === 'features'} data-tip="Features  Tab" aria-label="Features"
        onclick={() => { activeView = 'features'; }}>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <rect x="0" y="0" width="3" height="3" rx="1"/>
          <rect x="5" y="0" width="11" height="3" rx="1"/>
          <rect x="0" y="6" width="3" height="3" rx="1"/>
          <rect x="5" y="6" width="7" height="3" rx="1"/>
          <rect x="0" y="12" width="3" height="3" rx="1"/>
          <rect x="5" y="12" width="9" height="3" rx="1"/>
        </svg>
        <span class="icon-rail-key">⇥</span>
      </button>

      <button class="icon-rail-btn icon-rail-btn--keyed" class:icon-rail-btn--on={activeView === 'knowledge'} data-tip="Knowledge  F3" aria-label="Knowledge"
        onclick={() => { activeView = 'knowledge'; }}>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M1 2.828c.885-.37 2.154-.769 3.388-.893 1.33-.134 2.458.063 3.112.752v9.746c-.935-.53-2.12-.603-3.213-.493-1.18.12-2.37.461-3.287.811V2.828zm7.5-.141c.654-.689 1.782-.886 3.112-.752 1.234.124 2.503.523 3.388.893v9.966c-.918-.35-2.107-.692-3.287-.81-1.094-.111-2.278-.039-3.213.492V2.687zM8 1.783C7.015.936 5.587.81 4.287.94c-1.514.153-3.042.672-3.994 1.105A.5.5 0 000 2.5v11a.5.5 0 00.707.455c.882-.4 2.303-.881 3.68-1.02 1.409-.142 2.59.087 3.223.877a.5.5 0 00.78 0c.633-.79 1.814-1.019 3.222-.877 1.378.139 2.8.62 3.681 1.02A.5.5 0 0016 13.5v-11a.5.5 0 00-.293-.455c-.952-.433-2.48-.952-3.994-1.105C10.413.809 8.985.936 8 1.783z"/>
        </svg>
        <span class="icon-rail-key">F3</span>
      </button>

      <button class="icon-rail-btn icon-rail-btn--keyed" class:icon-rail-btn--on={activeView === 'timeline'} data-tip="Timeline  F4" aria-label="Timeline"
        onclick={() => { activeView = 'timeline'; }}>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 3.5a.5.5 0 00-1 0V9a.5.5 0 00.252.434l3.5 2a.5.5 0 00.496-.868L8 8.71V3.5z"/>
          <path d="M8 16A8 8 0 108 0a8 8 0 000 16zm7-8A7 7 0 111 8a7 7 0 0114 0z"/>
        </svg>
        <span class="icon-rail-key">F4</span>
      </button>

      <div class="icon-rail-sep"></div>
      <div class="icon-rail-spacer"></div>

      <button class="icon-rail-btn" data-tip="Search  Ctrl+T" aria-label="Search"
        onclick={() => (showSearch = true)}>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M11.742 10.344a6.5 6.5 0 10-1.397 1.398h-.001c.03.04.062.078.098.115l3.85 3.85a1 1 0 001.415-1.414l-3.85-3.85a1.007 1.007 0 00-.115-.099zM12 6.5a5.5 5.5 0 11-11 0 5.5 5.5 0 0111 0z"/>
        </svg>
      </button>

      <button
        class="icon-rail-btn"
        class:icon-rail-btn--on={showSessionsPanel}
        data-tip={showSessionsPanel ? "Hide Sessions" : "Show Sessions"}
        aria-label={showSessionsPanel ? "Hide sessions panel" : "Show sessions panel"}
        onclick={() => setSessionsPanelVisible(!showSessionsPanel)}
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M2.5 2A1.5 1.5 0 001 3.5v9A1.5 1.5 0 002.5 14h11a1.5 1.5 0 001.5-1.5v-9A1.5 1.5 0 0013.5 2h-11zM2 3.5a.5.5 0 01.5-.5H5v10H2.5a.5.5 0 01-.5-.5v-9zm4 9.5V3h7.5a.5.5 0 01.5.5v9a.5.5 0 01-.5.5H6z"/>
        </svg>
      </button>

      <button class="icon-rail-btn" data-tip="Settings" aria-label="Settings"
        onclick={() => (showSettings = true)}>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 4.754a3.246 3.246 0 100 6.492 3.246 3.246 0 000-6.492zM5.754 8a2.246 2.246 0 114.492 0 2.246 2.246 0 01-4.492 0z"/>
          <path d="M9.796 1.343c-.527-1.79-3.065-1.79-3.592 0l-.094.319a.873.873 0 01-1.255.52l-.292-.16c-1.64-.892-3.433.902-2.54 2.541l.159.292a.873.873 0 01-.52 1.255l-.319.094c-1.79.527-1.79 3.065 0 3.592l.319.094a.873.873 0 01.52 1.255l-.16.292c-.892 1.64.901 3.434 2.541 2.54l.292-.159a.873.873 0 011.255.52l.094.319c.527 1.79 3.065 1.79 3.592 0l.094-.319a.873.873 0 011.255-.52l.292.16c1.64.892 3.433-.902 2.54-2.541l-.159-.292a.873.873 0 01.52-1.255l.319-.094c1.79-.527 1.79-3.065 0-3.592l-.319-.094a.873.873 0 01-.52-1.255l.16-.292c.892-1.64-.901-3.433-2.541-2.54l-.292.159a.873.873 0 01-1.255-.52l-.094-.319z"/>
        </svg>
      </button>
    </div>

    <Sidebar
      {features}
      {featureGroups}
      {selectedId}
      {activeStorage}
      onSelect={handleSelect}
      onCreateNew={handleCreateNew}
      onStorageSwitch={handleStorageSwitch}
      onOpenSearch={() => (showSearch = true)}
      onOpenSettings={() => (showSettings = true)}
      onFeaturesChanged={loadFeatures}
      onSelectTerminal={(fid, tid) => {
        requestViewTerminal(tid);
        openTab(fid);
        initialTab = "ai";
        initialTabTargetId = fid;
      }}
      onSelectSessions={(fid) => {
        openTab(fid);
        initialTab = "ai";
        initialTabTargetId = fid;
      }}
      onSelectNewTab={handleSelectNewTab}
      onOpenBoard={() => {
        activeView = 'features';
        openBoardTab();
      }}
      onOpenKnowledge={() => activeView = 'knowledge'}
      onFinishTerminal={async (tid, sid) => {
        await ptyKill(tid).catch(() => {});
        await finishEmbeddedSession(sid).catch(() => {});
        removeTerminal(tid);
        emit("sessions:changed");
      }}
      width={sidebarWidth}
    />

    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="resize-handle" onmousedown={onResizeStart}></div>

    <div class="main-content">
      {#if activeView === 'dashboard'}
        <BoardPanel />
      {:else if activeView === 'timeline'}
        <GlobalTimeline />
      {:else if activeView === 'knowledge'}
        <KnowledgePanel onClose={() => activeView = 'features'} />
      {:else if workspaceTabs.length > 0}
        {#if workspaceTabs.length >= 2}
          <WorkspaceTabBar
            tabs={workspaceTabs}
            {activeTabId}
            {features}
            onSwitchTab={(tabId) => {
              switchToTab(tabId);
              const fid = getActiveFeatureId();
              if (fid) localStorage.setItem("featurehub:lastFeatureId", fid);
            }}
            onCloseTab={handleCloseTab}
            onCloseOtherTabs={closeOtherTabs}
            onCloseAllTabs={() => {
              closeAllTabs();
              localStorage.removeItem("featurehub:lastFeatureId");
            }}
          />
        {/if}
        {#each workspaceTabs as tab (tab.id)}
          {@const isActive = tab.id === activeTabId}
          <div class="tab-content-wrapper" style:display={isActive ? '' : 'none'}>
            {#if isBoardTab(tab)}
              <BoardPanel />
            {:else}
              <FeatureDetail
                featureId={tab.featureId}
                {isActive}
                onDeleted={handleDeleted}
                onUpdated={loadFeatures}
                onSessionsChanged={() => refreshSessionActivity()}
                {refreshFeatureId}
                onRefreshHandled={() => refreshFeatureId = null}
                {pendingPlanId}
                {pendingPlanFeatureId}
                onPendingPlanHandled={() => { pendingPlanId = null; pendingPlanFeatureId = null; }}
                initialTab={initialTabTargetId === tab.featureId ? initialTab : null}
                onInitialTabHandled={() => { initialTab = null; initialTabTargetId = null; }}
                onOpenSettings={(tab) => { settingsInitialTab = tab; showSettings = true; }}
              />
            {/if}
          </div>
        {/each}
      {:else}
        <div class="empty-state">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" opacity="0.2">
            <path d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/>
          </svg>
          <div style="font-size: 14px; color: var(--text-secondary);">Select a feature or create a new one</div>
          <div style="font-size: 12px; display: flex; gap: 16px; margin-top: 4px;">
            <span>
              <kbd class="search-trigger-key" style="font-size: 10px;">Ctrl T</kbd>
              <span style="margin-left: 4px; color: var(--text-muted);">Search</span>
            </span>
            <span>
              <kbd class="search-trigger-key" style="font-size: 10px;">Ctrl N</kbd>
              <span style="margin-left: 4px; color: var(--text-muted);">New</span>
            </span>
          </div>
        </div>
      {/if}
    </div>
    {#if showSessionsPanel}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="sessions-resize-handle" onmousedown={onResizePanelStart}></div>
      <SessionsPanel onSessionClick={handleSessionPanelClick} width={sessionsPanelWidth} />
    {/if}
  </div><!-- .app-shell -->

  {#if showSearch}
    <SearchBar
      onClose={() => (showSearch = false)}
      onSelect={handleSearchSelect}
    />
  {/if}

  {#if showCreateModal}
    <CreateFeatureModal
      onClose={() => (showCreateModal = false)}
      onCreated={handleCreated}
    />
  {/if}

  <ToastContainer {toasts} onClickToast={handleToastClick} />
  </div><!-- .app-frame -->
{/if}
