use std::process::Command;

use crate::config::McpServer;

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn shell_quote(s: &str) -> String {
    let clean = s.replace('\n', " ").replace('\r', " ");
    format!("'{}'", clean.replace('\'', "'\\''"))
}

/// Build a temporary MCP config JSON file for passing via `--mcp-config` flag.
/// This avoids polluting project `.mcp.json` files.
fn build_mcp_config_file(servers: &[McpServer]) -> Result<Option<std::path::PathBuf>, String> {
    if servers.is_empty() {
        return Ok(None);
    }

    let mut mcp_servers = serde_json::Map::new();
    for server in servers {
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

    let config = serde_json::json!({ "mcpServers": mcp_servers });
    let config_dir = crate::config::config_dir()?;
    let path = config_dir.join("session-mcp.json");
    let data = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize MCP config: {}", e))?;
    std::fs::write(&path, data).map_err(|e| format!("Failed to write MCP config: {}", e))?;

    Ok(Some(path))
}

pub fn push_default_allowed_tools(args: &mut Vec<String>) {
    let tools = [
        "mcp__featurehub__*",
        // git read-only
        "Bash(git status*)",
        "Bash(git log*)",
        "Bash(git diff*)",
        "Bash(git branch*)",
        "Bash(git show*)",
        "Bash(git remote*)",
        "Bash(git tag*)",
        "Bash(git stash list*)",
        // gh CLI read-only
        "Bash(gh pr list*)",
        "Bash(gh pr view*)",
        "Bash(gh pr status*)",
        "Bash(gh pr checks*)",
        "Bash(gh pr diff*)",
        "Bash(gh issue list*)",
        "Bash(gh issue view*)",
        "Bash(gh issue status*)",
        "Bash(gh repo view*)",
        "Bash(gh run list*)",
        "Bash(gh run view*)",
        "Bash(gh release list*)",
        "Bash(gh release view*)",
        "Bash(gh status*)",
    ];
    for tool in tools {
        args.push("--allowedTools".to_string());
        args.push(tool.to_string());
    }
}

/// Returns (program, args, cwd) for spawning claude in a PTY for resume.
pub fn build_resume_args(
    session_id: &str,
    project_path: &str,
    directories: &[String],
    servers: &[McpServer],
    dangerously_skip_permissions: bool,
) -> Result<(String, Vec<String>, String), String> {
    let mcp_config_path = build_mcp_config_file(servers)?;

    let mut args = vec!["--resume".to_string(), session_id.to_string()];
    if let Some(ref mcp_path) = mcp_config_path {
        args.push("--mcp-config".to_string());
        args.push(mcp_path.to_string_lossy().to_string());
    }
    if dangerously_skip_permissions {
        args.push("--dangerously-skip-permissions".to_string());
    } else {
        push_default_allowed_tools(&mut args);
    }

    for dir in directories {
        args.push("--add-dir".to_string());
        args.push(dir.clone());
    }

    Ok(("claude".to_string(), args, project_path.to_string()))
}

/// Returns (program, args, cwd) for spawning claude in a PTY for a new session.
pub fn build_new_session_args(
    feature_dir: &str,
    directories: &[String],
    feature_title: &str,
    context: Option<&str>,
    servers: &[McpServer],
    session_id: &str,
    dangerously_skip_permissions: bool,
) -> Result<(String, Vec<String>, String), String> {
    let mcp_config_path = build_mcp_config_file(servers)?;

    let mut args = vec!["--session-id".to_string(), session_id.to_string()];
    if let Some(ref mcp_path) = mcp_config_path {
        args.push("--mcp-config".to_string());
        args.push(mcp_path.to_string_lossy().to_string());
    }
    if dangerously_skip_permissions {
        args.push("--dangerously-skip-permissions".to_string());
    } else {
        push_default_allowed_tools(&mut args);
    }

    for dir in directories {
        args.push("--add-dir".to_string());
        args.push(dir.clone());
    }

    let mut prompt_parts = vec![format!("Working on feature: {}", feature_title)];
    if let Some(ctx) = context {
        if !ctx.is_empty() {
            prompt_parts.push(format!("Context: {}", ctx));
        }
    }
    // Pass prompt as positional argument (not a flag)
    args.push(prompt_parts.join("\n"));

    Ok(("claude".to_string(), args, feature_dir.to_string()))
}

pub fn resume_session(
    session_id: &str,
    project_path: &str,
    directories: &[String],
    servers: &[McpServer],
) -> Result<(), String> {
    let mcp_config_path = build_mcp_config_file(servers)?;

    let mut claude_args = vec![
        "claude".to_string(),
        "--resume".to_string(),
        session_id.to_string(),
    ];
    if let Some(ref mcp_path) = mcp_config_path {
        claude_args.push("--mcp-config".to_string());
        claude_args.push(mcp_path.to_string_lossy().to_string());
    }

    // Auto-approve FeatureHub MCP tools + git readonly
    push_default_allowed_tools(&mut claude_args);

    // Grant Claude access to all working directories
    for dir in directories {
        claude_args.push("--add-dir".to_string());
        claude_args.push(dir.clone());
    }

    #[cfg(target_os = "windows")]
    {
        let mut cmd_args = vec![
            "/c".to_string(),
            "start".to_string(),
            "wt".to_string(),
            "-d".to_string(),
            project_path.to_string(),
        ];
        cmd_args.extend(claude_args);

        Command::new("cmd")
            .args(&cmd_args)
            .spawn()
            .map_err(|e| format!("Failed to launch Claude session: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        let args_str = claude_args
            .iter()
            .map(|a| shell_quote(a))
            .collect::<Vec<_>>()
            .join(" ");

        let shell_cmd = format!("cd {} && {}", shell_quote(project_path), args_str);
        let script = format!(
            r#"tell application "Terminal"
                activate
                do script "{}"
            end tell"#,
            shell_cmd.replace('\\', "\\\\").replace('"', "\\\"")
        );

        Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map_err(|e| format!("Failed to launch Claude session: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        let args_str = claude_args
            .iter()
            .map(|a| shell_quote(a))
            .collect::<Vec<_>>()
            .join(" ");

        let terminals = [
            ("x-terminal-emulator", vec!["-e"]),
            ("gnome-terminal", vec!["--"]),
            ("konsole", vec!["-e"]),
            ("xterm", vec!["-e"]),
        ];

        let mut launched = false;
        for (term, args) in &terminals {
            let mut cmd = Command::new(term);
            for a in args {
                cmd.arg(a);
            }
            cmd.arg("bash").arg("-c").arg(format!(
                "cd {} && {}",
                shell_quote(project_path),
                args_str
            ));

            if cmd.spawn().is_ok() {
                launched = true;
                break;
            }
        }

        if !launched {
            return Err("Could not find a terminal emulator to launch Claude".to_string());
        }
    }

    Ok(())
}

pub fn start_new_session(
    feature_dir: &str,
    directories: &[String],
    feature_title: &str,
    context: Option<&str>,
    servers: &[McpServer],
    session_id: &str,
) -> Result<(), String> {
    let work_dir = feature_dir;

    let mcp_config_path = build_mcp_config_file(servers)?;

    let mut claude_args = vec![
        "claude".to_string(),
        "--session-id".to_string(),
        session_id.to_string(),
    ];
    if let Some(ref mcp_path) = mcp_config_path {
        claude_args.push("--mcp-config".to_string());
        claude_args.push(mcp_path.to_string_lossy().to_string());
    }

    // Auto-approve FeatureHub MCP tools + git readonly
    push_default_allowed_tools(&mut claude_args);

    // Grant Claude access to all working directories
    for dir in directories {
        claude_args.push("--add-dir".to_string());
        claude_args.push(dir.clone());
    }

    // Build a prompt with feature context
    let mut prompt_parts = vec![format!("Working on feature: {}", feature_title)];
    if let Some(ctx) = context {
        if !ctx.is_empty() {
            prompt_parts.push(format!("Context: {}", ctx));
        }
    }

    let prompt = prompt_parts.join("\n");
    // Pass prompt as positional argument (not a flag)
    claude_args.push(prompt);

    #[cfg(target_os = "windows")]
    {
        let mut cmd_args = vec![
            "/c".to_string(),
            "start".to_string(),
            "wt".to_string(),
            "-d".to_string(),
            work_dir.to_string(),
        ];
        cmd_args.extend(claude_args);

        Command::new("cmd")
            .args(&cmd_args)
            .spawn()
            .map_err(|e| format!("Failed to start Claude session: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        let args_str = claude_args
            .iter()
            .map(|a| shell_quote(a))
            .collect::<Vec<_>>()
            .join(" ");

        let shell_cmd = format!("cd {} && {}", shell_quote(work_dir), args_str);
        let script = format!(
            r#"tell application "Terminal"
                activate
                do script "{}"
            end tell"#,
            shell_cmd.replace('\\', "\\\\").replace('"', "\\\"")
        );

        Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map_err(|e| format!("Failed to start Claude session: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        let args_str = claude_args
            .iter()
            .map(|a| shell_quote(a))
            .collect::<Vec<_>>()
            .join(" ");

        let terminals = [
            ("x-terminal-emulator", vec!["-e"]),
            ("gnome-terminal", vec!["--"]),
            ("konsole", vec!["-e"]),
            ("xterm", vec!["-e"]),
        ];

        let mut launched = false;
        for (term, args) in &terminals {
            let mut cmd = Command::new(term);
            for a in args {
                cmd.arg(a);
            }
            cmd.arg("bash")
                .arg("-c")
                .arg(format!("cd {} && {}", shell_quote(work_dir), args_str));

            if cmd.spawn().is_ok() {
                launched = true;
                break;
            }
        }

        if !launched {
            return Err("Could not find a terminal emulator to launch Claude".to_string());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_resume_args_uses_default_allowed_tools_without_full_access() {
        let (_program, args, cwd) = build_resume_args(
            "session-1",
            "C:/work/project",
            &["C:/work/repo".to_string()],
            &[],
            false,
        )
        .expect("resume args");

        assert_eq!(cwd, "C:/work/project");
        assert!(args.contains(&"--resume".to_string()));
        assert!(args.contains(&"session-1".to_string()));
        assert!(args.contains(&"--allowedTools".to_string()));
        assert!(!args.contains(&"--dangerously-skip-permissions".to_string()));
    }

    #[test]
    fn build_resume_args_can_skip_permissions_for_full_access() {
        let (_program, args, _cwd) = build_resume_args(
            "session-1",
            "C:/work/project",
            &["C:/work/repo".to_string()],
            &[],
            true,
        )
        .expect("resume args");

        assert!(args.contains(&"--dangerously-skip-permissions".to_string()));
        assert!(!args.contains(&"--allowedTools".to_string()));
    }
}
