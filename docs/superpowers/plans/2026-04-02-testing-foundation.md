# Testing Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Set up testing infrastructure for both Rust backend and Svelte frontend, with representative tests proving each setup works.

**Architecture:** Rust tests use `#[cfg(test)]` modules in existing files with a shared test helper that creates in-memory SQLite databases. Frontend tests use Vitest + jsdom + @testing-library/svelte. Utility functions and one component get tests.

**Tech Stack:** rusqlite (in-memory), Vitest, @testing-library/svelte, jsdom

---

## File Structure

**Rust (new files):**
- `src-tauri/src/db/test_utils.rs` — shared test helper: creates in-memory DB, runs `initialize()`

**Rust (modified files — adding `#[cfg(test)]` modules):**
- `src-tauri/src/db/mod.rs` — declare `test_utils` module
- `src-tauri/src/db/features.rs` — feature CRUD tests
- `src-tauri/src/db/tasks.rs` — task CRUD tests
- `src-tauri/src/db/notes.rs` — note get/save tests

**Frontend (new files):**
- `src/lib/utils/format.test.ts` — tests for all 4 format functions
- `src/lib/utils/linkTypes.test.ts` — tests for URL detection and link type helpers
- `src/lib/components/ui/IconButton.test.ts` — component render/interaction tests

**Frontend (modified files):**
- `package.json` — add vitest, @testing-library/svelte, jsdom devDependencies + test script
- `vite.config.ts` — add vitest config via `/// <reference types="vitest" />`

---

### Task 1: Rust test helper module

**Files:**
- Create: `src-tauri/src/db/test_utils.rs`
- Modify: `src-tauri/src/db/mod.rs`

- [ ] **Step 1: Create the test helper module**

Create `src-tauri/src/db/test_utils.rs`:

```rust
use rusqlite::Connection;

/// Creates an in-memory SQLite database with the full schema initialized.
/// Use this in `#[cfg(test)]` modules to get a fresh DB for each test.
pub fn test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to open in-memory DB");
    super::initialize(&conn).expect("Failed to initialize schema");
    conn
}
```

- [ ] **Step 2: Declare the module in db/mod.rs**

Add to `src-tauri/src/db/mod.rs`, after the existing `pub mod` declarations:

```rust
#[cfg(test)]
pub mod test_utils;
```

- [ ] **Step 3: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles with no errors

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/db/test_utils.rs src-tauri/src/db/mod.rs
git commit -m "test: add in-memory DB test helper for Rust tests"
```

---

### Task 2: Feature CRUD tests

**Files:**
- Modify: `src-tauri/src/db/features.rs` (append `#[cfg(test)]` module)

- [ ] **Step 1: Write the tests**

Append to `src-tauri/src/db/features.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_utils::test_db;

    #[test]
    fn create_feature_returns_feature_with_defaults() {
        let conn = test_db();
        let feature = create_feature(&conn, "My Feature", None, None, None, None).unwrap();

        assert_eq!(feature.title, "My Feature");
        assert_eq!(feature.status, "active");
        assert!(feature.ticket_id.is_none());
        assert!(feature.description.is_none());
        assert!(!feature.pinned);
        assert!(!feature.archived);
        assert!(feature.parent_id.is_none());
    }

    #[test]
    fn get_feature_returns_created_feature() {
        let conn = test_db();
        let created = create_feature(&conn, "Test Feature", None, None, None, None).unwrap();
        let fetched = get_feature(&conn, &created.id).unwrap();

        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.title, "Test Feature");
    }

    #[test]
    fn get_feature_not_found_returns_error() {
        let conn = test_db();
        let result = get_feature(&conn, "nonexistent-id");

        assert!(result.is_err());
    }

    #[test]
    fn update_feature_changes_title() {
        let conn = test_db();
        let created = create_feature(&conn, "Original", None, None, None, None).unwrap();
        let updated = update_feature(
            &conn, &created.id,
            Some("Renamed".to_string()), None, None, None, None,
        ).unwrap();

        assert_eq!(updated.title, "Renamed");
    }

    #[test]
    fn delete_feature_removes_it() {
        let conn = test_db();
        let created = create_feature(&conn, "To Delete", None, None, None, None).unwrap();
        delete_feature(&conn, &created.id, None).unwrap();

        let result = get_feature(&conn, &created.id);
        assert!(result.is_err());
    }

    #[test]
    fn get_features_returns_all_created() {
        let conn = test_db();
        create_feature(&conn, "Feature A", None, None, None, None).unwrap();
        create_feature(&conn, "Feature B", None, None, None, None).unwrap();

        let features = get_features(&conn, None, None).unwrap();
        assert_eq!(features.len(), 2);
    }

    #[test]
    fn toggle_pin_flips_pinned_state() {
        let conn = test_db();
        let created = create_feature(&conn, "Pin Me", None, None, None, None).unwrap();
        assert!(!created.pinned);

        let pinned = toggle_pin_feature(&conn, &created.id).unwrap();
        assert!(pinned.pinned);

        let unpinned = toggle_pin_feature(&conn, &created.id).unwrap();
        assert!(!unpinned.pinned);
    }
}
```

