use tauri::State;

use crate::claude;
use crate::db;
use crate::AppState;

use super::ide::which_command;

#[derive(serde::Serialize)]
pub struct SessionActivity {
    pub counts: std::collections::HashMap<String, u32>,
    pub active_session_ids: Vec<String>,
}

#[derive(serde::Serialize, Clone)]
pub struct PanelSession {
    // Identity
    pub id: String,
    pub feature_id: String,
    pub feature_name: String,
    pub claude_session_id: String,

    // Display
    pub title: String,
    pub title_source: crate::claude::session_parser::TitleSource,
    pub branch: Option<String>,

    // Time
    pub last_activity: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,

    // Status
    pub is_active: bool,
    pub status: crate::claude::session_parser::SessionStatus,
    pub last_action: Option<String>,

    // Stats
    pub model: Option<String>,
    pub total_tokens: Option<u64>,
    pub context_tokens: Option<u64>,
    pub cost_usd: Option<f64>,
}

#[derive(serde::Serialize)]
pub struct SessionsPanelData {
    pub sessions: Vec<PanelSession>,
    pub active_count: usize,
}

#[derive(Clone)]
pub struct CachedStatusHint {
    pub mtime: std::time::SystemTime,
    pub hint: crate::claude::session_parser::ParsedStatusHint,
}

pub fn cached_status_hint(
    cache: &mut std::collections::HashMap<String, CachedStatusHint>,
    session_id: &str,
    mtime: std::time::SystemTime,
    parse: impl FnOnce() -> crate::claude::session_parser::ParsedStatusHint,
) -> crate::claude::session_parser::ParsedStatusHint {
    if let Some(cached) = cache.get(session_id) {
        if cached.mtime == mtime {
            return cached.hint.clone();
        }
    }

    let hint = parse();
    cache.insert(
        session_id.to_string(),
        CachedStatusHint {
            mtime,
            hint: hint.clone(),
        },
    );
    hint
}

#[cfg(test)]
mod session_panel_cache_tests {
    use std::collections::HashMap;
    use std::time::SystemTime;

    use crate::claude::session_parser::{ParsedStatusHint, StatusHint};

    use super::{cached_status_hint, CachedStatusHint};

    #[test]
    fn cached_status_hint_reuses_value_when_mtime_matches() {
        let mtime = SystemTime::UNIX_EPOCH;
        let mut cache = HashMap::<String, CachedStatusHint>::new();
        let mut parse_count = 0;

        let first = cached_status_hint(&mut cache, "s1", mtime, || {
            parse_count += 1;
            ParsedStatusHint {
                status_hint: StatusHint::UserPrompted,
                looks_like_waiting: true,
                last_action: Some("asked".to_string()),
            }
        });
        let second = cached_status_hint(&mut cache, "s1", mtime, || {
            parse_count += 1;
            ParsedStatusHint::default()
        });

        assert_eq!(parse_count, 1);
        assert_eq!(first, second);
        assert!(second.looks_like_waiting);
    }

    #[test]
    fn cached_status_hint_reparses_when_mtime_changes() {
        let mut cache = HashMap::<String, CachedStatusHint>::new();
        let mut parse_count = 0;

        let first = cached_status_hint(&mut cache, "s1", SystemTime::UNIX_EPOCH, || {
            parse_count += 1;
            ParsedStatusHint::default()
        });
        let second = cached_status_hint(
            &mut cache,
            "s1",
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1),
            || {
                parse_count += 1;
                ParsedStatusHint {
                    status_hint: StatusHint::ToolRunning,
                    looks_like_waiting: false,
                    last_action: Some("tool".to_string()),
                }
            },
        );

        assert_eq!(parse_count, 2);
        assert_ne!(first, second);
        assert_eq!(second.last_action.as_deref(), Some("tool"));
    }
}

