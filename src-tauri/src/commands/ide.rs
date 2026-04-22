use std::path::Path;

#[derive(serde::Serialize, Clone)]
pub struct DetectedIde {
    pub id: String,
    pub name: String,
    pub command: String,
}

#[tauri::command]
pub async fn detect_ides() -> Result<Vec<DetectedIde>, String> {
    tauri::async_runtime::spawn_blocking(|| detect_ides_sync())
        .await
        .map_err(|e| e.to_string())?
}

fn detect_ides_sync() -> Result<Vec<DetectedIde>, String> {
    let mut ides = Vec::new();
    let mut seen = std::collections::HashSet::<String>::new();

    // --- Code editors (PATH-based + install directory scan) ---

    let editors = [
        ("vscode", "VS Code", &["code"][..]),
        ("vscode-insiders", "VS Code Insiders", &["code-insiders"][..]),
        ("cursor", "Cursor", &["cursor"][..]),
        ("windsurf", "Windsurf", &["windsurf"][..]),
    ];

    for (id, name, cmds) in &editors {
        for cmd in *cmds {
            if let Some(path) = which_command(cmd) {
                if seen.insert(id.to_string()) {
                    ides.push(DetectedIde {
                        id: id.to_string(),
                        name: name.to_string(),
                        command: path,
                    });
                }
                break;
            }
        }
    }

    // Scan well-known install directories for code editors
    scan_editor_install_dirs(&mut ides, &mut seen);

    // --- JetBrains IDEs (PATH + Toolbox + install directory scan) ---

    let jetbrains_ides = [
        ("idea", "IntelliJ IDEA", "idea", &["idea", "idea64"][..]),
        ("webstorm", "WebStorm", "webstorm", &["webstorm", "webstorm64"][..]),
        ("pycharm", "PyCharm", "pycharm", &["pycharm", "pycharm64"][..]),
        ("clion", "CLion", "clion", &["clion", "clion64"][..]),
        ("goland", "GoLand", "goland", &["goland", "goland64"][..]),
        ("phpstorm", "PhpStorm", "phpstorm", &["phpstorm", "phpstorm64"][..]),
        ("rider", "Rider", "rider", &["rider", "rider64"][..]),
        ("rustrover", "RustRover", "rustrover", &["rustrover", "rustrover64"][..]),
        ("datagrip", "DataGrip", "datagrip", &["datagrip", "datagrip64"][..]),
        ("fleet", "Fleet", "fleet", &["fleet"][..]),
    ];

    for (id, name, _dir_pattern, cmds) in &jetbrains_ides {
        if seen.contains(*id) {
            continue;
        }
        for cmd in *cmds {
            if let Some(path) = which_command(cmd) {
                if seen.insert(id.to_string()) {
                    ides.push(DetectedIde {
                        id: id.to_string(),
                        name: name.to_string(),
                        command: path,
                    });
                }
                break;
            }
        }
    }

    // Check JetBrains Toolbox scripts
    scan_jetbrains_toolbox(&mut ides, &mut seen, &jetbrains_ides);

    // Scan JetBrains install directories (Program Files, etc.)
    scan_jetbrains_install_dirs(&mut ides, &mut seen, &jetbrains_ides);

    Ok(ides)
}

pub(crate) fn which_command(cmd: &str) -> Option<String> {
    #[cfg(target_os = "windows")]
    let which_cmd = "where";
    #[cfg(not(target_os = "windows"))]
    let which_cmd = "which";

    let output = std::process::Command::new(which_cmd)
        .arg(cmd)
        .output()
        .ok()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let first_line = stdout.lines().next()?.trim().to_string();
        if !first_line.is_empty() {
            return Some(first_line);
        }
    }
    None
}

