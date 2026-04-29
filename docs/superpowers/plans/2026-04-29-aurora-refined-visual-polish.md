# Aurora Refined Visual Polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refine the current Aurora Glass UI so it feels calmer, more consistent, and easier to scan.

**Architecture:** Keep the app structure unchanged. Make a CSS-first pass in `src/app.css`, using existing Aurora primitives and final scoped overrides for legacy selectors that still drive major shell areas.

**Tech Stack:** Svelte 5, TypeScript, Vite, Tauri, CSS custom properties.

---

## Files

- Modify: `src/app.css`
- Reference: `src/lib/components/Sidebar.svelte`
- Reference: `src/lib/components/FeatureDetail.svelte`
- Reference: `src/lib/modules/board/BoardPanel.svelte`
- Reference: `src/lib/components/SessionsPanel.svelte`
- Verify: `npm run build`

## Tasks

### Task 1: Refine Global Tokens

- [ ] Update Aurora color, surface, border, radius, shadow, and typography tokens in `src/app.css`.
- [ ] Remove negative letter spacing from shared token usage.
- [ ] Keep the palette dark and Aurora-like, but lower glow intensity and reduce purple dominance.

### Task 2: Normalize Shared Primitives

- [ ] Update `.glass-panel`, `.glass-panel--soft`, `.btn`, `.tab-bar`, `.tab`, `.modal`, and `.dropdown__panel`.
- [ ] Make hover states quieter and make active states clearer.
- [ ] Preserve existing class names so Svelte components do not need API changes.

### Task 3: Polish Main Shell Areas

- [ ] Add or adjust CSS for `.icon-rail`, `.sidebar`, `.feature-unit`, `.detail-header`, `.tab-panel`, `.board-panel`, `.board-column`, `.board-card`, `.sessions-panel`, `.session-row`, `.sessions-card`, and empty states.
- [ ] Keep the current information density; do not redesign workflows.
- [ ] Ensure feature rows with active sessions and selected states remain visibly distinct.

### Task 4: Verify

- [ ] Run `npm run build`.
- [ ] Review `git diff -- src/app.css docs/superpowers/specs/2026-04-29-aurora-refined-visual-polish-design.md docs/superpowers/plans/2026-04-29-aurora-refined-visual-polish.md`.
- [ ] Confirm unrelated user changes are not modified.