/// Find the fh-mcp binary path. Checks next to fh_cli_path from settings,
/// then next to the current executable.
pub(crate) fn fh_mcp_path(settings: &crate::config::AppSettings) -> Option<std::path::PathBuf> {
    let mcp_name = if cfg!(windows) {
        "fh-mcp.exe"
    } else {
        "fh-mcp"
    };

    let candidates: Vec<std::path::PathBuf> = {
        let mut paths = Vec::new();

        // 1. Next to fh_cli_path from settings
        if let Some(ref cli_path) = settings.fh_cli_path {
            let cli = std::path::Path::new(cli_path);
            if let Some(dir) = cli.parent() {
                paths.push(dir.join(mcp_name));
            }
        }

        // 2. Next to current executable
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                paths.push(dir.join(mcp_name));
            }
        }

        // 3. Cargo target/debug (dev mode) — CARGO_MANIFEST_DIR is src-tauri/
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        paths.push(manifest_dir.join("target").join("debug").join(mcp_name));
        if let Some(parent) = manifest_dir.parent() {
            paths.push(
                parent
                    .join("src-tauri")
                    .join("target")
                    .join("debug")
                    .join(mcp_name),
            );
        }

        // 4. Common install locations
        #[cfg(target_os = "windows")]
        {
            if let Ok(local) = std::env::var("LOCALAPPDATA") {
                paths.push(
                    std::path::PathBuf::from(local)
                        .join("Programs")
                        .join("FeatureHub")
                        .join(mcp_name),
                );
            }
        }

        // 5. Check PATH via where/which
        if let Some(found) = which_command(mcp_name) {
            paths.push(std::path::PathBuf::from(found));
        }

        paths
    };

    for path in &candidates {
        if path.exists() {
            return Some(path.clone());
        }
    }

    eprintln!(
        "[fh_mcp_path] Could not find {} in any of: {:?}",
        mcp_name,
        candidates
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
    );
    None
}

#[tauri::command]
pub fn get_sessions(
    state: State<'_, AppState>,
    feature_id: String,
) -> Result<Vec<db::sessions::Session>, String> {
    let mut sessions = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::sessions::get_sessions(&conn, &feature_id)?
    }; // DB lock released before disk I/O

    // Auto-fill/refresh titles from Claude Code's on-disk files
    // Skip sessions that were manually renamed by the user
    let mut updates = Vec::new();
    for session in &mut sessions {
        if !session.title_manual && !session.claude_session_id.is_empty() {
            let (title, summary) = claude::scanner::find_session_title(&session.claude_session_id);
            if title.is_some() && title != session.title {
                updates.push((session.id.clone(), title.clone(), summary.clone()));
                session.title = title;
                session.summary = summary;
            }
        }
    }

    // Batch-persist discovered titles
    if !updates.is_empty() {
        if let Ok(conn) = state.db.lock() {
            for (id, title, summary) in &updates {
                db::sessions::update_session_title_summary(
                    &conn,
                    id,
                    title.as_deref(),
                    summary.as_deref(),
                )
                .ok();
            }
        }
    }

    Ok(sessions)
}

#[tauri::command]
pub async fn scan_sessions(
    feature_id: Option<String>,
) -> Result<Vec<claude::scanner::ScannedSession>, String> {
    let _ = feature_id; // reserved for future filtering
    tauri::async_runtime::spawn_blocking(|| claude::scanner::scan_claude_sessions())
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn check_session_active(session_id: String) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || claude::scanner::is_session_active(&session_id))
        .await
        .map_err(|e| e.to_string())
}

