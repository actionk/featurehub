use rusqlite::Connection;
use crate::extensions::manifest::TableDecl;

/// Validate that a value is a safe SQL identifier: letters, digits, underscores only,
/// must start with a letter or underscore.
fn validate_identifier(value: &str, context: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{} identifier cannot be empty", context));
    }
    let first = value.chars().next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' {
        return Err(format!(
            "{} identifier '{}' must start with a letter or underscore",
            context, value
        ));
    }
    if !value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(format!(
            "{} identifier '{}' contains invalid characters (only letters, digits, underscores allowed)",
            context, value
        ));
    }
    Ok(())
}

/// Validate a SQL fragment (col_type, fk clause) is free of statement-terminating
/// or comment sequences that could enable injection.
fn validate_sql_fragment(value: &str, field: &str) -> Result<(), String> {
    for bad in &[";", "--", "/*", "*/"] {
        if value.contains(bad) {
            return Err(format!(
                "{} '{}' contains disallowed sequence '{}'",
                field, value, bad
            ));
        }
    }
    Ok(())
}

pub fn provision_tables(conn: &Connection, tables: &[TableDecl]) -> Result<(), String> {
    for table in tables {
        provision_table(conn, table)?;
    }
    Ok(())
}

pub fn provision_table(conn: &Connection, table: &TableDecl) -> Result<(), String> {
    if !table.name.starts_with("ext_") {
        return Err(format!(
            "table '{}' must be prefixed with 'ext_' to avoid collisions",
            table.name
        ));
    }
    // Validate the full table name (the ext_ prefix already passed; validate the whole thing)
    validate_identifier(&table.name, "table")?;

    // Validate all column names, types, and fk clauses before building any SQL
    for col in &table.columns {
        validate_identifier(&col.name, "column")?;
        validate_sql_fragment(&col.col_type, "col_type")?;
        if let Some(ref fk) = col.fk {
            validate_sql_fragment(fk, "fk")?;
        }
    }
    for idx_col in &table.indexes {
        validate_identifier(idx_col, "index column")?;
    }

    let mut col_defs: Vec<String> = table
        .columns
        .iter()
        .map(|c| format!("{} {}", c.name, c.col_type))
        .collect();

    for col in &table.columns {
        if let Some(ref fk) = col.fk {
            col_defs.push(format!("FOREIGN KEY ({}) REFERENCES {}", col.name, fk));
        }
    }

    let create_sql = format!(
        "CREATE TABLE IF NOT EXISTS {} ({})",
        table.name,
        col_defs.join(", ")
    );
    conn.execute_batch(&create_sql).map_err(|e| e.to_string())?;

    // Add any new columns not yet present (handles extension manifest version upgrades).
    // Uses the same probe-then-alter pattern as db/mod.rs migrations.
    for col in &table.columns {
        let probe = format!("SELECT {} FROM {} LIMIT 0", col.name, table.name);
        if conn.execute_batch(&probe).is_err() {
            let alter_sql = format!(
                "ALTER TABLE {} ADD COLUMN {} {}",
                table.name, col.name, col.col_type
            );
            conn.execute_batch(&alter_sql).map_err(|e| e.to_string())?;
        }
    }

    for col_name in &table.indexes {
        let idx_sql = format!(
            "CREATE INDEX IF NOT EXISTS idx_{}_{} ON {} ({})",
            table.name, col_name, table.name, col_name
        );
        conn.execute_batch(&idx_sql).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_utils::test_db;
    use crate::extensions::manifest::{TableDecl, ColumnDecl};

    fn make_table(name: &str) -> TableDecl {
        TableDecl {
            name: name.to_string(),
            columns: vec![
                ColumnDecl { name: "id".into(), col_type: "TEXT PRIMARY KEY".into(), fk: None },
                ColumnDecl { name: "feature_id".into(), col_type: "TEXT NOT NULL".into(), fk: Some("features(id) ON DELETE CASCADE".into()) },
                ColumnDecl { name: "title".into(), col_type: "TEXT NOT NULL".into(), fk: None },
            ],
            indexes: vec!["feature_id".into()],
        }
    }

    #[test]
    fn creates_table_and_index() {
        let conn = test_db();
        let table = make_table("ext_test_prs");
        provision_table(&conn, &table).unwrap();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='ext_test_prs'",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(count, 1);

        let idx_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='idx_ext_test_prs_feature_id'",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(idx_count, 1);
    }

    #[test]
    fn is_idempotent() {
        let conn = test_db();
        let table = make_table("ext_idempotent");
        provision_table(&conn, &table).unwrap();
        provision_table(&conn, &table).unwrap(); // Second call should not fail
    }

    #[test]
    fn rejects_unprefixed_table_name() {
        let conn = test_db();
        let table = make_table("no_prefix_table");
        let result = provision_table(&conn, &table);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ext_"));
    }

    #[test]
    fn adds_new_column_to_existing_table() {
        let conn = test_db();
        let table_v1 = TableDecl {
            name: "ext_migration_test".into(),
            columns: vec![
                ColumnDecl { name: "id".into(), col_type: "TEXT PRIMARY KEY".into(), fk: None },
                ColumnDecl { name: "title".into(), col_type: "TEXT NOT NULL".into(), fk: None },
            ],
            indexes: vec![],
        };
        provision_table(&conn, &table_v1).unwrap();

        let table_v2 = TableDecl {
            name: "ext_migration_test".into(),
            columns: vec![
                ColumnDecl { name: "id".into(), col_type: "TEXT PRIMARY KEY".into(), fk: None },
                ColumnDecl { name: "title".into(), col_type: "TEXT NOT NULL".into(), fk: None },
                ColumnDecl { name: "status".into(), col_type: "TEXT NOT NULL DEFAULT 'open'".into(), fk: None },
            ],
            indexes: vec![],
        };
        provision_table(&conn, &table_v2).unwrap();

        conn.execute_batch("SELECT status FROM ext_migration_test LIMIT 0").unwrap();
    }

    #[test]
    fn rejects_identifier_with_injection_chars() {
        let conn = test_db();
        let table = TableDecl {
            name: "ext_safe".into(),
            columns: vec![
                ColumnDecl {
                    name: "id) ; DROP TABLE features; --".into(),
                    col_type: "TEXT".into(),
                    fk: None,
                },
            ],
            indexes: vec![],
        };
        let result = provision_table(&conn, &table);
        assert!(result.is_err(), "should reject column name with injection chars");
    }

    #[test]
    fn rejects_col_type_with_semicolon() {
        let conn = test_db();
        let table = TableDecl {
            name: "ext_safe2".into(),
            columns: vec![
                ColumnDecl {
                    name: "id".into(),
                    col_type: "TEXT); DROP TABLE features; --".into(),
                    fk: None,
                },
            ],
            indexes: vec![],
        };
        let result = provision_table(&conn, &table);
        assert!(result.is_err(), "should reject col_type with semicolon");
    }
}
