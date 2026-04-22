use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::Emitter;
use tauri::State;
use tauri_plugin_dialog::DialogExt;

use crate::export_import;
use crate::AppState;

#[derive(Clone, serde::Serialize)]
struct ProgressPayload {
    stage: String,
    percent: u32,
}

/// Shared cancel flag — set by cancel_export, checked by export_storage.
static EXPORT_CANCELLED: std::sync::OnceLock<Arc<AtomicBool>> = std::sync::OnceLock::new();

fn get_cancel_flag() -> Arc<AtomicBool> {
    EXPORT_CANCELLED
        .get_or_init(|| Arc::new(AtomicBool::new(false)))
        .clone()
}

#[tauri::command]
pub fn cancel_export() {
    get_cancel_flag().store(true, Ordering::Relaxed);
}

#[tauri::command]
pub async fn export_storage(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    options: export_import::ExportOptions,
) -> Result<String, String> {
    // Reset cancel flag
    let cancelled = get_cancel_flag();
    cancelled.store(false, Ordering::Relaxed);

    let storage_path = {
        let sp = state.storage_path.lock().map_err(|e| e.to_string())?;
        sp.clone().ok_or("No active storage")?
    };

    // Show save dialog
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_file_name(&format!(
            "feature-hub-export-{}.zip",
            chrono::Local::now().format("%Y-%m-%d_%H%M")
        ))
        .add_filter("ZIP Archive", &["zip"])
        .save_file(move |path| {
            let _ = tx.send(path.map(|p| p.to_string()));
        });
    let result = rx.await.map_err(|e| e.to_string())?;
    let output_path = PathBuf::from(result.ok_or("Export cancelled")?);

    let app_handle = app.clone();
    let output_path_clone = output_path.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        export_import::export_storage(
            &storage_path,
            &output_path,
            &options,
            &cancelled,
            &|stage, percent| {
                let _ = app_handle.emit(
                    "export-progress",
                    ProgressPayload {
                        stage: stage.to_string(),
                        percent,
                    },
                );
            },
        )?;
        Ok(output_path.to_string_lossy().to_string())
    })
    .await
    .map_err(|e| e.to_string())?;

    // If cancelled or failed, clean up the partial zip file
    if result.is_err() {
        let _ = std::fs::remove_file(&output_path_clone);
    }

    result
}

/// Step 1 of import: open a file-picker dialog, inspect the ZIP for duplicates,
/// and return info the frontend needs to ask the user what to do.
#[tauri::command]
pub async fn check_import_zip(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<export_import::ImportCheckResult, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .add_filter("ZIP Archive", &["zip"])
        .pick_file(move |path| {
            let _ = tx.send(path.map(|p| p.to_string()));
        });
    let zip_path = PathBuf::from(
        rx.await
            .map_err(|e| e.to_string())?
            .ok_or("Import cancelled")?,
    );

    let db_path = {
        let sp = state.storage_path.lock().map_err(|e| e.to_string())?;
        sp.clone()
            .ok_or("No active storage")?
            .join("feature-hub.db")
    };

    tauri::async_runtime::spawn_blocking(move || {
        export_import::check_import_zip(&zip_path, &db_path)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Step 2 of import: merge the ZIP into the current active storage using `strategy`
/// ("replace" | "ignore" | "merge").
#[tauri::command]
pub async fn import_storage(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    zip_path: String,
    strategy: String,
) -> Result<export_import::ImportResult, String> {
    let zip_path = PathBuf::from(zip_path);

    let storage_path = {
        let sp = state.storage_path.lock().map_err(|e| e.to_string())?;
        sp.clone().ok_or("No active storage")?
    };

    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        export_import::import_into_current_storage(
            &zip_path,
            &storage_path,
            &strategy,
            &|stage, percent| {
                let _ = app_handle.emit(
                    "import-progress",
                    ProgressPayload {
                        stage: stage.to_string(),
                        percent,
                    },
                );
            },
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn restore_repo_from_export(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    zip_path: String,
    directory_id: String,
    target_path: String,
) -> Result<(), String> {
    let db_path = {
        let sp = state.storage_path.lock().map_err(|e| e.to_string())?;
        sp.clone()
            .ok_or("No active storage")?
            .join("feature-hub.db")
    };

    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _ = app_handle.emit(
            "import-progress",
            ProgressPayload {
                stage: "Cloning repository...".to_string(),
                percent: 30,
            },
        );

        export_import::restore_repo_from_export(
            &PathBuf::from(&zip_path),
            &directory_id,
            &PathBuf::from(&target_path),
            &db_path,
        )?;

        let _ = app_handle.emit(
            "import-progress",
            ProgressPayload {
                stage: "Repository restored!".to_string(),
                percent: 100,
            },
        );

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}