/// Returns a map of feature_id -> active session count (only features with >0 active sessions).
#[tauri::command]
pub async fn get_active_session_counts(
    state: State<'_, AppState>,
) -> Result<std::collections::HashMap<String, u32>, String> {
    let pairs = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::sessions::get_all_session_ids(&conn)?
    };

    // Process scanning off the main thread
    let active_ids = tauri::async_runtime::spawn_blocking(move || {
        let all_claude_ids: Vec<String> = pairs.iter().map(|(_, cid)| cid.clone()).collect();
        let active = claude::scanner::get_active_session_ids(&all_claude_ids);
        (pairs, active)
    })
    .await
    .map_err(|e| e.to_string())?;

    let (pairs, active_ids) = active_ids;
    let mut counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    for (feature_id, claude_id) in &pairs {
        if active_ids.contains(claude_id) {
            *counts.entry(feature_id.clone()).or_insert(0) += 1;
        }
    }

    Ok(counts)
}

/// Returns both per-feature active session counts AND the list of active claude_session_ids in one process scan.
#[tauri::command]
pub async fn get_active_session_activity(
    state: State<'_, AppState>,
) -> Result<SessionActivity, String> {
    // Read session IDs from DB, then release the lock before process scanning
    let pairs = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::sessions::get_all_session_ids(&conn)?
    };

    // Process scanning off the main thread (spawns PowerShell on Windows)
    let result = tauri::async_runtime::spawn_blocking(move || {
        let all_claude_ids: Vec<String> = pairs.iter().map(|(_, cid)| cid.clone()).collect();
        let active_ids = claude::scanner::get_active_session_ids(&all_claude_ids);
        let mut counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        for (feature_id, claude_id) in &pairs {
            if active_ids.contains(claude_id) {
                *counts.entry(feature_id.clone()).or_insert(0) += 1;
            }
        }
        SessionActivity {
            counts,
            active_session_ids: active_ids,
        }
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(result)
}

#[tauri::command]
pub fn link_session(
    state: State<'_, AppState>,
    feature_id: String,
    claude_session_id: String,
    title: Option<String>,
    summary: Option<String>,
    project_path: Option<String>,
    branch: Option<String>,
) -> Result<db::sessions::Session, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::sessions::link_session(
        &conn,
        &feature_id,
        &claude_session_id,
        title,
        summary,
        project_path,
        branch,
    )
}

#[tauri::command]
pub fn rename_session(state: State<'_, AppState>, id: String, title: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::sessions::rename_session(&conn, &id, &title)
}

#[tauri::command]
pub fn unlink_session(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::sessions::unlink_session(&conn, &id)
}

#[tauri::command]
pub fn resume_session(
    state: State<'_, AppState>,
    session_id: String,
    project_path: String,
    feature_id: Option<String>,
) -> Result<(), String> {
    let sp = state.storage_path.lock().map_err(|e| e.to_string())?;
    let storage_settings = match sp.as_ref() {
        Some(path) => crate::config::load_storage_settings(path).unwrap_or_default(),
        None => crate::config::StorageSettings::default(),
    };
    let all_servers = storage_settings.all_mcp_servers();
    let (servers, directories) = if let Some(ref fid) = feature_id {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let svrs = db::mcp_servers::resolve_servers_for_feature(&conn, fid, &all_servers)?;
        let dirs = db::directories::get_directories(&conn, fid)?;
        let storage_path = sp.as_ref().map(|p| p.as_path());
        let ready_dirs: Vec<String> = dirs
            .iter()
            .filter(|d| d.clone_status.as_deref().unwrap_or("ready") == "ready")
            .map(|d| match storage_path {
                Some(base) => crate::paths::resolve_path_string(&d.path, base),
                None => d.path.clone(),
            })
            .collect();
        (svrs, ready_dirs)
    } else {
        (all_servers, Vec::new())
    };

    claude::launcher::resume_session(&session_id, &project_path, &directories, &servers)
}

#[tauri::command]
pub fn ensure_mcp_config(
    _project_path: Option<String>,
    _claude_session_id: Option<String>,
) -> Result<(), String> {
    // MCP config is now passed via --mcp-config CLI flag when launching sessions,
    // so this command is a no-op. Kept for frontend compatibility.
    Ok(())
}

