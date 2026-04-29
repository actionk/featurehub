use std::path::{Path, PathBuf};

/// Convert an absolute path to a storage-relative path if it lives under the storage base.
/// Returns the relative portion (e.g., "workspaces/{fid}/repo") or the original path if external.
/// Always uses forward slashes for cross-platform portability.
pub fn to_storage_relative(path: &str, storage_base: &Path) -> String {
    let p = Path::new(path);
    if let Ok(rel) = p.strip_prefix(storage_base) {
        return rel.to_string_lossy().replace('\\', "/");
    }
    // Fallback: try to extract workspaces/... from an absolute path that doesn't match base
    if let Some(rel) = find_workspaces_relative(p) {
        return rel.to_string_lossy().replace('\\', "/");
    }
    // External path — keep as-is
    path.to_string()
}

/// Resolve a potentially-relative path to an absolute path using the storage base.
/// If the path is already absolute, return it as-is (external dirs, backward compat).
pub fn resolve_path(path: &str, storage_base: &Path) -> PathBuf {
    let p = Path::new(path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        storage_base.join(path)
    }
}

/// Resolve a path and return as String.
pub fn resolve_path_string(path: &str, storage_base: &Path) -> String {
    resolve_path(path, storage_base)
        .to_string_lossy()
        .to_string()
}

/// Extract relative path starting from "workspaces/..." in an absolute path.
pub fn find_workspaces_relative(path: &Path) -> Option<PathBuf> {
    let path_str = path.to_string_lossy();
    let normalized = path_str.replace('\\', "/");
    if let Some(idx) = normalized.find("/workspaces/") {
        Some(PathBuf::from(&normalized[idx + 1..]))
    } else if normalized.starts_with("workspaces/") {
        Some(PathBuf::from(&*normalized))
    } else {
        None
    }
}
