# Phase 8: Icon Rail + UI Polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a 52px icon rail to the left of the sidebar and polish the feature header, tab bar, bento cards, and sidebar items to match the design prototype.

**Architecture:** All changes are CSS and Svelte template modifications only — no new components, no Rust changes. The icon rail is a new `<div class="icon-rail">` prepended to `.app-shell` in `App.svelte`. Header restructure splits the existing 2-row layout into a top row (title + big stat cards) and a meta row (status + tags + actions). All other changes are CSS class updates and small HTML additions.

**Tech Stack:** Svelte 5, TypeScript, CSS custom properties (`src/app.css`), design tokens in `:root`

---

## File Map

| File | Changes |
|------|---------|
| `src/App.svelte` | Add icon rail HTML, fix resize offset |
| `src/lib/components/Sidebar.svelte` | Add mini progress bar to feature items |
| `src/lib/components/FeatureDetail.svelte` | Restructure header HTML (top row + meta row + big stat cards) |
| `src/lib/modules/ai/AiPanel.svelte` | Add Tasks card class, dot-field + glow to session card |
| `src/app.css` | CSS for all of the above |

## Key Design Tokens (all defined in `:root` in `src/app.css`)

- Gradients: `--grad-primary` (blue-violet), `--grad-success` (green), `--grad-warn` (amber)
- Backgrounds: `--bg-secondary` (sidebar bg), `--bg-card`, `--bg-raised`, `--bg-hover`
- Borders: `--border`, `--border-strong`
- Spacing: `--space-1` (4px) … `--space-6` (24px)
- Radius: `--radius-sm` (4px), `--radius-md` (6px), `--radius-lg` (12px), `--radius-full`
- Text sizes: `--text-xs` (11px), `--text-sm` (12px), `--text-base` (13px), `--text-xl` (18px)
- Colors: `--accent`, `--green`, `--amber`, `--text-primary`, `--text-secondary`, `--text-muted`, `--text-faint`
- Font: `--font-ui` (Space Grotesk), `--font-mono` (JetBrains Mono)
- Transitions: `--transition-fast`

---

### Task 1: Icon rail

**Files:**
- Modify: `src/App.svelte` (template ~line 395, resize fn ~line 84)
- Modify: `src/app.css` (after `.app-shell` section, ~line 168)

- [ ] **Step 1: Create a feature branch**

```bash
cd /d/LittleBrushGames/FeatureHub
git checkout -b feat/phase8-icon-rail-polish
```

- [ ] **Step 2: Add icon rail HTML to `src/App.svelte`**

Add `const ICON_RAIL_WIDTH = 52;` alongside the other sidebar constants (near `SIDEBAR_MIN = 200`):

```typescript
const SIDEBAR_MIN = 200;
const SIDEBAR_MAX = 500;
const SIDEBAR_DEFAULT = 272;
const ICON_RAIL_WIDTH = 52;
```

Fix the resize offset in `onResizeStart` — change `ev.clientX` to subtract the icon rail width:

```typescript
function onResizeStart(e: MouseEvent) {
  e.preventDefault();
  isResizing = true;
  const onMove = (ev: MouseEvent) => {
    const w = Math.min(SIDEBAR_MAX, Math.max(SIDEBAR_MIN, ev.clientX - ICON_RAIL_WIDTH));
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
```

In the template, insert the icon rail `<div>` as the FIRST child of `.app-shell`, before `<Sidebar`:

