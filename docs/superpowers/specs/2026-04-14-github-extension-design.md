# GitHub Extension Design

**Date**: 2026-04-14
**Status**: Draft
**Depends on**: `2026-04-03-extension-system-design.md` (Extension System)

## Overview

A first-party extension that brings GitHub pull requests into Feature Hub: list, view, refresh, and get notified on state changes for PRs linked to a feature. Ships as a directory-based extension under `<storage>/extensions/github-prs/` using the existing extension system — no changes to FH core beyond one new extension-system primitive (scheduled tasks) that is generally useful for any future polling extension.

Scope v1: **PRs + CI checks + reviews**. Adds one general primitive to the extension system: `schedules`. Uses the `gh` CLI for all GitHub access.

## Goals

- Zero new core FH UI — the extension is the UI.
- Parity with existing "Links" ergonomics: paste a PR URL, it shows up in the PRs tab automatically.
- Keep Claude sessions productive: Claude can list/fetch/add PRs via MCP tools.
- Low background cost by default; opt-in polling for users who want live updates.

## Extension Package

```
extensions/github-prs/
  extension.json
  handlers/
    add_pr.js               # MCP: add_github_pr
    list_prs.js             # MCP: list_github_prs
    get_pr.js               # MCP: get_github_pr
    refresh_pr.js           # MCP: refresh_github_pr
    on_link_created.js      # event: link_type=github-pr
    on_link_deleted.js      # event: link_type=github-pr
    poll.js                 # schedule: poll open PRs (opt-in)
  ui/
    tab.html
    tab.js
    tab.css
  lib/
    gh.js                   # shared: gh CLI invoker + normalizer
    db.js                   # shared: sqlite helpers
    notify.js               # shared: state-change → notifications[] builder
  handlers/__tests__/       # node --test
```

All handlers are Node.js scripts. Shared code lives in `lib/` and is required by handlers.

## Manifest

