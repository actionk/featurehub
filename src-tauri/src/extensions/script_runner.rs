use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ScriptInput {
    pub params: serde_json::Map<String, serde_json::Value>,
    pub db_path: String,
    pub storage_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Notification {
    #[serde(default)]
    pub feature_id: Option<String>,
    #[serde(default = "default_kind")]
    pub kind: String,
    pub message: String,
    #[serde(default)]
    pub plan_id: Option<String>,
}

fn default_kind() -> String {
    "info".into()
}

#[derive(Debug)]
pub struct ScriptResult {
    pub data: serde_json::Value,
    pub notifications: Vec<Notification>,
}

#[derive(Debug, serde::Deserialize)]
struct ScriptOutputFull {
    ok: bool,
    #[serde(default)]
    data: Option<serde_json::Value>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    notifications: Vec<Notification>,
}

/// Resolve the node binary path, working around version managers (e.g. nodist on Windows)
/// that use shim scripts requiring env vars that we strip for security.
/// Tries each PATH entry in order, returning the first real `node` executable found
/// that is not under a known shim directory (NODIST_PREFIX).
fn resolve_node_binary() -> std::ffi::OsString {
    #[cfg(windows)]
    let exe_name = "node.exe";
    #[cfg(not(windows))]
    let exe_name = "node";

    let shim_prefix = std::env::var("NODIST_PREFIX").ok();

    if let Ok(path_var) = std::env::var("PATH") {
        let sep = if cfg!(windows) { ';' } else { ':' };
        for dir in path_var.split(sep) {
            // Skip nodist shim directories when NODIST_PREFIX is set
            if let Some(ref prefix) = shim_prefix {
                let dir_lower = dir.to_lowercase();
                let prefix_lower = prefix.to_lowercase();
                if dir_lower.contains(&prefix_lower) {
                    continue;
                }
            }
            let candidate = std::path::Path::new(dir).join(exe_name);
            if candidate.is_file() {
                return candidate.into_os_string();
            }
        }
    }

    // Fall back to letting the OS resolve it
    std::ffi::OsString::from("node")
}

/// Run a script synchronously, blocking the calling thread.
/// Returns parsed `data` on success, or an error string.
/// Thin wrapper around `run_blocking_with_notifications` for backward compatibility.
pub fn run_blocking(
    script_path: &Path,
    input: &ScriptInput,
    timeout_secs: u64,
) -> Result<serde_json::Value, String> {
    run_blocking_with_notifications(script_path, input, timeout_secs).map(|r| r.data)
}

/// Run a script synchronously, returning both `data` and any `notifications` emitted.
pub fn run_blocking_with_notifications(
    script_path: &Path,
    input: &ScriptInput,
    timeout_secs: u64,
) -> Result<ScriptResult, String> {
    let input_json = serde_json::to_string(input).map_err(|e| e.to_string())?;

    let node_bin = resolve_node_binary();

    // Preserve minimal system env vars needed for the process to start on each OS,
    // while stripping everything else to avoid leaking secrets into scripts.
    let passthrough: &[&str] = &[
        "PATH",
        "SystemRoot",
        "SystemDrive",
        "TEMP",
        "TMP",
        "HOME",
        "USERPROFILE",
    ];
    let mut cmd = Command::new(&node_bin);
    cmd.arg(script_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_clear();
    for key in passthrough {
        if let Ok(val) = std::env::var(key) {
            cmd.env(key, val);
        }
    }
    cmd.env("FH_DB_PATH", &input.db_path)
        .env("FH_STORAGE_PATH", &input.storage_path);
    if let Some(ref fid) = input.feature_id {
        cmd.env("FH_FEATURE_ID", fid);
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn script {:?}: {}", script_path, e))?;

    // Write input to stdin and close it (signals EOF to the script).
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input_json.as_bytes())
            .map_err(|e| format!("Failed to write input to script {:?}: {}", script_path, e))?;
    }

    // Read stdout in a separate thread so we can apply a timeout while keeping
    // `child` on this thread for kill capability.
    let stdout_pipe = child.stdout.take().expect("stdout was piped");
    let (tx, rx) = std::sync::mpsc::channel::<std::io::Result<Vec<u8>>>();
    std::thread::spawn(move || {
        use std::io::Read;
        let mut buf = Vec::new();
        let result = {
            let mut r = stdout_pipe;
            r.read_to_end(&mut buf).map(|_| buf)
        };
        let _ = tx.send(result);
    });

    let stdout_bytes = match rx.recv_timeout(Duration::from_secs(timeout_secs)) {
        Ok(result) => result.map_err(|e| format!("Failed to read script output: {}", e))?,
        Err(_) => {
            // Kill the child so the stdout-reader thread unblocks and exits.
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("Script timed out after {}s", timeout_secs));
        }
    };

    // Reap the child process.
    let _ = child.wait();

    const MAX_OUTPUT: usize = 1024 * 1024;
    if stdout_bytes.len() > MAX_OUTPUT {
        return Err("Script output exceeded 1MB limit".to_string());
    }

    let stdout = String::from_utf8(stdout_bytes).map_err(|e| e.to_string())?;
    let result: ScriptOutputFull = serde_json::from_str(&stdout).map_err(|e| {
        let preview: String = stdout.chars().take(200).collect();
        format!("Failed to parse script output: {}. Raw: {}", e, preview)
    })?;

    if result.ok {
        Ok(ScriptResult {
            data: result.data.unwrap_or(serde_json::Value::Null),
            notifications: result.notifications,
        })
    } else {
        Err(result
            .error
            .unwrap_or_else(|| "Script returned ok=false".to_string()))
    }
}

