# Tier 1 Correctness Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix three confirmed correctness bugs: a startup crash in path migration, a phantom TypeScript field that doesn't exist in the database, and a dead Rust module that creates false confidence in branch-tracking functionality.

**Architecture:** All three fixes are surgical — no new files, no new dependencies, no behaviour changes visible to the user. Each fix is independent and can be committed separately.

**Tech Stack:** Rust (rusqlite, Tauri 2), TypeScript, Svelte 5

---

## File Map

| File | Change |
|------|--------|
| `src-tauri/src/db/mod.rs` | Replace two `unwrap_or_else(|_| panic!())` with graceful fallbacks |
| `src/lib/api/types.ts` | Remove phantom `sort_order: number` from `Link` interface |
| `src-tauri/src/db/branches.rs` | Delete (dead code — never imported, never called) |

---

## Task 1: Fix startup panic in `migrate_to_relative_paths`

**Files:**
- Modify: `src-tauri/src/db/mod.rs` (lines ~347–360 and ~378–392)

**Context:** `migrate_to_relative_paths` runs at app startup to convert any absolute DB paths to relative ones. It contains two identical `unwrap_or_else(|_| panic!())` calls on `query_map` results. If either query fails (e.g. locked DB, corrupt row), the app crashes instead of skipping the migration gracefully.

- [ ] **Step 1: Locate both panic sites**

Open `src-tauri/src/db/mod.rs`. Find the two blocks that follow this pattern (one for `files.stored_path`, one for `directories.path`, one for `sessions.project_path`):

```rust
let rows: Vec<(String, String)> = stmt
    .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
    .unwrap_or_else(|_| panic!())   // ← the bug
    .filter_map(|r| r.ok())
    .collect();
```

There are exactly **two** occurrences of `.unwrap_or_else(|_| panic!())` in this function.

- [ ] **Step 2: Replace both panics with graceful fallbacks**

Change each occurrence to use a `match` so a query failure skips that block instead of crashing:

```rust
let rows: Vec<(String, String)> = match stmt
    .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
{
    Ok(mapped) => mapped.filter_map(|r| r.ok()).collect(),
    Err(_) => Vec::new(),
};
```

Apply this to all three `if let Ok(mut stmt) = ...` blocks in `migrate_to_relative_paths` (files, directories, sessions). Each block has its own `stmt` and `rows` variable — replace the `unwrap_or_else(|_| panic!())` in each one identically.

- [ ] **Step 3: Verify compilation**

```bash
cd src-tauri && cargo check 2>&1
```

Expected: no errors or warnings related to `mod.rs`.

- [ ] **Step 4: Commit**

```bash
cd D:/LittleBrushGames/FeatureHub
git add src-tauri/src/db/mod.rs
git commit -m "fix: replace panic with graceful skip in migrate_to_relative_paths"
```

---

## Task 2: Remove phantom `sort_order` from TypeScript `Link` interface

**Files:**
- Modify: `src/lib/api/types.ts` (line 38)

**Context:** The `Link` interface in `types.ts` declares `sort_order: number`, but:
- The Rust `Link` struct in `db/links.rs` has no such field
- The `links` DB table has no such column
- No DB migration adds it
- No frontend code reads `link.sort_order`

The field is therefore always `undefined` at runtime while TypeScript claims it's `number`. This is a type lie that can cause silent bugs.

- [ ] **Step 1: Remove the field from the interface**

In `src/lib/api/types.ts`, find the `Link` interface (around line 29–40):

```typescript
export interface Link {
  id: string;
  feature_id: string;
  title: string;
  url: string;
  link_type: string;
  description: string | null;
  metadata: Record<string, any> | null;
  created_at: string;
  sort_order: number;   // ← remove this line
}
```

Remove the `sort_order: number;` line. The result:

```typescript
export interface Link {
  id: string;
  feature_id: string;
  title: string;
  url: string;
  link_type: string;
  description: string | null;
  metadata: Record<string, any> | null;
  created_at: string;
}
```

- [ ] **Step 2: Verify no code was using it**

```bash
cd D:/LittleBrushGames/FeatureHub
grep -r "\.sort_order" src/lib/modules/links/ src/lib/api/links.ts 2>/dev/null
```

Expected: no output. If anything appears, remove or fix those references.

- [ ] **Step 3: Run TypeScript check**

```bash
cd D:/LittleBrushGames/FeatureHub
npx svelte-check --tsconfig ./tsconfig.json 2>&1 | tail -20
```

Expected: 0 errors (warnings from other files are OK, but no errors mentioning `sort_order` or `Link`).

- [ ] **Step 4: Commit**

```bash
git add src/lib/api/types.ts
git commit -m "fix: remove phantom sort_order field from Link type"
```

---

## Task 3: Delete dead `db/branches.rs`

**Files:**
- Delete: `src-tauri/src/db/branches.rs`

**Context:** `branches.rs` provides `get_branches`, `add_branch`, `remove_branch` etc. for the `feature_branches` table. However:
- It is **never declared** in `db/mod.rs` (`pub mod branches` is absent)
- **No code anywhere** calls `db::branches::*`
- The `feature_branches` table is created and queried in `export_import.rs` via raw inline SQL (not through this module)
- The module compiles only coincidentally because Rust only compiles declared modules

The file creates false confidence that branch tracking is implemented when it is not. Deleting it makes the gap explicit.

- [ ] **Step 1: Confirm nothing references the module**

```bash
cd D:/LittleBrushGames/FeatureHub
grep -r "db::branches\|mod branches" src-tauri/src/ 2>/dev/null
```

Expected: no output. If anything appears, stop and investigate before deleting.

- [ ] **Step 2: Delete the file**

```bash
rm D:/LittleBrushGames/FeatureHub/src-tauri/src/db/branches.rs
```

- [ ] **Step 3: Verify compilation is unaffected**

```bash
cd D:/LittleBrushGames/FeatureHub/src-tauri && cargo check 2>&1
```

Expected: clean compile. (The `branches.rs` file was never in the module tree so its removal changes nothing.)

- [ ] **Step 4: Commit**

```bash
cd D:/LittleBrushGames/FeatureHub
git add -u src-tauri/src/db/branches.rs
git commit -m "remove: delete unused db/branches.rs module (never wired into db/mod.rs)"
```

---

## Self-Review

**Spec coverage:**
- ✅ Task 1 covers the startup panic
- ✅ Task 2 covers the phantom TypeScript field
- ✅ Task 3 covers the dead module

**Placeholder scan:** None found. All steps include exact file paths, exact code, and exact commands.

**Type consistency:** No new types introduced. Task 2 removes a field; Tasks 1 and 3 are Rust-only with no type surface.

**Gaps:** None. Each task is independent and produces a clean, verifiable commit.
