<script lang="ts">
  import type { Task } from "../../api/tauri";
  import { createTask, updateTask, deleteTask } from "../../api/tauri";

  let {
    featureId,
    tasks: tasksProp = [],
    onTasksChanged,
  }: {
    featureId: string;
    tasks?: Task[];
    onTasksChanged?: () => void;
  } = $props();

  let tasks = $state<Task[]>([]);
  let newTitle = $state("");
  let editingId = $state<string | null>(null);
  let editingTitle = $state("");
  let hoveredId = $state<string | null>(null);

  // Sync from parent prop
  $effect(() => {
    tasks = tasksProp;
  });

  async function handleAdd() {
    const t = newTitle.trim();
    if (!t) return;
    try {
      const task = await createTask(featureId, t);
      tasks = [...tasks, task];
      newTitle = "";
      onTasksChanged?.();
    } catch (e) {
      console.error("Failed to create task:", e);
    }
  }

  function handleAddKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") handleAdd();
  }

  async function handleToggle(task: Task) {
    try {
      const updated = await updateTask(task.id, undefined, !task.done);
      tasks = tasks.map((t) => (t.id === updated.id ? updated : t));
      onTasksChanged?.();
    } catch (e) {
      console.error("Failed to update task:", e);
    }
  }

  function startEdit(task: Task) {
    editingId = task.id;
    editingTitle = task.title;
  }

  async function saveEdit() {
    if (!editingId) return;
    const t = editingTitle.trim();
    if (!t) return;
    try {
      const updated = await updateTask(editingId, t);
      tasks = tasks.map((tk) => (tk.id === updated.id ? updated : tk));
    } catch (e) {
      console.error("Failed to update task:", e);
    }
    editingId = null;
  }

  function handleEditKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") saveEdit();
    if (e.key === "Escape") editingId = null;
  }

  async function handleDelete(id: string) {
    try {
      await deleteTask(id);
      tasks = tasks.filter((t) => t.id !== id);
      onTasksChanged?.();
    } catch (e) {
      console.error("Failed to delete task:", e);
    }
  }

  let manualTasks = $derived(tasks.filter((t) => t.source !== "jira"));
  let jiraTasks = $derived(tasks.filter((t) => t.source === "jira"));
  let doneTasks = $derived(manualTasks.filter((t) => t.done));
  let progress = $derived(manualTasks.length > 0 ? Math.round((doneTasks.length / manualTasks.length) * 100) : 0);
  let openTasks = $derived(manualTasks.filter((t) => !t.done));

  function jiraStatusColor(status: string | null): string {
    if (!status) return "var(--text-muted)";
    const s = status.toLowerCase();
    if (s === "done" || s === "closed" || s === "resolved") return "var(--green)";
    if (s.includes("progress") || s === "active") return "var(--blue, var(--accent))";
    if (s === "blocked") return "var(--red)";
    return "var(--text-secondary)";
  }
</script>

