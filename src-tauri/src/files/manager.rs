use std::path::{Path, PathBuf};

pub fn get_storage_path(base_path: &Path, feature_id: &str) -> PathBuf {
    base_path.join("workspaces").join(feature_id)
}

pub fn ensure_storage_dir(base_path: &Path, feature_id: &str) -> Result<PathBuf, String> {
    let path = get_storage_path(base_path, feature_id);
    std::fs::create_dir_all(&path).map_err(|e| format!("Failed to create storage directory: {}", e))?;
    Ok(path)
}

pub fn copy_file_to_storage(
    base_path: &Path,
    feature_id: &str,
    source_path: &str,
    subfolder: Option<&str>,
) -> Result<(String, String, i64), String> {
    let source = PathBuf::from(source_path);

    if !source.exists() {
        return Err(format!("Source file does not exist: {}", source_path));
    }

    let filename = source
        .file_name()
        .ok_or("Could not determine filename")?
        .to_string_lossy()
        .to_string();

    let storage_dir = ensure_storage_dir(base_path, feature_id)?;
    let target_dir = if let Some(sub) = subfolder {
        let dir = storage_dir.join(sub);
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create subfolder: {}", e))?;
        dir
    } else {
        storage_dir
    };

    let mut dest_path = target_dir.join(&filename);

    // Handle filename collisions by appending a number
    if dest_path.exists() {
        let stem = source
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let ext = source
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default();

        let mut counter = 1;
        loop {
            let new_name = format!("{}_{}{}", stem, counter, ext);
            dest_path = target_dir.join(&new_name);
            if !dest_path.exists() {
                break;
            }
            counter += 1;
        }
    }

    let size = std::fs::metadata(&source)
        .map_err(|e| format!("Failed to read file metadata: {}", e))?
        .len() as i64;

    std::fs::copy(&source, &dest_path)
        .map_err(|e| format!("Failed to copy file: {}", e))?;

    let dest_str = dest_path
        .strip_prefix(base_path)
        .map(|r| r.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| dest_path.to_string_lossy().to_string());
    let final_filename = dest_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    Ok((final_filename, dest_str, size))
}

pub fn create_folder_on_disk(
    base_path: &Path,
    feature_id: &str,
    relative_path: &str,
) -> Result<(), String> {
    let dir = get_storage_path(base_path, feature_id).join(relative_path);
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create folder on disk: {}", e))?;
    Ok(())
}

pub fn move_file_on_disk(
    old_stored_path: &str,
    new_dir_path: &str,
) -> Result<String, String> {
    let old_path = PathBuf::from(old_stored_path);
    let filename = old_path
        .file_name()
        .ok_or("Could not determine filename")?
        .to_string_lossy()
        .to_string();

    let new_dir = PathBuf::from(new_dir_path);
    std::fs::create_dir_all(&new_dir)
        .map_err(|e| format!("Failed to create target directory: {}", e))?;

    let mut new_path = new_dir.join(&filename);

    // Handle collision
    if new_path.exists() && new_path != old_path {
        let stem = old_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let ext = old_path
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default();
        let mut counter = 1;
        loop {
            let new_name = format!("{}_{}{}", stem, counter, ext);
            new_path = new_dir.join(&new_name);
            if !new_path.exists() {
                break;
            }
            counter += 1;
        }
    }

    if old_path != new_path {
        std::fs::rename(&old_path, &new_path)
            .map_err(|e| format!("Failed to move file: {}", e))?;
    }

    Ok(new_path.to_string_lossy().to_string())
}

pub fn rename_folder_on_disk(old_path: &str, new_path: &str) -> Result<(), String> {
    let old = PathBuf::from(old_path);
    let new = PathBuf::from(new_path);

    if old == new {
        return Ok(());
    }

    if !old.exists() {
        // Folder might not exist on disk yet
        std::fs::create_dir_all(&new)
            .map_err(|e| format!("Failed to create renamed folder: {}", e))?;
        return Ok(());
    }

    std::fs::rename(&old, &new)
        .map_err(|e| format!("Failed to rename folder on disk: {}", e))?;

    Ok(())
}

pub fn delete_folder_on_disk(
    base_path: &Path,
    feature_id: &str,
    relative_path: &str,
) -> Result<(), String> {
    let dir = get_storage_path(base_path, feature_id).join(relative_path);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)
            .map_err(|e| format!("Failed to delete folder from disk: {}", e))?;
    }
    Ok(())
}