#[tauri::command]
pub fn start_new_session(
    state: State<'_, AppState>,
    feature_id: String,
    directories: Vec<String>,
    feature_title: String,
    context: Option<String>,
) -> Result<(), String> {
    let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
    let base = storage.as_deref().ok_or("No active storage path")?;
    let feature_dir =
        crate::files::manager::ensure_storage_dir(std::path::Path::new(base), &feature_id)?;

    // Generate a session ID upfront so we can track it
    let claude_session_id = uuid::Uuid::new_v4().to_string();

    // Resolve which MCP servers are enabled for this feature
    let storage_settings = crate::config::load_storage_settings(base).unwrap_or_default();
    let all_servers = storage_settings.all_mcp_servers();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut servers =
        db::mcp_servers::resolve_servers_for_feature(&conn, &feature_id, &all_servers)?;

    // Add the built-in featurehub MCP server with --feature and --session-id
    let settings = crate::config::load_settings().unwrap_or_default();
    if let Some(mcp_bin) = fh_mcp_path(&settings) {
        let mcp_bin_str = mcp_bin.to_string_lossy().replace('\\', "/");
        servers.insert(
            0,
            crate::config::McpServer {
                name: "featurehub".to_string(),
                command: mcp_bin_str,
                args: vec![
                    "--feature".to_string(),
                    feature_id.clone(),
                    "--session-id".to_string(),
                    claude_session_id.clone(),
                ],
                env: std::collections::HashMap::new(),
                default_enabled: true,
                url: None,
            },
        );
    }

    // Create session record in the DB with relative path
    let project_path_rel = crate::paths::to_storage_relative(&feature_dir.to_string_lossy(), base);
    db::sessions::create_cli_session(
        &conn,
        &feature_id,
        Some(project_path_rel),
        &claude_session_id,
    )?;
    drop(conn);

    claude::launcher::start_new_session(
        &feature_dir.to_string_lossy(),
        &directories,
        &feature_title,
        context.as_deref(),
        &servers,
        &claude_session_id,
    )
}

/// Resolve session title using fallback chain.
fn resolve_session_title(
    claude_session_id: &str,
    db_title: Option<&str>,
    feature_name: &str,
    project_dir_hint: Option<&std::path::Path>,
) -> (String, crate::claude::session_parser::TitleSource) {
    use crate::claude::session_parser::TitleSource;

    // If DB has a title and it's not garbage, use it
    if let Some(t) = db_title {
        if !t.is_empty() && !crate::claude::session_parser::is_bad_title(t) {
            return (t.to_string(), TitleSource::FirstPrompt);
        }
    }

    if let Some(project_dir) = project_dir_hint {
        if let Some(resolved) =
            resolve_session_title_from_project_dir(project_dir, claude_session_id)
        {
            return resolved;
        }
    }

    // Try sessions-index.json
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => {
            return (
                format!("{} Session", feature_name),
                TitleSource::FeatureName,
            )
        }
    };
    let projects_dir = home.join(".claude").join("projects");

    for entry in std::fs::read_dir(&projects_dir)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
    {
        let project_dir = entry.path();
        if !project_dir.is_dir() {
            continue;
        }

        if let Some(resolved) =
            resolve_session_title_from_project_dir(&project_dir, claude_session_id)
        {
            return resolved;
        }
    }

    // Fallback to feature name
    (
        format!("{} Session", feature_name),
        TitleSource::FeatureName,
    )
}

fn resolve_session_title_from_project_dir(
    project_dir: &std::path::Path,
    claude_session_id: &str,
) -> Option<(String, crate::claude::session_parser::TitleSource)> {
    use crate::claude::session_parser::TitleSource;

    let (index_title, first_prompt) =
        crate::claude::session_parser::find_title_in_sessions_index(project_dir, claude_session_id);

    if let Some(title) = index_title {
        if !crate::claude::session_parser::is_bad_title(&title) {
            return Some((title, TitleSource::SessionsIndex));
        }
    }

    if let Some(prompt) = first_prompt {
        if !crate::claude::session_parser::is_bad_title(&prompt) {
            return Some((prompt, TitleSource::FirstPrompt));
        }
    }

    None
}

