/// Read and clear all pending notifications from the shared file.
#[tauri::command]
pub async fn poll_notifications() -> Result<Vec<crate::config::AppNotification>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let path = crate::config::notifications_path()?;
        if !path.exists() {
            return Ok(Vec::new());
        }
        let data = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read notifications: {}", e))?;
        // Clear the file
        std::fs::write(&path, "").map_err(|e| format!("Failed to clear notifications: {}", e))?;

        let notifs: Vec<crate::config::AppNotification> = data
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect();
        Ok(notifs)
    })
    .await
    .map_err(|e| e.to_string())?
}