- [ ] **Step 2: Run the tests**

Run: `cd src-tauri && cargo test --lib db::features::tests`
Expected: all 7 tests pass

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/db/features.rs
git commit -m "test: add feature CRUD unit tests"
```

---

### Task 3: Task CRUD tests

**Files:**
- Modify: `src-tauri/src/db/tasks.rs` (append `#[cfg(test)]` module)

- [ ] **Step 1: Write the tests**

Append to `src-tauri/src/db/tasks.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_utils::test_db;
    use crate::db::features::create_feature;

    fn setup() -> (rusqlite::Connection, String) {
        let conn = test_db();
        let feature = create_feature(&conn, "Test Feature", None, None, None, None).unwrap();
        let id = feature.id.clone();
        (conn, id)
    }

    #[test]
    fn create_task_returns_task_with_defaults() {
        let (conn, feature_id) = setup();
        let task = create_task(&conn, &feature_id, "My Task", None, None, None, None, None).unwrap();

        assert_eq!(task.title, "My Task");
        assert_eq!(task.feature_id, feature_id);
        assert!(!task.done);
        assert_eq!(task.source, "manual");
    }

    #[test]
    fn get_tasks_returns_all_for_feature() {
        let (conn, feature_id) = setup();
        create_task(&conn, &feature_id, "Task A", None, None, None, None, None).unwrap();
        create_task(&conn, &feature_id, "Task B", None, None, None, None, None).unwrap();

        let tasks = get_tasks(&conn, &feature_id).unwrap();
        assert_eq!(tasks.len(), 2);
    }

    #[test]
    fn update_task_marks_done() {
        let (conn, feature_id) = setup();
        let task = create_task(&conn, &feature_id, "Do This", None, None, None, None, None).unwrap();

        let updated = update_task(&conn, &task.id, None, Some(true), None, None).unwrap();
        assert!(updated.done);
    }

    #[test]
    fn delete_task_removes_it() {
        let (conn, feature_id) = setup();
        let task = create_task(&conn, &feature_id, "Remove Me", None, None, None, None, None).unwrap();

        delete_task(&conn, &task.id).unwrap();
        let tasks = get_tasks(&conn, &feature_id).unwrap();
        assert!(tasks.is_empty());
    }
}
```

- [ ] **Step 2: Run the tests**

Run: `cd src-tauri && cargo test --lib db::tasks::tests`
Expected: all 4 tests pass

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/db/tasks.rs
git commit -m "test: add task CRUD unit tests"
```

---

### Task 4: Note get/save tests

**Files:**
- Modify: `src-tauri/src/db/notes.rs` (append `#[cfg(test)]` module)

- [ ] **Step 1: Write the tests**

Append to `src-tauri/src/db/notes.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_utils::test_db;
    use crate::db::features::create_feature;

    fn setup() -> (rusqlite::Connection, String) {
        let conn = test_db();
        let feature = create_feature(&conn, "Test Feature", None, None, None, None).unwrap();
        let id = feature.id.clone();
        (conn, id)
    }

    #[test]
    fn get_note_returns_none_when_no_note() {
        let (conn, feature_id) = setup();
        let note = get_note(&conn, &feature_id).unwrap();
        assert!(note.is_none());
    }

    #[test]
    fn save_note_creates_and_returns_note() {
        let (conn, feature_id) = setup();
        let note = save_note(&conn, &feature_id, "Hello world").unwrap();

        assert_eq!(note.feature_id, feature_id);
        assert_eq!(note.content, "Hello world");
    }

    #[test]
    fn save_note_upserts_existing() {
        let (conn, feature_id) = setup();
        save_note(&conn, &feature_id, "Version 1").unwrap();
        let updated = save_note(&conn, &feature_id, "Version 2").unwrap();

        assert_eq!(updated.content, "Version 2");

        let fetched = get_note(&conn, &feature_id).unwrap().unwrap();
        assert_eq!(fetched.content, "Version 2");
    }
}
```

- [ ] **Step 2: Run the tests**