```svelte
<div class="app-shell" class:resizing={isResizing} class:storage-switching={storageSwitching}>
  <!-- Icon rail -->
  <div class="icon-rail">
    <div class="icon-rail-logo">FH</div>

    <button class="icon-rail-btn icon-rail-btn--on" data-tip="Features"
      onclick={() => { showKnowledge = false; }}>
      <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M1 2.5A1.5 1.5 0 012.5 1h3A1.5 1.5 0 017 2.5v3A1.5 1.5 0 015.5 7h-3A1.5 1.5 0 011 5.5v-3zm8 0A1.5 1.5 0 0110.5 1h3A1.5 1.5 0 0115 2.5v3A1.5 1.5 0 0113.5 7h-3A1.5 1.5 0 019 5.5v-3zm-8 8A1.5 1.5 0 012.5 9h3A1.5 1.5 0 017 10.5v3A1.5 1.5 0 015.5 15h-3A1.5 1.5 0 011 13.5v-3zm8 0A1.5 1.5 0 0110.5 9h3A1.5 1.5 0 0115 10.5v3A1.5 1.5 0 0113.5 15h-3A1.5 1.5 0 019 13.5v-3z"/>
      </svg>
    </button>

    <button class="icon-rail-btn {showKnowledge ? 'icon-rail-btn--on' : ''}" data-tip="Knowledge"
      onclick={() => { showKnowledge = true; }}>
      <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M1 2.5A1.5 1.5 0 012.5 1h11A1.5 1.5 0 0115 2.5v9A1.5 1.5 0 0113.5 13H9l.5 1H11a.5.5 0 010 1H5a.5.5 0 010-1h1.5l.5-1H2.5A1.5 1.5 0 011 11.5v-9zm13 9V2.5a.5.5 0 00-.5-.5h-11a.5.5 0 00-.5.5v9a.5.5 0 00.5.5h11a.5.5 0 00.5-.5z"/>
      </svg>
    </button>

    <div class="icon-rail-sep"></div>
    <div class="icon-rail-spacer"></div>

    <button class="icon-rail-btn" data-tip="Search  Ctrl+T"
      onclick={() => (showSearch = true)}>
      <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M11.742 10.344a6.5 6.5 0 10-1.397 1.398h-.001c.03.04.062.078.098.115l3.85 3.85a1 1 0 001.415-1.414l-3.85-3.85a1.007 1.007 0 00-.115-.099zM12 6.5a5.5 5.5 0 11-11 0 5.5 5.5 0 0111 0z"/>
      </svg>
    </button>

    <button class="icon-rail-btn" data-tip="Settings"
      onclick={() => (showSettings = true)}>
      <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M8 4.754a3.246 3.246 0 100 6.492 3.246 3.246 0 000-6.492zM5.754 8a2.246 2.246 0 114.492 0 2.246 2.246 0 01-4.492 0z"/>
        <path d="M9.796 1.343c-.527-1.79-3.065-1.79-3.592 0l-.094.319a.873.873 0 01-1.255.52l-.292-.16c-1.64-.892-3.433.902-2.54 2.541l.159.292a.873.873 0 01-.52 1.255l-.319.094c-1.79.527-1.79 3.065 0 3.592l.319.094a.873.873 0 01.52 1.255l-.16.292c-.892 1.64.901 3.434 2.541 2.54l.292-.159a.873.873 0 011.255.52l.094.319c.527 1.79 3.065 1.79 3.592 0l.094-.319a.873.873 0 011.255-.52l.292.16c1.64.892 3.433-.902 2.54-2.541l-.159-.292a.873.873 0 01.52-1.255l.319-.094c1.79-.527 1.79-3.065 0-3.592l-.319-.094a.873.873 0 01-.52-1.255l.16-.292c.892-1.64-.901-3.433-2.541-2.54l-.292.159a.873.873 0 01-1.255-.52l-.094-.319z"/>
      </svg>
    </button>
  </div>

  <Sidebar ...
```

- [ ] **Step 3: Add icon rail CSS to `src/app.css`**

Add after the `.app-shell` block (around line 168, before `/* ===== SIDEBAR =====`):

