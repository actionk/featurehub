use tauri::State;

use crate::claude;
use crate::db;
use crate::terminal::TerminalState;
use crate::AppState;

use super::sessions::fh_mcp_path;

#[tauri::command]
pub fn pty_spawn(
    terminal_state: State<'_, TerminalState>,
    app: tauri::AppHandle,
    feature_id: String,
    shell: Option<String>,
    args: Option<Vec<String>>,
    cols: u16,
    rows: u16,
    cwd: Option<String>,
) -> Result<String, String> {
    crate::terminal::spawn_pty(&app, &terminal_state, &feature_id, shell, args, cols, rows, cwd)
}

#[tauri::command]
pub fn pty_write(
    terminal_state: State<'_, TerminalState>,
    id: String,
    data: String,
) -> Result<(), String> {
    crate::terminal::write_pty(&terminal_state, &id, &data)
}

#[tauri::command]
pub fn pty_resize(
    terminal_state: State<'_, TerminalState>,
    id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    crate::terminal::resize_pty(&terminal_state, &id, cols, rows)
}

#[tauri::command]
pub fn pty_kill(
    terminal_state: State<'_, TerminalState>,
    id: String,
) -> Result<(), String> {
    crate::terminal::kill_pty(&terminal_state, &id)
}

#[tauri::command]
pub fn pty_kill_feature(
    terminal_state: State<'_, TerminalState>,
    feature_id: String,
) -> Result<(), String> {
    crate::terminal::kill_all_for_feature(&terminal_state, &feature_id)
}

#[tauri::command]
pub fn pty_spawn_session(
    state: State<'_, AppState>,
    terminal_state: State<'_, TerminalState>,
    app: tauri::AppHandle,
    feature_id: String,
    directories: Vec<String>,
    feature_title: String,
    context: Option<String>,
    dangerously_skip_permissions: Option<bool>,
    cols: u16,
    rows: u16,
) -> Result<serde_json::Value, String> {
    let storage = state.storage_path.lock().map_err(|e| e.to_string())?;
    let base = storage.as_deref().ok_or("No active storage path")?;
    let feature_dir = crate::files::manager::ensure_storage_dir(std::path::Path::new(base), &feature_id)?;

    let claude_session_id = uuid::Uuid::new_v4().to_string();

    let storage_settings = crate::config::load_storage_settings(base).unwrap_or_default();
    let all_servers = storage_settings.all_mcp_servers();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut servers = db::mcp_servers::resolve_servers_for_feature(&conn, &feature_id, &all_servers)?;

    let settings = crate::config::load_settings().unwrap_or_default();
    if let Some(mcp_bin) = fh_mcp_path(&settings) {
        let mcp_bin_str = mcp_bin.to_string_lossy().replace('\\', "/");
        servers.insert(0, crate::config::McpServer {
            name: "featurehub".to_string(),
            command: mcp_bin_str,
            args: vec!["--feature".to_string(), feature_id.clone(), "--session-id".to_string(), claude_session_id.clone()],
            env: std::collections::HashMap::new(),
            default_enabled: true,
            url: None,
        });
    } else {
        eprintln!("[pty_spawn_session] WARNING: fh-mcp binary not found — FeatureHub MCP server will not be available in this session. Run 'cargo build' or install CLI via Settings.");
    }

    eprintln!("[pty_spawn_session] MCP servers configured: {:?}", servers.iter().map(|s| &s.name).collect::<Vec<_>>());

    let project_path_rel = crate::paths::to_storage_relative(&feature_dir.to_string_lossy(), base);
    let session_db_id = db::sessions::create_cli_session(&conn, &feature_id, Some(project_path_rel), &claude_session_id)?;
    drop(conn);

    let (program, args, cwd) = claude::launcher::build_new_session_args(
        &feature_dir.to_string_lossy(),
        &directories,
        &feature_title,
        context.as_deref(),
        &servers,
        &claude_session_id,
        dangerously_skip_permissions.unwrap_or(false),
    )?;

    {
        let mut trust_dirs: Vec<&str> = Vec::with_capacity(directories.len() + 1);
        trust_dirs.push(cwd.as_str());
        for d in &directories {
            trust_dirs.push(d.as_str());
        }
        claude::trust::accept_dirs(&trust_dirs);
    }

    eprintln!("[pty_spawn_session] Spawning: {} {}", program, args.join(" "));

    let terminal_id = crate::terminal::spawn_pty(
        &app,
        &terminal_state,
        &feature_id,
        Some(program),
        Some(args),
        cols,
        rows,
        Some(cwd),
    )?;

    // Store session metadata so terminals can be restored after webview reload
    let label = format!("Session {}", 1);
    crate::terminal::set_session_metadata(&terminal_state, &terminal_id, &session_db_id, &label)?;

    Ok(serde_json::json!({
        "terminalId": terminal_id,
        "sessionDbId": session_db_id,
        "claudeSessionId": claude_session_id,
    }))
}