/// Forward notifications emitted by an extension script to the UI notification system.
pub fn forward_notifications(extension_id: &str, notifications: Vec<Notification>) {
    // Note: `n.kind` is intentionally not forwarded — `push_notification_ex` does not
    // accept a kind/severity argument yet. The field is preserved on the struct so
    // extensions can declare semantics; future API extension can pass it through.
    for n in notifications {
        if let Err(e) = crate::config::push_notification_ex(
            &n.message,
            n.feature_id.as_deref(),
            n.plan_id.as_deref(),
        ) {
            eprintln!("[ext:{}] push_notification failed: {}", extension_id, e);
        }
    }
}

/// Spawn an event hook script fire-and-forget. Errors are logged, not propagated.
pub fn run_event_hook(script_path: std::path::PathBuf, input: ScriptInput, extension_id: String) {
    std::thread::spawn(
        move || match run_blocking_with_notifications(&script_path, &input, 10) {
            Ok(result) => forward_notifications(&extension_id, result.notifications),
            Err(e) => eprintln!(
                "[ext:{}] Event hook {:?} failed: {}",
                extension_id, script_path, e
            ),
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn test_input() -> ScriptInput {
        let tmp = std::env::temp_dir();
        ScriptInput {
            params: Default::default(),
            db_path: tmp.join("test.db").to_string_lossy().into_owned(),
            storage_path: tmp.to_string_lossy().into_owned(),
            feature_id: None,
        }
    }

    fn echo_script() -> tempfile::TempPath {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            f,
            r#"
const chunks = [];
process.stdin.on('data', c => chunks.push(c));
process.stdin.on('end', () => {{
  const input = JSON.parse(Buffer.concat(chunks).toString());
  process.stdout.write(JSON.stringify({{ ok: true, data: input.params }}));
}});
"#
        )
        .unwrap();
        f.into_temp_path()
    }

    #[test]
    fn runs_script_and_returns_data() {
        let script = echo_script();
        let mut input = test_input();
        input.params = serde_json::json!({"key": "value"})
            .as_object()
            .unwrap()
            .clone();
        let result = run_blocking(script.as_ref(), &input, 5).unwrap();
        assert_eq!(result["key"], "value");
    }

    #[test]
    fn returns_error_for_nonexistent_script() {
        let result = run_blocking(
            std::path::Path::new("/nonexistent/script.js"),
            &test_input(),
            5,
        );
        assert!(result.is_err());
    }

    #[test]
    fn returns_error_when_script_returns_ok_false() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            f,
            r#"process.stdout.write(JSON.stringify({{ok: false, error: "something went wrong"}}));"#
        )
        .unwrap();
        let path = f.into_temp_path();
        let result = run_blocking(path.as_ref(), &test_input(), 5);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("something went wrong"));
    }

    #[test]
    fn returns_notifications_from_script() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            f,
            r#"process.stdout.write(JSON.stringify({{
            ok: true,
            data: null,
            notifications: [
                {{ feature_id: "f1", kind: "info", message: "hello" }},
                {{ feature_id: "f1", kind: "warn", message: "heads up" }}
            ]
        }}));"#
        )
        .unwrap();
        let path = f.into_temp_path();
        let out = run_blocking_with_notifications(path.as_ref(), &test_input(), 5).unwrap();
        assert_eq!(out.notifications.len(), 2);
        assert_eq!(out.notifications[0].message, "hello");
        assert_eq!(out.notifications[1].kind, "warn");
        assert_eq!(out.notifications[0].plan_id, None);
    }
}
