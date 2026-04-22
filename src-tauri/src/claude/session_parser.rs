use std::collections::HashMap;
use std::path::Path;

pub struct ParsedSession {
    pub title: Option<String>,
    pub summary: Option<String>,
}

/// Look up a session's title from Claude Code's `sessions-index.json` file.
/// Returns (summary_as_title, first_prompt) — `summary` is the Claude-generated
/// session name (e.g. "Add moderator popup showing current/next blank word").
pub fn find_title_in_sessions_index(project_dir: &Path, session_id: &str) -> (Option<String>, Option<String>) {
    let index_path = project_dir.join("sessions-index.json");
    let content = match std::fs::read_to_string(&index_path) {
        Ok(c) => c,
        Err(_) => return (None, None),
    };
    let val: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return (None, None),
    };
    let entries = match val.get("entries").and_then(|e| e.as_array()) {
        Some(a) => a,
        None => return (None, None),
    };
    for entry in entries {
        if entry.get("sessionId").and_then(|v| v.as_str()) == Some(session_id) {
            let summary = entry.get("summary").and_then(|v| v.as_str()).map(|s| s.to_string());
            let first_prompt = entry.get("firstPrompt").and_then(|v| v.as_str()).map(|s| {
                let trimmed = s.trim();
                if trimmed.len() > 120 {
                    format!("{}...", &trimmed[..117])
                } else {
                    trimmed.to_string()
                }
            });
            return (summary, first_prompt);
        }
    }
    (None, None)
}

/// Load all session titles from a `sessions-index.json` into a HashMap.
/// Key is session ID, value is the Claude-generated summary title.
pub fn load_sessions_index_titles(project_dir: &Path) -> HashMap<String, String> {
    let index_path = project_dir.join("sessions-index.json");
    let content = match std::fs::read_to_string(&index_path) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };
    let val: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return HashMap::new(),
    };
    let entries = match val.get("entries").and_then(|e| e.as_array()) {
        Some(a) => a,
        None => return HashMap::new(),
    };
    let mut map = HashMap::new();
    for entry in entries {
        if let (Some(id), Some(summary)) = (
            entry.get("sessionId").and_then(|v| v.as_str()),
            entry.get("summary").and_then(|v| v.as_str()),
        ) {
            if !summary.is_empty() {
                map.insert(id.to_string(), summary.to_string());
            }
        }
    }
    map
}

pub fn parse_session_summary(path: &Path) -> Result<ParsedSession, String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;

    if content.trim().is_empty() {
        return Ok(ParsedSession {
            title: None,
            summary: None,
        });
    }

    let lines: Vec<&str> = content.lines().collect();

    // Extract title: first heading (line starting with #) or first non-empty line
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

    // Extract summary: everything after the title, up to 500 characters
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

    Ok(ParsedSession { title, summary })
}

