use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionManifest {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub tables: Vec<TableDecl>,
    #[serde(default)]
    pub tools: Vec<ToolDecl>,
    #[serde(default)]
    pub events: Vec<EventDecl>,
    #[serde(default)]
    pub tabs: Vec<TabDecl>,
    #[serde(default)]
    pub instructions: String,
    #[serde(default)]
    pub schedules: Vec<ScheduleDecl>,
    #[serde(default)]
    pub storage_settings_key: Option<String>,
}

impl ExtensionManifest {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("missing 'id'".into());
        }
        if self.name.is_empty() {
            return Err("missing 'name'".into());
        }
        if self.version.is_empty() {
            return Err("missing 'version'".into());
        }
        for table in &self.tables {
            if !table.name.starts_with("ext_") {
                return Err(format!(
                    "table '{}' must be prefixed with 'ext_'",
                    table.name
                ));
            }
        }
        for s in &self.schedules {
            if s.id.is_empty() {
                return Err("schedule missing 'id'".into());
            }
            if s.handler.is_empty() {
                return Err(format!("schedule '{}' missing 'handler'", s.id));
            }
            if s.interval_secs < 60 {
                return Err(format!("schedule '{}': interval_secs must be >= 60", s.id));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleDecl {
    pub id: String,
    pub handler: String,
    pub interval_secs: u64,
    #[serde(default)]
    pub enabled_setting: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDecl {
    pub name: String,
    #[serde(default)]
    pub columns: Vec<ColumnDecl>,
    #[serde(default)]
    pub indexes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDecl {
    pub name: String,
    #[serde(rename = "type")]
    pub col_type: String,
    #[serde(default)]
    pub fk: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDecl {
    pub name: String,
    pub description: String,
    pub handler: String,
    #[serde(default)]
    pub params: HashMap<String, ParamDecl>,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDecl {
    #[serde(rename = "type")]
    pub param_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDecl {
    pub on: String,
    #[serde(default)]
    pub filter: HashMap<String, String>,
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabDecl {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub emoji: String,
    #[serde(rename = "sortOrder", default)]
    pub sort_order: u32,
    pub component: String,
    #[serde(default)]
    pub badge_query: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_manifest_json() -> &'static str {
        r#"{
            "id": "my-ext",
            "name": "My Extension",
            "version": "1.0.0",
            "tables": [
                {
                    "name": "ext_my_table",
                    "columns": [
                        {"name": "id", "type": "TEXT PRIMARY KEY"},
                        {"name": "feature_id", "type": "TEXT NOT NULL", "fk": "features(id) ON DELETE CASCADE"}
                    ],
                    "indexes": ["feature_id"]
                }
            ],
            "tools": [
                {"name": "do_thing", "description": "Does a thing", "handler": "handlers/do_thing.js", "params": {
                    "feature_id": {"type": "string", "required": true}
                }}
            ],
            "events": [
                {"on": "link_created", "filter": {"link_type": "github-pr"}, "handler": "handlers/on_link.js"}
            ],
            "tabs": [
                {"id": "prs", "label": "PRs", "emoji": "🔀", "sortOrder": 350, "component": "ui/tab.html",
                 "badge_query": "SELECT COUNT(*) FROM ext_my_table WHERE feature_id = ?"}
            ]
        }"#
    }

    #[test]
    fn parses_valid_manifest() {
        let m: ExtensionManifest = serde_json::from_str(valid_manifest_json()).unwrap();
        assert_eq!(m.id, "my-ext");
        assert_eq!(m.name, "My Extension");
        assert_eq!(m.version, "1.0.0");
        assert_eq!(m.tables.len(), 1);
        assert_eq!(m.tables[0].name, "ext_my_table");
        assert_eq!(m.tables[0].columns.len(), 2);
        assert_eq!(
            m.tables[0].columns[1].fk.as_deref(),
            Some("features(id) ON DELETE CASCADE")
        );
        assert_eq!(m.tools.len(), 1);
        assert_eq!(m.tools[0].name, "do_thing");
        assert!(m.tools[0].params["feature_id"].required);
        assert_eq!(m.events.len(), 1);
        assert_eq!(m.events[0].filter["link_type"], "github-pr");
        assert_eq!(m.tabs.len(), 1);
        assert_eq!(m.tabs[0].sort_order, 350);
    }

    #[test]
    fn validates_missing_id() {
        let json = r#"{"name": "x", "version": "1.0.0"}"#;
        let m: ExtensionManifest = serde_json::from_str(json).unwrap();
        assert!(m.validate().is_err());
    }

    #[test]
    fn validates_missing_name() {
        let json = r#"{"id": "x", "version": "1.0.0"}"#;
        let m: ExtensionManifest = serde_json::from_str(json).unwrap();
        assert!(m.validate().is_err());
    }

    #[test]
    fn validates_missing_version() {
        let json = r#"{"id": "x", "name": "x"}"#;
        let m: ExtensionManifest = serde_json::from_str(json).unwrap();
        assert!(m.validate().is_err());
    }

    #[test]
    fn validates_unprefixed_table() {
        let json = r#"{"id": "x", "name": "x", "version": "1.0.0", "tables": [{"name": "my_table", "columns": [], "indexes": []}]}"#;
        let m: ExtensionManifest = serde_json::from_str(json).unwrap();
        assert!(m.validate().is_err());
    }

    #[test]
    fn ignores_unknown_fields() {
        let json = r#"{"id": "x", "name": "x", "version": "1.0.0", "unknown_future_field": true}"#;
        let result: Result<ExtensionManifest, _> = serde_json::from_str(json);
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod schedule_tests {
    use super::*;

    #[test]
    fn parses_schedules_from_manifest() {
        let json = r#"{
            "id": "x", "name": "X", "version": "1.0.0",
            "schedules": [
                { "id": "poll", "handler": "p.js", "interval_secs": 300, "enabled_setting": "poll_enabled" }
            ]
        }"#;
        let m: ExtensionManifest = serde_json::from_str(json).unwrap();
        assert_eq!(m.schedules.len(), 1);
        let s = &m.schedules[0];
        assert_eq!(s.id, "poll");
        assert_eq!(s.handler, "p.js");
        assert_eq!(s.interval_secs, 300);
        assert_eq!(s.enabled_setting.as_deref(), Some("poll_enabled"));
    }

    #[test]
    fn schedule_without_enabled_setting_is_always_enabled() {
        let json = r#"{"id":"x","name":"X","version":"1.0.0","schedules":[{"id":"t","handler":"t.js","interval_secs":60}]}"#;
        let m: ExtensionManifest = serde_json::from_str(json).unwrap();
        assert!(m.schedules[0].enabled_setting.is_none());
    }

    #[test]
    fn validate_rejects_interval_below_60() {
        let m = ExtensionManifest {
            id: "x".into(),
            name: "X".into(),
            version: "1.0.0".into(),
            description: "".into(),
            author: "".into(),
            requires: vec![],
            tables: vec![],
            tools: vec![],
            events: vec![],
            tabs: vec![],
            instructions: "".into(),
            schedules: vec![ScheduleDecl {
                id: "s".into(),
                handler: "h.js".into(),
                interval_secs: 30,
                enabled_setting: None,
            }],
            storage_settings_key: None,
        };
        let err = m.validate().unwrap_err();
        assert!(err.contains("interval_secs"));
    }

    #[test]
    fn validate_rejects_empty_schedule_id() {
        let m = ExtensionManifest {
            id: "x".into(),
            name: "X".into(),
            version: "1.0.0".into(),
            description: "".into(),
            author: "".into(),
            requires: vec![],
            tables: vec![],
            tools: vec![],
            events: vec![],
            tabs: vec![],
            instructions: "".into(),
            schedules: vec![ScheduleDecl {
                id: "".into(),
                handler: "h.js".into(),
                interval_secs: 60,
                enabled_setting: None,
            }],
            storage_settings_key: None,
        };
        let err = m.validate().unwrap_err();
        assert!(err.contains("'id'") || err.contains("id"), "got: {}", err);
    }

    #[test]
    fn validate_rejects_empty_schedule_handler() {
        let m = ExtensionManifest {
            id: "x".into(),
            name: "X".into(),
            version: "1.0.0".into(),
            description: "".into(),
            author: "".into(),
            requires: vec![],
            tables: vec![],
            tools: vec![],
            events: vec![],
            tabs: vec![],
            instructions: "".into(),
            schedules: vec![ScheduleDecl {
                id: "s".into(),
                handler: "".into(),
                interval_secs: 60,
                enabled_setting: None,
            }],
            storage_settings_key: None,
        };
        let err = m.validate().unwrap_err();
        assert!(err.contains("handler"), "got: {}", err);
    }

    #[test]
    fn parses_storage_settings_key() {
        let with_key = r#"{"id":"x","name":"X","version":"1.0.0","storage_settings_key":"my_key"}"#;
        let m1: ExtensionManifest = serde_json::from_str(with_key).unwrap();
        assert_eq!(m1.storage_settings_key.as_deref(), Some("my_key"));

        let without_key = r#"{"id":"x","name":"X","version":"1.0.0"}"#;
        let m2: ExtensionManifest = serde_json::from_str(without_key).unwrap();
        assert!(m2.storage_settings_key.is_none());
    }
}
