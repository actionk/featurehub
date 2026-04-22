# Agents Display Improvements

**Date**: 2026-04-13  
**Status**: Approved  
**Scope**: Fix bugs + add rich status features to agents/sessions display

## Problem Statement

Three issues with the current agents display:

1. **Title garbage**: Shows `<local-command-caveat>Caveat:...` XML tags or unhelpful "Session" fallback instead of meaningful titles
2. **Stale time**: Shows "5d ago" for sessions used today because JSONL file not found, falls back to `linked_at`
3. **Status desync**: Same session shows as active in one panel but not another due to separate process scans

Additionally, users want richer status information:
- Waiting for input detection
- Idle vs finished distinction
- Cost/tokens visibility
- Last action indicator

## Approach

**Smart Unified Polling** — unify data paths, improve parsing, add rich status without architectural overhaul.

## Data Model

### Enhanced PanelSession

```rust
pub struct PanelSession {
    // Identity
    pub id: String,
    pub feature_id: String,
    pub feature_name: String,
    pub claude_session_id: String,
    
    // Display
    pub title: String,              // Never null - always has fallback
    pub title_source: TitleSource,  // Debugging: where title came from
    pub branch: Option<String>,
    
    // Time
    pub last_activity: String,      // Renamed from last_modified
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    
    // Status
    pub is_active: bool,            // Process running
    pub status: SessionStatus,      // Richer status enum
    pub last_action: Option<String>,// "Reading file", "Writing code", etc.
    
    // Stats
    pub model: Option<String>,
    pub total_tokens: Option<u64>,
    pub context_tokens: Option<u64>,
    pub cost_usd: Option<f64>,
}
```

### New Enums

```rust
pub enum SessionStatus {
    Active,           // Process running, Claude working
    WaitingForInput,  // Process running, Claude asked question
    Idle,             // Process not running, session can resume
    Finished,         // Explicitly ended (ended_at set)
    Lost,             // No claude_session_id, can't resume
}

pub enum TitleSource {
    SessionsIndex,    // Claude's auto-generated summary
    SessionMemory,    // From .md file
    FirstPrompt,      // From JSONL user message
    FeatureName,      // Fallback to feature name
    Default,          // "Session" fallback
}

pub enum TimeSource {
    JsonlMtime,       // JSONL file modification time
    EndedAt,          // DB ended_at field
    StartedAt,        // DB started_at field
    LinkedAt,         // DB linked_at (last resort)
}
```

## Title Extraction

### Bad Title Filter

```rust
fn is_bad_title(s: &str) -> bool {
    let trimmed = s.trim();
    
    // Too short
    if trimmed.len() < 5 { return true; }
    
    // Slash commands
    if trimmed.starts_with('/') { return true; }
    
    // XML/HTML tags
    if trimmed.starts_with('<') && trimmed.contains('>') { return true; }
    
    // File paths
    if looks_like_path(trimmed) { return true; }
    
    // Git commands
    if trimmed.starts_with("git ") { return true; }
    
    // System prefixes
    if trimmed.starts_with("Caveat:") { return true; }
    
    false
}
```

### Fallback Chain

Priority order (first match wins):