fn scan_editor_install_dirs(ides: &mut Vec<DetectedIde>, seen: &mut std::collections::HashSet<String>) {
    #[cfg(target_os = "windows")]
    {
        let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
        let program_files = std::env::var("PROGRAMFILES").unwrap_or_default();

        let checks: &[(&str, &str, &[String])] = &[
            ("vscode", "VS Code", &[
                format!("{}/Programs/Microsoft VS Code/bin/code.cmd", local_app_data),
                format!("{}/Microsoft VS Code/bin/code.cmd", program_files),
            ]),
            ("vscode-insiders", "VS Code Insiders", &[
                format!("{}/Programs/Microsoft VS Code Insiders/bin/code-insiders.cmd", local_app_data),
                format!("{}/Microsoft VS Code Insiders/bin/code-insiders.cmd", program_files),
            ]),
            ("cursor", "Cursor", &[
                format!("{}/Programs/cursor/resources/app/bin/cursor.cmd", local_app_data),
                format!("{}/cursor/Cursor.exe", local_app_data),
            ]),
        ];
        for (id, name, paths) in checks {
            if seen.contains(*id) { continue; }
            for p in *paths {
                if Path::new(p).exists() {
                    seen.insert(id.to_string());
                    ides.push(DetectedIde { id: id.to_string(), name: name.to_string(), command: p.clone() });
                    break;
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let macos_editors: &[(&str, &str, &str)] = &[
            ("vscode", "VS Code", "/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code"),
            ("vscode-insiders", "VS Code Insiders", "/Applications/Visual Studio Code - Insiders.app/Contents/Resources/app/bin/code-insiders"),
            ("cursor", "Cursor", "/Applications/Cursor.app/Contents/Resources/app/bin/cursor"),
            ("windsurf", "Windsurf", "/Applications/Windsurf.app/Contents/Resources/app/bin/windsurf"),
        ];
        for &(id, name, path) in macos_editors {
            if !seen.contains(id) && Path::new(path).exists() {
                seen.insert(id.to_string());
                ides.push(DetectedIde { id: id.into(), name: name.into(), command: path.into() });
            }
        }
    }
}

fn scan_jetbrains_toolbox(
    ides: &mut Vec<DetectedIde>,
    seen: &mut std::collections::HashSet<String>,
    jb_ides: &[(&str, &str, &str, &[&str])],
) {
    let scripts_dir = get_jetbrains_toolbox_scripts_dir();
    let scripts_dir = match scripts_dir {
        Some(d) if d.exists() => d,
        _ => return,
    };

    for (id, name, _dir_pattern, cmds) in jb_ides {
        if seen.contains(*id) {
            continue;
        }
        for cmd in *cmds {
            #[cfg(target_os = "windows")]
            let script = scripts_dir.join(format!("{}.cmd", cmd));
            #[cfg(not(target_os = "windows"))]
            let script = scripts_dir.join(cmd);

            if script.exists() {
                seen.insert(id.to_string());
                ides.push(DetectedIde {
                    id: id.to_string(),
                    name: name.to_string(),
                    command: script.to_string_lossy().into_owned(),
                });
                break;
            }
        }
    }
}

fn get_jetbrains_toolbox_scripts_dir() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("LOCALAPPDATA").ok()
            .map(|d| Path::new(&d).join("JetBrains").join("Toolbox").join("scripts"))
    }
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .map(|h| h.join("Library/Application Support/JetBrains/Toolbox/scripts"))
    }
    #[cfg(target_os = "linux")]
    {
        dirs::home_dir()
            .map(|h| h.join(".local/share/JetBrains/Toolbox/scripts"))
    }
}

fn scan_jetbrains_install_dirs(
    ides: &mut Vec<DetectedIde>,
    seen: &mut std::collections::HashSet<String>,
    jb_ides: &[(&str, &str, &str, &[&str])],
) {
    #[cfg(target_os = "windows")]
    {
        let program_files = std::env::var("PROGRAMFILES").unwrap_or_default();
        let jb_base = Path::new(&program_files).join("JetBrains");
        if !jb_base.exists() {
            return;
        }
        let entries = match std::fs::read_dir(&jb_base) {
            Ok(e) => e,
            Err(_) => return,
        };
        let mut dir_names: Vec<String> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();
        // Sort descending so newest version is found first
        dir_names.sort_by(|a, b| b.cmp(a));

        for (id, name, dir_pattern, _cmds) in jb_ides {
            if seen.contains(*id) {
                continue;
            }
            // Find the newest directory matching this IDE that has the exe
            let exe_name = format!("{}64.exe", dir_pattern);
            for dir_name in &dir_names {
                let lower = dir_name.to_lowercase();
                if !lower.contains(dir_pattern) {
                    continue;
                }
                let exe_path = jb_base.join(dir_name).join("bin").join(&exe_name);
                if exe_path.exists() {
                    seen.insert(id.to_string());
                    ides.push(DetectedIde {
                        id: id.to_string(),
                        name: name.to_string(),
                        command: exe_path.to_string_lossy().into_owned(),
                    });
                    break;
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let macos_jb_apps: &[(&str, &str, &[&str])] = &[
            ("idea", "IntelliJ IDEA", &["IntelliJ IDEA.app", "IntelliJ IDEA CE.app"]),
            ("webstorm", "WebStorm", &["WebStorm.app"]),
            ("pycharm", "PyCharm", &["PyCharm.app", "PyCharm CE.app"]),
            ("clion", "CLion", &["CLion.app"]),
            ("goland", "GoLand", &["GoLand.app"]),
            ("phpstorm", "PhpStorm", &["PhpStorm.app"]),
            ("rider", "Rider", &["Rider.app"]),
            ("rustrover", "RustRover", &["RustRover.app"]),
            ("datagrip", "DataGrip", &["DataGrip.app"]),
            ("fleet", "Fleet", &["Fleet.app"]),
        ];
        for &(id, name, app_names) in macos_jb_apps {
            if seen.contains(id) {
                continue;
            }
            for app_name in app_names {
                let app_path = format!("/Applications/{}", app_name);
                if Path::new(&app_path).exists() {
                    seen.insert(id.to_string());
                    ides.push(DetectedIde {
                        id: id.into(),
                        name: name.into(),
                        command: app_path,
                    });
                    break;
                }
            }
        }
    }
}

#[tauri::command]
pub fn open_in_ide(path: String, ide_command: String) -> Result<(), String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    // On macOS, .app bundles need `open -a`
    #[cfg(target_os = "macos")]
    if ide_command.ends_with(".app") {
        std::process::Command::new("open")
            .args(["-a", &ide_command, &path])
            .spawn()
            .map_err(|e| format!("Failed to open {} in IDE: {}", path, e))?;
        return Ok(());
    }

    std::process::Command::new(&ide_command)
        .arg(&path)
        .spawn()
        .map_err(|e| format!("Failed to open {} in IDE: {}", path, e))?;
    Ok(())
}