{#snippet manualTaskRow(task: Task)}
  {@const isDone = task.done}
  <div
    class="task-item"
    role="group"
    onmouseenter={() => (hoveredId = task.id)}
    onmouseleave={() => (hoveredId = null)}
  >
    <button
      class="task-checkbox{isDone ? ' task-checkbox--done' : ''}"
      onclick={() => handleToggle(task)}
      aria-label={isDone ? "Mark as not done" : "Mark as done"}
    >
      {#if isDone}
        <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M13.3 4.3L6 11.6 2.7 8.3l1.4-1.4L6 8.8l5.9-5.9z"/></svg>
      {/if}
    </button>

    {#if editingId === task.id}
      <input
        type="text"
        class="form-input"
        style="flex: 1; font-size: 12.5px; padding: 2px 6px;"
        bind:value={editingTitle}
        onblur={saveEdit}
        onkeydown={handleEditKeydown}
      />
    {:else}
      <span
        class="task-title{isDone ? ' task-title--done' : ''}"
        role="button"
        tabindex="0"
        ondblclick={() => startEdit(task)}
        onkeydown={(e) => { if (e.key === 'Enter') startEdit(task); }}
      >
        {task.title}
      </span>
    {/if}

    {#if hoveredId === task.id && editingId !== task.id}
      <button class="btn-ghost" style="color: var(--red); flex-shrink: 0;"
        onclick={() => handleDelete(task.id)} aria-label="Delete task">
        <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor"><path d="M4.5 3.1L8 6.6l3.5-3.5 1.4 1.4L9.4 8l3.5 3.5-1.4 1.4L8 9.4l-3.5 3.5-1.4-1.4L6.6 8 3.1 4.5z"/></svg>
      </button>
    {/if}
  </div>
{/snippet}

<div>
  {#if tasks.length === 0}
    <!-- Add task input -->
    <div style="display: flex; gap: 8px; margin-bottom: 16px;">
      <input
        type="text"
        class="form-input"
        style="flex: 1; font-size: 13px;"
        placeholder="Add a task..."
        bind:value={newTitle}
        onkeydown={handleAddKeydown}
      />
      <button class="btn-new" style="width: auto; padding: 7px 16px; font-size: 12px;" onclick={handleAdd} disabled={!newTitle.trim()}>
        Add
      </button>
    </div>
    <div style="text-align: center; padding: 40px 16px; color: var(--text-muted);">
      <div style="font-size: 13px;">No tasks yet</div>
      <div style="font-size: 12px; margin-top: 4px;">Add tasks to track work for this feature</div>
    </div>
  {:else}
    <!-- Progress bar (manual tasks only) -->
    {#if manualTasks.length > 0}
      <div style="margin-bottom: 16px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;">
          <span style="font-size: 11px; color: var(--text-muted);">{doneTasks.length} of {manualTasks.length} done</span>
          <span style="font-size: 11px; font-weight: 600; color: {progress === 100 ? 'var(--green)' : 'var(--text-secondary)'}; font-family: var(--font-mono);">{progress}%</span>
        </div>
        <div style="height: 3px; background: var(--bg-hover); border-radius: 2px; overflow: hidden;">
          <div style="height: 100%; width: {progress}%; background: {progress === 100 ? 'var(--green)' : 'var(--accent)'}; border-radius: 2px; transition: width 0.3s ease;"></div>
        </div>
      </div>
    {/if}

    <!-- Add task input -->
    <div style="display: flex; gap: 8px; margin-bottom: 12px;">
      <input
        type="text"
        class="form-input"
        style="flex: 1; font-size: 13px;"
        placeholder="Add a task..."
        bind:value={newTitle}
        onkeydown={handleAddKeydown}
      />
      <button class="btn-new" style="width: auto; padding: 7px 16px; font-size: 12px;" onclick={handleAdd} disabled={!newTitle.trim()}>
        Add
      </button>
    </div>

    {#if manualTasks.length > 0}
      <div style="display: flex; flex-direction: column; gap: 2px;">
        {#each openTasks as task (task.id)}
          {@render manualTaskRow(task)}
        {/each}
      </div>
      {#if doneTasks.length > 0}
        <div style="margin-top: 10px; padding-top: 10px; border-top: 1px solid var(--border);">
          <div style="font-size: 10px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 6px;">
            Completed
          </div>
          <div style="display: flex; flex-direction: column; gap: 2px;">
            {#each doneTasks as task (task.id)}
              {@render manualTaskRow(task)}
            {/each}
          </div>
        </div>
      {/if}
    {/if}

    <!-- Jira tasks section -->
    {#if jiraTasks.length > 0}
      <div style="margin-top: 16px; padding-top: 12px; border-top: 1px solid var(--border);">
        <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px;">
          <div style="display: flex; align-items: center; gap: 6px;">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
              <path d="M14.2 7.3l-5.5-5.5a1 1 0 00-1.4 0L5.8 3.3l1.8 1.8a1.2 1.2 0 011.5 1.5l1.7 1.7a1.2 1.2 0 11-.7.7L8.5 7.3v4a1.2 1.2 0 11-1-.1V7.1a1.2 1.2 0 01-.6-1.6L5.2 3.8 1.8 7.3a1 1 0 000 1.4l5.5 5.5a1 1 0 001.4 0l5.5-5.5a1 1 0 000-1.4z" fill="var(--blue, var(--accent))"/>
            </svg>
            <span style="font-size: 10px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.05em;">Jira Issues</span>
          </div>
        </div>
        <div style="display: flex; flex-direction: column; gap: 4px;">
          {#each jiraTasks as task (task.id)}
            <div class="jira-task-item">
              <div style="display: flex; align-items: center; gap: 8px; min-width: 0;">
                {#if task.external_key}
                  {#if task.external_url}
                    <a
                      href={task.external_url}
                      target="_blank"
                      rel="noopener"
                      class="jira-task-key"
                      title="Open in Jira"
                    >{task.external_key}</a>
                  {:else}
                    <span class="jira-task-key">{task.external_key}</span>
                  {/if}
                {/if}
                <span class="jira-task-title" title={task.title}>{task.title}</span>
              </div>
              {#if task.external_status}
                <span class="jira-task-status" style="color: {jiraStatusColor(task.external_status)};">
                  {task.external_status}
                </span>
              {/if}
            </div>
          {/each}
        </div>
      </div>
    {/if}
  {/if}
</div>
