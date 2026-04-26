<script lang="ts">
  import type { Task, Note } from "../../api/tauri";
  import type { TabContext } from "../registry";
  import { setToolbarActions, clearToolbarActions } from "../../stores/tabToolbar.svelte";
  import { onDestroy } from "svelte";
  import TaskList from "./TaskList.svelte";
  import NotesEditor from "./NotesEditor.svelte";

  let {
    featureId,
    tasks = [],
    note = null,
    onRefresh,
  }: TabContext = $props();

  let panelEl: HTMLDivElement | undefined = $state();

  function onTasksChanged() {
    onRefresh();
  }

  function onNoteChanged(_n: Note | null) {
    onRefresh();
  }

  function focusTaskInput() {
    const input = panelEl?.querySelector<HTMLInputElement>('input[placeholder="Add a task..."]');
    input?.focus();
  }

  // Register toolbar actions
  $effect(() => {
    setToolbarActions("notes", [
      {
        id: "add-task",
        label: "Add Task",
        icon: '<svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor"><path d="M8 1a1 1 0 011 1v5h5a1 1 0 110 2H9v5a1 1 0 11-2 0V9H2a1 1 0 010-2h5V2a1 1 0 011-1z"/></svg>',
        onClick: focusTaskInput,
        title: "Focus the task input",
      },
    ]);
  });
  onDestroy(() => clearToolbarActions("notes"));
</script>

<div class="tn-panel tasks-notes-panel" bind:this={panelEl}>
  <div class="tn-tasks glass-panel tasks-column">
    <TaskList {featureId} {tasks} {onTasksChanged} />
  </div>
  <div class="tn-notes glass-panel notes-column">
    <NotesEditor {featureId} {note} {onNoteChanged} />
  </div>
</div>