/// Resolve session status from multiple signals.
fn resolve_session_status(
    is_active: bool,
    status_hint: &crate::claude::session_parser::ParsedStatusHint,
    no_session_id: bool,
    has_ended: bool,
) -> crate::claude::session_parser::SessionStatus {
    use crate::claude::session_parser::SessionStatus;

    if no_session_id {
        return SessionStatus::Lost;
    }

    if is_active {
        if status_hint.looks_like_waiting {
            return SessionStatus::WaitingForInput;
        }
        return SessionStatus::Active;
    }

    if has_ended {
        return SessionStatus::Finished;
    }

    SessionStatus::Idle
}

/// Returns all sessions enriched with JSONL stats (model, tokens, cost) for the sessions panel.
/// Stats are cached by JSONL file mtime and only re-parsed when the file changes.
#[tauri::command]
pub async fn get_sessions_panel_data(
    state: State<'_, AppState>,
) -> Result<SessionsPanelData, String> {
    // 1. Read all sessions with feature names from DB
    let rows = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::sessions::get_all_sessions_for_panel(&conn)?
    };

    // 2. Determine which sessions are active via process scan
    let claude_ids: Vec<String> = rows.iter().map(|r| r.claude_session_id.clone()).collect();
    let active_ids: std::collections::HashSet<String> = {
        let ids = tauri::async_runtime::spawn_blocking(move || {
            claude::scanner::get_active_session_ids(&claude_ids)
        })
        .await
        .map_err(|e| e.to_string())?;
        ids.into_iter().collect()
    };

    // 3. Locate JSONL files, check mtime cache, collect what needs re-parsing
    let home = dirs::home_dir().ok_or("Cannot find home directory")?;
    let projects_dir = home.join(".claude").join("projects");

    // Phase A — collect cached paths + mtimes, only scanning Claude project dirs on cache miss.
    let path_map: Vec<(String, Option<(std::path::PathBuf, std::time::SystemTime)>)> = {
        let mut cache = state.jsonl_path_cache.lock().map_err(|e| e.to_string())?;
        rows.iter()
            .map(|row| {
                let cached_path = cache
                    .get(&row.claude_session_id)
                    .filter(|path| path.exists())
                    .cloned();
                let path = cached_path.or_else(|| {
                    let found = claude::scanner::find_jsonl_for_session(
                        &projects_dir,
                        &row.claude_session_id,
                    );
                    if let Some(path) = &found {
                        cache.insert(row.claude_session_id.clone(), path.clone());
                    }
                    found
                });
                let info = path.and_then(|path| {
                    let mtime = std::fs::metadata(&path)
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    Some((path, mtime))
                });
                (row.claude_session_id.clone(), info)
            })
            .collect()
    };
    let path_info_by_id: std::collections::HashMap<
        String,
        (std::path::PathBuf, std::time::SystemTime),
    > = path_map
        .iter()
        .filter_map(|(id, info)| info.clone().map(|info| (id.clone(), info)))
        .collect();

    // Phase B — cache lookup with lock (memory only, no I/O)
    let mut to_parse: Vec<(String, std::path::PathBuf)> = Vec::new();
    let mut all_stats: std::collections::HashMap<
        String,
        crate::claude::session_parser::CachedStats,
    > = {
        let cache = state.stats_cache.lock().map_err(|e| e.to_string())?;
        let mut hits = std::collections::HashMap::new();
        for (id, info) in &path_map {
            if let Some((path, mtime)) = info {
                if let Some(cached) = cache.get(id) {
                    if &cached.mtime == mtime {
                        hits.insert(id.clone(), cached.clone());
                        continue;
                    }
                }
                to_parse.push((id.clone(), path.clone()));
            }
        }
        hits
    }; // lock released here — only hash lookups held the lock

    // 4. Parse uncached JSONL files in a blocking task
    if !to_parse.is_empty() {
        let parsed: Vec<(String, crate::claude::session_parser::CachedStats)> =
            tauri::async_runtime::spawn_blocking(move || {
                to_parse
                    .into_iter()
                    .map(|(id, path)| {
                        let stats = crate::claude::session_parser::parse_session_stats(&path);
                        (id, stats)
                    })
                    .collect()
            })
            .await
            .map_err(|e| e.to_string())?;

        // Update cache and merge into all_stats
        let mut cache = state.stats_cache.lock().map_err(|e| e.to_string())?;
        for (id, stats) in parsed {
            cache.insert(id.clone(), stats.clone());
            all_stats.insert(id, stats);
        }
    }

    let mut status_hints: std::collections::HashMap<
        String,
        crate::claude::session_parser::ParsedStatusHint,
    > = std::collections::HashMap::new();
    {
        let mut cache = state.status_hint_cache.lock().map_err(|e| e.to_string())?;
        for row in &rows {
            let Some((path, mtime)) = path_info_by_id.get(&row.claude_session_id) else {
                continue;
            };
            let cached = cache.get(&row.claude_session_id);
            if let Some(cached) = cached {
                if &cached.mtime == mtime {
                    status_hints.insert(row.claude_session_id.clone(), cached.hint.clone());
                    continue;
                }
            }
            let is_active = active_ids.contains(&row.claude_session_id);
            if is_active || cached.is_some() {
                let hint = cached_status_hint(&mut cache, &row.claude_session_id, *mtime, || {
                    crate::claude::session_parser::parse_status_hint(path)
                });
                status_hints.insert(row.claude_session_id.clone(), hint);
            }
        }
    }

    // 5. Build PanelSession list with title resolution
    let mut sessions: Vec<PanelSession> = rows
        .iter()
        .map(|row| {
            let is_active = active_ids.contains(&row.claude_session_id);
            let stats = all_stats.get(&row.claude_session_id);
            // Resolve title with fallback chain
            let (title, title_source) = resolve_session_title(
                &row.claude_session_id,
                row.title.as_deref(),
                &row.feature_name,
                path_info_by_id
                    .get(&row.claude_session_id)
                    .and_then(|(path, _)| path.parent()),
            );

            let status_hint = status_hints
                .get(&row.claude_session_id)
                .cloned()
                .unwrap_or_default();

            // Determine session status
            let status = resolve_session_status(
                is_active,
                &status_hint,
                row.claude_session_id.is_empty(),
                false, // ended_at not in PanelSessionRow yet
            );

            let last_activity = stats
                .map(|s| {
                    let dt: chrono::DateTime<chrono::Utc> = s.mtime.into();
                    dt.to_rfc3339()
                })
                .unwrap_or_else(|| row.linked_at.clone());

            PanelSession {
                id: row.id.clone(),
                feature_id: row.feature_id.clone(),
                feature_name: row.feature_name.clone(),
                claude_session_id: row.claude_session_id.clone(),
                title,
                title_source,
                branch: row.branch.clone(),
                last_activity,
                started_at: None,
                ended_at: None,
                is_active,
                status,
                last_action: status_hint.last_action,
                model: stats.and_then(|s| s.model.clone()),
                total_tokens: stats.map(|s| s.total_tokens).filter(|&t| t > 0),
                context_tokens: stats.and_then(|s| s.context_tokens),
                cost_usd: stats.and_then(|s| s.cost_usd),
            }
        })
        .collect();

    // Sort: active first, then by last_activity descending
    sessions.sort_by(|a, b| {
        b.is_active
            .cmp(&a.is_active)
            .then_with(|| b.last_activity.cmp(&a.last_activity))
    });

    let active_count = sessions.iter().filter(|s| s.is_active).count();

    Ok(SessionsPanelData {
        sessions,
        active_count,
    })
}
