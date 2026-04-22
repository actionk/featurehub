use clap::{Parser, Subcommand};
use feature_hub::config;
use feature_hub::db;
use inquire::ui::{Attributes, Color, RenderConfig, StyleSheet, Styled};
use inquire::{InquireError, Select, Text};
use rusqlite::Connection;
use std::fmt;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(name = "fh", about = "FeatureHub CLI — launch Claude sessions linked to features")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a Claude Code session linked to a feature
    Start {
        /// Feature title (fuzzy match) or ID prefix
        feature: String,
        /// Optional initial prompt/context for Claude
        #[arg(short, long)]
        prompt: Option<String>,
    },
    /// Resume a previous Claude Code session
    Resume {
        /// Claude session ID
        session_id: String,
    },
    /// List features
    List {
        /// Filter by status (e.g. active, done, blocked, paused, todo)
        #[arg(short, long)]
        filter: Option<String>,
    },
}

fn open_db() -> Result<Connection, String> {
    let db_path = config::get_active_db_path()?;
    if !db_path.exists() {
        return Err(format!(
            "Database not found at {}. Open FeatureHub to create a storage first.",
            db_path.display()
        ));
    }
    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;
    db::initialize(&conn).map_err(|e| format!("Failed to initialize database: {}", e))?;
    // Migrate absolute paths to relative
    if let Ok(storage_path) = config::get_active_storage_path() {
        db::migrate_to_relative_paths(&conn, &storage_path);
    }
    Ok(conn)
}

fn find_feature(
    conn: &Connection,
    query: &str,
) -> Result<db::features::FeatureSummary, String> {
    let features = db::features::get_features(conn, None, None)?;

    // Try exact ID prefix match first
    if let Some(f) = features.iter().find(|f| f.id.starts_with(query)) {
        return Ok(f.clone());
    }

    // Fuzzy title match: case-insensitive contains
    let query_lower = query.to_lowercase();
    let matches: Vec<_> = features
        .iter()
        .filter(|f| f.title.to_lowercase().contains(&query_lower))
        .collect();

    match matches.len() {
        0 => Err(format!("No feature found matching '{}'", query)),
        1 => Ok(matches[0].clone()),
        _ => {
            let mut msg = format!(
                "Multiple features match '{}'. Be more specific:\n",
                query
            );
            for f in &matches {
                msg.push_str(&format!(
                    "  [{}] {} ({})\n",
                    &f.id[..8],
                    f.title,
                    f.status
                ));
            }
            Err(msg)
        }
    }
}

/// URL-encode a path the same way Claude Code does for project directories.
fn encode_path_for_claude(path: &str) -> String {
    let mut result = String::new();
    for byte in path.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

/// Check whether Claude actually created a transcript file for this session.
/// If no JSONL file exists, the session was never used (user exited without sending a message).
fn session_has_transcript(project_path: &str, session_id: &str) -> bool {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return false,
    };
    let encoded = encode_path_for_claude(project_path);
    let projects_dir = home.join(".claude").join("projects").join(&encoded);
    let jsonl_path = projects_dir.join(format!("{}.jsonl", session_id));
    jsonl_path.exists()
}

/// Look up the session summary for a known session ID.
/// Tries sessions-index.json first, then session-memory .md, then JSONL transcript.
fn find_session_summary(project_path: &str, session_id: &str) -> (Option<String>, Option<String>) {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return (None, None),
    };
    let encoded = encode_path_for_claude(project_path);
    let projects_dir = home.join(".claude").join("projects").join(&encoded);

    // Try sessions-index.json first (Claude Code's own title)
    let (index_title, _) = feature_hub::claude::session_parser::find_title_in_sessions_index(&projects_dir, session_id);
    if index_title.is_some() {
        return (index_title, None);
    }

    let summary_path = projects_dir
        .join("session-memory")
        .join(format!("{}.md", session_id));

    if summary_path.exists() {
        let result = parse_summary_file(&summary_path);
        if result.0.is_some() {
            return result;
        }
    }

    // Fall back to parsing the first user message from the JSONL transcript
    let jsonl_path = projects_dir.join(format!("{}.jsonl", session_id));
    if jsonl_path.exists() {
        let title = feature_hub::claude::session_parser::parse_title_from_jsonl(&jsonl_path);
        return (title, None);
    }

    (None, None)
}

