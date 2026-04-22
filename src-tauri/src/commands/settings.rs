use std::path::Path;
use tauri::State;

use crate::AppState;

/// Combined settings response sent to the frontend (merges global + storage).
#[derive(serde::Serialize)]
pub struct CombinedSettings {
    // Global fields
    pub fh_cli_path: Option<String>,
    pub mermaid_diagrams: bool,
    pub openfga_highlighting: bool,
    pub show_tab_emojis: bool,
    pub ui_font: Option<String>,
    pub mono_font: Option<String>,
    pub ui_font_size: Option<u32>,
    pub terminal_font_size: Option<u32>,
    pub preferred_ides: Vec<String>,
    // Storage-specific fields
    pub mcp_servers: Vec<crate::config::McpServer>,
    pub default_repositories: Vec<crate::config::Repository>,
    pub extensions: Vec<crate::config::Extension>,
    pub skills: Vec<crate::config::Skill>,
}

impl CombinedSettings {
    fn from(global: &crate::config::AppSettings, storage: &crate::config::StorageSettings) -> Self {
        Self {
            fh_cli_path: global.fh_cli_path.clone(),
            mermaid_diagrams: global.mermaid_diagrams,
            openfga_highlighting: global.openfga_highlighting,
            show_tab_emojis: global.show_tab_emojis,
            ui_font: global.ui_font.clone(),
            mono_font: global.mono_font.clone(),
            ui_font_size: global.ui_font_size,
            terminal_font_size: global.terminal_font_size,
            preferred_ides: global.preferred_ides.clone(),
            mcp_servers: storage.mcp_servers.clone(),
            default_repositories: storage.default_repositories.clone(),
            extensions: storage.extensions.clone(),
            skills: storage.skills.clone(),
        }
    }
}

#[derive(serde::Serialize)]
pub struct CliInstallResult {
    pub install_dir: String,
    pub binaries: Vec<String>,
    pub path_updated: bool,
}

