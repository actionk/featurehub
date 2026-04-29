use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeatureSkill {
    pub skill_id: String,
    pub enabled: bool,
}

/// Get all per-feature skill overrides for a given feature.
pub fn get_feature_skills(
    conn: &Connection,
    feature_id: &str,
) -> Result<Vec<FeatureSkill>, String> {
    let mut stmt = conn
        .prepare("SELECT skill_id, enabled FROM feature_skills WHERE feature_id = ?1")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([feature_id], |row| {
            Ok(FeatureSkill {
                skill_id: row.get(0)?,
                enabled: row.get::<_, i32>(1)? != 0,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

/// Set whether a specific skill is enabled for a feature (upsert).
pub fn set_feature_skill(
    conn: &Connection,
    feature_id: &str,
    skill_id: &str,
    enabled: bool,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO feature_skills (feature_id, skill_id, enabled)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(feature_id, skill_id) DO UPDATE SET enabled = ?3",
        rusqlite::params![feature_id, skill_id, enabled as i32],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Resolve which skills are active for a given feature.
/// Returns filtered list based on default_enabled + per-feature overrides.
pub fn resolve_skills_for_feature(
    conn: &Connection,
    feature_id: &str,
    all_skills: &[crate::config::Skill],
) -> Result<Vec<crate::config::Skill>, String> {
    let overrides = get_feature_skills(conn, feature_id)?;
    let override_map: std::collections::HashMap<&str, bool> = overrides
        .iter()
        .map(|o| (o.skill_id.as_str(), o.enabled))
        .collect();

    Ok(all_skills
        .iter()
        .filter(|s| match override_map.get(s.id.as_str()) {
            Some(&enabled) => enabled,
            None => s.default_enabled,
        })
        .cloned()
        .collect())
}