```json
{
  "id": "github-prs",
  "name": "GitHub Pull Requests",
  "version": "1.0.0",
  "description": "Track GitHub PRs, CI checks, and reviews for features.",
  "author": "LittleBrushGames",
  "requires": ["gh"],
  "storage_settings_key": "github_prs",

  "tables": [
    {
      "name": "ext_github_prs",
      "columns": [
        { "name": "id",              "type": "TEXT PRIMARY KEY" },
        { "name": "feature_id",      "type": "TEXT NOT NULL", "fk": "features(id) ON DELETE CASCADE" },
        { "name": "directory_id",    "type": "TEXT", "fk": "directories(id) ON DELETE SET NULL" },
        { "name": "url",             "type": "TEXT NOT NULL UNIQUE" },
        { "name": "repo_owner",      "type": "TEXT NOT NULL" },
        { "name": "repo_name",       "type": "TEXT NOT NULL" },
        { "name": "pr_number",       "type": "INTEGER NOT NULL" },
        { "name": "title",           "type": "TEXT NOT NULL" },
        { "name": "state",           "type": "TEXT NOT NULL" },
        { "name": "is_draft",        "type": "INTEGER NOT NULL DEFAULT 0" },
        { "name": "author",          "type": "TEXT" },
        { "name": "base_branch",     "type": "TEXT" },
        { "name": "head_branch",     "type": "TEXT" },
        { "name": "additions",       "type": "INTEGER" },
        { "name": "deletions",       "type": "INTEGER" },
        { "name": "changed_files",   "type": "INTEGER" },
        { "name": "created_at",      "type": "TEXT NOT NULL" },
        { "name": "updated_at",      "type": "TEXT NOT NULL" },
        { "name": "merged_at",       "type": "TEXT" },
        { "name": "closed_at",       "type": "TEXT" },
        { "name": "last_fetched_at", "type": "TEXT NOT NULL" },
        { "name": "raw_json",        "type": "TEXT" }
      ],
      "indexes": ["feature_id", "url"]
    },
    {
      "name": "ext_github_pr_checks",
      "columns": [
        { "name": "id",           "type": "TEXT PRIMARY KEY" },
        { "name": "pr_id",        "type": "TEXT NOT NULL", "fk": "ext_github_prs(id) ON DELETE CASCADE" },
        { "name": "name",         "type": "TEXT NOT NULL" },
        { "name": "status",       "type": "TEXT NOT NULL" },
        { "name": "conclusion",   "type": "TEXT" },
        { "name": "url",          "type": "TEXT" },
        { "name": "completed_at", "type": "TEXT" }
      ],
      "indexes": ["pr_id"]
    },
    {
      "name": "ext_github_pr_reviews",
      "columns": [
        { "name": "id",           "type": "TEXT PRIMARY KEY" },
        { "name": "pr_id",        "type": "TEXT NOT NULL", "fk": "ext_github_prs(id) ON DELETE CASCADE" },
        { "name": "reviewer",     "type": "TEXT NOT NULL" },
        { "name": "state",        "type": "TEXT NOT NULL" },
        { "name": "submitted_at", "type": "TEXT" }
      ],
      "indexes": ["pr_id"]
    }
  ],

  "tools": [
    { "name": "list_github_prs",    "description": "List PRs linked to a feature (or current feature).", "handler": "handlers/list_prs.js",
      "params": { "feature_id": { "type": "string", "required": false } } },
    { "name": "get_github_pr",      "description": "Get full PR details including checks and reviews.",    "handler": "handlers/get_pr.js",
      "params": { "pr_id": { "type": "string", "required": false }, "url": { "type": "string", "required": false } } },
    { "name": "add_github_pr",      "description": "Attach a GitHub PR URL to a feature.",                "handler": "handlers/add_pr.js",
      "params": { "feature_id": { "type": "string", "required": true }, "url": { "type": "string", "required": true } } },
    { "name": "refresh_github_pr",  "description": "Refresh a PR's metadata, checks, and reviews.",       "handler": "handlers/refresh_pr.js",
      "params": { "pr_id": { "type": "string", "required": false }, "url": { "type": "string", "required": false } } }
  ],

  "events": [
    { "on": "link_created", "filter": { "link_type": "github-pr" }, "handler": "handlers/on_link_created.js" },
    { "on": "link_deleted", "filter": { "link_type": "github-pr" }, "handler": "handlers/on_link_deleted.js" }
  ],

  "schedules": [
    { "id": "poll_prs", "handler": "handlers/poll.js", "interval_secs": 300, "enabled_setting": "poll_enabled" }
  ],

  "tabs": [
    {
      "id": "github-prs",
      "label": "PRs",
      "emoji": "🔀",
      "sortOrder": 350,
      "component": "ui/tab.html",
      "badge_query": "SELECT COUNT(*) FROM ext_github_prs WHERE feature_id = ? AND state = 'open'"
    }
  ],

  "instructions": "GitHub PRs extension: use list_github_prs / get_github_pr to inspect PRs linked to the current feature. Use add_github_pr after creating a PR with `gh pr create` so it is tracked by the feature. Use refresh_github_pr to get latest state on demand. Commenting, approving, and merging are deliberately not exposed."
}
```

## New Extension-System Primitive: Schedules

GitHub PRs needs periodic polling. The existing extension system has no scheduling primitive, so this spec introduces one — generally useful for any future extension that wants periodic work.

**Manifest shape:**
```json
"schedules": [
  { "id": "<id>", "handler": "<path>", "interval_secs": <int>, "enabled_setting": "<setting-key>" }
]
```

