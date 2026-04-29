use std::path::PathBuf;

fn claude_json_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude.json"))
}

/// Pre-accept the "Do you trust the files in this folder?" dialog for the given
/// directories by writing `projects[dir].hasTrustDialogAccepted = true` into
/// the user's `~/.claude.json`. Best-effort — failures are logged and ignored.
pub fn accept_dirs(dirs: &[&str]) {
    if dirs.is_empty() {
        return;
    }
    let path = match claude_json_path() {
        Some(p) => p,
        None => return,
    };

    let mut root: serde_json::Value = match std::fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_else(|_| serde_json::json!({})),
        Err(_) => serde_json::json!({}),
    };

    if !root.is_object() {
        root = serde_json::json!({});
    }

    let Some(root_obj) = root.as_object_mut() else {
        return;
    };
    let projects = root_obj
        .entry("projects")
        .or_insert_with(|| serde_json::json!({}));
    if !projects.is_object() {
        *projects = serde_json::json!({});
    }
    let Some(projects_obj) = projects.as_object_mut() else {
        return;
    };

    let mut changed = false;
    for dir in dirs {
        let key = dir.replace('\\', "/");
        let entry = projects_obj
            .entry(&key)
            .or_insert_with(|| serde_json::json!({}));
        if !entry.is_object() {
            *entry = serde_json::json!({});
        }
        let Some(obj) = entry.as_object_mut() else {
            continue;
        };
        let already = obj
            .get("hasTrustDialogAccepted")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if !already {
            obj.insert(
                "hasTrustDialogAccepted".to_string(),
                serde_json::json!(true),
            );
            changed = true;
        }
    }

    if !changed {
        return;
    }

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match serde_json::to_string_pretty(&root) {
        Ok(s) => {
            if let Err(e) = std::fs::write(&path, s) {
                eprintln!("[claude::trust] failed to write {}: {}", path.display(), e);
            }
        }
        Err(e) => eprintln!("[claude::trust] failed to serialize: {}", e),
    }
}