fn parse_summary_file(path: &PathBuf) -> (Option<String>, Option<String>) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (None, None),
    };

    if content.trim().is_empty() {
        return (None, None);
    }

    let lines: Vec<&str> = content.lines().collect();

    let title = lines
        .iter()
        .find(|line| !line.trim().is_empty())
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                trimmed.trim_start_matches('#').trim().to_string()
            } else {
                trimmed.to_string()
            }
        });

    let summary = if lines.len() > 1 {
        let body: String = lines
            .iter()
            .skip(1)
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect::<Vec<&str>>()
            .join(" ");
        if body.is_empty() {
            None
        } else if body.len() > 500 {
            Some(format!("{}...", &body[..497]))
        } else {
            Some(body)
        }
    } else {
        None
    };

    (title, summary)
}

fn cmd_list(filter: Option<String>) -> Result<(), String> {
    let conn = open_db()?;
    let features = db::features::get_features(&conn, filter, None)?;

    if features.is_empty() {
        println!("No features found.");
        return Ok(());
    }

    let max_title = features
        .iter()
        .map(|f| f.title.len())
        .max()
        .unwrap_or(5)
        .min(50);

    println!(
        "{:<8}  {:<width$}  {:<12}  {}",
        "ID",
        "TITLE",
        "STATUS",
        "TAGS",
        width = max_title
    );
    println!("{}", "-".repeat(8 + 2 + max_title + 2 + 12 + 2 + 20));

    for f in &features {
        let title_display = if f.title.len() > 50 {
            format!("{}...", &f.title[..47])
        } else {
            f.title.clone()
        };
        let tags: Vec<_> = f.tags.iter().map(|t| t.name.as_str()).collect();
        println!(
            "{:<8}  {:<width$}  {:<12}  {}",
            &f.id[..8],
            title_display,
            f.status,
            tags.join(", "),
            width = max_title
        );
    }

    Ok(())
}

/// Determine the path to the fh-mcp binary (sibling of the current executable).
fn fh_mcp_path() -> Option<PathBuf> {
    let current_exe = std::env::current_exe().ok()?;
    let dir = current_exe.parent()?;
    let mcp_name = if cfg!(windows) { "fh-mcp.exe" } else { "fh-mcp" };
    let mcp_path = dir.join(mcp_name);
    if mcp_path.exists() {
        Some(mcp_path)
    } else {
        None
    }
}

/// Build a temporary MCP config JSON file containing the featurehub server and
/// any user-configured MCP servers (filtered per-feature). Returns CLI args to pass to `claude`.
fn build_mcp_args(feature_id: &str, claude_session_id: &str, conn: &rusqlite::Connection) -> Vec<String> {
    let mut mcp_servers = serde_json::Map::new();

    // Add featurehub MCP server
    if let Some(mcp_bin) = fh_mcp_path() {
        let mcp_bin_str = mcp_bin.to_string_lossy().replace('\\', "/");
        mcp_servers.insert("featurehub".to_string(), serde_json::json!({
            "command": mcp_bin_str,
            "args": ["--feature", feature_id, "--session-id", claude_session_id]
        }));
    }

    // Add user-configured + extension MCP servers, filtered by per-feature overrides
    if let Ok(storage_path) = config::get_active_storage_path() {
        let storage_settings = config::load_storage_settings(&storage_path).unwrap_or_default();
        let all_servers = storage_settings.all_mcp_servers();
        let resolved = db::mcp_servers::resolve_servers_for_feature(conn, feature_id, &all_servers)
            .unwrap_or_else(|_| all_servers);
        for server in &resolved {
            let mut entry = if let Some(ref url) = server.url {
                serde_json::json!({ "type": "http", "url": url })
            } else {
                serde_json::json!({
                    "command": server.command,
                    "args": server.args,
                })
            };
            if !server.env.is_empty() {
                if let Some(obj) = entry.as_object_mut() {
                    if let Ok(env_val) = serde_json::to_value(&server.env) {
                        obj.insert("env".to_string(), env_val);
                    }
                }
            }
            mcp_servers.insert(server.name.clone(), entry);
        }
    }

    if mcp_servers.is_empty() {
        return Vec::new();
    }

    let config_obj = serde_json::json!({ "mcpServers": mcp_servers });
    match config::config_dir() {
        Ok(dir) => {
            let path = dir.join("session-mcp.json");
            match serde_json::to_string_pretty(&config_obj) {
                Ok(data) => {
                    if let Err(e) = std::fs::write(&path, &data) {
                        eprintln!("Warning: Failed to write MCP config: {}", e);
                        return Vec::new();
                    }
                    vec!["--mcp-config".to_string(), path.to_string_lossy().to_string()]
                }
                Err(e) => {
                    eprintln!("Warning: Failed to serialize MCP config: {}", e);
                    Vec::new()
                }
            }
        }
        Err(e) => {
            eprintln!("Warning: Failed to get config dir: {}", e);
            Vec::new()
        }
    }
}