1. `sessions-index.json` → `summary` field (Claude's auto-title)
2. `sessions-index.json` → `firstPrompt` field (truncated to 120 chars)
3. `session-memory/{id}.md` → first heading
4. JSONL → first valid user message (filtered by `is_bad_title`)
5. Feature name + " Session" (e.g., "Auth Layer Session")
6. "Untitled Session"

Always return a title — never `None`.

## JSONL Discovery

### Path Cache

Add to `AppState`:

```rust
pub jsonl_path_cache: Mutex<HashMap<String, PathBuf>>,
```

### Improved Search

```rust
pub fn find_jsonl_for_session(
    cache: &Mutex<HashMap<String, PathBuf>>,
    session_id: &str,
) -> Option<PathBuf> {
    // 1. Check cache first
    if let Ok(c) = cache.lock() {
        if let Some(path) = c.get(session_id) {
            if path.exists() {
                return Some(path.clone());
            }
        }
    }
    
    // 2. Search projects directory
    let home = dirs::home_dir()?;
    let projects_dir = home.join(".claude").join("projects");
    let filename = format!("{}.jsonl", session_id);
    
    for entry in std::fs::read_dir(&projects_dir).ok()?.flatten() {
        let dir = entry.path();
        if !dir.is_dir() { continue; }
        
        // Direct child
        let jsonl = dir.join(&filename);
        if jsonl.exists() {
            cache.lock().ok()?.insert(session_id.to_string(), jsonl.clone());
            return Some(jsonl);
        }
        
        // Subdirectories (one level deeper)
        for subentry in std::fs::read_dir(&dir).ok()?.flatten() {
            let subdir = subentry.path();
            if subdir.is_dir() {
                let jsonl = subdir.join(&filename);
                if jsonl.exists() {
                    cache.lock().ok()?.insert(session_id.to_string(), jsonl.clone());
                    return Some(jsonl);
                }
            }
        }
    }
    
    None
}
```

### Time Source Priority

1. JSONL file mtime (actual last write)
2. `ended_at` from DB (if session finished)
3. `started_at` from DB
4. `linked_at` from DB (last resort)

## Rich Status Detection

### Parsed State

```rust
pub struct ParsedSessionState {
    pub model: Option<String>,
    pub total_tokens: u64,
    pub cost_usd: Option<f64>,
    pub context_tokens: Option<u64>,
    pub status_hint: StatusHint,
    pub last_action: Option<String>,
}

pub enum StatusHint {
    ClaudeResponded,  // Last message was assistant
    UserPrompted,     // Last message was user
    ToolRunning,      // Tool use in progress
}
```

### Status Resolution

| Process Running | JSONL State | DB State | → SessionStatus |
|-----------------|-------------|----------|-----------------|
| Yes | Assistant ended with `?` | - | `WaitingForInput` |
| Yes | User message pending | - | `Active` |
| Yes | Tool in progress | - | `Active` |
| No | Any | `ended_at` null | `Idle` |
| No | Any | `ended_at` set | `Finished` |
| - | - | No `claude_session_id` | `Lost` |

### Waiting for Input Detection

Parse last assistant message. Waiting if ends with `?` or contains:
- "Would you like me to..."
- "Should I..."
- "Do you want..."
- "Let me know..."
- "What would you prefer?"

### Last Action Extraction

From last JSONL tool calls:

| Tool | → Last Action |
|------|---------------|
| `Read` | "Reading files" |
| `Edit`, `Write` | "Writing code" |
| `Bash` | "Running command" |
| `Grep`, `Glob` | "Searching" |
| `Agent` | "Running subagent" |
| Assistant text only | "Thinking" |

**Performance**: Only parse last ~20 lines of JSONL for status detection.

## Frontend Unification

### Single Data Path

Remove separate process scans. `get_sessions_panel_data` becomes the only endpoint.

### Store Changes (`sessionActivity.svelte.ts`)

```typescript
async function refresh() {
  const data = await getSessionsPanelData();
  
  panelSessions = data.sessions;
  panelActiveCount = data.active_count;
  
  // Derive all state from single source
  counts = {};
  activeSessionIds = new Set();
  
  for (const s of data.sessions) {
    if (s.is_active || s.status === 'Active' || s.status === 'WaitingForInput') {
      counts[s.feature_id] = (counts[s.feature_id] ?? 0) + 1;
      activeSessionIds.add(s.claude_session_id);
    }
  }
}
```

### Component Updates

Both `SessionCard.svelte` and `SessionsPanel.svelte` read from same store — no more desync.

## Files to Modify

### Backend (Rust)

- `src-tauri/src/claude/session_parser.rs` — Add `is_bad_title`, status parsing
- `src-tauri/src/claude/scanner.rs` — Improve JSONL discovery, add path cache
- `src-tauri/src/commands/sessions.rs` — Unify endpoints, enhanced `PanelSession`
- `src-tauri/src/lib.rs` — Add `jsonl_path_cache` to `AppState`

### Frontend (TypeScript/Svelte)

- `src/lib/api/types.ts` — Update `PanelSession` interface, add enums
- `src/lib/stores/sessionActivity.svelte.ts` — Derive all state from single call
- `src/lib/components/SessionsPanel.svelte` — Display new status/last_action
- `src/lib/modules/ai/SessionCard.svelte` — Display new status/last_action

## Testing

### Rust Unit Tests

```rust
#[test]
fn test_is_bad_title_filters_xml_tags() {
    assert!(is_bad_title("<local-command-caveat>Caveat: ..."));
    assert!(!is_bad_title("Add authentication to login flow"));
}

#[test]
fn test_title_fallback_chain() {
    // Test each source in priority order
}

#[test]
fn test_status_hint_detects_waiting() {
    // Assistant message ending with question
}

#[test]
fn test_jsonl_path_cache() {
    // First call searches, second uses cache
}
```

### Frontend Tests

```typescript
test('refresh populates all derived state from single call', async () => {
  // Mock and verify unified data flow
});
```

### Manual Testing Checklist

- [ ] Active session shows green in both panels simultaneously
- [ ] Title shows Claude's summary, not XML garbage
- [ ] Time updates after session activity
- [ ] "Waiting for input" status appears when Claude asks question
- [ ] Cost/tokens display correctly
- [ ] "Idle" vs "Finished" distinction works

## Out of Scope

- File watching (polling is sufficient for 10s refresh)
- Historical session analytics
- Session grouping/filtering beyond current UI
