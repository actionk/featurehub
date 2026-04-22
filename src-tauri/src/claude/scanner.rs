use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::session_parser;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScannedSession {
    pub session_id: String,
    pub project_path: String,
    pub project_name: String,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub last_modified: String,
    pub is_active: bool,
}

pub fn scan_claude_sessions() -> Result<Vec<ScannedSession>, String> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let projects_dir = home.join(".claude").join("projects");

    if !projects_dir.exists() {
        return Ok(Vec::new());
    }

    // Single process scan for all active sessions
    let active_ids = get_running_session_ids();

    let mut sessions = Vec::new();

    // Iterate over project directories
    let project_entries = std::fs::read_dir(&projects_dir).map_err(|e| e.to_string())?;

    for project_entry in project_entries {
        let project_entry = match project_entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let project_path = project_entry.path();
        if !project_path.is_dir() {
            continue;
        }

        let project_name = project_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Decode the project directory name back to a real path
        // Claude encodes paths by replacing path separators with dashes or similar
        let decoded_project_path = decode_project_dir_name(&project_name);

        // Look for session directories or session files within the project
        let session_results = scan_project_sessions(&project_path, &decoded_project_path, &project_name, &active_ids);
        if let Ok(mut project_sessions) = session_results {
            sessions.append(&mut project_sessions);
        }
    }

    // Sort by last_modified descending
    sessions.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

    Ok(sessions)
}

/// Extract session IDs from running claude processes by looking for
/// `--session-id <id>` or `--resume <id>` in their command lines.
/// Returns the set of session IDs that are currently running.
fn get_running_session_ids() -> std::collections::HashSet<String> {
    let output = if cfg!(target_os = "windows") {
        std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "Get-CimInstance Win32_Process -Filter \"Name='claude.exe'\" | Select-Object -ExpandProperty CommandLine",
            ])
            .output()
    } else if cfg!(target_os = "macos") {
        // On macOS, `pgrep -a` means "include ancestors" (BSD semantics), NOT "show full
        // command line" like Linux procps-ng pgrep. So `pgrep -af claude` only returns PIDs
        // on macOS, making it impossible to extract --session-id / --resume from output.
        // Use `ps -ax -o args` instead, which always includes the full argument list.
        std::process::Command::new("ps")
            .args(["-ax", "-o", "args"])
            .output()
    } else {
        // Linux: pgrep -af outputs "PID full-command-line" per line
        std::process::Command::new("pgrep")
            .args(["-af", "claude"])
            .output()
    };

    let mut active = std::collections::HashSet::new();

    let stdout = match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
        Err(_) => return active,
    };

    for line in stdout.lines() {
        // When using `ps` on macOS, skip lines that don't mention claude (e.g. the header row
        // "ARGS" and unrelated processes) to avoid spurious matches.
        if cfg!(target_os = "macos") && !line.contains("claude") {
            continue;
        }

        // Extract session ID from --session-id <id> or --resume <id>
        let parts: Vec<&str> = line.split_whitespace().collect();
        for window in parts.windows(2) {
            if window[0] == "--session-id" || window[0] == "--resume" {
                active.insert(window[1].to_string());
            }
        }
    }

    active
}

/// Check if a Claude session is currently active by looking for a running
/// `claude` process with `--session-id <id>` or `--resume <id>` in its arguments.
pub fn is_session_active(session_id: &str) -> bool {
    let active = get_running_session_ids();
    active.contains(session_id)
}

/// Check which session IDs from the given list are currently active.
/// Does a single process scan and matches all IDs against it.
pub fn get_active_session_ids(session_ids: &[String]) -> Vec<String> {
    let active = get_running_session_ids();
    session_ids
        .iter()
        .filter(|id| active.contains(id.as_str()))
        .cloned()
        .collect()
}

/// Find the decoded project path for a given session ID by locating its .jsonl file.
pub fn find_project_path_for_session(session_id: &str) -> Option<String> {
    let home = dirs::home_dir()?;
    let projects_dir = home.join(".claude").join("projects");

    if !projects_dir.exists() {
        return None;
    }

    for entry in std::fs::read_dir(&projects_dir).ok()?.flatten() {
        let jsonl_path = entry.path().join(format!("{}.jsonl", session_id));
        if jsonl_path.exists() {
            let dir_name = entry.path().file_name()?.to_string_lossy().to_string();
            return Some(decode_project_dir_name(&dir_name));
        }
    }

    None
}