fn get_feature_dir(feature_id: &str) -> Result<PathBuf, String> {
    let db_path = config::get_active_db_path()?;
    let storage_dir = db_path.parent().ok_or("Failed to get storage directory")?;
    let feature_dir = storage_dir.join("workspaces").join(feature_id);
    std::fs::create_dir_all(&feature_dir)
        .map_err(|e| format!("Failed to create feature directory: {}", e))?;
    Ok(feature_dir)
}

/// Core logic for starting a Claude session on a feature.
/// Used by both `cmd_start` (CLI subcommand) and `cmd_interactive` (interactive menu).
fn cmd_start_feature(feature: &db::features::FeatureSummary, prompt: Option<String>) -> Result<(), String> {
    let conn = open_db()?;

    println!("Feature: {} [{}]", feature.title, feature.status);

    // Get feature directories
    let directories = db::directories::get_directories(&conn, &feature.id)?;

    // Use the per-feature storage directory as the working directory
    let feature_dir = get_feature_dir(&feature.id)?;
    let work_dir = feature_dir.to_string_lossy().to_string();

    // Store relative project_path in DB
    let storage_path = config::get_active_storage_path()?;
    let work_dir_rel = feature_hub::paths::to_storage_relative(&work_dir, &storage_path);

    // Generate a session ID upfront and pass it to Claude via --session-id
    let claude_session_id = uuid::Uuid::new_v4().to_string();

    // Create session record with the known ID immediately
    let session_id =
        db::sessions::create_cli_session(&conn, &feature.id, Some(work_dir_rel), &claude_session_id)?;

    println!("Starting Claude in {}...", work_dir);
    println!("Session ID: {}", &claude_session_id[..8]);

    // Build claude args
    let mut args: Vec<String> = vec![
        "--session-id".to_string(),
        claude_session_id.clone(),
    ];

    // Add MCP config (featurehub + user-configured servers, filtered per-feature)
    args.extend(build_mcp_args(&feature.id, &claude_session_id, &conn));

    // Auto-approve FeatureHub MCP tools + git readonly
    feature_hub::claude::launcher::push_default_allowed_tools(&mut args);

    // Grant Claude access to all ready repositories
    let ready_dirs: Vec<_> = directories.iter().filter(|d| {
        d.clone_status.as_deref().unwrap_or("ready") == "ready"
    }).collect();
    let not_ready: Vec<_> = directories.iter().filter(|d| {
        d.clone_status.as_deref().unwrap_or("ready") != "ready"
    }).collect();
    if !not_ready.is_empty() {
        for d in &not_ready {
            let status = d.clone_status.as_deref().unwrap_or("unknown");
            let name = d.label.as_deref().unwrap_or(&d.path);
            eprintln!("Warning: {} is {} — skipping", name, status);
        }
    }
    for dir in &ready_dirs {
        let resolved = feature_hub::paths::resolve_path_string(&dir.path, &storage_path);
        args.push("--add-dir".to_string());
        args.push(resolved);
    }

    if let Some(p) = prompt {
        // Pass prompt as positional argument
        args.push(p);
    }

    {
        let mut trust_dirs: Vec<&str> = Vec::with_capacity(ready_dirs.len() + 1);
        trust_dirs.push(work_dir.as_str());
        let resolved_dirs: Vec<String> = ready_dirs.iter()
            .map(|d| feature_hub::paths::resolve_path_string(&d.path, &storage_path))
            .collect();
        for d in &resolved_dirs {
            trust_dirs.push(d.as_str());
        }
        feature_hub::claude::trust::accept_dirs(&trust_dirs);
    }

    // Spawn claude as a child process inheriting stdin/stdout/stderr
    let status = Command::new("claude")
        .args(&args)
        .current_dir(&work_dir)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map_err(|e| format!("Failed to launch claude: {}", e))?;

    println!(
        "\nClaude exited with {}",
        if status.success() {
            "success".to_string()
        } else {
            format!("code {}", status.code().unwrap_or(-1))
        }
    );

    // Try to parse session summary
    let (title, summary) = find_session_summary(&work_dir, &claude_session_id);

    // Re-open DB (the connection may have been idle a long time)
    let conn = open_db()?;

    // If the session has no title/summary, check if Claude actually created any transcript.
    // When the user exits without sending a single message, Claude won't create a JSONL file
    // or index entry — in that case, automatically unlink the empty session.
    if title.is_none() && summary.is_none() && !session_has_transcript(&work_dir, &claude_session_id) {
        db::sessions::unlink_session(&conn, &session_id)?;
        println!("Session was empty — automatically unlinked.");
    } else {
        db::sessions::finish_session(
            &conn,
            &session_id,
            Some(&claude_session_id),
            title.as_deref(),
            summary.as_deref(),
        )?;
        println!("Session recorded: {}", &claude_session_id[..8]);
    }

    Ok(())
}

