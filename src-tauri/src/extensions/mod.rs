pub mod manifest;
pub mod scheduler;
pub mod script_runner;
pub mod table_provisioner;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use manifest::{ExtensionManifest, ToolDecl};

#[derive(Debug, Clone)]
pub struct RequiresStatus {
    pub name: String,
    pub found: bool,
    pub path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LoadedExtension {
    pub manifest: ExtensionManifest,
    pub enabled: bool,
    pub dir: PathBuf,
    pub requires_status: Vec<RequiresStatus>,
}

#[derive(Debug, Clone, Default)]
pub struct ExtensionRegistry {
    pub extensions: Vec<LoadedExtension>,
}

impl ExtensionRegistry {
    pub fn load_from_dir(extensions_dir: &Path) -> Self {
        Self::load_from_dirs(&[extensions_dir], None)
    }

    /// Load extensions from multiple directories. Later dirs override earlier ones on id collision
    /// (so pass `[builtin, storage]` to let user-installed extensions override bundled ones).
    /// If `storage_path` is provided, reads `extension_settings.<id>.enabled` from that storage's
    /// settings.json to set per-extension enabled state (defaults true).
    pub fn load_from_dirs(dirs: &[&Path], storage_path: Option<&Path>) -> Self {
        let mut by_id: std::collections::HashMap<String, LoadedExtension> =
            std::collections::HashMap::new();
        for dir in dirs {
            Self::scan_into(dir, &mut by_id);
        }
        let mut extensions: Vec<LoadedExtension> = by_id.into_values().collect();
        if let Some(sp) = storage_path {
            for ext in extensions.iter_mut() {
                if let Some(flag) = read_enabled_flag(sp, &ext.manifest.id) {
                    ext.enabled = flag;
                }
            }
        }
        Self { extensions }
    }

    fn scan_into(
        extensions_dir: &Path,
        by_id: &mut std::collections::HashMap<String, LoadedExtension>,
    ) {
        if !extensions_dir.exists() {
            return;
        }
        let entries = match std::fs::read_dir(extensions_dir) {
            Ok(e) => e,
            Err(e) => {
                eprintln!(
                    "[extensions] Failed to read dir {:?}: {}",
                    extensions_dir, e
                );
                return;
            }
        };
        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            let manifest_path = dir.join("extension.json");
            if !manifest_path.exists() {
                continue;
            }
            let content = match std::fs::read_to_string(&manifest_path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[extensions] Failed to read {:?}: {}", manifest_path, e);
                    continue;
                }
            };
            let manifest: ExtensionManifest = match serde_json::from_str(&content) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("[extensions] Failed to parse {:?}: {}", manifest_path, e);
                    continue;
                }
            };
            if let Err(e) = manifest.validate() {
                eprintln!("[extensions] Invalid manifest in {:?}: {}", dir, e);
                continue;
            }
            // Table name collision check across all previously accepted extensions
            let claimed_tables: std::collections::HashSet<&str> = by_id
                .values()
                .filter(|e| e.manifest.id != manifest.id)
                .flat_map(|e| e.manifest.tables.iter().map(|t| t.name.as_str()))
                .collect();
            let collision = manifest
                .tables
                .iter()
                .find(|t| claimed_tables.contains(t.name.as_str()));
            if let Some(table) = collision {
                eprintln!(
                    "[extensions] Table '{}' declared by '{}' conflicts with an already-loaded extension, skipping {:?}",
                    table.name, manifest.id, dir
                );
                continue;
            }
            let requires_status = check_requires(&manifest.requires);
            let id = manifest.id.clone();
            by_id.insert(
                id,
                LoadedExtension {
                    manifest,
                    enabled: true,
                    dir,
                    requires_status,
                },
            );
        }
    }

    pub fn find_tool<'a>(&'a self, tool_name: &str) -> Option<(&'a LoadedExtension, &'a ToolDecl)> {
        for ext in &self.extensions {
            if !ext.enabled {
                continue;
            }
            for tool in &ext.manifest.tools {
                if tool.name == tool_name {
                    return Some((ext, tool));
                }
            }
        }
        None
    }

    pub fn handlers_for_event(
        &self,
        event_type: &str,
        payload: &serde_json::Value,
    ) -> Vec<(PathBuf, String)> {
        let mut out = Vec::new();
        for ext in &self.extensions {
            if !ext.enabled {
                continue;
            }
            for event_decl in &ext.manifest.events {
                if event_decl.on != event_type {
                    continue;
                }
                if !event_filter_matches(&event_decl.filter, payload) {
                    continue;
                }
                out.push((ext.dir.join(&event_decl.handler), ext.manifest.id.clone()));
            }
        }
        out
    }
}