/// Parse a suitable title from a Claude Code JSONL transcript file.
/// Skips very short or slash-command-like messages, preferring the first
/// substantive user message. Falls back to the first message if nothing better.
pub fn parse_title_from_jsonl(path: &Path) -> Option<String> {
    use std::io::{BufRead, BufReader};

    let file = std::fs::File::open(path).ok()?;
    let reader = BufReader::new(file);

    let mut fallback: Option<String> = None;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.is_empty() {
            continue;
        }

        // Quick check before parsing JSON
        if !line.contains("\"type\":\"user\"") {
            continue;
        }

        let val: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if val.get("type").and_then(|v| v.as_str()) != Some("user") {
            continue;
        }

        // Extract the user message content
        let content = match val.get("message").and_then(|m| m.get("content")) {
            Some(c) => c,
            None => continue,
        };

        let text = if let Some(s) = content.as_str() {
            s.to_string()
        } else if let Some(arr) = content.as_array() {
            arr.iter()
                .filter_map(|part| {
                    if part.get("type")?.as_str()? == "text" {
                        part.get("text")?.as_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            continue;
        };

        let text = text.trim().to_string();
        if text.is_empty() {
            continue;
        }

        let first_line = text.lines().next().unwrap_or(&text);
        let title = if first_line.len() > 120 {
            format!("{}...", &first_line[..117])
        } else {
            first_line.to_string()
        };

        // Keep the very first message as fallback
        if fallback.is_none() {
            fallback = Some(title.clone());
        }

        // Skip messages that don't make good titles
        if is_bad_title(&title) {
            continue;
        }

        // This is a good title
        return Some(title);
    }

    fallback
}

#[derive(Clone, Debug)]
pub struct CachedStats {
    pub mtime: std::time::SystemTime,
    pub model: Option<String>,
    pub total_tokens: u64,
    pub cost_usd: Option<f64>,
    pub context_tokens: Option<u64>,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq)]
pub enum SessionStatus {
    Active,
    WaitingForInput,
    Idle,
    Finished,
    Lost,
}

#[derive(Clone, Debug, serde::Serialize, PartialEq)]
pub enum TitleSource {
    SessionsIndex,
    SessionMemory,
    FirstPrompt,
    FeatureName,
    Default,
}

#[derive(Clone, Debug, Default, serde::Serialize, PartialEq)]
pub enum StatusHint {
    #[default]
    ClaudeResponded,
    UserPrompted,
    ToolRunning,
}

#[derive(Clone, Debug, Default)]
pub struct ParsedStatusHint {
    pub status_hint: StatusHint,
    pub looks_like_waiting: bool,
    pub last_action: Option<String>,
}

/// Parse the last ~20 lines of a JSONL file to determine status hints.
pub fn parse_status_hint(path: &std::path::Path) -> ParsedStatusHint {
    use std::io::{BufRead, BufReader, Seek, SeekFrom};

    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return ParsedStatusHint::default(),
    };

    // Read last ~8KB to get recent entries
    let file_len = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mut reader = BufReader::new(file);
    if file_len > 8192 {
        reader.seek(SeekFrom::End(-8192)).ok();
        // Skip partial line
        let mut skip = String::new();
        reader.read_line(&mut skip).ok();
    }

    let mut last_type: Option<&'static str> = None;
    let mut last_assistant_content: Option<String> = None;
    let mut last_tool: Option<String> = None;

    for line in reader.lines().map_while(Result::ok) {
        if line.is_empty() {
            continue;
        }

        let val: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let msg_type = val.get("type").and_then(|v| v.as_str());

        match msg_type {
            Some("user") => {
                last_type = Some("user");
            }
            Some("assistant") => {
                last_type = Some("assistant");
                // Extract content for waiting detection
                if let Some(msg) = val.get("message") {
                    if let Some(content) = msg.get("content") {
                        if let Some(s) = content.as_str() {
                            last_assistant_content = Some(s.to_string());
                        } else if let Some(arr) = content.as_array() {
                            // Check for tool use
                            for item in arr {
                                if item.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                                    last_tool = item.get("name").and_then(|n| n.as_str()).map(|s| s.to_string());
                                }
                                if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                                    if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                                        last_assistant_content = Some(text.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let status_hint = match last_type {
        Some("user") => StatusHint::UserPrompted,
        Some("assistant") if last_tool.is_some() => StatusHint::ToolRunning,
        _ => StatusHint::ClaudeResponded,
    };

    let looks_like_waiting = last_assistant_content.as_ref().is_some_and(|c| {
        c.ends_with('?')
            || c.contains("Would you like")
            || c.contains("Should I")
            || c.contains("Do you want")
            || c.contains("Let me know")
            || c.contains("What would you prefer")
    });

    let last_action = last_tool.map(|tool| match tool.as_str() {
        "Read" => "Reading files".to_string(),
        "Edit" | "Write" => "Writing code".to_string(),
        "Bash" => "Running command".to_string(),
        "Grep" | "Glob" => "Searching".to_string(),
        "Agent" => "Running subagent".to_string(),
        other => format!("Using {}", other),
    });

    ParsedStatusHint {
        status_hint,
        looks_like_waiting,
        last_action,
    }
}

/// Parse model, token counts, and cost from a Claude JSONL transcript.
/// Skips files larger than 100 MB. Results should be cached by mtime.
pub fn parse_session_stats(path: &std::path::Path) -> CachedStats {
    use std::io::{BufRead, BufReader};

    let meta = std::fs::metadata(path).ok();
    let mtime = meta
        .as_ref()
        .and_then(|m| m.modified().ok())
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

    // Skip very large files to prevent memory pressure
    if meta.as_ref().is_some_and(|m| m.len() > 100 * 1024 * 1024) {
        return CachedStats { mtime, model: None, total_tokens: 0, cost_usd: None, context_tokens: None };
    }

    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return CachedStats { mtime, model: None, total_tokens: 0, cost_usd: None, context_tokens: None },
    };

    let reader = BufReader::new(file);
    let mut model: Option<String> = None;
    let mut total_tokens: u64 = 0;
    let mut cost_usd: Option<f64> = None;
    let mut context_tokens: Option<u64> = None;

    for line in reader.lines() {
        let line = match line {
            Ok(l) if !l.is_empty() => l,
            _ => continue,
        };

        // Quick pre-filter before JSON parsing
        if !line.contains("\"type\":\"assistant\"") {
            continue;
        }

        let val: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if val.get("type").and_then(|v| v.as_str()) != Some("assistant") {
            continue;
        }

        let msg = match val.get("message") {
            Some(m) => m,
            None => continue,
        };

        // Extract model from each assistant message — last one wins (most recent model)
        if let Some(m) = msg.get("model").and_then(|v| v.as_str()) {
            model = Some(m.to_string());
        }

        // Accumulate token counts; track last message's full context size
        // (input + cache reads + cache creation — matches Claude's own context-window display)
        if let Some(usage) = msg.get("usage") {
            let input = usage.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            let output = usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            let cache_read = usage.get("cache_read_input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            let cache_create = usage.get("cache_creation_input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            total_tokens = total_tokens.saturating_add(input + output);
            let ctx = input + cache_read + cache_create;
            if ctx > 0 {
                context_tokens = Some(ctx);
            }
        }

        // Accumulate cost if present
        if let Some(cost) = val.get("costUSD").and_then(|v| v.as_f64()) {
            *cost_usd.get_or_insert(0.0) += cost;
        }
    }

    CachedStats { mtime, model, total_tokens, cost_usd, context_tokens }
}

/// Check if a string is unsuitable as a session title.
pub fn is_bad_title(s: &str) -> bool {
    let trimmed = s.trim();

    // Too short to be meaningful
    if trimmed.len() < 5 {
        return true;
    }

    // Slash commands
    if trimmed.starts_with('/') {
        return true;
    }

    // XML/HTML tags (covers <local-command-caveat>, <system>, etc.)
    if trimmed.starts_with('<') && trimmed.contains('>') {
        return true;
    }

    // File paths
    if looks_like_path(trimmed) {
        return true;
    }

    // Git commands
    if trimmed.starts_with("git ") {
        return true;
    }

    // System prefixes that leak through
    if trimmed.starts_with("Caveat:") {
        return true;
    }

    false
}

/// Check if a string looks like a bare file path rather than a real title.
fn looks_like_path(s: &str) -> bool {
    // Windows absolute path: C:\... or D:\...
    if s.len() >= 3 && s.as_bytes()[1] == b':' && (s.as_bytes()[2] == b'\\' || s.as_bytes()[2] == b'/') {
        return true;
    }
    // Unix absolute path with multiple segments
    if s.starts_with('/') && s.contains('/') && !s.contains(' ') {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp_jsonl(content: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f.flush().unwrap();
        f
    }

    #[test]
    fn test_parse_session_stats_empty_file() {
        let f = write_temp_jsonl("");
        let stats = parse_session_stats(f.path());
        assert_eq!(stats.total_tokens, 0);
        assert!(stats.model.is_none());
        assert!(stats.cost_usd.is_none());
    }

    #[test]
    fn test_parse_session_stats_extracts_model_and_tokens() {
        let line = r#"{"type":"assistant","message":{"model":"claude-opus-4-6","usage":{"input_tokens":100,"output_tokens":50}}}"#;
        let f = write_temp_jsonl(&format!("{}\n", line));
        let stats = parse_session_stats(f.path());
        assert_eq!(stats.model.as_deref(), Some("claude-opus-4-6"));
        assert_eq!(stats.total_tokens, 150);
        assert!(stats.cost_usd.is_none());
    }

    #[test]
    fn test_parse_session_stats_accumulates_tokens_across_messages() {
        let line1 = r#"{"type":"assistant","message":{"model":"claude-opus-4-6","usage":{"input_tokens":100,"output_tokens":50}}}"#;
        let line2 = r#"{"type":"assistant","message":{"model":"claude-opus-4-6","usage":{"input_tokens":200,"output_tokens":80}}}"#;
        let f = write_temp_jsonl(&format!("{}\n{}\n", line1, line2));
        let stats = parse_session_stats(f.path());
        assert_eq!(stats.total_tokens, 430);
    }

    #[test]
    fn test_parse_session_stats_accumulates_cost() {
        let line1 = r#"{"type":"assistant","costUSD":0.05,"message":{"model":"claude-opus-4-6","usage":{"input_tokens":10,"output_tokens":5}}}"#;
        let line2 = r#"{"type":"assistant","costUSD":0.03,"message":{"model":"claude-opus-4-6","usage":{"input_tokens":10,"output_tokens":5}}}"#;
        let f = write_temp_jsonl(&format!("{}\n{}\n", line1, line2));
        let stats = parse_session_stats(f.path());
        assert!((stats.cost_usd.unwrap() - 0.08).abs() < 1e-6);
    }

    #[test]
    fn test_is_bad_title_filters_xml_tags() {
        assert!(is_bad_title("<local-command-caveat>Caveat: The messages..."));
        assert!(is_bad_title("<system>Some system text</system>"));
        assert!(is_bad_title("<user-prompt>test</user-prompt>"));
    }

    #[test]
    fn test_is_bad_title_filters_short_and_commands() {
        assert!(is_bad_title("hi"));  // too short
        assert!(is_bad_title("/help"));  // slash command
        assert!(is_bad_title("git status"));  // git command
        assert!(is_bad_title("Caveat: something"));  // system prefix
    }

    #[test]
    fn test_is_bad_title_allows_good_titles() {
        assert!(!is_bad_title("Add authentication to login flow"));
        assert!(!is_bad_title("Fix the bug in session parsing"));
        assert!(!is_bad_title("implement user management feature"));
    }

    #[test]
    fn test_parse_title_from_jsonl_skips_xml_tags() {
        let content = r#"{"type":"user","message":{"content":"<local-command-caveat>Caveat: The messages below were...</local-command-caveat>"}}
{"type":"user","message":{"content":"implement the authentication feature"}}"#;
        let f = write_temp_jsonl(content);
        let title = parse_title_from_jsonl(f.path());
        assert_eq!(title, Some("implement the authentication feature".to_string()));
    }

    #[test]
    fn test_parse_title_from_jsonl_uses_fallback_if_all_bad() {
        let content = r#"{"type":"user","message":{"content":"<system>skip</system>"}}
{"type":"user","message":{"content":"/help"}}"#;
        let f = write_temp_jsonl(content);
        let title = parse_title_from_jsonl(f.path());
        // Falls back to first message since all are "bad"
        assert_eq!(title, Some("<system>skip</system>".to_string()));
    }

    #[test]
    fn test_parse_status_hint_detects_waiting() {
        let content = r#"{"type":"assistant","message":{"content":"Would you like me to proceed with the implementation?"}}"#;
        let f = write_temp_jsonl(content);
        let hint = parse_status_hint(f.path());
        assert_eq!(hint.status_hint, StatusHint::ClaudeResponded);
        assert!(hint.looks_like_waiting);
    }

    #[test]
    fn test_parse_status_hint_detects_user_prompted() {
        let content = r#"{"type":"assistant","message":{"content":"Done!"}}
{"type":"user","message":{"content":"now do the next thing"}}"#;
        let f = write_temp_jsonl(content);
        let hint = parse_status_hint(f.path());
        assert_eq!(hint.status_hint, StatusHint::UserPrompted);
    }

    #[test]
    fn test_parse_last_action_from_tool() {
        let content = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Read"}]}}"#;
        let f = write_temp_jsonl(content);
        let hint = parse_status_hint(f.path());
        assert_eq!(hint.last_action, Some("Reading files".to_string()));
    }

    #[test]
    fn test_parse_session_stats_skips_non_assistant_lines() {
        let content = r#"{"type":"user","message":{"role":"user","content":"hello"}}
{"type":"assistant","message":{"model":"claude-sonnet-4-6","usage":{"input_tokens":5,"output_tokens":10}}}
{"type":"progress"}
"#;
        let f = write_temp_jsonl(content);
        let stats = parse_session_stats(f.path());
        assert_eq!(stats.total_tokens, 15);
        assert_eq!(stats.model.as_deref(), Some("claude-sonnet-4-6"));
    }
}