fn cmd_start(feature_query: &str, prompt: Option<String>) -> Result<(), String> {
    let conn = open_db()?;
    let feature = find_feature(&conn, feature_query)?;
    cmd_start_feature(&feature, prompt)
}

fn cmd_resume(session_id: &str) -> Result<(), String> {
    let conn = open_db()?;

    // Look up the session to find its project_path and feature_id
    // Query directly since we need to search across all features
    let (_project_path, claude_session_id, feature_id): (Option<String>, String, String) = conn
        .query_row(
            "SELECT project_path, claude_session_id, feature_id FROM sessions WHERE claude_session_id = ?1 OR id = ?1",
            rusqlite::params![session_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|_| format!("Session '{}' not found in database", session_id))?;

    cmd_resume_session(&claude_session_id, &feature_id)
}

/// Core logic for resuming a Claude session. Used by both `cmd_resume` and interactive mode.
fn cmd_resume_session(claude_session_id: &str, feature_id: &str) -> Result<(), String> {
    let conn = open_db()?;

    // Use the per-feature storage directory as the working directory
    let feature_dir = get_feature_dir(feature_id)?;
    let work_dir = feature_dir.to_string_lossy().to_string();

    println!("Resuming session {} in {}...", &claude_session_id[..8.min(claude_session_id.len())], work_dir);

    // Build claude args with MCP config
    let mut args = vec!["--resume".to_string(), claude_session_id.to_string()];
    args.extend(build_mcp_args(feature_id, claude_session_id, &conn));

    // Auto-approve FeatureHub MCP tools + git readonly
    feature_hub::claude::launcher::push_default_allowed_tools(&mut args);

    // Grant Claude access to all ready repositories (resolve relative paths)
    let storage_path = config::get_active_storage_path()?;
    let directories = db::directories::get_directories(&conn, feature_id)?;
    let mut resolved_dirs: Vec<String> = Vec::new();
    for dir in &directories {
        if dir.clone_status.as_deref().unwrap_or("ready") == "ready" {
            let resolved = feature_hub::paths::resolve_path_string(&dir.path, &storage_path);
            args.push("--add-dir".to_string());
            args.push(resolved.clone());
            resolved_dirs.push(resolved);
        }
    }

    {
        let mut trust_dirs: Vec<&str> = Vec::with_capacity(resolved_dirs.len() + 1);
        trust_dirs.push(work_dir.as_str());
        for d in &resolved_dirs {
            trust_dirs.push(d.as_str());
        }
        feature_hub::claude::trust::accept_dirs(&trust_dirs);
    }

    let status = Command::new("claude")
        .args(&args)
        .current_dir(&work_dir)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map_err(|e| format!("Failed to launch claude: {}", e))?;

    println!(
        "\nClaude exited with {}",
        if status.success() {
            "success".to_string()
        } else {
            format!("code {}", status.code().unwrap_or(-1))
        }
    );

    Ok(())
}

// --- Render config ---

fn render_config() -> RenderConfig<'static> {
    RenderConfig::default()
        .with_prompt_prefix(Styled::new("?").with_style_sheet(
            StyleSheet::new().with_fg(Color::LightMagenta).with_attr(Attributes::BOLD),
        ))
        .with_answered_prompt_prefix(Styled::new("✓").with_style_sheet(
            StyleSheet::new().with_fg(Color::LightGreen).with_attr(Attributes::BOLD),
        ))
        .with_highlighted_option_prefix(Styled::new("›").with_style_sheet(
            StyleSheet::new().with_fg(Color::LightCyan).with_attr(Attributes::BOLD),
        ))
        .with_help_message(StyleSheet::new().with_fg(Color::DarkGrey))
        .with_text_input(StyleSheet::new().with_fg(Color::LightCyan))
        .with_answer(StyleSheet::new().with_fg(Color::LightGreen).with_attr(Attributes::BOLD))
        .with_scroll_up_prefix(Styled::new("↑").with_style_sheet(
            StyleSheet::new().with_fg(Color::DarkGrey),
        ))
        .with_scroll_down_prefix(Styled::new("↓").with_style_sheet(
            StyleSheet::new().with_fg(Color::DarkGrey),
        ))
}