```css
/* ===== ICON RAIL ===== */

.icon-rail {
  width: 52px;
  min-width: 52px;
  height: 100%;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px 0 10px;
  gap: 2px;
  user-select: none;
  flex-shrink: 0;
  z-index: 10;
}

.icon-rail-logo {
  width: 32px;
  height: 32px;
  background: var(--grad-primary);
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 700;
  color: #fff;
  letter-spacing: -0.04em;
  margin-bottom: 10px;
  flex-shrink: 0;
  box-shadow: 0 2px 12px rgba(77, 124, 255, 0.3);
}

.icon-rail-btn {
  width: 36px;
  height: 36px;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background var(--transition-fast), color var(--transition-fast);
  color: var(--text-muted);
  border: 1px solid transparent;
  background: none;
  position: relative;
  flex-shrink: 0;
}

/* Tooltip shown on hover via data-tip attribute */
.icon-rail-btn::after {
  content: attr(data-tip);
  position: absolute;
  left: calc(100% + 10px);
  top: 50%;
  transform: translateY(-50%) scale(0.92);
  background: var(--bg-raised);
  border: 1px solid var(--border-strong);
  color: var(--text-primary);
  font-family: var(--font-ui);
  font-size: 11px;
  font-weight: 500;
  padding: 4px 9px;
  border-radius: var(--radius-sm);
  white-space: nowrap;
  pointer-events: none;
  opacity: 0;
  transition: opacity var(--transition-fast), transform var(--transition-fast);
  z-index: 200;
}

.icon-rail-btn:hover::after {
  opacity: 1;
  transform: translateY(-50%) scale(1);
}

.icon-rail-btn:hover {
  background: var(--bg-hover);
  color: var(--text-secondary);
}

.icon-rail-btn--on {
  background: linear-gradient(135deg, rgba(77, 124, 255, 0.18), rgba(139, 92, 246, 0.11));
  color: var(--accent);
  border-color: rgba(77, 124, 255, 0.22);
}

.icon-rail-btn--on:hover {
  background: linear-gradient(135deg, rgba(77, 124, 255, 0.25), rgba(139, 92, 246, 0.16));
  color: var(--accent);
}

.icon-rail-sep {
  width: 24px;
  height: 1px;
  background: var(--border);
  margin: 4px 0;
  flex-shrink: 0;
}

.icon-rail-spacer {
  flex: 1;
}
```

- [ ] **Step 4: Run tests to confirm no regressions**

```bash
cd /d/LittleBrushGames/FeatureHub && npm run test
```

