<script lang="ts">
  import type { Feature, FeatureGroup, StorageInfo } from "../api/tauri";
  import { duplicateFeature, togglePinFeature, setFeatureArchived, updateFeature, deleteFeature, setFeatureParent, reorderFeatures, createFeatureGroup, updateFeatureGroup, deleteFeatureGroup, reorderFeatureGroups, setFeatureGroup, updateStorageIcon, getStorages } from "../api/tauri";
  import StorageSelector from "./StorageSelector.svelte";
  import { formatRelativeTime } from "../utils/format";
  import { getActiveTerminals, getViewingTerminal } from "../stores/terminals.svelte";
  import { getActiveCountForFeature } from "../stores/sessionActivity.svelte";

  let {
    features,
    featureGroups = [],
    selectedId,
    activeStorage,
    onSelect,
    onCreateNew,
    onStorageSwitch,
    onOpenSearch,
    onOpenSettings,
    onOpenKnowledge,
    onOpenBoard,
    onFeaturesChanged,
    onSelectTerminal,
    onSelectSessions,
    onSelectNewTab,
    onFinishTerminal,
    width,
  }: {
    features: Feature[];
    featureGroups?: FeatureGroup[];
    selectedId: string | null;
    activeStorage: StorageInfo;
    onSelect: (id: string) => void;
    onCreateNew: () => void;
    onStorageSwitch: () => void;
    onOpenSearch?: () => void;
    onOpenSettings?: () => void;
    onOpenKnowledge?: () => void;
    onOpenBoard?: () => void;
    onFeaturesChanged?: () => void;
    onSelectTerminal?: (featureId: string, terminalId: string) => void;
    onSelectSessions?: (featureId: string) => void;
    onSelectNewTab?: (featureId: string) => void;
    onFinishTerminal?: (terminalId: string, sessionDbId: string) => void;
    width?: number;
  } = $props();

  let activeTerminals = $derived(getActiveTerminals());
  let viewingTerminalId = $derived(getViewingTerminal());

  // Group terminals by feature for inline rendering
  let terminalsByFeature = $derived.by(() => {
    const map = new Map<string, typeof activeTerminals>();
    for (const t of activeTerminals) {
      if (!map.has(t.featureId)) map.set(t.featureId, []);
      map.get(t.featureId)!.push(t);
    }
    return map;
  });

  // Storage index for shortcut display
  let storageIndex = $state<number | null>(null);
  $effect(() => {
    // Re-run when activeStorage changes
    const id = activeStorage.id;
    getStorages().then(list => {
      const idx = list.findIndex(s => s.id === id);
      storageIndex = idx >= 0 && idx < 9 ? idx : null;
    }).catch(() => {});
  });

  // Icon picker
  let showIconPicker = $state(false);
  let iconInput = $state("");
  let fileInputEl: HTMLInputElement | null = $state(null);

  const emojiOptions = [
    "📁", "💼", "🏠", "🔧", "🚀", "💡", "🎮", "🎨",
    "📊", "🔬", "📝", "⚡", "🌟", "🔥", "💎", "🧪",
    "🏗️", "📦", "🎯", "🛠️", "🌍", "🔒", "📱", "💻",
  ];

  let logoEl: HTMLButtonElement;
  let pickerPos = $state({ top: 0, left: 0 });

  function toggleIconPicker() {
    showIconPicker = !showIconPicker;
    if (showIconPicker) {
      iconInput = activeStorage.icon || "";
      if (logoEl) {
        const rect = logoEl.getBoundingClientRect();
        pickerPos = { top: rect.bottom + 6, left: rect.left };
      }
    }
  }

  async function selectIcon(icon: string) {
    try {
      await updateStorageIcon(activeStorage.id, icon);
      showIconPicker = false;
      onStorageSwitch();
    } catch (e) {
      console.error("Failed to set icon:", e);
    }
  }

  async function clearIcon() {
    try {
      await updateStorageIcon(activeStorage.id, null);
      showIconPicker = false;
      onStorageSwitch();
    } catch (e) {
      console.error("Failed to clear icon:", e);
    }
  }

  async function commitIconInput() {
    if (iconInput.trim()) {
      await selectIcon(iconInput.trim());
    }
  }

  async function handleImageUpload(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    // Resize to 64x64 and convert to base64 data URL
    const img = new Image();
    const url = URL.createObjectURL(file);
    img.onload = async () => {
      const canvas = document.createElement("canvas");
      canvas.width = 64;
      canvas.height = 64;
      const ctx = canvas.getContext("2d")!;
      ctx.drawImage(img, 0, 0, 64, 64);
      const dataUrl = canvas.toDataURL("image/png");
      URL.revokeObjectURL(url);
      await selectIcon(dataUrl);
    };
    img.src = url;
    input.value = "";
  }

  let statusFilter = $state("all");


  // Context menu state
  let contextMenu = $state<{ x: number; y: number; feature: Feature } | null>(null);
  let groupContextMenu = $state<{ x: number; y: number; group: FeatureGroup } | null>(null);
  let deleteConfirm = $state<Feature | null>(null);
  let deleteGroupConfirm = $state<FeatureGroup | null>(null);

  // Inline group rename state
  let renamingGroupId = $state<string | null>(null);
  let renamingGroupName = $state("");

  // New group creation state
  let creatingGroup = $state(false);
  let newGroupName = $state("");

  // Tree expand/collapse state — persisted in localStorage
  let expandedIds = $state<Set<string>>(loadExpanded());

  function loadExpanded(): Set<string> {
    try {
      const raw = localStorage.getItem("featurehub:expandedFeatures");
      if (raw) return new Set(JSON.parse(raw));
    } catch {}
    return new Set();
  }

  function saveExpanded() {
    localStorage.setItem("featurehub:expandedFeatures", JSON.stringify([...expandedIds]));
  }

  function toggleExpanded(id: string) {
    const next = new Set(expandedIds);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    expandedIds = next;
    saveExpanded();
  }

  // Group collapse state — persisted in localStorage
  let collapsedGroups = $state<Set<string>>(loadCollapsedGroups());

  function loadCollapsedGroups(): Set<string> {
    try {
      const raw = localStorage.getItem("featurehub:collapsedGroups");
      if (raw) return new Set(JSON.parse(raw));
    } catch {}
    return new Set();
  }

  function saveCollapsedGroups() {
    localStorage.setItem("featurehub:collapsedGroups", JSON.stringify([...collapsedGroups]));
  }

  function toggleGroupCollapsed(id: string) {
    const next = new Set(collapsedGroups);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    collapsedGroups = next;
    saveCollapsedGroups();
  }

  // ── Custom mouse-based drag state ─────────────────────────────────
  let draggingId = $state<string | null>(null);
  let dragGhost = $state<{ x: number; y: number; title: string } | null>(null);
  let dropTargetId = $state<string | null>(null);
  let dropZone = $state<"above" | "child" | "below" | null>(null);
  let dragStartPos: { x: number; y: number } | null = null;
  let pendingDragFeature: Feature | null = null;
  let justDropped = $state(false);
  let autoExpandTimer: ReturnType<typeof setTimeout> | null = null;
  const DRAG_THRESHOLD = 5;

  // ── Group drag state ──────────────────────────────────────────────
  let draggingGroupId = $state<string | null>(null);
  let dragGroupGhost = $state<{ x: number; y: number; title: string } | null>(null);
  let dropGroupTargetId = $state<string | null>(null);
  let dropGroupZone = $state<"above" | "below" | null>(null);
  let dragGroupStartPos: { x: number; y: number } | null = null;
  let pendingDragGroup: FeatureGroup | null = null;

  const filters = [
    { value: "all", label: "All" },
    { value: "active", label: "Active" },
    { value: "done", label: "Done" },
  ];

  const statusColors: Record<string, string> = {
    todo: "var(--text-muted)",
    in_progress: "var(--amber)",
    in_review: "var(--blue)",
    done: "var(--green)",
    active: "var(--accent)",
    blocked: "var(--red)",
    paused: "var(--purple)",
  };

  const ACTIVE_STATUSES = new Set(["active", "in_progress", "in_review"]);

  // Derive status counts for filter pills
  let statusCounts = $derived.by(() => {
    const nonArchived = features.filter((f) => !f.archived);
    return {
      all: nonArchived.length,
      active: nonArchived.filter((f) => ACTIVE_STATUSES.has(f.status)).length,
      done: features.filter((f) => f.archived || f.status === "done").length, // includes archived
    };
  });

  // Filter features first
  let filteredFeatures = $derived(
    statusFilter === "all"
      ? features.filter((f) => !f.archived)
      : statusFilter === "done"
        ? features.filter((f) => f.archived || f.status === "done")
        : features.filter((f) => ACTIVE_STATUSES.has(f.status) && !f.archived)
  );

  // Build feature map for quick lookup
  let featureMap = $derived(new Map(filteredFeatures.map((f) => [f.id, f])));

  // Build flattened visible tree nodes
  interface TreeNode {
    feature: Feature;
    depth: number;
    hasChildren: boolean;
    isExpanded: boolean;
  }

  interface GroupedSection {
    groupId: string | null;
    group: FeatureGroup | null;
    nodes: TreeNode[];
    isCollapsed: boolean;
  }

  function buildTreeNodes(featureList: Feature[]): TreeNode[] {
    const nodes: TreeNode[] = [];
    const childrenMap = new Map<string, Feature[]>();

    for (const f of featureList) {
      const pid = f.parent_id ?? "__root__";
      if (!childrenMap.has(pid)) childrenMap.set(pid, []);
      childrenMap.get(pid)!.push(f);
    }

    // Sort each group: running terminals first, then pinned, then by sort_order
    for (const [, children] of childrenMap) {
      children.sort((a, b) => {
        const aRunning = terminalsByFeature.has(a.id) ? 1 : 0;
        const bRunning = terminalsByFeature.has(b.id) ? 1 : 0;
        if (aRunning !== bRunning) return bRunning - aRunning;
        if (a.pinned !== b.pinned) return a.pinned ? -1 : 1;
        return a.sort_order - b.sort_order;
      });
    }

    function addNodes(parentId: string, depth: number) {
      const children = childrenMap.get(parentId) ?? [];
      for (const f of children) {
        const hc = childrenMap.has(f.id) && childrenMap.get(f.id)!.length > 0;
        const expanded = expandedIds.has(f.id);
        nodes.push({ feature: f, depth, hasChildren: hc, isExpanded: expanded });
        if (hc && expanded) {
          addNodes(f.id, depth + 1);
        }
      }
    }

    addNodes("__root__", 0);
    return nodes;
  }

  let groupedSections = $derived.by((): GroupedSection[] => {
    // When a filter is active (not "all"), flatten to a simple list (ignore groups)
    if (statusFilter !== "all") {
      const sorted = [...filteredFeatures].sort((a, b) => {
        const aHas = terminalsByFeature.has(a.id) ? 1 : 0;
        const bHas = terminalsByFeature.has(b.id) ? 1 : 0;
        if (aHas !== bHas) return bHas - aHas;
        return 0;
      });
      return [{
        groupId: null,
        group: null,
        isCollapsed: false,
        nodes: sorted.map((f) => ({
          feature: f,
          depth: 0,
          hasChildren: false,
          isExpanded: false,
        })),
      }];
    }

    // Split features by group — treat orphaned group_ids (no matching group record) as ungrouped
    const knownGroupIds = new Set(featureGroups.map((g) => g.id));
    const ungrouped = filteredFeatures.filter((f) => !f.group_id || !knownGroupIds.has(f.group_id));
    const byGroup = new Map<string, Feature[]>();
    for (const f of filteredFeatures) {
      if (f.group_id && knownGroupIds.has(f.group_id)) {
        if (!byGroup.has(f.group_id)) byGroup.set(f.group_id, []);
        byGroup.get(f.group_id)!.push(f);
      }
    }

    const sections: GroupedSection[] = [];

    // Ungrouped section (no header)
    const ungroupedNodes = buildTreeNodes(ungrouped);
    if (ungroupedNodes.length > 0) {
      sections.push({
        groupId: null,
        group: null,
        isCollapsed: false,
        nodes: ungroupedNodes,
      });
    }

    // Each group section
    for (const group of featureGroups) {
      const groupFeatures = byGroup.get(group.id) ?? [];
      const isCollapsed = collapsedGroups.has(group.id);
      sections.push({
        groupId: group.id,
        group,
        isCollapsed,
        nodes: isCollapsed ? [] : buildTreeNodes(groupFeatures),
      });
    }

    return sections;
  });

  // Flat list for drag hit-testing
  let allVisibleNodes = $derived(groupedSections.flatMap((s) => s.nodes));

  // Check if targetId is a descendant of featureId
  function isDescendant(featureId: string, targetId: string): boolean {
    let current = featureMap.get(targetId);
    while (current) {
      if (current.parent_id === featureId) return true;
      if (!current.parent_id) break;
      current = featureMap.get(current.parent_id);
    }
    return false;
  }

  // ── Custom mouse drag handlers ────────────────────────────────────

  function handleMouseDown(e: MouseEvent, feature: Feature) {
    if (e.button !== 0) return;
    if ((e.target as HTMLElement).closest(".tree-chevron")) return;
    e.preventDefault();
    dragStartPos = { x: e.clientX, y: e.clientY };
    pendingDragFeature = feature;
    window.addEventListener("mousemove", handleGlobalMouseMove);
    window.addEventListener("mouseup", handleGlobalMouseUp);
  }

  function handleGlobalMouseMove(e: MouseEvent) {
    if (pendingDragFeature && dragStartPos && !draggingId) {
      const dx = e.clientX - dragStartPos.x;
      const dy = e.clientY - dragStartPos.y;
      if (Math.abs(dx) > DRAG_THRESHOLD || Math.abs(dy) > DRAG_THRESHOLD) {
        draggingId = pendingDragFeature.id;
        dragGhost = { x: e.clientX, y: e.clientY, title: pendingDragFeature.title };
      }
    }
    if (draggingId && dragGhost) {
      dragGhost = { ...dragGhost, x: e.clientX, y: e.clientY };

      // Hit-test which element we're over
      const el = document.elementFromPoint(e.clientX, e.clientY);

      // Check if over a group header
      const groupEl = el?.closest("[data-group-id]") as HTMLElement | null;
      if (groupEl) {
        const gid = groupEl.dataset.groupId!;
        dropTargetId = `group:${gid}`;
        dropZone = "child";
        clearAutoExpand();
        return;
      }

      const wrapperEl = el?.closest("[data-feature-id]") as HTMLElement | null;

      if (wrapperEl) {
        const targetFid = wrapperEl.dataset.featureId!;
        if (targetFid !== draggingId && !isDescendant(draggingId, targetFid)) {
          const rect = wrapperEl.getBoundingClientRect();
          const y = e.clientY - rect.top;
          const h = rect.height;

          let zone: "above" | "child" | "below";
          if (y < h * 0.25) zone = "above";
          else if (y > h * 0.75) zone = "below";
          else zone = "child";

          dropTargetId = targetFid;
          dropZone = zone;

          // Auto-expand collapsed parents
          if (zone === "child") {
            const node = allVisibleNodes.find((n) => n.feature.id === targetFid);
            if (node && node.hasChildren && !node.isExpanded && !autoExpandTimer) {
              autoExpandTimer = setTimeout(() => {
                toggleExpanded(targetFid);
                autoExpandTimer = null;
              }, 600);
            }
          } else {
            clearAutoExpand();
          }
        } else {
          dropTargetId = null;
          dropZone = null;
          clearAutoExpand();
        }
      } else {
        // Check if over root drop zone
        const rootEl = el?.closest(".root-drop-zone") as HTMLElement | null;
        if (rootEl) {
          dropTargetId = "__root__";
          dropZone = "child";
        } else {
          dropTargetId = null;
          dropZone = null;
        }
        clearAutoExpand();
      }
    }
  }

  function clearAutoExpand() {
    if (autoExpandTimer) {
      clearTimeout(autoExpandTimer);
      autoExpandTimer = null;
    }
  }

  async function handleGlobalMouseUp() {
    window.removeEventListener("mousemove", handleGlobalMouseMove);
    window.removeEventListener("mouseup", handleGlobalMouseUp);

    if (draggingId && dropTargetId && dropZone) {
      justDropped = true;
      requestAnimationFrame(() => { justDropped = false; });

      if (dropTargetId.startsWith("group:")) {
        // Drop onto a group header — move feature into that group
        const groupId = dropTargetId.slice(6);
        await setFeatureGroup(draggingId, groupId);
        // Also unparent if nested
        const f = featureMap.get(draggingId);
        if (f && f.parent_id) {
          await setFeatureParent(draggingId, null);
        }
        onFeaturesChanged?.();
      } else if (dropTargetId === "__root__") {
        // Unparent and ungroup
        const f = featureMap.get(draggingId);
        if (f && f.parent_id) {
          await setFeatureParent(draggingId, null);
        }
        if (f && f.group_id) {
          await setFeatureGroup(draggingId, null);
        }
        onFeaturesChanged?.();
      } else if (dropZone === "child") {
        // Make child of target
        await setFeatureParent(draggingId, dropTargetId);
        // Inherit target's group_id
        const targetFeature = featureMap.get(dropTargetId);
        const draggedFeature = featureMap.get(draggingId);
        if (targetFeature && draggedFeature && draggedFeature.group_id !== targetFeature.group_id) {
          await setFeatureGroup(draggingId, targetFeature.group_id ?? null);
        }
        if (!expandedIds.has(dropTargetId)) {
          toggleExpanded(dropTargetId);
        }
        onFeaturesChanged?.();
      } else {
        // Reorder as sibling
        const targetFeature = featureMap.get(dropTargetId);
        const newParentId = targetFeature?.parent_id ?? null;
        const draggedFeature = featureMap.get(draggingId);

        if (draggedFeature?.parent_id !== newParentId) {
          await setFeatureParent(draggingId, newParentId);
        }
        // Match target's group
        if (draggedFeature && targetFeature && draggedFeature.group_id !== targetFeature.group_id) {
          await setFeatureGroup(draggingId, targetFeature.group_id ?? null);
        }

        const siblings = filteredFeatures
          .filter((f) => (f.parent_id ?? null) === newParentId && f.id !== draggingId && (f.group_id ?? null) === (targetFeature?.group_id ?? null))
          .sort((a, b) => a.sort_order - b.sort_order);

        const targetIdx = siblings.findIndex((f) => f.id === dropTargetId);
        const insertIdx = dropZone === "above" ? targetIdx : targetIdx + 1;
        const newOrder = [...siblings];
        newOrder.splice(insertIdx, 0, draggedFeature!);
        await reorderFeatures(newOrder.map((f) => f.id));
        onFeaturesChanged?.();
      }
    }

    draggingId = null;
    dragGhost = null;
    dropTargetId = null;
    dropZone = null;
    dragStartPos = null;
    pendingDragFeature = null;
    clearAutoExpand();
  }

  // ── Group drag handlers ────────────────────────────────────────────────

  function handleGroupMouseDown(e: MouseEvent, group: FeatureGroup) {
    if (e.button !== 0) return;
    if ((e.target as HTMLElement).closest("input")) return;
    e.preventDefault();
    dragGroupStartPos = { x: e.clientX, y: e.clientY };
    pendingDragGroup = group;
    window.addEventListener("mousemove", handleGroupGlobalMouseMove);
    window.addEventListener("mouseup", handleGroupGlobalMouseUp);
  }

  function handleGroupGlobalMouseMove(e: MouseEvent) {
    if (pendingDragGroup && dragGroupStartPos && !draggingGroupId) {
      const dx = e.clientX - dragGroupStartPos.x;
      const dy = e.clientY - dragGroupStartPos.y;
      if (Math.abs(dx) > DRAG_THRESHOLD || Math.abs(dy) > DRAG_THRESHOLD) {
        draggingGroupId = pendingDragGroup.id;
        dragGroupGhost = { x: e.clientX, y: e.clientY, title: pendingDragGroup.name };
      }
    }
    if (draggingGroupId && dragGroupGhost) {
      dragGroupGhost = { ...dragGroupGhost, x: e.clientX, y: e.clientY };

      const el = document.elementFromPoint(e.clientX, e.clientY);
      const groupEl = el?.closest("[data-group-id]") as HTMLElement | null;
      if (groupEl) {
        const gid = groupEl.dataset.groupId!;
        if (gid !== draggingGroupId) {
          const rect = groupEl.getBoundingClientRect();
          const y = e.clientY - rect.top;
          dropGroupTargetId = gid;
          dropGroupZone = y < rect.height / 2 ? "above" : "below";
        } else {
          dropGroupTargetId = null;
          dropGroupZone = null;
        }
      } else {
        dropGroupTargetId = null;
        dropGroupZone = null;
      }
    }
  }

  async function handleGroupGlobalMouseUp() {
    window.removeEventListener("mousemove", handleGroupGlobalMouseMove);
    window.removeEventListener("mouseup", handleGroupGlobalMouseUp);

    if (draggingGroupId && dropGroupTargetId && dropGroupZone) {
      const currentOrder = featureGroups.map((g) => g.id);
      const fromIdx = currentOrder.indexOf(draggingGroupId);
      if (fromIdx !== -1) {
        currentOrder.splice(fromIdx, 1);
        let toIdx = currentOrder.indexOf(dropGroupTargetId);
        if (dropGroupZone === "below") toIdx += 1;
        currentOrder.splice(toIdx, 0, draggingGroupId);
        await reorderFeatureGroups(currentOrder);
        onFeaturesChanged?.();
      }
    }

    draggingGroupId = null;
    dragGroupGhost = null;
    dropGroupTargetId = null;
    dropGroupZone = null;
    dragGroupStartPos = null;
    pendingDragGroup = null;
  }

  // ── Context menu handlers ──────────────────────────────────────────────

  function handleContextMenu(e: MouseEvent, feature: Feature) {
    e.preventDefault();
    groupContextMenu = null;
    contextMenu = { x: e.clientX, y: e.clientY, feature };
  }

  function handleGroupContextMenu(e: MouseEvent, group: FeatureGroup) {
    e.preventDefault();
    contextMenu = null;
    groupContextMenu = { x: e.clientX, y: e.clientY, group };
  }

  function closeContextMenu() {
    contextMenu = null;
    groupContextMenu = null;
  }

  async function handleDuplicate() {
    if (!contextMenu) return;
    const f = contextMenu.feature;
    closeContextMenu();
    const newFeature = await duplicateFeature(f.id);
    onFeaturesChanged?.();
    onSelect(newFeature.id);
  }

  async function handleTogglePin() {
    if (!contextMenu) return;
    const f = contextMenu.feature;
    closeContextMenu();
    await togglePinFeature(f.id);
    onFeaturesChanged?.();
  }

  async function handleArchive() {
    if (!contextMenu) return;
    const f = contextMenu.feature;
    closeContextMenu();
    if (f.archived) {
      await setFeatureArchived(f.id, false);
      await updateFeature(f.id, { status: "todo" });
    } else {
      await updateFeature(f.id, { status: "done" });
    }
    onFeaturesChanged?.();
    if (selectedId === f.id && !f.archived) {
      onSelect("");
    }
  }

  async function handleMoveToRoot() {
    if (!contextMenu) return;
    const f = contextMenu.feature;
    closeContextMenu();
    if (f.parent_id) {
      await setFeatureParent(f.id, null);
    }
    if (f.group_id) {
      await setFeatureGroup(f.id, null);
    }
    onFeaturesChanged?.();
  }

  async function handleMoveToGroup(groupId: string | null) {
    if (!contextMenu) return;
    const f = contextMenu.feature;
    closeContextMenu();
    await setFeatureGroup(f.id, groupId);
    // If moving to a group and was a child, unparent unless parent is in same group
    if (groupId && f.parent_id) {
      const parent = featureMap.get(f.parent_id);
      if (parent && parent.group_id !== groupId) {
        await setFeatureParent(f.id, null);
      }
    }
    onFeaturesChanged?.();
  }

  function handleDelete() {
    if (!contextMenu) return;
    deleteConfirm = contextMenu.feature;
    closeContextMenu();
  }

  async function confirmDelete() {
    if (!deleteConfirm) return;
    const f = deleteConfirm;
    deleteConfirm = null;
    await deleteFeature(f.id);
    onFeaturesChanged?.();
    if (selectedId === f.id) {
      onSelect("");
    }
  }

  // ── Group actions ──────────────────────────────────────────────

  async function handleCreateGroup() {
    const name = newGroupName.trim();
    if (!name) return;
    await createFeatureGroup(name);
    newGroupName = "";
    creatingGroup = false;
    onFeaturesChanged?.();
  }

  function startRenameGroup(group: FeatureGroup) {
    closeContextMenu();
    renamingGroupId = group.id;
    renamingGroupName = group.name;
  }

  async function finishRenameGroup() {
    if (!renamingGroupId) return;
    const name = renamingGroupName.trim();
    if (name) {
      await updateFeatureGroup(renamingGroupId, name);
      onFeaturesChanged?.();
    }
    renamingGroupId = null;
    renamingGroupName = "";
  }

  function handleDeleteGroup() {
    if (!groupContextMenu) return;
    deleteGroupConfirm = groupContextMenu.group;
    closeContextMenu();
  }

  async function confirmDeleteGroup() {
    if (!deleteGroupConfirm) return;
    const g = deleteGroupConfirm;
    deleteGroupConfirm = null;
    await deleteFeatureGroup(g.id);
    onFeaturesChanged?.();
  }

  // Close context menu on click anywhere
  $effect(() => {
    if (!contextMenu && !groupContextMenu) return;
    function handleClick() { contextMenu = null; groupContextMenu = null; }
    window.addEventListener("click", handleClick);
    return () => window.removeEventListener("click", handleClick);
  });

  // Has any groups at all
  let hasGroups = $derived(featureGroups.length > 0);
  let totalNodes = $derived(groupedSections.reduce((n, s) => n + s.nodes.length, 0) + groupedSections.filter(s => s.group).length);