// --- Interactive mode ---

fn status_symbol(status: &str) -> &'static str {
    match status {
        "active" | "in_progress" => "●",
        "todo" => "○",
        "blocked" => "✕",
        "paused" => "◑",
        "done" => "✓",
        _ => "·",
    }
}

struct FeatureChoice {
    feature: db::features::FeatureSummary,
    max_title_len: usize,
}

impl fmt::Display for FeatureChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sym = status_symbol(&self.feature.status);

        let title = if self.feature.title.len() > 50 {
            format!("{}...", &self.feature.title[..47])
        } else {
            self.feature.title.clone()
        };
        let title_padded = format!("{:width$}", title, width = self.max_title_len.min(50));

        // Status with fixed-width padding (11 chars covers "in_progress")
        let status_padded = format!("{:<11}", self.feature.status);

        let ticket = match self.feature.ticket_id.as_deref() {
            Some(t) if !t.is_empty() => format!("  {}", t),
            _ => String::new(),
        };

        let tags: Vec<_> = self.feature.tags.iter().map(|t| t.name.as_str()).collect();
        let tags_str = if tags.is_empty() {
            String::new()
        } else {
            format!("  [{}]", tags.join(", "))
        };

        write!(f, "{} {}  {}{}{}",
            sym, status_padded, title_padded, ticket, tags_str)
    }
}

struct SessionChoice {
    session: db::sessions::RecentSession,
    is_active: bool,
    max_feature_len: usize,
}

impl fmt::Display for SessionChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Active indicator
        let indicator = if self.is_active { "● " } else { "  " };

        // Feature name (padded)
        let feature_title = if self.session.feature_title.len() > 30 {
            format!("{}...", &self.session.feature_title[..27])
        } else {
            self.session.feature_title.clone()
        };
        let feature_padded = format!("{:width$}", feature_title, width = self.max_feature_len.min(30));

        // Session title
        let session_title = self.session.title.as_deref().unwrap_or("untitled");
        let session_display = if session_title.len() > 40 {
            format!("{}...", &session_title[..37])
        } else {
            session_title.to_string()
        };

        // Relative time (right-aligned feel via padding)
        let time_str = format_relative_time(
            self.session.ended_at.as_deref()
                .or(self.session.started_at.as_deref()),
        );
        let time_display = if time_str.is_empty() {
            String::new()
        } else {
            format!("  ({})", time_str)
        };

        write!(f, "{}{}  {}{}",
            indicator, feature_padded, session_display, time_display)
    }
}

fn format_relative_time(iso_str: Option<&str>) -> String {
    let Some(s) = iso_str else {
        return String::new();
    };
    let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) else {
        return String::new();
    };
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(dt);

    let minutes = duration.num_minutes();
    if minutes < 1 {
        return "just now".to_string();
    }
    if minutes < 60 {
        return format!("{}m ago", minutes);
    }
    let hours = duration.num_hours();
    if hours < 24 {
        return format!("{}h ago", hours);
    }
    let days = duration.num_days();
    if days == 1 {
        return "yesterday".to_string();
    }
    if days < 30 {
        return format!("{}d ago", days);
    }
    format!("{}mo ago", days / 30)
}