Run: `cd src-tauri && cargo test --lib db::notes::tests`
Expected: all 3 tests pass

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/db/notes.rs
git commit -m "test: add note get/save unit tests"
```

---

### Task 5: Frontend test infrastructure (Vitest + Svelte Testing Library)

**Files:**
- Modify: `package.json` — add devDependencies and test script
- Modify: `vite.config.ts` — add vitest config

- [ ] **Step 1: Install test dependencies**

Run from project root:
```bash
npm install -D vitest @testing-library/svelte @testing-library/jest-dom jsdom
```

- [ ] **Step 2: Add test script to package.json**

Add to `scripts` in `package.json`:
```json
"test": "vitest run",
"test:watch": "vitest"
```

- [ ] **Step 3: Add Vitest config to vite.config.ts**

Update `vite.config.ts` to:

```typescript
/// <reference types="vitest" />
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [svelte(), tailwindcss()],
  clearScreen: false,
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.ts"],
  },
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
```

- [ ] **Step 4: Verify vitest runs (no tests yet, should exit cleanly)**

Run: `npm run test`
Expected: "No test files found" or similar clean exit

- [ ] **Step 5: Commit**

```bash
git add package.json package-lock.json vite.config.ts
git commit -m "test: add Vitest + Svelte Testing Library infrastructure"
```

---

### Task 6: Utility function tests — format.ts

**Files:**
- Create: `src/lib/utils/format.test.ts`

- [ ] **Step 1: Write the tests**

Create `src/lib/utils/format.test.ts`:

```typescript
import { describe, it, expect, vi, afterEach } from "vitest";
import { formatDate, formatRelativeTime, formatDuration, formatFileSize } from "./format";

describe("formatDate", () => {
  it("formats an ISO date string", () => {
    expect(formatDate("2026-01-15T10:30:00Z")).toBe("Jan 15, 2026");
  });

  it("formats a date in a different month", () => {
    expect(formatDate("2025-12-03T00:00:00Z")).toBe("Dec 3, 2025");
  });
});

describe("formatRelativeTime", () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it('returns "just now" for recent timestamps', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-04-02T12:00:30Z"));
    expect(formatRelativeTime("2026-04-02T12:00:00Z")).toBe("just now");
  });

  it("returns minutes ago", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-04-02T12:05:00Z"));
    expect(formatRelativeTime("2026-04-02T12:00:00Z")).toBe("5m ago");
  });

  it("returns hours ago", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-04-02T15:00:00Z"));
    expect(formatRelativeTime("2026-04-02T12:00:00Z")).toBe("3h ago");
  });

  it("returns days ago", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-04-05T12:00:00Z"));
    expect(formatRelativeTime("2026-04-02T12:00:00Z")).toBe("3d ago");
  });
});

describe("formatDuration", () => {
  it("formats minutes only", () => {
    expect(formatDuration(45)).toBe("45m");
  });

  it("formats hours only", () => {
    expect(formatDuration(120)).toBe("2h");
  });

  it("formats hours and minutes", () => {
    expect(formatDuration(90)).toBe("1h 30m");
  });
});

describe("formatFileSize", () => {
  it("formats bytes", () => {
    expect(formatFileSize(500)).toBe("500 B");
  });

  it("formats kilobytes", () => {
    expect(formatFileSize(1536)).toBe("1.5 KB");
  });

  it("formats megabytes", () => {
    expect(formatFileSize(2.5 * 1024 * 1024)).toBe("2.5 MB");
  });

  it("formats gigabytes", () => {
    expect(formatFileSize(1.5 * 1024 * 1024 * 1024)).toBe("1.50 GB");
  });
});
```

- [ ] **Step 2: Run the tests**

Run: `npm run test`
Expected: all tests pass

- [ ] **Step 3: Commit**

```bash
git add src/lib/utils/format.test.ts
git commit -m "test: add format utility tests"
```

---

### Task 7: Utility function tests — linkTypes.ts

**Files:**
- Create: `src/lib/utils/linkTypes.test.ts`

- [ ] **Step 1: Write the tests**

Create `src/lib/utils/linkTypes.test.ts`:

```typescript
import { describe, it, expect } from "vitest";
import { getLinkTypeFromUrl, getLinkTypeInfo, isTicketLink } from "./linkTypes";

describe("getLinkTypeFromUrl", () => {
  it("detects GitHub URLs", () => {
    expect(getLinkTypeFromUrl("https://github.com/org/repo")).toBe("github");
  });

  it("detects Jira URLs", () => {
    expect(getLinkTypeFromUrl("https://myteam.atlassian.net/browse/PROJ-123")).toBe("jira");
  });

  it("detects Linear URLs", () => {
    expect(getLinkTypeFromUrl("https://linear.app/team/issue/ENG-456")).toBe("linear");
  });

  it("detects Figma URLs", () => {
    expect(getLinkTypeFromUrl("https://figma.com/file/abc123")).toBe("figma");
  });

  it("detects Slack URLs", () => {
    expect(getLinkTypeFromUrl("https://myteam.slack.com/archives/C123")).toBe("slack");
  });

  it('returns "other" for unknown URLs', () => {
    expect(getLinkTypeFromUrl("https://example.com")).toBe("other");
  });

  it('returns "other" for invalid URLs', () => {
    expect(getLinkTypeFromUrl("not-a-url")).toBe("other");
  });
});