</script>

<aside class="sidebar" style:width={width ? `${width}px` : undefined} style:min-width={width ? `${width}px` : undefined} style:container-type="inline-size" style:container-name="sidebar">
  <div class="sidebar-header">
    <button bind:this={logoEl} class="sidebar-logo" class:sidebar-logo--emoji={activeStorage.icon && !activeStorage.icon.startsWith("data:")} class:sidebar-logo--img={activeStorage.icon?.startsWith("data:")} onclick={(e: MouseEvent) => { e.stopPropagation(); toggleIconPicker(); }} title="Change icon">
      {#if activeStorage.icon?.startsWith("data:")}
        <img src={activeStorage.icon} alt="" class="sidebar-logo-img" />
      {:else}
        {activeStorage.icon || 'FH'}
      {/if}
    </button>
    {#if !width || width >= 250}
      <div class="sidebar-title">{activeStorage.name}</div>
      {#if storageIndex !== null}
        <kbd class="sidebar-shortcut">Shift+{storageIndex + 1}</kbd>
      {/if}
    {/if}
  </div>

  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="search-trigger" onclick={() => onOpenSearch?.()}
    onkeydown={(e) => { if (e.key === 'Enter') onOpenSearch?.(); }}
  >
    <svg width="13" height="13" viewBox="0 0 16 16" fill="var(--text-muted)">
      <path d="M6.5 1a5.5 5.5 0 014.38 8.82l3.65 3.65a.75.75 0 01-1.06 1.06l-3.65-3.65A5.5 5.5 0 116.5 1zm0 1.5a4 4 0 100 8 4 4 0 000-8z"/>
    </svg>
    <span class="search-trigger-text">Search...</span>
    <span class="search-trigger-key">Ctrl T</span>
  </div>

  <div class="sidebar-filters">
    {#each filters as f}
      <button
        class="filter-pill {statusFilter === f.value ? 'filter-pill--active' : ''}"
        onclick={() => (statusFilter = f.value)}
      >
        {f.label}{#if statusCounts[f.value]}<span class="filter-pill-count"> ({statusCounts[f.value]})</span>{/if}
      </button>
    {/each}
  </div>

  <div class="sidebar-list">
    {#if totalNodes === 0 && !creatingGroup}
      <div style="padding: 40px 16px; text-align: center; font-size: 12px; color: var(--text-muted);">
        {statusFilter === "all" ? "No features yet" : statusFilter === "done" ? "No completed features" : "No matching features"}
      </div>
    {:else}
      {#each groupedSections as section (section.groupId ?? "__ungrouped__")}
        <!-- Group header -->
        {#if section.group}
          <div
            class="group-header"
            class:group-header--drop={dropTargetId === `group:${section.group.id}`}
            class:group-header--dragging={draggingGroupId === section.group.id}
            class:group-header--drop-above={dropGroupTargetId === section.group.id && dropGroupZone === "above"}
            class:group-header--drop-below={dropGroupTargetId === section.group.id && dropGroupZone === "below"}
            data-group-id={section.group.id}
            onmousedown={(e) => handleGroupMouseDown(e, section.group!)}
            onclick={() => { if (!draggingGroupId) toggleGroupCollapsed(section.group!.id); }}
            onkeydown={(e) => { if (e.key === 'Enter' && !draggingGroupId) toggleGroupCollapsed(section.group!.id); }}
            oncontextmenu={(e) => handleGroupContextMenu(e, section.group!)}
            role="button"
            tabindex="0"
          >
            <span
              class="group-chevron"
              class:group-chevron--collapsed={section.isCollapsed}
            >
              <svg width="10" height="10" viewBox="0 0 16 16" fill="var(--text-muted)">
                <path d="M6 3.5l5 4.5-5 4.5V3.5z"/>
              </svg>
            </span>
            {#if renamingGroupId === section.group.id}
              <!-- svelte-ignore a11y_autofocus -->
              <input
                class="group-rename-input"
                type="text"
                bind:value={renamingGroupName}
                onclick={(e) => e.stopPropagation()}
                onkeydown={(e) => {
                  if (e.key === 'Enter') finishRenameGroup();
                  if (e.key === 'Escape') { renamingGroupId = null; renamingGroupName = ""; }
                }}
                onblur={() => finishRenameGroup()}
                autofocus
              />
            {:else}
              {#if section.group.color}
                <div class="group-color-dot" style="background: {section.group.color};"></div>
              {/if}
              <span class="group-name">{section.group.name}</span>
              <span class="group-count">
                {filteredFeatures.filter((f) => f.group_id === section.group!.id).length}
              </span>
            {/if}
          </div>
        {/if}

        <!-- Features in this section -->
        {#if !section.isCollapsed}
          {#each section.nodes as node (node.feature.id)}
            {@const feature = node.feature}
            {@const isOver = dropTargetId === feature.id}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="feature-item-wrapper"
              class:feature-item-drop-above={isOver && dropZone === "above"}
              class:feature-item-drop-child={isOver && dropZone === "child"}
              class:feature-item-drop-below={isOver && dropZone === "below"}
              class:feature-item-dragging={draggingId === feature.id}
              data-feature-id={feature.id}
            >
              <button
                class="feature-item feature-item--status-{feature.status} {selectedId === feature.id ? 'feature-item--selected' : ''} {feature.archived ? 'feature-item--archived' : ''} {viewingTerminalId && terminalsByFeature.get(feature.id)?.some(t => t.terminalId === viewingTerminalId) ? 'feature-item--session-active' : ''}"
                style="padding-left: {10 + node.depth * 16}px;"
                onmousedown={(e) => handleMouseDown(e, feature)}
                onclick={(e: MouseEvent) => {
                  if (draggingId || justDropped) return;
                  // Ctrl/Cmd+Click opens in a new workspace tab
                  if ((e.ctrlKey || e.metaKey) && onSelectNewTab) {
                    onSelectNewTab(feature.id);
                    return;
                  }
                  // If already selected and has embedded terminals, open the first one
                  if (selectedId === feature.id) {
                    const terms = terminalsByFeature.get(feature.id);
                    if (terms && terms.length > 0) {
                      onSelectTerminal?.(feature.id, terms[0].terminalId);
                      return;
                    }
                  }
                  onSelect(feature.id);
                }}
                oncontextmenu={(e) => handleContextMenu(e, feature)}
              >
                {#if feature.pinned}
                  <svg class="pin-icon" width="10" height="10" viewBox="0 0 16 16" fill="var(--accent)">
                    <path d="M9.828.722a.5.5 0 01.354.146l4.95 4.95a.5.5 0 01-.707.707l-.707-.707-3.182 3.182a3.5 3.5 0 01-.564.41l-2.05 1.166a.5.5 0 01-.639-.112l-.41-.41-3.96 3.96a.5.5 0 01-.707-.707l3.96-3.96-.41-.41a.5.5 0 01-.112-.639l1.166-2.05a3.5 3.5 0 01.41-.564l3.182-3.182-.707-.707a.5.5 0 01.146-.854z"/>
                  </svg>
                {/if}
                  <div class="feature-item-compact">
                    <span class="feature-item-status-dot" style="background: {statusColors[feature.status] ?? 'var(--text-muted)'};"></span>
                    <span class="feature-item-title">{feature.title}</span>
                    {#if (feature.task_count_total ?? 0) > 0}
                      <div class="feature-item-pmb">
                        <div class="feature-item-pmb-track">
                          <div class="feature-item-pmb-fill" style="width: {Math.round(((feature.task_count_done ?? 0) / (feature.task_count_total ?? 1)) * 100)}%;"></div>
                        </div>
                        <span class="feature-item-pmb-label">{feature.task_count_done ?? 0}/{feature.task_count_total}</span>
                      </div>
                    {/if}
                    {#if getActiveCountForFeature(feature.id) > 0}
                      {@const count = getActiveCountForFeature(feature.id)}
                      <!-- svelte-ignore a11y_no_static_element_interactions -->
                      <span class="active-sessions-badge" title="{count} active session{count > 1 ? 's' : ''} — click to open"
                        role="button"
                        tabindex="-1"
                        onclick={(e: MouseEvent) => {
                          e.stopPropagation();
                          // Ctrl/Cmd+Click opens in a new workspace tab first
                          if ((e.ctrlKey || e.metaKey) && onSelectNewTab) {
                            onSelectNewTab(feature.id);
                          }
                          const terms = terminalsByFeature.get(feature.id);
                          if (terms && terms.length > 0) {
                            onSelectTerminal?.(feature.id, terms[0].terminalId);
                          } else {
                            onSelectSessions?.(feature.id);
                          }
                        }}
                        onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') { e.stopPropagation(); const terms = terminalsByFeature.get(feature.id); if (terms && terms.length > 0) { onSelectTerminal?.(feature.id, terms[0].terminalId); } else { onSelectSessions?.(feature.id); } } }}
                      >
                        <span class="active-sessions-dot"></span>{count}
                      </span>
                    {/if}
                    {#if node.hasChildren}
                      <!-- svelte-ignore a11y_no_static_element_interactions -->
                      <span class="tree-chevron" class:tree-chevron--expanded={node.isExpanded}
                        onclick={(e: MouseEvent) => { e.stopPropagation(); toggleExpanded(feature.id); }}
                        onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') { e.stopPropagation(); toggleExpanded(feature.id); } }}
                        role="button" tabindex="-1">
                        <svg width="10" height="10" viewBox="0 0 16 16" fill="var(--text-muted)"><path d="M6 3.5l5 4.5-5 4.5V3.5z"/></svg>
                      </span>
                    {/if}
                  </div>
              </button>
            </div>
            <!-- Running terminal sessions under this feature -->
            {#if terminalsByFeature.has(feature.id)}
              {#each terminalsByFeature.get(feature.id) ?? [] as term (term.terminalId)}
                <div
                  class="feature-session-item {term.exited ? 'feature-session-item--exited' : ''} {term.needsInput ? 'feature-session-item--input' : ''} {viewingTerminalId === term.terminalId ? 'feature-session-item--viewing' : ''}"
                  style="padding-left: {26 + node.depth * 16}px;"
                  onmousedown={(e) => e.stopPropagation()}
                  onclick={(e: MouseEvent) => {
                    e.stopPropagation();
                    // Ctrl/Cmd+Click opens in a new workspace tab, then navigates to terminal
                    if ((e.ctrlKey || e.metaKey) && onSelectNewTab) {
                      onSelectNewTab(term.featureId);
                    }
                    onSelectTerminal?.(term.featureId, term.terminalId);
                  }}
                  onkeydown={(e) => { if (e.key === 'Enter') { e.stopPropagation(); onSelectTerminal?.(term.featureId, term.terminalId); } }}
                  role="button"
                  tabindex="0"
                >
                  <button
                    class="feature-session-main"
                  >
                    <span class="sidebar-terminal-dot-wrap">
                      {#if term.exited}
                        <span class="terminal-tab-dot"></span>
                      {:else if term.needsInput}
                        <span class="sidebar-terminal-input-dot"></span>
                      {:else}
                        <span class="terminal-tab-dot terminal-tab-dot--live"></span>
                      {/if}
                    </span>
                    <span class="feature-session-label">{term.label}</span>
                    {#if term.needsInput}
                      <span class="sidebar-terminal-input-badge">Input</span>
                    {/if}
                    {#if term.statusLine && !term.needsInput}
                      <span class="feature-session-status">{term.statusLine}</span>
                    {/if}
                  </button>
                  <button
                    class="feature-session-finish"
                    onclick={(e) => { e.stopPropagation(); onFinishTerminal?.(term.terminalId, term.sessionDbId); }}
                    title="Finish session"
                  >
                    <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M3.72 3.72a.75.75 0 011.06 0L8 6.94l3.22-3.22a.749.749 0 011.275.326.749.749 0 01-.215.734L9.06 8l3.22 3.22a.749.749 0 01-.326 1.275.749.749 0 01-.734-.215L8 9.06l-3.22 3.22a.751.751 0 01-1.042-.018.751.751 0 01-.018-1.042L6.94 8 3.72 4.78a.75.75 0 010-1.06z"/></svg>
                  </button>
                </div>
                {#if term.needsInput && term.statusLine}
                  <div class="feature-session-status-line {term.needsInput ? 'sidebar-terminal-status--input' : ''}" style="padding-left: {40 + node.depth * 16}px;">
                    {term.statusLine}
                  </div>
                {/if}
              {/each}
            {/if}
          {/each}
        {/if}
      {/each}

      <!-- New group inline input -->
      {#if creatingGroup}
        <div class="group-header group-header--creating">
          <!-- svelte-ignore a11y_autofocus -->
          <input
            class="group-rename-input"
            type="text"
            placeholder="Group name..."
            bind:value={newGroupName}
            onkeydown={(e) => {
              if (e.key === 'Enter') handleCreateGroup();
              if (e.key === 'Escape') { creatingGroup = false; newGroupName = ""; }
            }}
            onblur={() => { if (!newGroupName.trim()) { creatingGroup = false; newGroupName = ""; } else { handleCreateGroup(); } }}
            autofocus
          />
        </div>
      {/if}

      <!-- Root drop zone for unparenting/ungrouping -->
      {#if draggingId}
        <div
          class="root-drop-zone"
          class:root-drop-zone--active={dropTargetId === "__root__"}
        >
          Drop here to move to root
        </div>
      {/if}
    {/if}
  </div>

  <div class="sidebar-footer">
    <button class="btn-new-feature" onclick={onCreateNew}>
      <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M8 2a.75.75 0 01.75.75v4.5h4.5a.75.75 0 010 1.5h-4.5v4.5a.75.75 0 01-1.5 0v-4.5h-4.5a.75.75 0 010-1.5h4.5v-4.5A.75.75 0 018 2z"/></svg>
      New Feature
    </button>
    <button class="btn-ghost sidebar-footer-btn" onclick={() => { creatingGroup = true; }} title="New Group">
      <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
        <path d="M1 3.5A1.5 1.5 0 012.5 2h3.879a1.5 1.5 0 011.06.44l1.122 1.12A.5.5 0 008.914 4H13.5A1.5 1.5 0 0115 5.5v7a1.5 1.5 0 01-1.5 1.5h-11A1.5 1.5 0 011 12.5v-9zM2.5 3a.5.5 0 00-.5.5v9a.5.5 0 00.5.5h11a.5.5 0 00.5-.5v-7a.5.5 0 00-.5-.5H8.914a1.5 1.5 0 01-1.06-.44L6.732 3.44A.5.5 0 006.379 3H2.5z"/>
      </svg>
    </button>
  </div>

  <StorageSelector {activeStorage} onSwitch={onStorageSwitch} />
</aside>

{#if contextMenu}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="context-menu" style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
    oncontextmenu={(e) => e.preventDefault()}
  >
    <button class="context-menu-item" onclick={handleTogglePin}>
      <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
        <path d="M9.828.722a.5.5 0 01.354.146l4.95 4.95a.5.5 0 01-.707.707l-.707-.707-3.182 3.182a3.5 3.5 0 01-.564.41l-2.05 1.166a.5.5 0 01-.639-.112l-.41-.41-3.96 3.96a.5.5 0 01-.707-.707l3.96-3.96-.41-.41a.5.5 0 01-.112-.639l1.166-2.05a3.5 3.5 0 01.41-.564l3.182-3.182-.707-.707a.5.5 0 01.146-.854z"/>
      </svg>
      {contextMenu.feature.pinned ? "Unpin" : "Pin to top"}
    </button>
    <button class="context-menu-item" onclick={handleDuplicate}>
      <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
        <path d="M4 2a2 2 0 012-2h8a2 2 0 012 2v8a2 2 0 01-2 2H6a2 2 0 01-2-2V2zm2-1a1 1 0 00-1 1v8a1 1 0 001 1h8a1 1 0 001-1V2a1 1 0 00-1-1H6z"/>
        <path d="M2 6a2 2 0 012-2v1a1 1 0 00-1 1v8a1 1 0 001 1h8a1 1 0 001-1h1a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"/>
      </svg>
      Duplicate
    </button>
    {#if contextMenu.feature.parent_id || contextMenu.feature.group_id}
      <button class="context-menu-item" onclick={handleMoveToRoot}>
        <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 1a.5.5 0 01.5.5v11.793l3.146-3.147a.5.5 0 01.708.708l-4 4a.5.5 0 01-.708 0l-4-4a.5.5 0 01.708-.708L7.5 13.293V1.5A.5.5 0 018 1z" transform="rotate(180 8 8)"/>
        </svg>
        Move to root
      </button>
    {/if}
    <!-- Move to group submenu -->
    {#if featureGroups.length > 0}
      <div class="context-menu-separator"></div>
      <div class="context-menu-label">Move to group</div>
      {#each featureGroups as group}
        {#if group.id !== contextMenu.feature.group_id}
          <button class="context-menu-item context-menu-item--indent" onclick={() => handleMoveToGroup(group.id)}>
            {#if group.color}
              <div style="width: 8px; height: 8px; border-radius: 2px; background: {group.color}; flex-shrink: 0;"></div>
            {:else}
              <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
                <path d="M1 3.5A1.5 1.5 0 012.5 2h3.879a1.5 1.5 0 011.06.44l1.122 1.12A.5.5 0 008.914 4H13.5A1.5 1.5 0 0115 5.5v7a1.5 1.5 0 01-1.5 1.5h-11A1.5 1.5 0 011 12.5v-9z"/>
              </svg>
            {/if}
            {group.name}
          </button>
        {/if}
      {/each}
      {#if contextMenu.feature.group_id}
        <button class="context-menu-item context-menu-item--indent" onclick={() => handleMoveToGroup(null)}>
          <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
            <path d="M3.72 3.72a.75.75 0 011.06 0L8 6.94l3.22-3.22a.749.749 0 011.275.326.749.749 0 01-.215.734L9.06 8l3.22 3.22a.749.749 0 01-.326 1.275.749.749 0 01-.734-.215L8 9.06l-3.22 3.22a.751.751 0 01-1.042-.018.751.751 0 01-.018-1.042L6.94 8 3.72 4.78a.75.75 0 010-1.06z"/>
          </svg>
          Remove from group
        </button>
      {/if}
    {/if}
    <div class="context-menu-separator"></div>
    <button class="context-menu-item" onclick={handleArchive}>
      <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
        {#if contextMenu.feature.archived}
          <path d="M8 1a.5.5 0 01.5.5v11.793l3.146-3.147a.5.5 0 01.708.708l-4 4a.5.5 0 01-.708 0l-4-4a.5.5 0 01.708-.708L7.5 13.293V1.5A.5.5 0 018 1z" transform="rotate(180 8 8)"/>
        {:else}
          <path d="M13.3 4.3L6 11.6 2.7 8.3l1.4-1.4L6 8.8l5.9-5.9z"/>
        {/if}
      </svg>
      {contextMenu.feature.archived ? "Reopen" : "Mark Done"}
    </button>
    <button class="context-menu-item context-menu-item--danger" onclick={handleDelete}>
      <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
        <path d="M5.5 5.5A.5.5 0 016 6v6a.5.5 0 01-1 0V6a.5.5 0 01.5-.5zm2.5 0a.5.5 0 01.5.5v6a.5.5 0 01-1 0V6a.5.5 0 01.5-.5zm3 .5a.5.5 0 00-1 0v6a.5.5 0 001 0V6z"/>
        <path d="M14.5 3a1 1 0 01-1 1H13v9a2 2 0 01-2 2H5a2 2 0 01-2-2V4h-.5a1 1 0 01-1-1V2a1 1 0 011-1H5a1 1 0 011-1h4a1 1 0 011 1h2.5a1 1 0 011 1v1zM4.118 4L4 4.059V13a1 1 0 001 1h6a1 1 0 001-1V4.059L11.882 4H4.118zM2.5 3h11V2h-11v1z"/>
      </svg>
      Delete
    </button>
  </div>
{/if}

{#if groupContextMenu}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="context-menu" style="left: {groupContextMenu.x}px; top: {groupContextMenu.y}px;"
    oncontextmenu={(e) => e.preventDefault()}
  >
    <button class="context-menu-item" onclick={() => startRenameGroup(groupContextMenu!.group)}>
      <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
        <path d="M12.146.146a.5.5 0 01.708 0l3 3a.5.5 0 010 .708l-10 10a.5.5 0 01-.168.11l-5 2a.5.5 0 01-.65-.65l2-5a.5.5 0 01.11-.168l10-10zM11.207 2.5L13.5 4.793 14.793 3.5 12.5 1.207 11.207 2.5zm1.586 3L10.5 3.207 4 9.707V10h.5a.5.5 0 01.5.5v.5h.5a.5.5 0 01.5.5v.5h.293l6.5-6.5z"/>
      </svg>
      Rename
    </button>
    <button class="context-menu-item context-menu-item--danger" onclick={handleDeleteGroup}>
      <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
        <path d="M5.5 5.5A.5.5 0 016 6v6a.5.5 0 01-1 0V6a.5.5 0 01.5-.5zm2.5 0a.5.5 0 01.5.5v6a.5.5 0 01-1 0V6a.5.5 0 01.5-.5zm3 .5a.5.5 0 00-1 0v6a.5.5 0 001 0V6z"/>
        <path d="M14.5 3a1 1 0 01-1 1H13v9a2 2 0 01-2 2H5a2 2 0 01-2-2V4h-.5a1 1 0 01-1-1V2a1 1 0 011-1H5a1 1 0 011-1h4a1 1 0 011 1h2.5a1 1 0 011 1v1zM4.118 4L4 4.059V13a1 1 0 001 1h6a1 1 0 001-1V4.059L11.882 4H4.118zM2.5 3h11V2h-11v1z"/>
      </svg>
      Delete Group
    </button>
  </div>
{/if}

{#if deleteConfirm}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal-backdrop" onclick={(e) => { if (e.target === e.currentTarget) deleteConfirm = null; }} onkeydown={(e) => { if (e.key === 'Escape') deleteConfirm = null; }}>
    <div class="modal-content" style="width: 380px;">
      <h2 class="modal-title">Delete Feature</h2>
      <p class="modal-body-text" style="margin-bottom: var(--space-6);">
        Are you sure you want to delete <strong style="color: var(--text-primary);">{deleteConfirm.title}</strong>? This action cannot be undone.
      </p>
      <div class="modal-actions">
        <button class="btn-subtle" style="padding: 7px 16px;" onclick={() => (deleteConfirm = null)}>Cancel</button>
        <button class="btn-danger" onclick={confirmDelete}>Delete</button>
      </div>
    </div>
  </div>
{/if}

{#if deleteGroupConfirm}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal-backdrop" onclick={(e) => { if (e.target === e.currentTarget) deleteGroupConfirm = null; }} onkeydown={(e) => { if (e.key === 'Escape') deleteGroupConfirm = null; }}>
    <div class="modal-content" style="width: 380px;">
      <h2 class="modal-title">Delete Group</h2>
      <p class="modal-body-text" style="margin-bottom: var(--space-6);">
        Delete the group <strong style="color: var(--text-primary);">{deleteGroupConfirm.name}</strong>? Features in this group will be moved to the root level.
      </p>
      <div class="modal-actions">
        <button class="btn-subtle" style="padding: 7px 16px;" onclick={() => (deleteGroupConfirm = null)}>Cancel</button>
        <button class="btn-danger" onclick={confirmDeleteGroup}>Delete</button>
      </div>
    </div>
  </div>
{/if}

{#if dragGhost}
  <div class="sidebar-drag-ghost" style="left: {dragGhost.x}px; top: {dragGhost.y}px;">
    {dragGhost.title}
  </div>
{/if}

{#if dragGroupGhost}
  <div class="sidebar-drag-ghost" style="left: {dragGroupGhost.x}px; top: {dragGroupGhost.y}px;">
    {dragGroupGhost.title}
  </div>
{/if}

{#if showIconPicker}
  <div class="icon-picker-backdrop" onclick={() => showIconPicker = false} onkeydown={(e) => { if (e.key === 'Escape') showIconPicker = false; }} role="button" tabindex="-1"></div>
  <div class="icon-picker-popover" style="top: {pickerPos.top}px; left: {pickerPos.left}px;">
    <div class="icon-picker-grid">
      {#each emojiOptions as emoji}
        <button class="icon-picker-emoji" onclick={() => selectIcon(emoji)}>{emoji}</button>
      {/each}
    </div>
    <div class="icon-picker-custom">
      <input
        class="icon-picker-input"
        type="text"
        placeholder="Type emoji..."
        bind:value={iconInput}
        onkeydown={(e) => { if (e.key === 'Enter') commitIconInput(); if (e.key === 'Escape') showIconPicker = false; }}
      />
      <button class="icon-picker-btn" onclick={commitIconInput}>Set</button>
    </div>
    <div class="icon-picker-actions">
      <button class="icon-picker-btn" onclick={() => fileInputEl.click()}>Upload image</button>
      {#if activeStorage.icon}
        <button class="icon-picker-btn icon-picker-btn--danger" onclick={clearIcon}>Clear</button>
      {/if}
    </div>
    <input bind:this={fileInputEl} type="file" accept="image/*" style="display:none" onchange={handleImageUpload} />
  </div>
{/if}
