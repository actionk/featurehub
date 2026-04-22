use std::path::Path;

use crate::git;

#[derive(serde::Serialize)]
pub struct GitStatusSummary {
    pub branch: String,
    pub modified: usize,
    pub untracked: usize,
    pub staged: usize,
    pub ahead: Option<usize>,
    pub behind: Option<usize>,
}

#[tauri::command]
pub async fn get_git_current_branch(directory_path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || git::get_current_branch(Path::new(&directory_path)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn list_git_branches(directory_path: String) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || git::list_local_branches(Path::new(&directory_path)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn checkout_git_branch(directory_path: String, branch_name: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || git::checkout_branch(Path::new(&directory_path), &branch_name))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_git_status(directory_path: String) -> Result<GitStatusSummary, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let dir = Path::new(&directory_path);
        let branch = git::get_current_branch(dir)?;
        let (modified, untracked, staged) = git::get_status_counts(dir)?;
        let (ahead, behind) = git::get_ahead_behind(dir)
            .map(|(a, b)| (Some(a), Some(b)))
            .unwrap_or((None, None));
        Ok(GitStatusSummary { branch, modified, untracked, staged, ahead, behind })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
#[allow(dead_code)] // Used via tauri::generate_handler! macro in lib.rs
pub async fn git_fetch(directory_path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || git::fetch(Path::new(&directory_path)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
#[allow(dead_code)] // Used via tauri::generate_handler! macro in lib.rs
pub async fn create_git_branch(directory_path: String, branch_name: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        git::create_branch(Path::new(&directory_path), &branch_name)
    })
    .await
    .map_err(|e| e.to_string())?
}