/// Find the JSONL transcript file for a given session ID.
/// Searches one level deep under `projects_dir` for `<session_id>.jsonl`.
pub fn find_jsonl_for_session(
    projects_dir: &std::path::Path,
    session_id: &str,
) -> Option<std::path::PathBuf> {
    let filename = format!("{}.jsonl", session_id);
    for entry in std::fs::read_dir(projects_dir).ok()?.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let jsonl = dir.join(&filename);
        if jsonl.exists() {
            return Some(jsonl);
        }
    }
    None
}

/// Extract title and summary for a session from Claude Code's on-disk files.
/// Priority: sessions-index.json summary → session-memory .md → JSONL first user message.
pub fn find_session_title(session_id: &str) -> (Option<String>, Option<String>) {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return (None, None),
    };
    let projects_dir = home.join(".claude").join("projects");
    if !projects_dir.exists() {
        return (None, None);
    }

    for entry in std::fs::read_dir(&projects_dir).ok().into_iter().flatten().flatten() {
        let project_dir = entry.path();
        if !project_dir.is_dir() {
            continue;
        }

        // Check sessions-index.json first (Claude Code's own title)
        let (index_title, _first_prompt) = session_parser::find_title_in_sessions_index(&project_dir, session_id);
        if index_title.is_some() {
            return (index_title, None);
        }

        // Check session-memory .md file
        let summary_path = project_dir
            .join("session-memory")
            .join(format!("{}.md", session_id));
        if summary_path.exists() {
            if let Ok(parsed) = session_parser::parse_session_summary(&summary_path) {
                if parsed.title.is_some() {
                    return (parsed.title, parsed.summary);
                }
            }
        }

        // Fall back to JSONL transcript
        let jsonl_path = project_dir.join(format!("{}.jsonl", session_id));
        if jsonl_path.exists() {
            let title = session_parser::parse_title_from_jsonl(&jsonl_path);
            return (title, None);
        }
    }

    (None, None)
}

fn decode_project_dir_name(encoded: &str) -> String {
    // Claude projects directory names are URL-encoded paths
    // e.g., "C%3A%5CUsers%5Cuser%5Cproject" -> "C:\Users\user\project"
    // or on Unix: "%2Fhome%2Fuser%2Fproject" -> "/home/user/project"
    urlish_decode(encoded)
}

fn urlish_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let mut hex = String::new();
            if let Some(&h1) = chars.peek() {
                hex.push(h1);
                chars.next();
            }
            if let Some(&h2) = chars.peek() {
                hex.push(h2);
                chars.next();
            }
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Find JSONL file with caching. Searches cache first, then filesystem.
pub fn find_jsonl_with_cache(
    cache: &std::sync::Mutex<std::collections::HashMap<String, std::path::PathBuf>>,
    projects_dir: &std::path::Path,
    session_id: &str,
) -> Option<std::path::PathBuf> {
    // 1. Check cache first
    if let Ok(c) = cache.lock() {
        if let Some(path) = c.get(session_id) {
            if path.exists() {
                return Some(path.clone());
            }
        }
    }

    // 2. Search filesystem
    let filename = format!("{}.jsonl", session_id);

    for entry in std::fs::read_dir(projects_dir).ok()?.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }

        // Direct child
        let jsonl = dir.join(&filename);
        if jsonl.exists() {
            if let Ok(mut c) = cache.lock() {
                c.insert(session_id.to_string(), jsonl.clone());
            }
            return Some(jsonl);
        }

        // Subdirectories (one level deeper)
        for subentry in std::fs::read_dir(&dir).ok()?.flatten() {
            let subdir = subentry.path();
            if subdir.is_dir() && subdir.file_name().is_some_and(|n| n != "session-memory") {
                let jsonl = subdir.join(&filename);
                if jsonl.exists() {
                    if let Ok(mut c) = cache.lock() {
                        c.insert(session_id.to_string(), jsonl.clone());
                    }
                    return Some(jsonl);
                }
            }
        }
    }

    None
}