#[tauri::command]
pub fn get_fh_cli_path() -> Result<String, String> {
    // Check user-configured path first
    if let Ok(settings) = crate::config::load_settings() {
        if let Some(ref custom) = settings.fh_cli_path {
            if !custom.is_empty() && std::path::Path::new(custom).exists() {
                return Ok(custom.clone());
            }
        }
    }

    // Fall back to binary next to the running executable
    let exe = std::env::current_exe().map_err(|e| format!("Failed to get exe path: {}", e))?;
    let dir = exe.parent().ok_or("Failed to get exe directory")?;
    let fh = dir.join(if cfg!(windows) { "fh.exe" } else { "fh" });
    if fh.exists() {
        Ok(fh.to_string_lossy().to_string())
    } else {
        Err(format!("fh CLI not found at {}", fh.display()))
    }
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<CombinedSettings, String> {
    let global = crate::config::load_settings().unwrap_or_default();
    let sp = state.storage_path.lock().map_err(|e| e.to_string())?;
    let storage = match sp.as_ref() {
        Some(path) => crate::config::load_storage_settings(path)?,
        None => crate::config::StorageSettings::default(),
    };
    Ok(CombinedSettings::from(&global, &storage))
}

#[tauri::command]
pub fn save_settings(
    state: State<'_, AppState>,
    fh_cli_path: Option<String>,
    mcp_servers: Option<Vec<crate::config::McpServer>>,
    default_repositories: Option<Vec<crate::config::Repository>>,
    mermaid_diagrams: Option<bool>,
    openfga_highlighting: Option<bool>,
    show_tab_emojis: Option<bool>,
    ui_font: Option<String>,
    mono_font: Option<String>,
    ui_font_size: Option<u32>,
    terminal_font_size: Option<u32>,
    extensions: Option<Vec<crate::config::Extension>>,
    preferred_ides: Option<Vec<String>>,
    skills: Option<Vec<crate::config::Skill>>,
) -> Result<CombinedSettings, String> {
    // Update global settings
    let mut global = crate::config::load_settings().unwrap_or_default();
    if let Some(p) = fh_cli_path {
        global.fh_cli_path = if p.is_empty() { None } else { Some(p) };
    }
    if let Some(mermaid) = mermaid_diagrams {
        global.mermaid_diagrams = mermaid;
    }
    if let Some(openfga) = openfga_highlighting {
        global.openfga_highlighting = openfga;
    }
    if let Some(emojis) = show_tab_emojis {
        global.show_tab_emojis = emojis;
    }
    if let Some(f) = ui_font {
        global.ui_font = if f.is_empty() { None } else { Some(f) };
    }
    if let Some(f) = mono_font {
        global.mono_font = if f.is_empty() { None } else { Some(f) };
    }
    if let Some(size) = ui_font_size {
        global.ui_font_size = if size == 0 { None } else { Some(size) };
    }
    if let Some(size) = terminal_font_size {
        global.terminal_font_size = if size == 0 { None } else { Some(size) };
    }
    if let Some(ides) = preferred_ides {
        global.preferred_ides = ides;
    }
    crate::config::save_settings(&global)?;

    // Update storage-specific settings
    let sp = state.storage_path.lock().map_err(|e| e.to_string())?;
    let storage = match sp.as_ref() {
        Some(path) => {
            let mut storage = crate::config::load_storage_settings(path).unwrap_or_default();
            if let Some(servers) = mcp_servers {
                storage.mcp_servers = servers;
            }
            if let Some(repos) = default_repositories {
                storage.default_repositories = repos;
            }
            if let Some(exts) = extensions {
                storage.extensions = exts;
            }
            if let Some(s) = skills {
                storage.skills = s;
            }
            crate::config::save_storage_settings(path, &storage)?;
            storage
        }
        None => crate::config::StorageSettings::default(),
    };

    Ok(CombinedSettings::from(&global, &storage))
}

#[tauri::command]
pub async fn install_cli_to_path() -> Result<CliInstallResult, String> {
    tauri::async_runtime::spawn_blocking(|| install_cli_to_path_sync())
        .await
        .map_err(|e| e.to_string())?
}

fn install_cli_to_path_sync() -> Result<CliInstallResult, String> {
    // Find the source binaries (siblings of the running exe)
    let exe = std::env::current_exe().map_err(|e| format!("Failed to get exe path: {}", e))?;
    let exe_dir = exe.parent().ok_or("Failed to get exe directory")?;

    let (fh_name, fh_mcp_name) = if cfg!(windows) {
        ("fh.exe", "fh-mcp.exe")
    } else {
        ("fh", "fh-mcp")
    };

    let fh_src = exe_dir.join(fh_name);
    let fh_mcp_src = exe_dir.join(fh_mcp_name);

    if !fh_src.exists() {
        return Err(format!(
            "fh binary not found at {}. Build it first with: cd src-tauri && cargo build",
            fh_src.display()
        ));
    }

    if cfg!(windows) {
        install_cli_windows(&fh_src, &fh_mcp_src, fh_name, fh_mcp_name)
    } else {
        install_cli_unix(&fh_src, &fh_mcp_src, fh_name, fh_mcp_name)
    }
}

fn install_cli_windows(
    fh_src: &std::path::Path,
    fh_mcp_src: &std::path::Path,
    fh_name: &str,
    fh_mcp_name: &str,
) -> Result<CliInstallResult, String> {
    // Install to %LOCALAPPDATA%\Programs\FeatureHub\
    let local_app_data = std::env::var("LOCALAPPDATA")
        .map_err(|_| "Could not find LOCALAPPDATA directory")?;
    let install_dir = std::path::PathBuf::from(&local_app_data)
        .join("Programs")
        .join("FeatureHub");

    std::fs::create_dir_all(&install_dir)
        .map_err(|e| format!("Failed to create install directory: {}", e))?;

    // Copy binaries
    let mut binaries = Vec::new();
    std::fs::copy(fh_src, install_dir.join(fh_name))
        .map_err(|e| format!("Failed to copy {}: {}", fh_name, e))?;
    binaries.push(fh_name.to_string());

    if fh_mcp_src.exists() {
        std::fs::copy(fh_mcp_src, install_dir.join(fh_mcp_name))
            .map_err(|e| format!("Failed to copy {}: {}", fh_mcp_name, e))?;
        binaries.push(fh_mcp_name.to_string());
    }

    // Add to user PATH via PowerShell (reads current user PATH from registry, appends if missing)
    let install_dir_str = install_dir.to_string_lossy().to_string();
    let ps_script = format!(
        "$p = [Environment]::GetEnvironmentVariable('Path','User'); \
         if ($p -and $p.Split(';') -contains '{}') {{ exit 0 }}; \
         $newPath = if ($p) {{ $p + ';{}' }} else {{ '{}' }}; \
         [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')",
        install_dir_str.replace("'", "''"),
        install_dir_str.replace("'", "''"),
        install_dir_str.replace("'", "''"),
    );

    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_script])
        .status()
        .map_err(|e| format!("Failed to update PATH: {}", e))?;

    let path_updated = status.success();

    // Also update fh_cli_path in settings
    let fh_installed = install_dir.join(fh_name);
    if let Ok(mut settings) = crate::config::load_settings() {
        settings.fh_cli_path = Some(fh_installed.to_string_lossy().to_string());
        let _ = crate::config::save_settings(&settings);
    }

    Ok(CliInstallResult {
        install_dir: install_dir_str,
        binaries,
        path_updated,
    })
}