/// Read `extension_settings.<id>.enabled` from the storage's settings.json. Returns None
/// if the file, keys, or value are missing.
fn read_enabled_flag(storage_path: &Path, extension_id: &str) -> Option<bool> {
    let path = storage_path.join("settings.json");
    let text = std::fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    v.get("extension_settings")?
        .get(extension_id)?
        .get("enabled")?
        .as_bool()
}

pub fn check_requires(requires: &[String]) -> Vec<RequiresStatus> {
    requires
        .iter()
        .map(|name| {
            let result = if cfg!(windows) {
                std::process::Command::new("where").arg(name).output()
            } else {
                std::process::Command::new("which").arg(name).output()
            };
            match result {
                Ok(output) if output.status.success() => RequiresStatus {
                    name: name.clone(),
                    found: true,
                    // Take only the first line: `where` on Windows can return multiple paths.
                    path: String::from_utf8(output.stdout)
                        .ok()
                        .and_then(|s| s.lines().next().map(|l| l.trim().to_string())),
                },
                _ => RequiresStatus {
                    name: name.clone(),
                    found: false,
                    path: None,
                },
            }
        })
        .collect()
}

pub(crate) fn event_filter_matches(
    filter: &HashMap<String, String>,
    payload: &serde_json::Value,
) -> bool {
    for (key, expected) in filter {
        match payload.get(key).and_then(|v| v.as_str()) {
            Some(val) if val == expected => {}
            _ => return false,
        }
    }
    true
}

pub fn extension_setting_bool(
    storage_path: &std::path::Path,
    manifest: &manifest::ExtensionManifest,
    key: &str,
) -> Option<bool> {
    let settings_key = manifest.storage_settings_key.as_deref()?;
    let path = storage_path.join("settings.json");
    let text = std::fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    v.get("extension_settings")?
        .get(settings_key)?
        .get(key)?
        .as_bool()
}