fn scan_project_sessions(
    project_dir: &PathBuf,
    decoded_project_path: &str,
    project_name: &str,
    active_ids: &std::collections::HashSet<String>,
) -> Result<Vec<ScannedSession>, String> {
    let mut sessions = Vec::new();

    // Load sessions-index.json once for all sessions in this project
    let index_titles = session_parser::load_sessions_index_titles(project_dir);

    // Look for .json session files directly in the project directory
    let entries = std::fs::read_dir(project_dir).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();

        if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
            let session_id = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let metadata = std::fs::metadata(&path).ok();
            let last_modified = metadata
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    let datetime: chrono::DateTime<chrono::Utc> = t.into();
                    datetime.to_rfc3339()
                })
                .unwrap_or_default();

            // Try sessions-index.json first (Claude Code's own title)
            let title = index_titles.get(&session_id).cloned().or_else(|| {
                // Try session-memory .md file
                let summary_path = project_dir
                    .join("session-memory")
                    .join(format!("{}.md", session_id));
                if summary_path.exists() {
                    if let Ok(parsed) = session_parser::parse_session_summary(&summary_path) {
                        if parsed.title.is_some() {
                            return parsed.title;
                        }
                    }
                }
                // Fall back to JSONL transcript
                let jsonl_path = project_dir.join(format!("{}.jsonl", session_id));
                session_parser::parse_title_from_jsonl(&jsonl_path)
            });

            let is_active = active_ids.contains(&session_id);

            sessions.push(ScannedSession {
                session_id,
                project_path: decoded_project_path.to_string(),
                project_name: project_name.to_string(),
                title,
                summary: None,
                last_modified,
                is_active,
            });
        }

        // Also check subdirectories that might contain sessions
        if path.is_dir() {
            let dirname = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            // Skip known non-session directories
            if dirname == "session-memory" || dirname == ".git" {
                continue;
            }

            // Check for session files within subdirectory
            if let Ok(sub_entries) = std::fs::read_dir(&path) {
                for sub_entry in sub_entries {
                    let sub_entry = match sub_entry {
                        Ok(e) => e,
                        Err(_) => continue,
                    };

                    let sub_path = sub_entry.path();
                    if sub_path.is_file()
                        && sub_path.extension().is_some_and(|ext| ext == "json")
                    {
                        let session_id = sub_path
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();

                        let metadata = std::fs::metadata(&sub_path).ok();
                        let last_modified = metadata
                            .and_then(|m| m.modified().ok())
                            .map(|t| {
                                let datetime: chrono::DateTime<chrono::Utc> = t.into();
                                datetime.to_rfc3339()
                            })
                            .unwrap_or_default();

                        // Try sessions-index.json first (already loaded)
                        let title = index_titles.get(&session_id).cloned().or_else(|| {
                            let summary_path = path.join("session-memory").join(format!("{}.md", session_id));
                            if summary_path.exists() {
                                if let Ok(parsed) = session_parser::parse_session_summary(&summary_path) {
                                    if parsed.title.is_some() {
                                        return parsed.title;
                                    }
                                }
                            }
                            let jsonl_path = path.join(format!("{}.jsonl", session_id));
                            session_parser::parse_title_from_jsonl(&jsonl_path)
                        });

                        let is_active = active_ids.contains(&session_id);

                        sessions.push(ScannedSession {
                            session_id,
                            project_path: decoded_project_path.to_string(),
                            project_name: project_name.to_string(),
                            title,
                            summary: None,
                            last_modified,
                            is_active,
                        });
                    }
                }
            }
        }
    }

    Ok(sessions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::TempDir;

    #[test]
    fn test_find_jsonl_uses_cache() {
        let cache = Mutex::new(std::collections::HashMap::new());
        let tmp = TempDir::new().unwrap();
        let jsonl_path = tmp.path().join("test-session.jsonl");
        std::fs::write(&jsonl_path, "{}").unwrap();

        // Prime the cache
        cache.lock().unwrap().insert("test-session".to_string(), jsonl_path.clone());

        // Should find from cache
        let result = find_jsonl_with_cache(&cache, tmp.path(), "test-session");
        assert_eq!(result, Some(jsonl_path));
    }
}
