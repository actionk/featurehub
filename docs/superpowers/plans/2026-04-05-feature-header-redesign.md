# Feature Header Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the 3-zone header (large title / stat cards / meta row) with a compact 2-row layout: title+chips+actions on row 1, status+tags+time+description on row 2.

**Architecture:** All changes are confined to two files — `src/app.css` (CSS classes) and `src/lib/components/FeatureDetail.svelte` (template). No new files, no backend changes. Old stat-card CSS classes are removed after the template stops using them.

**Tech Stack:** Svelte 5, TypeScript, CSS custom properties (no Tailwind in templates)

---

## Files

- Modify: `src/app.css` — lines 1374–1609 (DETAIL HEADER section)
- Modify: `src/lib/components/FeatureDetail.svelte` — lines 393–536 (header + description template)

---

### Task 1: Add new CSS classes to `app.css`

**Files:**
- Modify: `src/app.css`

- [ ] **Step 1: Replace the DETAIL HEADER CSS block**

Open `src/app.css`. Find the block starting at the comment `/* ===== DETAIL HEADER ===== */` (around line 1375) and ending just before `/* ===== TAB BAR ===== */` (around line 1610). Replace the entire block with:

```css
/* ===== DETAIL HEADER ===== */

.detail-header {
  padding: var(--space-5) var(--space-6) 0;
  flex-shrink: 0;
  background: linear-gradient(to bottom, color-mix(in srgb, var(--accent) 3%, var(--bg-primary)), var(--bg-primary));
}

.detail-header-title {
  min-width: 0;
  font-size: 20px;
  line-height: 1.2;
  font-weight: 700;
  letter-spacing: -0.03em;
  color: var(--text-primary);
  cursor: pointer;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.detail-title-last-word {
  background-image: var(--grad-primary);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.detail-title-input {
  flex: 1;
  font-size: var(--text-xl);
  font-weight: 700;
  padding: 2px 8px;
  letter-spacing: -0.03em;
}

.status-trigger {
  cursor: pointer;
  background: none;
  border: none;
  padding: 0;
  transition: opacity var(--transition-fast);
}
.status-trigger:hover { opacity: 0.85; }

/* Row 1: title (flex:1) + chips + actions */
.detail-row1 {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  margin-bottom: var(--space-2);
}

/* Row 2: status + tags + time + desc */
.detail-row2 {
  display: flex;
  align-items: center;
  gap: 7px;
  padding-bottom: var(--space-3);
  overflow: hidden;
}

/* Compact stat chips */
.detail-chip {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  position: relative;
  overflow: hidden;
  flex-shrink: 0;
}

.detail-chip::before {
  content: '';
  position: absolute;
  left: 0; top: 0; bottom: 0;
  width: 2px;
  border-radius: 1px;
}

.detail-chip--green::before { background: var(--grad-success); }
.detail-chip--accent::before { background: var(--grad-primary); }
.detail-chip--amber::before { background: var(--grad-warn); }

.detail-chip-num {
  font-size: 15px;
  font-weight: 700;
  letter-spacing: -0.03em;
  line-height: 1;
  color: var(--text-muted);
}

.detail-chip--green .detail-chip-num { color: var(--green); }
.detail-chip--accent .detail-chip-num { color: var(--accent); }
.detail-chip--amber .detail-chip-num { color: var(--amber); }

.detail-chip-lbl {
  font-size: 9.5px;
  color: var(--text-faint);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  line-height: 1;
}

.detail-chip-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--text-faint);
  flex-shrink: 0;
}

.detail-chip-dot--on {
  background: var(--green);
  box-shadow: 0 0 4px var(--green);
}

/* Vertical separator between chips and icon buttons */
.detail-chip-sep {
  width: 1px;
  height: 16px;
  background: var(--border);
  flex-shrink: 0;
  margin: 0 var(--space-1);
}

/* Row 2 inline elements */
.detail-row2-sep {
  color: var(--text-faint);
  font-size: var(--text-xs);
  flex-shrink: 0;
}

.detail-row2-time {
  font-size: var(--text-xs);
  color: var(--text-faint);
  flex-shrink: 0;
  white-space: nowrap;
}

.detail-row2-desc {
  font-size: var(--text-xs);
  color: var(--text-faint);
  flex: 1;
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  cursor: default;
}

.detail-ticket-id {
  font-size: var(--text-xs);
  color: var(--text-muted);
  font-family: var(--font-mono);
  cursor: pointer;
  transition: color var(--transition-fast);
  flex-shrink: 0;
  white-space: nowrap;
}

.detail-ticket-id:hover {
  color: var(--text-secondary);
}

.detail-ticket-id::after {
  content: " ⎘";
  opacity: 0;
  color: var(--accent);
  transition: opacity var(--transition-fast);
}

.detail-ticket-id:hover::after {
  opacity: 1;
}

/* Actions group (right side of row 1) */
.detail-header-actions {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  flex-shrink: 0;
}

.detail-description-edit {
  padding-bottom: var(--space-3);
}
```