Expected: all 38 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/App.svelte src/app.css
git commit -m "feat: add icon rail to app shell"
```

---

### Task 2: Header layout — big stat cards, status to meta-row

**Files:**
- Modify: `src/lib/components/FeatureDetail.svelte` (~lines 393–498)
- Modify: `src/app.css` (~lines 1120–1285)

The current header has two rows:
- **Row 1** (`detail-header-row1`): `[title h1]` `[status dropdown, margin-left:auto]` `[stat chips (small pills)]`
- **Row 2** (`detail-header-row2`): `[ticket · updated · tags · actions]`

The new structure:
- **Top** (`detail-header-top`, flex + space-between): `[detail-header-left: title + sub (ticket · updated)]` | `[detail-stat-chips: 3 big stat cards]`
- **Meta row** (`detail-meta-row`): `[status dropdown]` `[tags + tag-picker]` `[detail-header-actions]`

- [ ] **Step 1: Replace header HTML in `src/lib/components/FeatureDetail.svelte`**

Find the `{:else if feature}` block that opens with `<div class="detail-header">`. Replace the entire block from `<div class="detail-header">` through `</div>` closing that element (up to the `<!-- Description -->` comment, NOT including it). The replacement is:

```svelte
  <!-- Header -->
  <div class="detail-header">
    <div class="detail-header-top">
      <div class="detail-header-left">
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
        <div class="detail-header-sub">
          {#if feature.ticket_id}
            <span class="detail-ticket-id">{feature.ticket_id}</span>
            <span class="detail-separator">·</span>
          {/if}
          <span class="detail-updated-at">{formatRelativeTime(feature.updated_at)}</span>
        </div>
      </div>

      <div class="detail-stat-chips">
        <div class="detail-stat-card detail-stat-card--green">
          <div class="detail-stat-card-icon" style="background: rgba(52,211,153,0.07); border: 1px solid rgba(52,211,153,0.12);">
            <svg width="12" height="12" viewBox="0 0 16 16" fill="var(--green)"><path d="M13.3 4.3L6 11.6 2.7 8.3l1.4-1.4L6 8.8l5.9-5.9z"/></svg>
          </div>
          <div>
            <div class="detail-stat-card-num">{tasksDone}</div>
            <div class="detail-stat-card-lbl">Done</div>
          </div>
        </div>

        <div class="detail-stat-card {activeSessionCount > 0 ? 'detail-stat-card--accent' : ''}">
          <div class="detail-stat-card-icon" style="background: {activeSessionCount > 0 ? 'rgba(77,124,255,0.07)' : 'rgba(255,255,255,0.04)'}; border: 1px solid {activeSessionCount > 0 ? 'rgba(77,124,255,0.14)' : 'rgba(255,255,255,0.07)'};  color: {activeSessionCount > 0 ? 'var(--accent)' : 'var(--text-muted)'};">
            <span class="detail-stat-live-dot {activeSessionCount > 0 ? 'detail-stat-live-dot--on' : ''}"></span>
          </div>
          <div>
            <div class="detail-stat-card-num">{activeSessionCount}</div>
            <div class="detail-stat-card-lbl">Agents</div>
          </div>
        </div>

        {#if pendingPlanCount > 0}
          <div class="detail-stat-card detail-stat-card--amber">
            <div class="detail-stat-card-icon" style="background: rgba(251,191,36,0.07); border: 1px solid rgba(251,191,36,0.12);">
              <svg width="12" height="12" viewBox="0 0 16 16" fill="var(--amber)"><path d="M8 1L10 6H15L11 9.5L12.5 14.5L8 11.5L3.5 14.5L5 9.5L1 6H6Z"/></svg>
            </div>
            <div>
              <div class="detail-stat-card-num">{pendingPlanCount}</div>
              <div class="detail-stat-card-lbl">Plans</div>
            </div>
          </div>
        {/if}
      </div>
    </div>

    <div class="detail-meta-row">
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

      <div class="detail-header-actions">
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
  </div>
```

- [ ] **Step 2: Update header CSS in `src/app.css`**

Find the `.detail-header` section (around line 1120). Replace the existing `.detail-header-row1`, `.detail-header-row2`, `.detail-stat-chips`, `.detail-stat-chip*` rules and add the new ones. Keep `.detail-header`, `.detail-header-title`, `.detail-title-last-word`, `.detail-ticket-id`, `.detail-separator`, `.detail-updated-at`, `.detail-header-actions`, `.detail-stat-live-dot*` as-is.

Add/replace with:

```css
/* detail-header-top: title+sub LEFT, stat chips RIGHT */
.detail-header-top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
}

.detail-header-left {
  min-width: 0;
  flex: 1;
}

.detail-header-sub {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-top: var(--space-1);
}

/* Stat chips container (right side of top row) */
.detail-stat-chips {
  display: flex;
  gap: var(--space-2);
  flex-shrink: 0;
  align-items: flex-start;
}

/* Big stat card */
.detail-stat-card {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  min-width: 68px;
  position: relative;
  overflow: hidden;
  transition: transform var(--transition-fast), border-color var(--transition-fast);
}

/* Thin gradient top-edge accent line */
.detail-stat-card::before {
  content: '';
  position: absolute;
  top: 0; left: 0; right: 0;
  height: 1.5px;
  border-radius: 1px;
}
.detail-stat-card--green::before { background: var(--grad-success); }
.detail-stat-card--accent::before { background: var(--grad-primary); }
.detail-stat-card--amber::before { background: var(--grad-warn); }

.detail-stat-card:hover {
  transform: translateY(-1px);
  border-color: var(--border-strong);
}

.detail-stat-card-icon {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.detail-stat-card-num {
  font-size: var(--text-xl);
  font-weight: 700;
  letter-spacing: -0.04em;
  line-height: 1;
  color: var(--text-primary);
}
.detail-stat-card--green .detail-stat-card-num { color: var(--green); }
.detail-stat-card--accent .detail-stat-card-num { color: var(--accent); }
.detail-stat-card--amber .detail-stat-card-num { color: var(--amber); }

.detail-stat-card-lbl {
  font-size: 10px;
  color: var(--text-muted);
  margin-top: 2px;
}

/* Meta row: status + tags + actions */
.detail-meta-row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-top: var(--space-3);
  flex-wrap: wrap;
}
```

Also update `.detail-description` to add max-width and 2-line clamp:

```css
.detail-description {
  padding: 0 var(--space-6) var(--space-2);
  font-size: var(--text-sm);
  line-height: 1.55;
  color: var(--text-muted);
  max-width: 560px;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  word-break: break-word;
  cursor: default;
}
```

Remove (delete) the old CSS rules: `.detail-header-row1`, `.detail-header-row2`, `.detail-stat-chip`, `.detail-stat-chip--green`, `.detail-stat-chip--accent`, `.detail-stat-chip--amber`.

- [ ] **Step 3: Run tests**

```bash
cd /d/LittleBrushGames/FeatureHub && npm run test
```

Expected: 38 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/FeatureDetail.svelte src/app.css
git commit -m "feat: header big stat cards, status to meta-row, description clamp"
```

---

### Task 3: Tab bar pill style

**Files:**
- Modify: `src/app.css` (`.tab-btn` and `.tab-btn--active` blocks, ~lines 1307–1387)

The current tab bar uses an underline indicator (`border-bottom: 2px solid`) with gradient text on the active tab. The prototype uses pill-shaped tabs with a gradient background on the active tab. This is a CSS-only change.

- [ ] **Step 1: Replace tab button CSS in `src/app.css`**

Find and replace the `.tab-btn` block and all related active/count rules. The tab-bar itself keeps its bottom border. Replace `.tab-btn` through `.tab-btn--active .tab-count--active` with:

```css
.tab-btn {
  padding: 5px var(--space-3);
  font-size: var(--text-sm);
  font-weight: 500;
  color: var(--text-muted);
  cursor: pointer;
  border: 1px solid transparent;
  background: transparent;
  transition: all var(--transition-fast);
  user-select: none;
  font-family: inherit;
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  position: relative;
  border-radius: var(--radius-full);
  margin-bottom: 0;
}

.tab-btn:hover {
  background: var(--bg-hover);
  color: var(--text-secondary);
}

.tab-btn--active {
  background: linear-gradient(135deg, rgba(77, 124, 255, 0.16), rgba(139, 92, 246, 0.10));
  color: var(--accent);
  border-color: rgba(77, 124, 255, 0.22);
  font-weight: 600;
}

.tab-btn--active:hover {
  background: linear-gradient(135deg, rgba(77, 124, 255, 0.22), rgba(139, 92, 246, 0.14));
}

.tab-count {
  background: var(--accent-dim);
  color: var(--accent);
  padding: 0 5px;
  border-radius: var(--radius-full);
  font-size: 10px;
  font-family: var(--font-mono);
  line-height: 16px;
  min-width: 16px;
  text-align: center;
}

.tab-btn--active .tab-count {
  background: rgba(77, 124, 255, 0.25);
  color: var(--accent);
}

.tab-count--active {
  background: var(--green-dim);
  color: var(--green);
}

.tab-btn--active .tab-count--active {
  background: rgba(52, 211, 153, 0.2);
  color: var(--green);
}
```

Also update `.tab-bar` to add vertical padding for the pill tabs:

```css
.tab-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-2) var(--space-6);
  flex-shrink: 0;
  border-bottom: 1px solid var(--border);
}
```

And update `.tab-bar-tabs`:

```css
.tab-bar-tabs {
  display: flex;
  gap: var(--space-1);
}
```

- [ ] **Step 2: Run tests**

```bash
cd /d/LittleBrushGames/FeatureHub && npm run test
```

Expected: 38 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/app.css
git commit -m "style: tab bar pill style matching prototype"
```

---

### Task 4: Bento card gradient titles + Tasks card tint

**Files:**
- Modify: `src/app.css` (`.bento-title` and new `.bento-card--tasks`, ~lines 1645–1660)
- Modify: `src/lib/modules/ai/AiPanel.svelte` (~line 379, Tasks card div)

The prototype shows card titles with gradient text (`Tasks`, `Agent History`, `Links`) at `font-size: 13.5px, font-weight: 700`. The Tasks card has a subtle blue tint background. The warning plan card already has gradient text from `.bento-card--warn .bento-title`.

- [ ] **Step 1: Update `.bento-title` in `src/app.css`**

Find the `.bento-title` rule (~line 1645) and replace:

```css
.bento-title {
  font-size: var(--text-sm);
  font-weight: 700;
  letter-spacing: -0.01em;
  background-image: var(--grad-primary);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  flex: 1;
}
```

(Remove `color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.08em;` from the original.)

- [ ] **Step 2: Add `.bento-card--tasks` CSS after `.bento-card--full`**

Find the `.bento-card--full` rule (~line 1634) and add after it:

```css
.bento-card--tasks {
  background: linear-gradient(155deg, rgba(77, 124, 255, 0.04) 0%, var(--bg-card) 40%);
}

.bento-card--tasks:hover {
  border-color: rgba(77, 124, 255, 0.18);
}
```

- [ ] **Step 3: Apply `.bento-card--tasks` in `src/lib/modules/ai/AiPanel.svelte`**

Find the Tasks card div (~line 379):
```svelte
<div class="bento-card bento-card--span2" style="grid-area: tasks;">
```

Replace with:
```svelte
<div class="bento-card bento-card--span2 bento-card--tasks" style="grid-area: tasks;">
```

- [ ] **Step 4: Run tests**

```bash
cd /d/LittleBrushGames/FeatureHub && npm run test
```

Expected: 38 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/app.css src/lib/modules/ai/AiPanel.svelte
git commit -m "style: bento card gradient titles and tasks card blue tint"
```

---

### Task 5: Active session card dot-field + glow animation

**Files:**
- Modify: `src/lib/modules/ai/AiPanel.svelte` (~line 410, active session card)
- Modify: `src/app.css` (after `.bento-card--live` block, ~line 1755)

The prototype's active session card has a pulsing radial glow and an animated dot-field pattern background. These are decorative CSS-only elements inside `.bento-card--live`.

- [ ] **Step 1: Add decorative elements to session card in `src/lib/modules/ai/AiPanel.svelte`**

Find the session card div (~line 410):
```svelte
        <!-- Active Session (col 2, row 1) -->
        <div class="bento-card {activeSessions.length > 0 ? 'bento-card--live' : ''}" style="grid-area: session;">
          <div class="bento-header">
```

Replace with:
```svelte
        <!-- Active Session (col 2, row 1) -->
        <div class="bento-card {activeSessions.length > 0 ? 'bento-card--live' : ''}" style="grid-area: session;">
          {#if activeSessions.length > 0}
            <div class="bento-card-glow"></div>
            <div class="bento-card-dot-field"></div>
          {/if}
          <div class="bento-header" style="position: relative; z-index: 1;">
```

Also wrap the remaining session card content (after `bento-header`) in `style="position: relative; z-index: 1;"` by adding `style="position: relative; z-index: 1;"` to the existing `{#if activeSessions.length > 0}` content wrapper. Since the content is not wrapped in a div currently, add a wrapper:

After `</div>` (closing `bento-header`) and before `{#if activeSessions.length > 0}` (the data-showing branch), change the structure so the session content is inside a `<div style="position: relative; z-index: 1; display: flex; flex-direction: column; flex: 1;">`.

The full new session card block:

```svelte
        <!-- Active Session (col 2, row 1) -->
        <div class="bento-card {activeSessions.length > 0 ? 'bento-card--live' : ''}" style="grid-area: session;">
          {#if activeSessions.length > 0}
            <div class="bento-card-glow"></div>
            <div class="bento-card-dot-field"></div>
          {/if}
          <div style="position: relative; z-index: 1; display: flex; flex-direction: column; height: 100%;">
            <div class="bento-header">
              <span class="bento-title">Active Session</span>
              {#if activeSessions.length > 0}
                <span class="bento-live-pill">
                  <span class="bento-live-ring"></span>
                  ACTIVE NOW
                </span>
              {/if}
            </div>
            {#if activeSessions.length > 0}
              {@const s = activeSessions[0]}
              <div class="bento-session-title">{s.title ?? 'Running session'}</div>
              {#if s.summary}
                <div class="bento-session-summary">{s.summary}</div>
              {/if}
              <div class="bento-session-footer">
                {#if sessionElapsed}
                  <span class="bento-session-timer">
                    <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor"><path d="M8 1a7 7 0 100 14A7 7 0 008 1zm0 1.5a5.5 5.5 0 110 11 5.5 5.5 0 010-11zM8 4v4.25l2.75 1.75-.75 1.25L7 9V4h1z"/></svg>
                    {sessionElapsed}
                  </span>
                {/if}
                <button class="bento-session-open" onclick={() => handleResumeSession(s)}>
                  Open Terminal →
                </button>
              </div>
            {:else}
              <div class="bento-empty">
                <span class="bento-empty-text">No active session</span>
                <button class="bento-start-btn" onclick={handleStartSession} disabled={launching}>
                  {launching ? 'Starting…' : '▶ Start Session'}
                </button>
              </div>
            {/if}
          </div>
        </div>
```

- [ ] **Step 2: Add dot-field + glow CSS to `src/app.css`**

Find `.bento-card--live` (~line 1746). First, add `position: relative; overflow: hidden;` to `.bento-card--live`:

```css
.bento-card--live {
  background: linear-gradient(135deg, rgba(13, 15, 24, 0.95), rgba(18, 20, 31, 0.98));
  border-color: rgba(77, 124, 255, 0.3);
  box-shadow: 0 0 0 1px rgba(77, 124, 255, 0.08), inset 0 0 40px rgba(77, 124, 255, 0.03);
  position: relative;
  overflow: hidden;
}
```

Then add the new animation rules after `.bento-card--live:hover`:

```css
.bento-card-glow {
  position: absolute;
  inset: -40px;
  background: radial-gradient(ellipse at 40% 40%, rgba(77, 124, 255, 0.06) 0%, transparent 60%);
  pointer-events: none;
  animation: bento-glow-pulse 4s ease-in-out infinite;
}

@keyframes bento-glow-pulse {
  0%, 100% { opacity: 0.6; }
  50% { opacity: 1; }
}

.bento-card-dot-field {
  position: absolute;
  inset: 0;
  background-image: radial-gradient(circle, rgba(77, 124, 255, 0.18) 1px, transparent 1px);
  background-size: 20px 20px;
  mask-image: radial-gradient(ellipse 90% 90% at 50% 50%, black 0%, transparent 100%);
  -webkit-mask-image: radial-gradient(ellipse 90% 90% at 50% 50%, black 0%, transparent 100%);
  pointer-events: none;
  opacity: 0.15;
  animation: bento-dot-scroll 14s linear infinite;
}

@keyframes bento-dot-scroll {
  0%   { background-position: 0 0; }
  100% { background-position: 20px 20px; }
}
```

- [ ] **Step 3: Run tests**

```bash
cd /d/LittleBrushGames/FeatureHub && npm run test
```

Expected: 38 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/lib/modules/ai/AiPanel.svelte src/app.css
git commit -m "style: active session card dot-field and glow animation"
```

---

### Task 6: Sidebar mini progress bars

**Files:**
- Modify: `src/lib/components/Sidebar.svelte` (~line 920, feature-item-compact block)
- Modify: `src/app.css` (after `.feature-item-status-dot`, ~line 449)

`FeatureSummary` (the type used in the sidebar feature list) has `task_count_total?: number` and `task_count_done?: number`. Show a small `X/Y` progress bar on feature items that have tasks.

- [ ] **Step 1: Add progress bar HTML to `src/lib/components/Sidebar.svelte`**

Find the `.feature-item-compact` block (~line 918). Add the PMB after `<span class="feature-item-title">`:

```svelte
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
```

- [ ] **Step 2: Add PMB CSS to `src/app.css`**

Add after `.feature-item-status-dot` (~line 449):

```css
.feature-item-pmb {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.feature-item-pmb-track {
  width: 26px;
  height: 3px;
  background: rgba(255, 255, 255, 0.07);
  border-radius: 2px;
  overflow: hidden;
}

.feature-item-pmb-fill {
  height: 100%;
  background: var(--grad-primary);
  border-radius: 2px;
  min-width: 2px;
}

.feature-item-pmb-label {
  font-size: 9px;
  font-family: var(--font-mono);
  color: var(--text-faint);
}
```

- [ ] **Step 3: Run tests**

```bash
cd /d/LittleBrushGames/FeatureHub && npm run test
```

Expected: 38 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/Sidebar.svelte src/app.css
git commit -m "feat: sidebar feature items show task progress bar"
```

---

### Task 7: Final verification and merge

**Files:** None modified.

- [ ] **Step 1: Run frontend tests**

```bash
cd /d/LittleBrushGames/FeatureHub && npm run test
```

Expected: 38 tests pass across 6 test files.

- [ ] **Step 2: Run Rust check**

```bash
cd /d/LittleBrushGames/FeatureHub/src-tauri && cargo check
```

Expected: no errors.

- [ ] **Step 3: Merge to master**

```bash
cd /d/LittleBrushGames/FeatureHub
git checkout master
git merge feat/phase8-icon-rail-polish --no-ff -m "feat: phase 8 — icon rail, header polish, tab pills, bento effects, sidebar progress"
```

- [ ] **Step 4: Confirm merge**

```bash
git log --oneline -3
```

Expected: merge commit is at HEAD.

---

## Self-Review

**Spec coverage:**
- ✅ Icon rail (60px) with logo, nav icons, tooltips — Task 1
- ✅ Resize offset fix — Task 1
- ✅ Header top row (title+sub LEFT, stat cards RIGHT) — Task 2
- ✅ Status badge moves to meta-row — Task 2
- ✅ Tags move to meta-row — Task 2
- ✅ Description 2-line clamp + max-width — Task 2
- ✅ Tab bar pill style — Task 3
- ✅ Bento title gradient text — Task 4
- ✅ Tasks card blue tint — Task 4
- ✅ Session card dot-field + glow — Task 5
- ✅ Sidebar mini progress bar — Task 6

**Placeholder scan:** No TBDs or vague steps. All code is complete.

**Type consistency:**
- `feature.task_count_total` and `feature.task_count_done` — used with `?? 0` fallback. Both are `number | undefined` on `FeatureSummary`. Correct.
- `titleLastSpace` — derived in script, used in template. Already exists from Phase 7.
- `tasksDone`, `activeSessionCount`, `pendingPlanCount` — already defined from Phase 7 in FeatureDetail.svelte script.