describe("getLinkTypeInfo", () => {
  it("returns info for known type", () => {
    const info = getLinkTypeInfo("github");
    expect(info.label).toBe("GitHub");
    expect(info.color).toBeTruthy();
  });

  it("returns fallback for unknown type", () => {
    const info = getLinkTypeInfo("unknown-type");
    expect(info.label).toBe("Link");
  });
});

describe("isTicketLink", () => {
  it("returns true for ticket types", () => {
    expect(isTicketLink("jira")).toBe(true);
    expect(isTicketLink("linear")).toBe(true);
    expect(isTicketLink("github-issue")).toBe(true);
  });

  it("returns false for non-ticket types", () => {
    expect(isTicketLink("github")).toBe(false);
    expect(isTicketLink("figma")).toBe(false);
    expect(isTicketLink("other")).toBe(false);
  });
});
```

- [ ] **Step 2: Run the tests**

Run: `npm run test`
Expected: all tests pass

- [ ] **Step 3: Commit**

```bash
git add src/lib/utils/linkTypes.test.ts
git commit -m "test: add link type utility tests"
```

---

### Task 8: Component test — IconButton

**Files:**
- Create: `src/lib/components/ui/IconButton.test.ts`

- [ ] **Step 1: Write the tests**

Create `src/lib/components/ui/IconButton.test.ts`:

```typescript
import { describe, it, expect, vi } from "vitest";
import { render, fireEvent } from "@testing-library/svelte";
import IconButton from "./IconButton.svelte";

describe("IconButton", () => {
  it("renders with default props", () => {
    const { container } = render(IconButton, {
      props: { children: (($$anchor: any) => { $$anchor.textContent = "X"; }) as any },
    });
    const button = container.querySelector("button");
    expect(button).toBeTruthy();
    expect(button!.classList.contains("icon-btn")).toBe(true);
    expect(button!.classList.contains("icon-btn--sm")).toBe(true);
    expect(button!.classList.contains("icon-btn--ghost")).toBe(true);
  });

  it("applies size and variant classes", () => {
    const { container } = render(IconButton, {
      props: {
        size: "md",
        variant: "accent",
        children: (($$anchor: any) => { $$anchor.textContent = "X"; }) as any,
      },
    });
    const button = container.querySelector("button");
    expect(button!.classList.contains("icon-btn--md")).toBe(true);
    expect(button!.classList.contains("icon-btn--accent")).toBe(true);
  });

  it("sets disabled attribute", () => {
    const { container } = render(IconButton, {
      props: {
        disabled: true,
        children: (($$anchor: any) => { $$anchor.textContent = "X"; }) as any,
      },
    });
    const button = container.querySelector("button") as HTMLButtonElement;
    expect(button.disabled).toBe(true);
  });

  it("sets title attribute", () => {
    const { container } = render(IconButton, {
      props: {
        title: "Click me",
        children: (($$anchor: any) => { $$anchor.textContent = "X"; }) as any,
      },
    });
    const button = container.querySelector("button");
    expect(button!.getAttribute("title")).toBe("Click me");
  });
});
```

Note: Svelte 5 Snippet props are tricky to test. If the snippet approach above doesn't work with @testing-library/svelte, create a thin wrapper component instead:

Create `src/lib/components/ui/IconButtonTestWrapper.svelte`:
```svelte
<script lang="ts">
  import IconButton from "./IconButton.svelte";

  let { onclick, title, variant, size, disabled, label = "X" }: {
    onclick?: (e: MouseEvent) => void;
    title?: string;
    variant?: "ghost" | "subtle" | "accent";
    size?: "xs" | "sm" | "md";
    disabled?: boolean;
    label?: string;
  } = $props();
</script>

<IconButton {onclick} {title} {variant} {size} {disabled}>
  {label}
</IconButton>
```

Then update the test to import `IconButtonTestWrapper` instead and pass `label` as a prop. This is the standard workaround for testing Svelte 5 snippet-based components.

- [ ] **Step 2: Run the tests**

Run: `npm run test`
Expected: all tests pass (adjust snippet approach if needed — see note above)

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/ui/IconButton.test.ts
# If wrapper was needed:
# git add src/lib/components/ui/IconButtonTestWrapper.svelte
git commit -m "test: add IconButton component tests"
```

---

### Task 9: Run all tests end-to-end

- [ ] **Step 1: Run all Rust tests**

Run: `cd src-tauri && cargo test --lib`
Expected: all 14 Rust tests pass (7 features + 4 tasks + 3 notes)

- [ ] **Step 2: Run all frontend tests**

Run: `npm run test`
Expected: all frontend tests pass across 3 test files

- [ ] **Step 3: Verify no regressions**

Run: `cd src-tauri && cargo check`
Expected: clean compilation, no warnings from test code