#[tauri::command]
pub fn pty_resume_session(
    state: State<'_, AppState>,
    terminal_state: State<'_, TerminalState>,
    app: tauri::AppHandle,
    session_db_id: String,
    cols: u16,
    rows: u16,
) -> Result<serde_json::Value, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let session = db::sessions::get_session(&conn, &session_db_id)?;

    let claude_session_id = &session.claude_session_id;
    if claude_session_id.is_empty() {
        return Err("Session has no claude_session_id".to_string());
    }
    let project_path_raw = session.project_path
        .as_deref()
        .ok_or("Session has no project_path")?;

    let sp = state.storage_path.lock().map_err(|e| e.to_string())?;
    // Resolve relative project_path to absolute
    let project_path_resolved = match sp.as_ref() {
        Some(base) => crate::paths::resolve_path_string(project_path_raw, base),
        None => project_path_raw.to_string(),
    };
    let project_path = project_path_resolved.as_str();
    let storage_settings = match sp.as_ref() {
        Some(path) => crate::config::load_storage_settings(path).unwrap_or_default(),
        None => crate::config::StorageSettings::default(),
    };
    let all_servers = storage_settings.all_mcp_servers();
    let mut servers = db::mcp_servers::resolve_servers_for_feature(&conn, &session.feature_id, &all_servers)?;

    let settings = crate::config::load_settings().unwrap_or_default();
    if let Some(mcp_bin) = fh_mcp_path(&settings) {
        let mcp_bin_str = mcp_bin.to_string_lossy().replace('\\', "/");
        servers.insert(0, crate::config::McpServer {
            name: "featurehub".to_string(),
            command: mcp_bin_str,
            args: vec![
                "--feature".to_string(),
                session.feature_id.clone(),
                "--session-id".to_string(),
                claude_session_id.clone(),
            ],
            env: std::collections::HashMap::new(),
            default_enabled: true,
            url: None,
        });
    } else {
        eprintln!("[pty_resume_session] WARNING: fh-mcp binary not found — FeatureHub MCP server will not be available in this session.");
    }

    eprintln!("[pty_resume_session] MCP servers configured: {:?}", servers.iter().map(|s| &s.name).collect::<Vec<_>>());

    let dirs = db::directories::get_directories(&conn, &session.feature_id)?;
    let ready_dirs: Vec<String> = dirs.iter()
        .filter(|d| d.clone_status.as_deref().unwrap_or("ready") == "ready")
        .map(|d| match sp.as_ref() {
            Some(base) => crate::paths::resolve_path_string(&d.path, base),
            None => d.path.clone(),
        })
        .collect();
    drop(conn);
    drop(sp);

    let (program, args, cwd) = claude::launcher::build_resume_args(
        claude_session_id,
        project_path,
        &ready_dirs,
        &servers,
    )?;

    {
        let mut trust_dirs: Vec<&str> = Vec::with_capacity(ready_dirs.len() + 1);
        trust_dirs.push(cwd.as_str());
        for d in &ready_dirs {
            trust_dirs.push(d.as_str());
        }
        claude::trust::accept_dirs(&trust_dirs);
    }

    let terminal_id = crate::terminal::spawn_pty(
        &app,
        &terminal_state,
        &session.feature_id,
        Some(program),
        Some(args),
        cols,
        rows,
        Some(cwd),
    )?;

    // Stamp started_at so sorting reflects when the session was last resumed
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE sessions SET started_at = ?1 WHERE id = ?2",
            rusqlite::params![now, session_db_id],
        ).ok();
    }

    // Store session metadata so terminals can be restored after webview reload
    let label = session.title.as_deref().unwrap_or("Resumed Session").to_string();
    crate::terminal::set_session_metadata(&terminal_state, &terminal_id, &session_db_id, &label)?;

    Ok(serde_json::json!({
        "terminalId": terminal_id,
        "sessionDbId": session_db_id,
    }))
}

#[tauri::command]
pub fn finish_embedded_session(
    state: State<'_, AppState>,
    session_db_id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::sessions::finish_session(&conn, &session_db_id, None, None, None)
}

#[tauri::command]
pub fn pty_get_scrollback(
    terminal_state: State<'_, TerminalState>,
    id: String,
) -> Result<String, String> {
    crate::terminal::get_scrollback(&terminal_state, &id)
}

#[tauri::command]
pub fn pty_list_active(
    terminal_state: State<'_, TerminalState>,
) -> Result<Vec<crate::terminal::ActiveTerminalInfo>, String> {
    crate::terminal::list_active(&terminal_state)
}

/// Clean up sessions that were left open (started_at set, ended_at null)
/// but whose claude process is no longer running. Happens on app reload.
#[tauri::command]
pub fn cleanup_orphaned_sessions(state: State<'_, AppState>) -> Result<u32, String> {
    // Step 1: Get session IDs from DB, then release the lock
    let pairs = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::sessions::get_all_session_ids(&conn)?
    };

    if pairs.is_empty() {
        return Ok(0);
    }

    // Step 2: Scan processes ONCE (expensive on Windows — spawns PowerShell)
    let all_claude_ids: Vec<String> = pairs.iter().map(|(_, cid)| cid.clone()).collect();
    let active_ids = claude::scanner::get_active_session_ids(&all_claude_ids);

    // Step 3: Re-acquire lock and clean up orphaned sessions
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut cleaned = 0u32;

    for (_, claude_id) in &pairs {
        if !active_ids.contains(claude_id) {
            let result: Result<String, _> = conn.query_row(
                "SELECT id FROM sessions WHERE claude_session_id = ?1 AND started_at IS NOT NULL AND ended_at IS NULL",
                rusqlite::params![claude_id],
                |row| row.get(0),
            );
            if let Ok(session_id) = result {
                let _ = db::sessions::finish_session(&conn, &session_id, None, None, None);
                cleaned += 1;
            }
        }
    }

    Ok(cleaned)
}