fn install_cli_unix(
    fh_src: &std::path::Path,
    fh_mcp_src: &std::path::Path,
    fh_name: &str,
    fh_mcp_name: &str,
) -> Result<CliInstallResult, String> {
    // Try /usr/local/bin first, fall back to ~/.local/bin
    let (install_dir, path_needs_update) = {
        let usr_local = std::path::PathBuf::from("/usr/local/bin");
        if usr_local.exists() && is_writable(&usr_local) {
            (usr_local, false)
        } else {
            let home = dirs::home_dir().ok_or("Could not find home directory")?;
            let local_bin = home.join(".local").join("bin");
            std::fs::create_dir_all(&local_bin)
                .map_err(|e| format!("Failed to create ~/.local/bin: {}", e))?;
            // Check if ~/.local/bin is in PATH
            let in_path = std::env::var("PATH")
                .unwrap_or_default()
                .split(':')
                .any(|p| p == local_bin.to_string_lossy());
            (local_bin, !in_path)
        }
    };

    // Copy binaries (symlinks can break if source moves, copies are more robust)
    let mut binaries = Vec::new();
    std::fs::copy(fh_src, install_dir.join(fh_name))
        .map_err(|e| format!("Failed to copy {}: {}", fh_name, e))?;
    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            install_dir.join(fh_name),
            std::fs::Permissions::from_mode(0o755),
        )
        .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }
    binaries.push(fh_name.to_string());

    if fh_mcp_src.exists() {
        std::fs::copy(fh_mcp_src, install_dir.join(fh_mcp_name))
            .map_err(|e| format!("Failed to copy {}: {}", fh_mcp_name, e))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                install_dir.join(fh_mcp_name),
                std::fs::Permissions::from_mode(0o755),
            )
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
        }
        binaries.push(fh_mcp_name.to_string());
    }

    // Update fh_cli_path in settings
    let fh_installed = install_dir.join(fh_name);
    if let Ok(mut settings) = crate::config::load_settings() {
        settings.fh_cli_path = Some(fh_installed.to_string_lossy().to_string());
        let _ = crate::config::save_settings(&settings);
    }

    let install_dir_str = install_dir.to_string_lossy().to_string();

    Ok(CliInstallResult {
        install_dir: install_dir_str,
        binaries,
        path_updated: !path_needs_update,
    })
}

fn is_writable(path: &Path) -> bool {
    // Try creating a temp file to check write access
    let test_path = path.join(".fh-write-test");
    match std::fs::write(&test_path, "") {
        Ok(_) => {
            let _ = std::fs::remove_file(&test_path);
            true
        }
        Err(_) => false,
    }
}

#[tauri::command]
pub async fn check_cli_installed() -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(|| check_cli_installed_sync())
        .await
        .map_err(|e| e.to_string())?
}

fn check_cli_installed_sync() -> Result<Option<String>, String> {
    // Check if fh is available in PATH
    let fh_name = if cfg!(windows) { "fh.exe" } else { "fh" };

    if cfg!(windows) {
        let output = std::process::Command::new("where")
            .arg(fh_name)
            .output()
            .map_err(|e| format!("Failed to run where: {}", e))?;
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            if !path.is_empty() {
                return Ok(Some(path));
            }
        }
    } else {
        let output = std::process::Command::new("which")
            .arg(fh_name)
            .output()
            .map_err(|e| format!("Failed to run which: {}", e))?;
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Ok(Some(path));
            }
        }
    }

    Ok(None)
}