pub fn dispatch_event(
    registry: &ExtensionRegistry,
    event_type: &str,
    payload: serde_json::Value,
    db_path: String,
    storage_path: String,
    feature_id: Option<String>,
) {
    let params = match payload.as_object() {
        Some(obj) => obj.clone(),
        None => {
            eprintln!("[extensions] dispatch_event called with non-object payload for '{}', params will be empty", event_type);
            Default::default()
        }
    };
    let handlers = registry.handlers_for_event(event_type, &payload);
    for (script_path, extension_id) in handlers {
        let input = script_runner::ScriptInput {
            params: params.clone(),
            db_path: db_path.clone(),
            storage_path: storage_path.clone(),
            feature_id: feature_id.clone(),
        };
        script_runner::run_event_hook(script_path, input, extension_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_manifest(dir: &TempDir, ext_id: &str, json: &str) {
        let ext_dir = dir.path().join(ext_id);
        fs::create_dir_all(&ext_dir).unwrap();
        fs::write(ext_dir.join("extension.json"), json).unwrap();
    }

    fn valid_json(id: &str) -> String {
        format!(
            r#"{{"id": "{}", "name": "Ext {}", "version": "1.0.0"}}"#,
            id, id
        )
    }

    #[test]
    fn loads_valid_extension() {
        let tmp = TempDir::new().unwrap();
        write_manifest(&tmp, "my-ext", &valid_json("my-ext"));
        let registry = ExtensionRegistry::load_from_dir(tmp.path());
        assert_eq!(registry.extensions.len(), 1);
        assert_eq!(registry.extensions[0].manifest.id, "my-ext");
        assert!(registry.extensions[0].enabled);
    }

    #[test]
    fn skips_missing_required_fields() {
        let tmp = TempDir::new().unwrap();
        write_manifest(&tmp, "bad-ext", r#"{"name": "x", "version": "1.0.0"}"#); // missing id
        let registry = ExtensionRegistry::load_from_dir(tmp.path());
        assert_eq!(registry.extensions.len(), 0);
    }

    #[test]
    fn skips_duplicate_ids() {
        let tmp = TempDir::new().unwrap();
        write_manifest(&tmp, "ext-a", &valid_json("same-id"));
        write_manifest(&tmp, "ext-b", &valid_json("same-id"));
        let registry = ExtensionRegistry::load_from_dir(tmp.path());
        assert_eq!(registry.extensions.len(), 1);
    }

    #[test]
    fn skips_unprefixed_table() {
        let tmp = TempDir::new().unwrap();
        write_manifest(
            &tmp,
            "bad-table",
            r#"{"id": "x", "name": "x", "version": "1.0.0", "tables": [{"name": "no_prefix", "columns": [], "indexes": []}]}"#,
        );
        let registry = ExtensionRegistry::load_from_dir(tmp.path());
        assert_eq!(registry.extensions.len(), 0);
    }

    #[test]
    fn returns_empty_for_nonexistent_dir() {
        let registry = ExtensionRegistry::load_from_dir(std::path::Path::new("/nonexistent/path"));
        assert_eq!(registry.extensions.len(), 0);
    }

    #[test]
    fn find_tool_returns_matching_extension_and_decl() {
        let tmp = TempDir::new().unwrap();
        write_manifest(
            &tmp,
            "my-ext",
            r#"{
            "id": "my-ext", "name": "My Ext", "version": "1.0.0",
            "tools": [{"name": "my_tool", "description": "test", "handler": "h.js", "params": {}}]
        }"#,
        );
        let registry = ExtensionRegistry::load_from_dir(tmp.path());
        let result = registry.find_tool("my_tool");
        assert!(result.is_some());
        let (ext, tool) = result.unwrap();
        assert_eq!(ext.manifest.id, "my-ext");
        assert_eq!(tool.name, "my_tool");
    }

    #[test]
    fn skips_extension_with_duplicate_table_name() {
        let tmp = TempDir::new().unwrap();
        let table_json = r#"{"id": "ext-a", "name": "A", "version": "1.0.0", "tables": [{"name": "ext_shared", "columns": [], "indexes": []}]}"#;
        let table_json2 = r#"{"id": "ext-b", "name": "B", "version": "1.0.0", "tables": [{"name": "ext_shared", "columns": [], "indexes": []}]}"#;
        write_manifest(&tmp, "ext-a", table_json);
        write_manifest(&tmp, "ext-b", table_json2);
        let registry = ExtensionRegistry::load_from_dir(tmp.path());
        // One is accepted, the other (whichever loads second) is rejected
        assert_eq!(registry.extensions.len(), 1);
    }

    #[test]
    fn event_filter_matches_correctly() {
        let payload = serde_json::json!({"link_type": "github-pr", "feature_id": "abc"});
        let mut filter = std::collections::HashMap::new();
        filter.insert("link_type".to_string(), "github-pr".to_string());
        assert!(event_filter_matches(&filter, &payload));

        let mut bad_filter = std::collections::HashMap::new();
        bad_filter.insert("link_type".to_string(), "jira".to_string());
        assert!(!event_filter_matches(&bad_filter, &payload));
    }
}
