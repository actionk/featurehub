use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeatureMcpServer {
    pub server_name: String,
    pub enabled: bool,
}

/// Get all per-feature MCP server overrides for a given feature.
pub fn get_feature_mcp_servers(
    conn: &Connection,
    feature_id: &str,
) -> Result<Vec<FeatureMcpServer>, String> {
    let mut stmt = conn
        .prepare("SELECT server_name, enabled FROM feature_mcp_servers WHERE feature_id = ?1")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([feature_id], |row| {
            Ok(FeatureMcpServer {
                server_name: row.get(0)?,
                enabled: row.get::<_, i32>(1)? != 0,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

/// Set whether a specific MCP server is enabled for a feature (upsert).
pub fn set_feature_mcp_server(
    conn: &Connection,
    feature_id: &str,
    server_name: &str,
    enabled: bool,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO feature_mcp_servers (feature_id, server_name, enabled)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(feature_id, server_name) DO UPDATE SET enabled = ?3",
        rusqlite::params![feature_id, server_name, enabled as i32],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Resolve which MCP servers should be active for a given feature.
/// Returns filtered list based on default_enabled + per-feature overrides.
pub fn resolve_servers_for_feature(
    conn: &Connection,
    feature_id: &str,
    all_servers: &[crate::config::McpServer],
) -> Result<Vec<crate::config::McpServer>, String> {
    let overrides = get_feature_mcp_servers(conn, feature_id)?;
    let override_map: std::collections::HashMap<&str, bool> = overrides
        .iter()
        .map(|o| (o.server_name.as_str(), o.enabled))
        .collect();

    Ok(all_servers
        .iter()
        .filter(|s| match override_map.get(s.name.as_str()) {
            Some(&enabled) => enabled,
            None => s.default_enabled,
        })
        .cloned()
        .collect())
}