**Semantics:**
- On extension load, backend spawns one tokio task per schedule whose `enabled_setting` is `true` in the extension's storage-scoped settings (see **Settings** below).
- Task wakes every `interval_secs`, invokes the handler via the existing script runner with an **empty `feature_id`** (it's a global tick) and the standard `db_path` / `storage_path`.
- **Per-schedule mutex**: if a tick is still running when the next tick fires, the new tick is skipped.
- On setting toggle or extension disable, tasks are cancelled.
- Minimum `interval_secs`: 60. Values below are clamped with a warning.

**Extended script stdout protocol:**
Handlers (any — tools, events, schedules) may include an optional `notifications` array in their success response:

```json
{
  "ok": true,
  "data": { ... },
  "notifications": [
    { "feature_id": "...", "kind": "info|warn|error", "message": "PR #123 merged", "plan_id": null }
  ]
}
```

The script runner iterates the array and calls `config::push_notification_ex()` for each after the handler completes. This replaces any ad-hoc "notify from inside an extension script" mechanism and is generally available to every extension.

## Data Model

Three `ext_`-prefixed tables (schemas in manifest above).

**Refresh strategy:** for each PR, on refresh we `UPDATE` the row in `ext_github_prs` and `DELETE + INSERT` rows in `ext_github_pr_checks` and `ext_github_pr_reviews`. Simple, avoids reconciliation bugs, and GitHub returns the full check/review sets per PR anyway.

**Rollups** (computed in SQL via aggregate queries; not stored):
- `checks_rollup`: counts by `conclusion` over the PR's rows in `ext_github_pr_checks` → `{ total, success, failure, pending }` + a single worst-case symbol.
- `review_rollup`: counts by `state` over `ext_github_pr_reviews` → `{ approved, changes_requested, pending }`.

`raw_json` stores the full `gh pr view --json ...` blob for forward compatibility — new display fields can be derived without re-fetching.

## Fetching via `gh`

One canonical call in `lib/gh.js`:

```
gh pr view <number> --repo <owner>/<repo> \
  --json number,url,title,state,isDraft,author,headRefName,baseRefName,\
         additions,deletions,changedFiles,createdAt,updatedAt,mergedAt,closedAt,\
         statusCheckRollup,reviews,reviewRequests
```

Normalizer in `lib/gh.js` converts the `gh` JSON into `{ pr, checks[], reviews[] }`. `statusCheckRollup` supplies the checks rows directly. `reviews` supplies submitted reviews (latest per reviewer wins); `reviewRequests` supplies pending-requested reviewers (mapped to `state = "requested"`).

For the poll path, `poll.js` first queries all distinct `(repo_owner, repo_name)` pairs from `ext_github_prs WHERE state='open'` and calls `gh pr list --repo <owner>/<repo> --state all --json number,url,state,updatedAt` per repo to cheaply find PRs whose `updatedAt` changed since `last_fetched_at`; only those are then fully re-fetched via the single-PR call above. Minimizes API usage under rate limits.

## Flows

**Paste a PR URL into Links tab** → `add_link` command writes link → `extensions::dispatch_event("link_created", { link_type: "github-pr", feature_id, url, ... })` fires → `on_link_created.js`:
1. Parse URL → `{ owner, repo, number }`. Invalid → return error (but don't fail the link creation).
2. Fetch via `lib/gh.js`.
3. Upsert `ext_github_prs` by `url`.
4. Delete & insert `ext_github_pr_checks` / `ext_github_pr_reviews`.
5. Emit one info notification: `"Linked PR #{n}: {title}"`.

**Delete the link** → `on_link_deleted.js` removes the `ext_github_prs` row (cascades to checks/reviews).

**User clicks "Refresh" on a card** → iframe calls `refresh_github_pr` via the `fh:invoke` bridge → `refresh_pr.js` runs the same fetch+upsert+diff path.

**Poll tick** → `poll.js` runs the list-per-repo probe, identifies changed PRs, calls the refresh core for each, collects `notifications[]` from state-change detection, returns them. The script runner forwards them to `push_notification_ex`.

**Claude invokes `add_github_pr`** → same handler as the link path; creates the link row too (so the PR is also visible in the Links tab). Idempotent by URL.

## State-Change Detection

`lib/notify.js` compares `(old, new)` snapshots of PR + checks + reviews and emits notifications for:
- `state` transition `open` → `merged` (msg: `PR #N merged: {title}`)
- `state` transition `open` → `closed` (without merge) (msg: `PR #N closed: {title}`)
- Any check conclusion transitions to `failure` that wasn't previously `failure` (msg: `Checks failed on PR #N`)
- New review with `state = changes_requested` (msg: `Changes requested on PR #N by @{reviewer}`)
- New review with `state = approved` (msg: `PR #N approved by @{reviewer}`)
- New entry in `reviewRequests` not previously present (msg: `Review requested on PR #N`)

Each notification is `{ feature_id, kind: "info"|"warn", message }`. The UI toast and sidebar refresh already handle everything downstream via the existing 2s poll loop.

## UI

### PRs tab

Rendered in an iframe per the existing extension-system tab mechanism. Layout: single-column wrapper with a header row (`[+ Add PR]` button left, `[⟳ Refresh all]` right) and a responsive card grid below (min card width 320px, wraps to fill).

**Card** (class: `sc-card github-pr-card`):
- Header: state pill (colors: `open`=success, `draft`=neutral, `merged`=violet, `closed`=danger), `#{pr_number}`, `{owner}/{repo}`, spacer, overflow menu (⋮).
- Title: 2-line truncate. Click → opens `url` via the parent's `fh:shellOpen` bridge message (Tauri `shell.open`).
- Footer row 1 — **rollups**:
  - Checks: worst-case icon (`✓` all success / `✗` any failure / `⏳` any pending) + `{success}/{total}`. Color matches worst case.
  - Reviews: `👍 {approved}` + `⚠ {changes_requested}` (if > 0) + `👀 {pending}` (if > 0).
- Footer row 2: `@{author}` · relative time (`opened 2h ago` / `updated 1h ago` / `merged 1d ago` based on state).
- Click body (not title / not menu) → inline expand:
  - Branches: `{head_branch} → {base_branch}`
  - Diff stats: `+{additions} / -{deletions}` across `{changed_files} files`
  - Checks list: each row = status icon + name + "view" link (check's `url`)
  - Reviews list: each row = state icon + `@{reviewer}` + relative timestamp
- Overflow menu items: `Refresh`, `Open on GitHub`, `Remove from feature` (confirm dialog → deletes the underlying `github-pr` link, which cascades via `on_link_deleted`).

**Add PR modal** — uses shared `Modal` primitive. Single URL input, regex validator `^https://github\.com/[^/]+/[^/]+/pull/\d+$`, submit button disabled until valid. On submit → `add_github_pr` tool via iframe bridge. Inline error on failure, close on success.

**Tab mount behavior:**
- Initial render: load rows from DB (via `list_github_prs`), render immediately.
- If any PR has `last_fetched_at > 5 min ago`, fire a background "Refresh all" and update cards as they return.
- No polling here — polling is the opt-in backend schedule.

### Settings card

Added to the existing extensions section of `SettingsModal.svelte`:
- Name, version, description, source label ("Installed").
- Enabled toggle.
- `requires` check: `✓ gh found at {path}` or `✗ gh not found — install from https://cli.github.com`.
- Auth status: runs `gh auth status` once on modal open (cached for the modal lifetime), shows `✓ Logged in to github.com as @{login}` or `✗ Not authenticated — run: gh auth login`.
- Polling controls (only enabled if gh found + authenticated):
  - Checkbox `Auto-refresh open PRs in background` → writes `poll_enabled` to the extension's storage-scoped settings key (`github_prs`).
  - Interval dropdown `3m / 5m / 10m / 30m` → writes `poll_interval_secs`. Changing this restarts the schedule task.

## Settings Storage

A new manifest field `storage_settings_key` names an object under the storage's existing `settings.json` `extensions` slice. For this extension: `extensions.github_prs = { "poll_enabled": false, "poll_interval_secs": 300 }`. Defaults are applied on first load.

Backend exposes two small Tauri commands (generally useful for any extension):
- `get_extension_settings(extension_id) -> JSON`
- `set_extension_settings(extension_id, settings_json)`

The existing CombinedSettings flow already loads `extensions`, so this is a thin accessor.

## MCP Tool Surface

Exposed by `fh-mcp` through the existing extension-tool registration path. Tool descriptions are tuned for Claude:

- `list_github_prs(feature_id?)` — lists PRs for the feature (defaults to scoped feature when available). Returns summaries with rollups.
- `get_github_pr(pr_id? | url?)` — full detail (PR + checks array + reviews array). At least one of the two must be provided.
- `add_github_pr(feature_id, url)` — attaches a PR. Idempotent by URL. Also creates the underlying `github-pr` link if absent.
- `refresh_github_pr(pr_id? | url?)` — re-fetches and updates.

Commenting, approving, merging, and closing are deliberately out of scope.

## Error Handling

- `gh` missing — extension loads; `requires_status` shows missing. Tools return `{ ok: false, error: "gh CLI not found — install from https://cli.github.com" }`. Tab shows inline banner.
- `gh` not authenticated — stderr contains `authentication` / `login` → `{ ok: false, error: "gh not authenticated — run: gh auth login" }`. Tab banner shows same.
- PR URL malformed — `add_github_pr` regex-validates before shelling out.
- PR not found / no access (404) — surface as user-facing toast; no row inserted.
- Rate limit (`403` + `X-RateLimit-Remaining: 0`) — detected in `lib/gh.js`, back off polling for 15 minutes, emit one warning toast.
- Partial fetch failure (PR ok, checks fail) — save PR row, skip checks/reviews, do NOT bump `last_fetched_at` so next refresh retries. Log to `<storage>/extensions/github-prs/logs/`.
- Concurrent poll ticks — skipped via per-schedule mutex (in extension-system core).
- Link deleted while refresh in flight — use `UPDATE ... WHERE id = ?` and check affected rows; zero affected = no-op (prevents resurrected ghosts).

## Testing

### Extension-system additions (Rust)

- `schedules` manifest parsing: valid shape accepted; missing `id` / non-positive `interval_secs` rejected.
- Scheduled task runner: spawns one task per enabled schedule; `enabled_setting = false` → no task.
- Per-schedule mutex prevents overlapping ticks (drive with a script that sleeps 2s on a 1s interval; verify only one runs).
- Script stdout `notifications` array forwarded to `push_notification_ex`.
- `get_extension_settings` / `set_extension_settings` round-trip.

### GitHub extension (Node, `handlers/__tests__/`)

- `lib/gh.js` URL parser: valid PR URLs → `{ owner, repo, number }`; invalid → `null`.
- `lib/gh.js` normalizer: given fixture `gh pr view --json` blobs, returns expected `{ pr, checks, reviews }`.
- `lib/notify.js` state-change detector: table-driven test covering each transition listed above; no notifications on no-op diffs.
- `on_link_created.js`: running twice with the same URL produces one row.
- `poll.js`: with fixture repos and a stubbed `gh`, only changed PRs are refreshed.

### Manual integration test (documented)

1. Fresh storage, install extension, open Settings — card shows `gh` + auth status.
2. Paste a PR URL into a feature's Links tab — PRs tab gets a card within seconds.
3. Open the tab, click a card — inline details expand.
4. Click Refresh on the card — metadata updates.
5. Enable polling (3m). On GitHub, close/merge the PR. Within ≤3m, toast appears and card updates.
6. From a Claude session, call `list_github_prs`, `get_github_pr`, `add_github_pr`, `refresh_github_pr` — each returns expected shape.

## Out of Scope (deferred)

- PR comments, review submission, approvals, merges from within FH.
- Issues, discussions, releases.
- Webhook-based push updates.
- Branch-based auto-discovery of PRs for a feature's linked directories.
- GitHub Enterprise custom hosts (inherits whatever `gh` is configured with, but no explicit UI).
- REST API fallback without `gh`.
- Multi-account selection beyond `gh`'s default account.
