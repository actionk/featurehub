pub mod claude;
mod commands;
pub mod config;
pub mod db;
pub mod export_import;
pub mod extensions;
mod files;
pub mod git;
pub mod paths;
pub mod storage;
pub mod terminal;

use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Emitter, Manager, WindowEvent};
use tauri_plugin_window_state::{AppHandleExt, StateFlags};

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub storage_path: Mutex<Option<PathBuf>>,
    pub extensions: Mutex<extensions::ExtensionRegistry>,
    pub schedule_handles: Mutex<Vec<extensions::scheduler::ScheduleHandle>>,
    pub stats_cache: Mutex<std::collections::HashMap<String, claude::session_parser::CachedStats>>,
    pub jsonl_path_cache: Mutex<std::collections::HashMap<String, PathBuf>>,
}

pub fn spawn_extension_schedules(
    extensions: &extensions::ExtensionRegistry,
    storage_path: &std::path::Path,
) -> Vec<extensions::scheduler::ScheduleHandle> {
    let mut handles = Vec::new();
    let db_path_str = storage_path
        .join("feature-hub.db")
        .to_string_lossy()
        .to_string();
    let storage_path_str = storage_path.to_string_lossy().to_string();
    for ext in &extensions.extensions {
        if !ext.enabled {
            continue;
        }
        for schedule in &ext.manifest.schedules {
            let enabled = match &schedule.enabled_setting {
                None => true,
                Some(key) => extensions::extension_setting_bool(storage_path, &ext.manifest, key)
                    .unwrap_or(false),
            };
            if !enabled {
                continue;
            }
            handles.push(extensions::scheduler::spawn_schedule(
                ext.manifest.id.clone(),
                ext.dir.clone(),
                schedule.clone(),
                db_path_str.clone(),
                storage_path_str.clone(),
            ));
        }
    }
    handles
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .setup(|app| {
            // Try to open the active storage's DB
            let (conn, storage_path, extension_registry, schedule_handles) = match storage::get_active_storage(&app.handle()) {
                Ok(Some(entry)) => {
                    let path = PathBuf::from(&entry.path);
                    let db_path = path.join("feature-hub.db");
                    std::fs::create_dir_all(&path).ok();
                    std::fs::create_dir_all(path.join("workspaces")).ok();
                    let conn = rusqlite::Connection::open(&db_path)
                        .expect("Failed to open database");
                    db::initialize(&conn).expect("Failed to initialize database");
                    db::migrate_to_relative_paths(&conn, &path);
                    // Load extensions from both built-in (app resources) and user storage.
                    // Storage extensions override built-in on id collision.
                    let extension_registry = {
                        let builtin_dir = builtin_extensions_dir(&app.handle());
                        let storage_ext_dir = path.join("extensions");
                        let dirs: Vec<&std::path::Path> = [
                            builtin_dir.as_deref(),
                            Some(storage_ext_dir.as_path()),
                        ]
                        .into_iter()
                        .flatten()
                        .collect();
                        let registry = extensions::ExtensionRegistry::load_from_dirs(&dirs, Some(&path));
                        let table_decls: Vec<_> = registry
                            .extensions
                            .iter()
                            .flat_map(|e| &e.manifest.tables)
                            .collect();
                        for table in &table_decls {
                            if let Err(e) = extensions::table_provisioner::provision_table(&conn, table) {
                                eprintln!("[extensions] Table provisioning failed: {}", e);
                            }
                        }
                        registry
                    };

                    // Spawn schedules for loaded extensions.
                    let schedule_handles = spawn_extension_schedules(&extension_registry, &path);

                    (conn, Some(path), extension_registry, schedule_handles)
                }
                _ => {
                    // No active storage — open in-memory placeholder
                    let conn = rusqlite::Connection::open_in_memory()
                        .expect("Failed to open in-memory database");
                    db::initialize(&conn).expect("Failed to initialize database");
                    (conn, None, extensions::ExtensionRegistry::default(), Vec::new())
                }
            };

            let state = AppState {
                db: Mutex::new(conn),
                storage_path: Mutex::new(storage_path),
                extensions: Mutex::new(extension_registry),
                schedule_handles: Mutex::new(schedule_handles),
                stats_cache: Mutex::new(std::collections::HashMap::new()),
                jsonl_path_cache: Mutex::new(std::collections::HashMap::new()),
            };

            app.manage(state);
            app.manage(terminal::TerminalState::new());

            // Save window state to disk on every move/resize so it survives
            // unclean shutdowns (common in dev mode with Ctrl+C / hot reload).
            let handle = app.handle().clone();
            if let Some(window) = app.get_webview_window("main") {
                let window_handle = window.clone();
                window.on_window_event(move |event| {
                    match event {
                        WindowEvent::Moved(_) | WindowEvent::Resized(_) => {
                            let _ = handle.save_window_state(StateFlags::all());
                            if let Ok(maximized) = window_handle.is_maximized() {
                                let _ = window_handle.emit("window-maximized", maximized);
                            }
                        }
                        _ => {}
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_features,
            commands::get_feature,
            commands::get_feature_data,
            commands::create_feature,
            commands::update_feature,
            commands::delete_feature,
            commands::reorder_features,
            commands::duplicate_feature,
            commands::toggle_pin_feature,
            commands::set_feature_archived,
            commands::set_feature_parent,
            commands::get_feature_groups,
            commands::create_feature_group,
            commands::update_feature_group,
            commands::delete_feature_group,
            commands::reorder_feature_groups,
            commands::set_feature_group,
            commands::add_link,
            commands::update_link,
            commands::delete_link,
            commands::delete_link_by_url,
            commands::get_tags,
            commands::create_tag,
            commands::delete_tag,
            commands::toggle_tag,
            commands::add_directory,
            commands::remove_directory,
            commands::clone_repository,
            commands::retry_clone,
            commands::get_git_current_branch,
            commands::get_git_status,
            commands::list_git_branches,
            commands::checkout_git_branch,
            commands::git_fetch,
            commands::create_git_branch,
            commands::get_files,
            commands::add_files,
            commands::delete_file,
            commands::open_file,
            commands::reveal_file,
            commands::get_file_path,
            commands::get_files_directory,
            commands::open_files_directory,
            commands::sync_workspace_files,
            commands::open_path,
            commands::detect_ides,
            commands::open_in_ide,
            commands::get_folders,
            commands::create_folder,
            commands::rename_folder,
            commands::delete_folder,
            commands::move_folder,
            commands::move_file,
            commands::rename_file,
            commands::preview_file,
            commands::save_file_content,
            commands::get_sessions,
            commands::scan_sessions,
            commands::check_session_active,
            commands::get_active_session_counts,
            commands::get_active_session_activity,
            commands::get_sessions_panel_data,
            commands::link_session,
            commands::rename_session,
            commands::unlink_session,
            commands::resume_session,
            commands::ensure_mcp_config,
            commands::start_new_session,
            commands::get_tasks,
            commands::create_task,
            commands::update_task,
            commands::delete_task,
            commands::get_plans,
            commands::get_plan,
            commands::resolve_plan,
            commands::delete_plan,

            commands::get_note,
            commands::save_note,
            commands::get_context,
            commands::save_context,
            commands::get_timeline,
            commands::get_global_timeline,
            commands::global_search,
            commands::rebuild_search_index,
            commands::get_storages,
            commands::get_active_storage,
            commands::create_storage,
            commands::switch_storage,
            commands::remove_storage,
            commands::rename_storage,
            commands::update_storage_icon,
            commands::get_storage_git_status,
            commands::pick_storage_folder,
            commands::get_feature_mcp_servers,
            commands::set_feature_mcp_server,
            commands::get_feature_skills,
            commands::set_feature_skill,
            commands::get_fh_cli_path,
            commands::get_settings,
            commands::save_settings,
            commands::install_cli_to_path,
            commands::check_cli_installed,
            commands::poll_notifications,
            commands::pty_spawn,
            commands::pty_write,
            commands::pty_resize,
            commands::pty_kill,
            commands::pty_kill_feature,
            commands::cleanup_feature_repos,
            commands::pty_spawn_session,
            commands::pty_resume_session,
            commands::pty_list_active,
            commands::pty_get_scrollback,
            commands::finish_embedded_session,
            commands::cleanup_orphaned_sessions,
            commands::export_storage,
            commands::cancel_export,
            commands::check_import_zip,
            commands::import_storage,
            commands::restore_repo_from_export,
            commands::get_knowledge_folders,
            commands::create_knowledge_folder,
            commands::rename_knowledge_folder,
            commands::delete_knowledge_folder,
            commands::get_knowledge_entries,
            commands::get_all_knowledge_entries,
            commands::get_knowledge_entry,
            commands::create_knowledge_entry,
            commands::update_knowledge_entry,
            commands::delete_knowledge_entry,
            commands::get_extensions,
            commands::get_extension_badge,
            commands::get_extension_settings,
            commands::set_extension_settings,
            commands::invoke_extension_tool,
            commands::shell_open_url,

            commands::restart_extension_schedules,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Locate the directory holding built-in extensions shipped with the app.
/// In debug builds, point directly at the repo's `extensions/` dir so edits are live.
/// In release builds, use Tauri's resource_dir (where `bundle.resources` copies them).
fn builtin_extensions_dir(app: &tauri::AppHandle) -> Option<PathBuf> {
    #[cfg(debug_assertions)]
    {
        let _ = app;
        let repo_ext = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("extensions");
        if repo_ext.exists() {
            return Some(repo_ext);
        }
        None
    }
    #[cfg(not(debug_assertions))]
    {
        use tauri::Manager;
        let resource_dir = app.path().resource_dir().ok()?;
        let candidate = resource_dir.join("extensions");
        if candidate.exists() {
            Some(candidate)
        } else {
            None
        }
    }
}