- [ ] **Step 2: Verify no CSS syntax errors**

Run: `npm run build 2>&1 | head -40`

Expected: Build succeeds or only errors about missing template classes (old ones still in Svelte template). CSS errors would show file + line number.

---

### Task 2: Rewrite the header template in `FeatureDetail.svelte`

**Files:**
- Modify: `src/lib/components/FeatureDetail.svelte`

- [ ] **Step 1: Replace the header + description blocks in the template**

In `src/lib/components/FeatureDetail.svelte`, find this comment on approximately line 392:

```svelte
  <!-- Header — top row (title+sub | stat cards) + meta row -->
```

The section to replace runs from that comment all the way through the closing `</div>` of `detail-header` (around line 522) AND the `<!-- Description -->` block below it (lines 524–536 approximately). Replace it all with:

```svelte
  <!-- Header — row 1 (title | chips + actions) + row 2 (status + tags + time + desc) -->
  <div class="detail-header">
    <!-- Row 1: title | chips + actions -->
    <div class="detail-row1">
      {#if editingTitle}
        <input type="text" class="form-input detail-title-input"
          bind:value={titleInput} onblur={saveTitle} onkeydown={handleTitleKeydown} />
      {:else}
        <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
        <h1 class="detail-header-title"
          ondblclick={toggleEditTitle} role="button" tabindex="0"
          onkeydown={(e) => e.key === 'Enter' && toggleEditTitle()} title="Double-click to edit">
          {#if titleLastSpace >= 0}{feature.title.slice(0, titleLastSpace + 1)}<span class="detail-title-last-word">{feature.title.slice(titleLastSpace + 1)}</span>{:else}<span class="detail-title-last-word">{feature.title}</span>{/if}
        </h1>
      {/if}

      <div class="detail-header-actions">
        <div class="detail-chip detail-chip--green">
          <svg width="10" height="10" viewBox="0 0 16 16" fill="var(--green)"><path d="M13.3 4.3L6 11.6 2.7 8.3l1.4-1.4L6 8.8l5.9-5.9z"/></svg>
          <div class="detail-chip-num">{tasksDone}</div>
          <div class="detail-chip-lbl">Done</div>
        </div>

        <div class="detail-chip" class:detail-chip--accent={activeSessionCount > 0}>
          <div class="detail-chip-dot" class:detail-chip-dot--on={activeSessionCount > 0}></div>
          <div class="detail-chip-num">{activeSessionCount}</div>
          <div class="detail-chip-lbl">Agents</div>
        </div>

        {#if pendingPlanCount > 0}
          <div class="detail-chip detail-chip--amber">
            <svg width="10" height="10" viewBox="0 0 16 16" fill="var(--amber)"><path d="M8 1L10 6H15L11 9.5L12.5 14.5L8 11.5L3.5 14.5L5 9.5L1 6H6Z"/></svg>
            <div class="detail-chip-num">{pendingPlanCount}</div>
            <div class="detail-chip-lbl">Plans</div>
          </div>
        {/if}

        <div class="detail-chip-sep"></div>

        <button class="btn-ghost" onclick={() => { if (feature) { navigator.clipboard.writeText(feature.id); copiedId = true; setTimeout(() => { copiedId = false; }, 1500); } }}
          title="Copy Feature ID">
          {#if copiedId}
            <svg width="13" height="13" viewBox="0 0 16 16" fill="var(--green)"><path d="M13.3 4.3L6 11.6 2.7 8.3l1.4-1.4L6 8.8l5.9-5.9z"/></svg>
          {:else}
            <svg width="13" height="13" viewBox="0 0 16 16" fill="currentColor"><path d="M4 2a2 2 0 00-2 2v6h2V4h6V2H4zm3 3a2 2 0 00-2 2v6a2 2 0 002 2h5a2 2 0 002-2V7a2 2 0 00-2-2H7z"/></svg>
          {/if}
        </button>
        <button class="btn-ghost" onclick={async () => { if (feature) { try { const p = await getFilesDirectory(feature.id); await navigator.clipboard.writeText(p); copiedPath = true; setTimeout(() => { copiedPath = false; }, 1500); } catch {} } }}
          title="Copy feature directory path">
          {#if copiedPath}
            <svg width="13" height="13" viewBox="0 0 16 16" fill="var(--green)"><path d="M13.3 4.3L6 11.6 2.7 8.3l1.4-1.4L6 8.8l5.9-5.9z"/></svg>
          {:else}
            <svg width="13" height="13" viewBox="0 0 16 16" fill="currentColor"><path d="M1.75 1A1.75 1.75 0 000 2.75v10.5C0 14.216.784 15 1.75 15h12.5A1.75 1.75 0 0016 13.25v-8.5A1.75 1.75 0 0014.25 3H7.5a.25.25 0 01-.2-.1l-.9-1.2c-.33-.44-.85-.7-1.4-.7H1.75z"/></svg>
          {/if}
        </button>
        <button class="btn-ghost" onclick={toggleEditTitle} title="Edit">
          <svg width="13" height="13" viewBox="0 0 16 16" fill="currentColor"><path d="M12.1 1.3a1 1 0 011.4 0l1.2 1.2a1 1 0 010 1.4L5.8 12.8l-3.5.9.9-3.5z"/></svg>
        </button>
        <button class="btn-ghost btn-ghost--danger" onclick={() => (showDeleteConfirm = true)}
          title="Delete">
          <svg width="13" height="13" viewBox="0 0 16 16" fill="currentColor"><path d="M5 2V1h6v1h4v1H1V2h4zm1 3v8h1V5H6zm3 0v8h1V5H9zM2 4l1 11h10l1-11H2z"/></svg>
        </button>
      </div>
    </div>

    <!-- Row 2: status + tags + time + desc (or description edit textarea) -->
    {#if editingDescription}
      <div class="detail-description-edit">
        <textarea class="form-input" style="resize: vertical; font-size: 12.5px; line-height: 1.5; min-height: 48px;" rows="3"
          bind:value={descriptionInput} onblur={saveDescription} onkeydown={handleDescriptionKeydown}
          placeholder="Feature description..."></textarea>
      </div>
    {:else}
      <div class="detail-row2">
        <div class="status-dropdown-wrapper" style="position: relative; flex-shrink: 0;">
          <button class="status-trigger"
            onclick={() => (showStatusDropdown = !showStatusDropdown)}>
            <StatusBadge status={feature.status} />
          </button>
          {#if showStatusDropdown}
            <div class="dropdown" style="left: 0;">
              {#each statuses as s}
                <button class="dropdown-item" onclick={() => setStatus(s.value)}>
                  <StatusBadge status={s.value} />
                </button>
              {/each}
            </div>
          {/if}
        </div>

        {#each featureTags as tag (tag.id)}
          <TagBadge {tag} removable onRemove={handleRemoveTag} />
        {/each}
        <div class="tag-picker-wrapper" style="position: relative;">
          <button class="btn-add" onclick={async () => { showTagPicker = !showTagPicker; if (showTagPicker) { await tick(); tagInputEl?.focus(); } }}>+ Tag</button>
          {#if showTagPicker}
            <div class="dropdown">
              <div style="padding: 4px 6px; border-bottom: 1px solid var(--border);">
                <input
                  type="text"
                  class="form-input"
                  style="font-size: 12px; padding: 4px 8px;"
                  placeholder="New tag name..."
                  bind:this={tagInputEl}
                  bind:value={newTagName}
                  onkeydown={(e) => { if (e.key === 'Enter') handleCreateTag(); if (e.key === 'Escape') { showTagPicker = false; newTagName = ''; } }}
                />
              </div>
              {#each availableTags as tag (tag.id)}
                <button class="dropdown-item"
                  onclick={() => { handleToggleTag(tag.id); showTagPicker = false; }}>
                  <TagBadge {tag} />
                </button>
              {/each}
            </div>
          {/if}
        </div>

        {#if feature.ticket_id}
          <span class="detail-row2-sep">·</span>
          <span class="detail-ticket-id">{feature.ticket_id}</span>
        {/if}

        <span class="detail-row2-sep">·</span>
        <span class="detail-row2-time">{formatRelativeTime(feature.updated_at)}</span>

        {#if feature.description}
          <span class="detail-row2-sep">·</span>
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <span class="detail-row2-desc" ondblclick={startEditDescription} title="Double-click to edit">{feature.description}</span>
        {/if}
      </div>
    {/if}
  </div>
```

- [ ] **Step 2: Verify the Svelte template compiles**

Run: `npm run build 2>&1 | head -60`

Expected: Clean build. No errors about missing classes or undefined variables. Warnings about unused CSS selectors (from old classes still in CSS being purged) are fine.

- [ ] **Step 3: Start dev server and manually verify**

Run: `npm run tauri dev`

Check in the app:
- Feature header shows 2 rows (title + chips + actions on top; status + tags + time + desc below)
- Chips have left accent bar (green for Done, accent color when agents > 0, amber for Plans)
- Description truncates with ellipsis on one line
- Double-click title → edits inline ✓
- Double-click description → textarea appears (replaces row 2) ✓
- Status badge opens dropdown ✓
- + Tag opens picker ✓
- Copy ID / Copy Path / Edit / Delete buttons work ✓
- Ticket ID shows when present, with hover clipboard icon ✓

- [ ] **Step 4: Commit**

```bash
git add src/app.css src/lib/components/FeatureDetail.svelte
git commit -m "feat: compact 2-row feature header with inline stat chips"
```