fn cmd_interactive() -> Result<(), String> {
    use feature_hub::claude::scanner::is_session_active;

    let rc = render_config();

    let conn = open_db()?;
    let features = db::features::get_features(&conn, None, None)?;

    if features.is_empty() {
        println!("\n  No features found. Create one in FeatureHub first.\n");
        return Ok(());
    }

    let recent_sessions = db::sessions::get_recent_sessions(&conn, 20)?;
    drop(conn);

    // Check active status for each recent session
    let session_active_states: Vec<bool> = recent_sessions
        .iter()
        .map(|s| is_session_active(&s.claude_session_id))
        .collect();

    let has_sessions = !recent_sessions.is_empty();
    let active_count = session_active_states.iter().filter(|&&a| a).count();

    // Print header
    println!();
    println!("  \x1b[1;35mFeatureHub\x1b[0m");
    if active_count > 0 {
        println!("  \x1b[32m● {} session{} running\x1b[0m", active_count, if active_count == 1 { "" } else { "s" });
    }
    println!();

    // Top-level action menu
    let mut options = vec!["Start a new session".to_string()];
    if has_sessions {
        options.push("Resume a recent session".to_string());
    }

    let action = match Select::new("What would you like to do?", options)
        .with_render_config(rc)
        .with_help_message("↑↓ navigate  ⏎ select  Esc cancel")
        .prompt()
    {
        Ok(a) => a,
        Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => return Ok(()),
        Err(e) => return Err(format!("Prompt error: {}", e)),
    };

    if action.starts_with("Start") {
        interactive_start(features, rc)
    } else {
        interactive_resume(recent_sessions, session_active_states, rc)
    }
}

fn interactive_start(
    features: Vec<db::features::FeatureSummary>,
    rc: RenderConfig<'static>,
) -> Result<(), String> {
    let max_title_len = features.iter().map(|f| f.title.len().min(50)).max().unwrap_or(20);

    let choices: Vec<FeatureChoice> = features
        .into_iter()
        .map(|feature| FeatureChoice { feature, max_title_len })
        .collect();

    let selected = match Select::new("Select a feature:", choices)
        .with_render_config(rc)
        .with_help_message("type to filter  ⏎ select  Esc back")
        .prompt()
    {
        Ok(s) => s,
        Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => return Ok(()),
        Err(e) => return Err(format!("Prompt error: {}", e)),
    };

    let prompt = match Text::new("Initial prompt:")
        .with_render_config(rc)
        .with_help_message("optional — press Enter to skip")
        .prompt()
    {
        Ok(p) if p.trim().is_empty() => None,
        Ok(p) => Some(p),
        Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => return Ok(()),
        Err(e) => return Err(format!("Prompt error: {}", e)),
    };

    println!();
    cmd_start_feature(&selected.feature, prompt)
}

fn interactive_resume(
    recent_sessions: Vec<db::sessions::RecentSession>,
    session_active_states: Vec<bool>,
    rc: RenderConfig<'static>,
) -> Result<(), String> {
    let max_feature_len = recent_sessions
        .iter()
        .map(|s| s.feature_title.len().min(30))
        .max()
        .unwrap_or(15);

    let choices: Vec<SessionChoice> = recent_sessions
        .into_iter()
        .zip(session_active_states.iter())
        .map(|(session, &is_active)| SessionChoice {
            session,
            is_active,
            max_feature_len,
        })
        .collect();

    let selected = match Select::new("Select a session to resume:", choices)
        .with_render_config(rc)
        .with_help_message("● = running  type to filter  ⏎ select  Esc back")
        .prompt()
    {
        Ok(s) => s,
        Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => return Ok(()),
        Err(e) => return Err(format!("Prompt error: {}", e)),
    };

    // Warn if session is already running
    if selected.is_active {
        let confirm_options = vec![
            "Yes, open in this terminal too".to_string(),
            "No, go back".to_string(),
        ];
        let confirm = match Select::new(
            "⚠ This session is already running in another terminal.",
            confirm_options,
        )
        .with_render_config(rc)
        .prompt()
        {
            Ok(c) => c,
            Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => return Ok(()),
            Err(e) => return Err(format!("Prompt error: {}", e)),
        };
        if confirm.starts_with("No") {
            return cmd_interactive();
        }
    }

    println!();
    cmd_resume_session(&selected.session.claude_session_id, &selected.session.feature_id)
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Start { feature, prompt }) => cmd_start(&feature, prompt),
        Some(Commands::Resume { session_id }) => cmd_resume(&session_id),
        Some(Commands::List { filter }) => cmd_list(filter),
        None => cmd_interactive(),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
