use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Mutex;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use tauri::{AppHandle, Emitter};

pub struct TerminalInstance {
    writer: Box<dyn Write + Send>,
    master: Box<dyn portable_pty::MasterPty + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    pub feature_id: String,
    pub session_db_id: Option<String>,
    pub label: Option<String>,
}

#[derive(serde::Serialize)]
pub struct ActiveTerminalInfo {
    pub terminal_id: String,
    pub feature_id: String,
    pub session_db_id: Option<String>,
    pub label: Option<String>,
}

pub struct TerminalState {
    pub terminals: Mutex<HashMap<String, TerminalInstance>>,
}

impl TerminalState {
    pub fn new() -> Self {
        Self {
            terminals: Mutex::new(HashMap::new()),
        }
    }
}

pub fn default_shell() -> (String, Vec<String>) {
    if cfg!(windows) {
        ("powershell.exe".to_string(), vec!["-NoLogo".to_string()])
    } else {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
        (shell, vec!["-l".to_string()])
    }
}

pub fn spawn_pty(
    app: &AppHandle,
    state: &TerminalState,
    feature_id: &str,
    shell: Option<String>,
    args: Option<Vec<String>>,
    cols: u16,
    rows: u16,
    cwd: Option<String>,
) -> Result<String, String> {
    let pty_system = native_pty_system();

    let size = PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    };

    let pair = pty_system
        .openpty(size)
        .map_err(|e| format!("Failed to open PTY: {}", e))?;

    let (shell_cmd, shell_args) = match (shell, args) {
        (Some(s), Some(a)) => (s, a),
        (Some(s), None) => (s, vec![]),
        _ => default_shell(),
    };

    // Use CommandBuilder::from_argv to get a clean command,
    // then copy the current process environment so child inherits PATH etc.
    let mut cmd = CommandBuilder::new(&shell_cmd);
    for arg in &shell_args {
        cmd.arg(arg);
    }

    // Inherit current process environment — critical for MCP servers
    // that need to be found via PATH or use env vars
    for (key, value) in std::env::vars() {
        cmd.env(key, value);
    }

    if let Some(ref dir) = cwd {
        cmd.cwd(dir);
    }

    let child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| format!("Failed to spawn shell: {}", e))?;

    // Drop the slave — we only need the master side
    drop(pair.slave);

    let writer = pair
        .master
        .take_writer()
        .map_err(|e| format!("Failed to get PTY writer: {}", e))?;

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| format!("Failed to get PTY reader: {}", e))?;

    let id = uuid::Uuid::new_v4().to_string();

    let instance = TerminalInstance {
        writer,
        master: pair.master,
        child,
        feature_id: feature_id.to_string(),
        session_db_id: None,
        label: None,
    };

    {
        let mut terminals = state.terminals.lock().map_err(|e| e.to_string())?;
        terminals.insert(id.clone(), instance);
    }

    // Spawn reader thread
    let app_handle = app.clone();
    let terminal_id = id.clone();
    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => {
                    let _ = app_handle.emit(&format!("pty-exit-{}", terminal_id), ());
                    break;
                }
                Ok(n) => {
                    let encoded = BASE64.encode(&buf[..n]);
                    let _ = app_handle.emit(&format!("pty-data-{}", terminal_id), encoded);
                }
                Err(_) => {
                    let _ = app_handle.emit(&format!("pty-exit-{}", terminal_id), ());
                    break;
                }
            }
        }
    });

    Ok(id)
}

pub fn write_pty(state: &TerminalState, id: &str, data: &str) -> Result<(), String> {
    let bytes = BASE64
        .decode(data)
        .map_err(|e| format!("Invalid base64: {}", e))?;
    let mut terminals = state.terminals.lock().map_err(|e| e.to_string())?;
    let instance = terminals
        .get_mut(id)
        .ok_or_else(|| format!("Terminal {} not found", id))?;
    instance
        .writer
        .write_all(&bytes)
        .map_err(|e| format!("Write failed: {}", e))?;
    instance
        .writer
        .flush()
        .map_err(|e| format!("Flush failed: {}", e))?;
    Ok(())
}

pub fn resize_pty(state: &TerminalState, id: &str, cols: u16, rows: u16) -> Result<(), String> {
    let terminals = state.terminals.lock().map_err(|e| e.to_string())?;
    let instance = terminals
        .get(id)
        .ok_or_else(|| format!("Terminal {} not found", id))?;
    instance
        .master
        .resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("Resize failed: {}", e))?;
    Ok(())
}

pub fn kill_pty(state: &TerminalState, id: &str) -> Result<(), String> {
    let mut terminals = state.terminals.lock().map_err(|e| e.to_string())?;
    if let Some(mut instance) = terminals.remove(id) {
        let _ = instance.child.kill();
    }
    Ok(())
}

pub fn kill_all_for_feature(state: &TerminalState, feature_id: &str) -> Result<(), String> {
    let mut terminals = state.terminals.lock().map_err(|e| e.to_string())?;
    let ids: Vec<String> = terminals
        .iter()
        .filter(|(_, inst)| inst.feature_id == feature_id)
        .map(|(id, _)| id.clone())
        .collect();
    for id in ids {
        if let Some(mut instance) = terminals.remove(&id) {
            let _ = instance.child.kill();
        }
    }
    Ok(())
}

pub fn kill_all(state: &TerminalState) -> Result<(), String> {
    let mut terminals = state.terminals.lock().map_err(|e| e.to_string())?;
    for (_, mut instance) in terminals.drain() {
        let _ = instance.child.kill();
    }
    Ok(())
}

pub fn set_session_metadata(
    state: &TerminalState,
    id: &str,
    session_db_id: &str,
    label: &str,
) -> Result<(), String> {
    let mut terminals = state.terminals.lock().map_err(|e| e.to_string())?;
    let instance = terminals
        .get_mut(id)
        .ok_or_else(|| format!("Terminal {} not found", id))?;
    instance.session_db_id = Some(session_db_id.to_string());
    instance.label = Some(label.to_string());
    Ok(())
}

pub fn list_active(state: &TerminalState) -> Result<Vec<ActiveTerminalInfo>, String> {
    let terminals = state.terminals.lock().map_err(|e| e.to_string())?;
    Ok(terminals
        .iter()
        .filter(|(_, inst)| inst.session_db_id.is_some())
        .map(|(id, inst)| ActiveTerminalInfo {
            terminal_id: id.clone(),
            feature_id: inst.feature_id.clone(),
            session_db_id: inst.session_db_id.clone(),
            label: inst.label.clone(),
        })
        .collect())
}
