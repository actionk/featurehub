use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use zip::write::SimpleFileOptions;

use crate::db;
use crate::git;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExportOptions {
    pub include_done: bool,
    pub include_archived: bool,
    pub include_files: bool,
    pub include_sessions: bool,
    pub include_tasks: bool,
    pub include_notes: bool,
    pub include_context: bool,
    pub include_patches: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportManifest {
    pub version: String,
    pub app_version: String,
    pub exported_at: String,
    pub os: String,
    pub feature_count: usize,
    pub has_patches: bool,
}

#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub feature_count: usize,
    pub file_count: usize,
    pub storage_path: String,
    pub directories_with_repos: Vec<RepoDirectory>,
}

#[derive(Debug, Serialize)]
pub struct ImportCheckResult {
    pub zip_path: String,
    pub total_features: usize,
    pub duplicate_count: usize,
    pub duplicate_titles: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RepoDirectory {
    pub directory_id: String,
    pub feature_id: String,
    pub feature_title: String,
    pub repo_url: String,
    pub label: Option<String>,
    pub has_patch: bool,
}

/// Export the current storage to a ZIP archive.
pub fn export_storage(
    storage_path: &Path,
    output_path: &Path,
    opts: &ExportOptions,
    cancelled: &Arc<AtomicBool>,
    progress_fn: &dyn Fn(&str, u32),
) -> Result<PathBuf, String> {
    let db_path = storage_path.join("feature-hub.db");
    if !db_path.exists() {
        return Err("No database found in storage".to_string());
    }

    let file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create archive: {}", e))?;
    let mut zip = zip::ZipWriter::new(file);
    let zip_opts = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // Helper to check cancellation
    let check_cancel = || -> Result<(), String> {
        if cancelled.load(Ordering::Relaxed) {
            Err("Export cancelled".to_string())
        } else {
            Ok(())
        }
    };

    // Stage 1: Backup database to temp file
    progress_fn("Reading storage...", 5);
    check_cancel()?;
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let temp_db = std::env::temp_dir().join(format!("fh-export-{}.db", uuid::Uuid::new_v4()));
    {
        let mut dest = rusqlite::Connection::open(&temp_db)
            .map_err(|e| format!("Failed to create temp DB: {}", e))?;
        let backup = rusqlite::backup::Backup::new(&conn, &mut dest)
            .map_err(|e| format!("Failed to start backup: {}", e))?;
        backup
            .run_to_completion(100, std::time::Duration::from_millis(10), None)
            .map_err(|e| format!("Database backup failed: {}", e))?;
    }

    // Stage 2: Strip excluded data from the temp DB copy
    progress_fn("Filtering data...", 10);
    check_cancel()?;
    let excluded_feature_ids = {
        let temp_conn = rusqlite::Connection::open(&temp_db)
            .map_err(|e| format!("Failed to open temp DB: {}", e))?;
        strip_excluded_data(&temp_conn, opts)?
    };

    // Count remaining features
    let feature_count: usize = {
        let temp_conn = rusqlite::Connection::open(&temp_db)
            .map_err(|e| format!("Failed to open temp DB: {}", e))?;
        temp_conn
            .query_row("SELECT COUNT(*) FROM features", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count features: {}", e))?
    };
    progress_fn(
        &format!("Backing up database ({} features)...", feature_count),
        18,
    );

    // Add filtered DB to zip
    let db_bytes = fs::read(&temp_db)
        .map_err(|e| format!("Failed to read temp DB: {}", e))?;
    zip.start_file("feature-hub.db", zip_opts)
        .map_err(|e| format!("Failed to add DB to archive: {}", e))?;
    zip.write_all(&db_bytes)
        .map_err(|e| format!("Failed to write DB to archive: {}", e))?;
    let _ = fs::remove_file(&temp_db);

    // Stage 3: Add only DB-tracked files (not entire workspace dirs which contain repos)
    if opts.include_files {
        check_cancel()?;
        let tracked_files = collect_tracked_files(&conn, &excluded_feature_ids)?;
        let total = tracked_files.len();

        if total > 0 {
            progress_fn(
                &format!("Archiving {} tracked files...", total),
                25,
            );

            for (i, (stored_path, filename)) in tracked_files.iter().enumerate() {
                check_cancel()?;
                // Resolve relative DB path to absolute for disk access
                let resolved = crate::paths::resolve_path(stored_path, storage_path);
                if !resolved.exists() {
                    continue;
                }

                // Compute relative path for ZIP archive
                let rel_path = if let Ok(rel) = resolved.strip_prefix(storage_path) {
                    rel.to_string_lossy().to_string().replace('\\', "/")
                } else if !Path::new(stored_path).is_absolute() {
                    // Already relative
                    stored_path.replace('\\', "/")
                } else {
                    // Absolute path outside storage — store under workspaces/orphan/
                    format!("workspaces/orphan/{}", filename)
                };

                let data = fs::read(&resolved)
                    .map_err(|e| format!("Failed to read {}: {}", stored_path, e))?;
                zip.start_file(&rel_path, zip_opts)
                    .map_err(|e| format!("Failed to add {}: {}", rel_path, e))?;
                zip.write_all(&data)
                    .map_err(|e| format!("Failed to write {}: {}", rel_path, e))?;

                let pct = 25 + ((i as u32 * 50) / total.max(1) as u32).min(50);
                progress_fn(
                    &format!("Archiving file {}/{} — {}", i + 1, total, filename),
                    pct,
                );
            }
            progress_fn(&format!("Archived {} files", total), 78);
        }
    }

    // Stage 4: Optional git patches
    let mut has_patches = false;
    if opts.include_patches {
        check_cancel()?;
        progress_fn("Scanning repositories for uncommitted changes...", 80);
        has_patches =
            capture_patches(&conn, &excluded_feature_ids, storage_path, &mut zip, zip_opts, cancelled, progress_fn)?;
    }

    // Stage 5: Write manifest
    check_cancel()?;
    progress_fn("Finalizing archive...", 92);
    let manifest = ExportManifest {
        version: "1".to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        os: std::env::consts::OS.to_string(),
        feature_count,
        has_patches,
    };
    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
    zip.start_file("manifest.json", zip_opts)
        .map_err(|e| format!("Failed to add manifest: {}", e))?;
    zip.write_all(manifest_json.as_bytes())
        .map_err(|e| format!("Failed to write manifest: {}", e))?;

    zip.finish()
        .map_err(|e| format!("Failed to finalize archive: {}", e))?;

    progress_fn("Export complete!", 100);
    Ok(output_path.to_path_buf())
}

/// Import a storage from a ZIP archive.
pub fn import_storage(
    zip_path: &Path,
    target_dir: &Path,
    progress_fn: &dyn Fn(&str, u32),
) -> Result<ImportResult, String> {
    let file = fs::File::open(zip_path)
        .map_err(|e| format!("Failed to open archive: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Invalid ZIP archive: {}", e))?;

    // Stage 1: Validate manifest
    progress_fn("Reading manifest...", 5);
    let manifest: ExportManifest = {
        let mut entry = archive
            .by_name("manifest.json")
            .map_err(|_| {
                "Archive missing manifest.json — not a valid Feature Hub export".to_string()
            })?;
        let mut buf = String::new();
        entry
            .read_to_string(&mut buf)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;
        serde_json::from_str(&buf).map_err(|e| format!("Invalid manifest: {}", e))?
    };

    if manifest.version != "1" {
        return Err(format!("Unsupported export version: {}", manifest.version));
    }

    // Stage 2: Create target directory
    progress_fn("Creating storage directory...", 10);
    fs::create_dir_all(target_dir)
        .map_err(|e| format!("Failed to create target directory: {}", e))?;
    fs::create_dir_all(target_dir.join("workspaces"))
        .map_err(|e| format!("Failed to create workspaces directory: {}", e))?;

    // Stage 3: Extract all files
    progress_fn("Extracting files...", 20);
    let total = archive.len();
    let mut file_count = 0u64;
    for i in 0..total {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read archive entry: {}", e))?;

        let name = entry.name().to_string();

        if name == "manifest.json" || entry.is_dir() {
            continue;
        }

        // Skip patches dir — only used during repo restore
        if name.starts_with("patches/") {
            continue;
        }

        let out_path = target_dir.join(&name);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory for {}: {}", name, e))?;
        }

        let mut out_file = fs::File::create(&out_path)
            .map_err(|e| format!("Failed to create file {}: {}", name, e))?;
        std::io::copy(&mut entry, &mut out_file)
            .map_err(|e| format!("Failed to extract {}: {}", name, e))?;
        file_count += 1;

        if i % 50 == 0 {
            let pct = 20 + ((i as u32 * 60) / total as u32);
            progress_fn(&format!("Extracting files... ({}/{})", i, total), pct);
        }
    }

    // Stage 4: Initialize DB for migration compatibility
    progress_fn("Initializing database...", 85);
    let db_path = target_dir.join("feature-hub.db");
    if db_path.exists() {
        let conn = rusqlite::Connection::open(&db_path)
            .map_err(|e| format!("Failed to open imported database: {}", e))?;
        db::initialize(&conn)
            .map_err(|e| format!("Failed to initialize imported database: {}", e))?;

        // Rewrite stored_path values to point to the new storage location
        rewrite_file_paths(&conn, target_dir)?;
    }

    // Stage 5: Collect repo directories
    progress_fn("Analyzing repositories...", 92);
    let directories_with_repos = if db_path.exists() {
        collect_repo_directories(&db_path, &archive_has_patches(zip_path)?)?
    } else {
        vec![]
    };

    progress_fn("Import complete!", 100);
    Ok(ImportResult {
        feature_count: manifest.feature_count,
        file_count: file_count as usize,
        storage_path: target_dir.to_string_lossy().to_string(),
        directories_with_repos,
    })
}

/// Restore a single repository from the export archive.
pub fn restore_repo_from_export(
    zip_path: &Path,
    directory_id: &str,
    target_path: &Path,
    db_path: &Path,
) -> Result<(), String> {
    let conn = rusqlite::Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    let dir = db::directories::get_directory(&conn, directory_id)?;

    let repo_url = dir.repo_url.ok_or("Directory has no repo_url")?;

    git::clone_repo(&repo_url, target_path)?;

    if let Ok(branch) = conn.query_row(
        "SELECT branch_name FROM feature_branches WHERE directory_id = ?1 ORDER BY created_at DESC LIMIT 1",
        rusqlite::params![directory_id],
        |row| row.get::<_, String>(0),
    ) {
        let _ = git::checkout_branch(target_path, &branch);
    }

    let file = fs::File::open(zip_path)
        .map_err(|e| format!("Failed to open archive: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Failed to open archive: {}", e))?;

    let patch_name = format!("patches/{}.diff", directory_id);
    if let Ok(mut entry) = archive.by_name(&patch_name) {
        let mut patch = String::new();
        if entry.read_to_string(&mut patch).is_ok() && !patch.trim().is_empty() {
            let temp_patch = std::env::temp_dir().join(format!("fh-patch-{}.diff", directory_id));
            if fs::write(&temp_patch, &patch).is_ok() {
                let _ = git::run_git(
                    target_path,
                    &["apply", "--allow-empty", &temp_patch.to_string_lossy()],
                );
                let _ = fs::remove_file(&temp_patch);
            }
        }
    }

    // Store relative path: extract storage base from db_path's parent
    let rel_path = if let Some(storage_base) = db_path.parent() {
        crate::paths::to_storage_relative(&target_path.to_string_lossy(), storage_base)
    } else {
        target_path.to_string_lossy().to_string()
    };
    conn.execute(
        "UPDATE directories SET path = ?1, clone_status = 'ready', clone_error = NULL WHERE id = ?2",
        rusqlite::params![rel_path, directory_id],
    )
    .map_err(|e| format!("Failed to update directory: {}", e))?;

    Ok(())
}

/// Check a ZIP archive for duplicates against the current DB without importing.
pub fn check_import_zip(
    zip_path: &Path,
    current_db: &Path,
) -> Result<ImportCheckResult, String> {
    let temp_db = extract_db_to_temp(zip_path)?;

    let result = (|| -> Result<ImportCheckResult, String> {
        let import_conn = rusqlite::Connection::open(&temp_db)
            .map_err(|e| format!("Failed to open imported DB: {}", e))?;
        db::initialize(&import_conn)
            .map_err(|e| format!("Failed to initialize imported DB: {}", e))?;

        let current_conn = rusqlite::Connection::open(current_db)
            .map_err(|e| format!("Failed to open current DB: {}", e))?;

        let imported: Vec<(String, String)> = {
            let mut stmt = import_conn
                .prepare("SELECT id, title FROM features")
                .map_err(|e| e.to_string())?;
            let v: Vec<(String, String)> = stmt
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok())
                .collect();
            v
        };

        let total = imported.len();
        let mut dup_titles = Vec::new();
        for (id, title) in &imported {
            let n: i64 = current_conn
                .query_row(
                    "SELECT COUNT(*) FROM features WHERE id = ?1",
                    rusqlite::params![id],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            if n > 0 {
                dup_titles.push(title.clone());
            }
        }

        Ok(ImportCheckResult {
            zip_path: zip_path.to_string_lossy().to_string(),
            total_features: total,
            duplicate_count: dup_titles.len(),
            duplicate_titles: dup_titles,
        })
    })();

    let _ = fs::remove_file(&temp_db);
    result
}

/// Import a ZIP archive into the current storage directory.
/// `strategy` is one of: "replace", "ignore", "merge".
pub fn import_into_current_storage(
    zip_path: &Path,
    current_storage: &Path,
    strategy: &str,
    progress_fn: &dyn Fn(&str, u32),
) -> Result<ImportResult, String> {
    progress_fn("Reading archive...", 5);

    let manifest = read_manifest_from_zip(zip_path)?;
    if manifest.version != "1" {
        return Err(format!("Unsupported export version: {}", manifest.version));
    }

    progress_fn("Extracting database...", 10);
    let temp_db = extract_db_to_temp(zip_path)?;

    let result = (|| -> Result<ImportResult, String> {
        // Run migrations on the imported DB so schemas match.
        {
            let import_conn = rusqlite::Connection::open(&temp_db)
                .map_err(|e| format!("Failed to open imported DB: {}", e))?;
            db::initialize(&import_conn)
                .map_err(|e| format!("Failed to initialize imported DB: {}", e))?;
        }

        let current_db_path = current_storage.join("feature-hub.db");
        let current_conn = rusqlite::Connection::open(&current_db_path)
            .map_err(|e| format!("Failed to open current DB: {}", e))?;

        // Attach the import DB so we can run cross-DB SQL.
        let attach_path = temp_db.to_string_lossy().replace('\'', "''");
        current_conn
            .execute_batch(&format!("ATTACH DATABASE '{}' AS import;", attach_path))
            .map_err(|e| format!("Failed to attach import DB: {}", e))?;

        progress_fn("Importing tags...", 15);
        let tag_id_map = import_tags_with_map(&current_conn)?;

        progress_fn("Importing features...", 25);

        // Disable FK checks for the entire import block:
        // - features.parent_id is self-referential (ordering unknown)
        // - features.group_id references feature_groups which may not exist in current DB
        // - folders.parent_id is self-referential
        // - sub-data for ignored features is silently dropped via INSERT OR IGNORE
        current_conn
            .execute_batch("PRAGMA foreign_keys=OFF;")
            .map_err(|e| e.to_string())?;

        // Import feature groups before features (features.group_id references them).
        // OR IGNORE: keep existing groups if already present, add new ones.
        // Older exports may not have this table — skip gracefully if missing.
        let has_groups_table = current_conn
            .query_row(
                "SELECT COUNT(*) FROM import.sqlite_master WHERE type='table' AND name='feature_groups'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0) > 0;
        if has_groups_table {
            current_conn
                .execute_batch(
                    "INSERT OR IGNORE INTO main.feature_groups SELECT * FROM import.feature_groups;",
                )
                .map_err(|e| format!("Failed to insert feature groups: {}", e))?;
        }

        match strategy {
            "replace" => {
                // Delete duplicates — manually cascade since FK checks are off.
                current_conn
                    .execute_batch(
                        "DELETE FROM main.feature_tags   WHERE feature_id IN (SELECT id FROM import.features); \
                         DELETE FROM main.tasks          WHERE feature_id IN (SELECT id FROM import.features); \
                         DELETE FROM main.links          WHERE feature_id IN (SELECT id FROM import.features); \
                         DELETE FROM main.directories    WHERE feature_id IN (SELECT id FROM import.features); \
                         DELETE FROM main.files          WHERE feature_id IN (SELECT id FROM import.features); \
                         DELETE FROM main.folders        WHERE feature_id IN (SELECT id FROM import.features); \
                         DELETE FROM main.sessions       WHERE feature_id IN (SELECT id FROM import.features); \
                         DELETE FROM main.notes          WHERE feature_id IN (SELECT id FROM import.features); \
                         DELETE FROM main.context        WHERE feature_id IN (SELECT id FROM import.features); \
                         DELETE FROM main.plans          WHERE feature_id IN (SELECT id FROM import.features); \
                         DELETE FROM main.feature_mcp_servers WHERE feature_id IN (SELECT id FROM import.features); \
                         DELETE FROM main.feature_skills      WHERE feature_id IN (SELECT id FROM import.features); \
                         DELETE FROM main.feature_branches    WHERE feature_id IN (SELECT id FROM import.features); \
                         DELETE FROM main.features WHERE id IN (SELECT id FROM import.features);",
                    )
                    .map_err(|e| format!("Failed to delete duplicates: {}", e))?;
                current_conn
                    .execute_batch(
                        "INSERT OR IGNORE INTO main.features SELECT * FROM import.features;",
                    )
                    .map_err(|e| format!("Failed to insert features: {}", e))?;
            }
            "merge" => {
                // Update existing feature metadata (keep local sort_order / pinned / archived / group_id).
                current_conn
                    .execute_batch(
                        "UPDATE main.features SET \
                            title         = (SELECT title         FROM import.features i WHERE i.id = main.features.id), \
                            ticket_id     = (SELECT ticket_id     FROM import.features i WHERE i.id = main.features.id), \
                            status        = (SELECT status        FROM import.features i WHERE i.id = main.features.id), \
                            left_off_text = (SELECT left_off_text FROM import.features i WHERE i.id = main.features.id), \
                            description   = (SELECT description   FROM import.features i WHERE i.id = main.features.id), \
                            updated_at    = (SELECT updated_at    FROM import.features i WHERE i.id = main.features.id) \
                         WHERE id IN (SELECT id FROM import.features);",
                    )
                    .map_err(|e| format!("Failed to update features: {}", e))?;
                // Insert new (non-duplicate) features.
                current_conn
                    .execute_batch(
                        "INSERT OR IGNORE INTO main.features SELECT * FROM import.features;",
                    )
                    .map_err(|e| format!("Failed to insert new features: {}", e))?;
            }
            _ => {
                // "ignore": only insert features that don't already exist.
                current_conn
                    .execute_batch(
                        "INSERT OR IGNORE INTO main.features SELECT * FROM import.features;",
                    )
                    .map_err(|e| format!("Failed to insert features: {}", e))?;
            }
        }

        progress_fn("Importing feature data...", 45);

        // Insert feature_tags with remapped tag IDs (handles cross-storage tag ID divergence).
        import_feature_tags_mapped(&current_conn, &tag_id_map)?;

        // Notes and context: overwrite for replace/merge, add-only for ignore.
        let (notes_sql, context_sql) = if strategy == "ignore" {
            (
                "INSERT OR IGNORE INTO main.notes SELECT * FROM import.notes;",
                "INSERT OR IGNORE INTO main.context SELECT * FROM import.context;",
            )
        } else {
            (
                "INSERT OR REPLACE INTO main.notes SELECT * FROM import.notes;",
                "INSERT OR REPLACE INTO main.context SELECT * FROM import.context;",
            )
        };

        current_conn
            .execute_batch(&format!(
                "INSERT OR IGNORE INTO main.tasks SELECT * FROM import.tasks; \
                 INSERT OR IGNORE INTO main.links SELECT * FROM import.links; \
                 INSERT OR IGNORE INTO main.directories SELECT * FROM import.directories; \
                 INSERT OR IGNORE INTO main.folders SELECT * FROM import.folders; \
                 INSERT OR IGNORE INTO main.files SELECT * FROM import.files; \
                 INSERT OR IGNORE INTO main.sessions SELECT * FROM import.sessions; \
                 INSERT OR IGNORE INTO main.plans SELECT * FROM import.plans; \
                 INSERT OR REPLACE INTO main.feature_mcp_servers SELECT * FROM import.feature_mcp_servers; \
                 INSERT OR REPLACE INTO main.feature_skills SELECT * FROM import.feature_skills; \
                 INSERT OR IGNORE INTO main.feature_branches SELECT * FROM import.feature_branches; \
                 INSERT OR IGNORE INTO main.knowledge_folders SELECT * FROM import.knowledge_folders; \
                 INSERT OR IGNORE INTO main.knowledge_entries SELECT * FROM import.knowledge_entries; \
                 {notes_sql} \
                 {context_sql}",
            ))
            .map_err(|e| format!("Failed to insert feature data: {}", e))?;

        current_conn
            .execute_batch("PRAGMA foreign_keys=ON;")
            .map_err(|e| e.to_string())?;

        progress_fn("Extracting files...", 72);
        let file_count = extract_files_to_storage(zip_path, current_storage)?;

        progress_fn("Updating file paths...", 87);
        rewrite_file_paths(&current_conn, current_storage)?;

        let _ = current_conn.execute_batch("DETACH DATABASE import;");

        progress_fn("Analyzing repositories...", 93);
        let patch_ids = archive_has_patches(zip_path)?;
        let directories_with_repos =
            collect_repo_directories_from_conn(&current_conn, &patch_ids)?;

        progress_fn("Import complete!", 100);
        Ok(ImportResult {
            feature_count: manifest.feature_count,
            file_count,
            storage_path: current_storage.to_string_lossy().to_string(),
            directories_with_repos,
        })
    })();

    let _ = fs::remove_file(&temp_db);
    result
}

// ── Helpers ──────────────────────────────────────────────────────────────

/// Strip excluded data from the temp DB copy. Returns the set of excluded feature IDs.
fn strip_excluded_data(
    conn: &rusqlite::Connection,
    opts: &ExportOptions,
) -> Result<HashSet<String>, String> {
    let mut excluded_ids = HashSet::new();

    if !opts.include_done {
        let mut stmt = conn
            .prepare("SELECT id FROM features WHERE status = 'done'")
            .map_err(|e| e.to_string())?;
        let ids: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        excluded_ids.extend(ids);
    }
    if !opts.include_archived {
        let mut stmt = conn
            .prepare("SELECT id FROM features WHERE archived = 1")
            .map_err(|e| e.to_string())?;
        let ids: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        excluded_ids.extend(ids);
    }

    // Enable FK cascades so ON DELETE CASCADE propagates to child tables.
    conn.execute_batch("PRAGMA foreign_keys=ON;")
        .map_err(|e| format!("Failed to enable FK cascades: {}", e))?;

    for fid in &excluded_ids {
        conn.execute("DELETE FROM features WHERE id = ?1", rusqlite::params![fid])
            .map_err(|e| format!("Failed to exclude feature: {}", e))?;
    }

    if !opts.include_sessions {
        conn.execute_batch("DELETE FROM sessions;")
            .map_err(|e| format!("Failed to strip sessions: {}", e))?;
    }
    if !opts.include_tasks {
        conn.execute_batch("DELETE FROM tasks;")
            .map_err(|e| format!("Failed to strip tasks: {}", e))?;
    }
    if !opts.include_notes {
        conn.execute_batch("DELETE FROM notes;")
            .map_err(|e| format!("Failed to strip notes: {}", e))?;
    }
    if !opts.include_context {
        conn.execute_batch("DELETE FROM context;")
            .map_err(|e| format!("Failed to strip context: {}", e))?;
    }
    if !opts.include_files {
        conn.execute_batch("DELETE FROM files;")
            .map_err(|e| format!("Failed to strip files: {}", e))?;
        conn.execute_batch("DELETE FROM folders;")
            .map_err(|e| format!("Failed to strip folders: {}", e))?;
    }

    conn.execute_batch("DELETE FROM search_index;")
        .map_err(|e| format!("Failed to clear search index: {}", e))?;
    conn.execute_batch("VACUUM;")
        .map_err(|e| format!("Failed to vacuum: {}", e))?;

    Ok(excluded_ids)
}

/// Collect only the files tracked in the DB's `files` table (not entire workspace dirs).
fn collect_tracked_files(
    conn: &rusqlite::Connection,
    excluded_feature_ids: &HashSet<String>,
) -> Result<Vec<(String, String)>, String> {
    let mut stmt = conn
        .prepare("SELECT stored_path, filename, feature_id FROM files")
        .map_err(|e| format!("Failed to query files: {}", e))?;

    let files: Vec<(String, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| format!("Failed to read files: {}", e))?
        .filter_map(|r| r.ok())
        .filter(|(_, _, fid)| !excluded_feature_ids.contains(fid))
        .map(|(path, name, _)| (path, name))
        .collect();

    Ok(files)
}

/// After import, rewrite stored_path values so they point to the new storage location.
fn rewrite_file_paths(conn: &rusqlite::Connection, _new_storage: &Path) -> Result<(), String> {
    // Get all files with their stored paths
    let mut stmt = conn
        .prepare("SELECT id, stored_path FROM files")
        .map_err(|e| e.to_string())?;

    let files: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    for (id, old_path) in &files {
        // Find the relative part (workspaces/feature-id/...)
        // The stored_path may be an absolute path from the old machine
        let old = Path::new(old_path);

        if !old.is_absolute() {
            continue; // Already relative — no rewrite needed
        }

        // Try to find "workspaces" component in the path and store as relative
        let rel = find_workspaces_relative(old);
        if let Some(rel_path) = rel {
            let rel_str = rel_path.to_string_lossy().replace('\\', "/");
            conn.execute(
                "UPDATE files SET stored_path = ?1 WHERE id = ?2",
                rusqlite::params![rel_str, id],
            )
            .map_err(|e| format!("Failed to update file path: {}", e))?;
        }
    }

    Ok(())
}

/// Extract the relative path starting from "workspaces/..." in an absolute stored_path.
fn find_workspaces_relative(path: &Path) -> Option<PathBuf> {
    let path_str = path.to_string_lossy();
    // Find "workspaces" separator in path (works for both / and \)
    let normalized = path_str.replace('\\', "/");
    if let Some(idx) = normalized.find("/workspaces/") {
        Some(PathBuf::from(&normalized[idx + 1..]))
    } else if normalized.starts_with("workspaces/") {
        Some(PathBuf::from(&*normalized))
    } else {
        None
    }
}

fn capture_patches(
    conn: &rusqlite::Connection,
    excluded_feature_ids: &HashSet<String>,
    storage_path: &Path,
    zip: &mut zip::ZipWriter<fs::File>,
    zip_opts: SimpleFileOptions,
    cancelled: &Arc<AtomicBool>,
    progress_fn: &dyn Fn(&str, u32),
) -> Result<bool, String> {
    let mut stmt = conn
        .prepare(
            "SELECT d.id, d.feature_id, d.path, d.label FROM directories d WHERE d.repo_url IS NOT NULL AND d.repo_url != ''",
        )
        .map_err(|e| format!("Failed to query directories: {}", e))?;

    let dirs: Vec<(String, String, String, Option<String>)> = stmt
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(|e| format!("Failed to read directories: {}", e))?
        .filter_map(|r| r.ok())
        .filter(|(_, fid, _, _)| !excluded_feature_ids.contains(fid))
        .collect();

    if dirs.is_empty() {
        return Ok(false);
    }

    let total = dirs.len();
    let mut any_patches = false;

    for (i, (dir_id, _, dir_path, label)) in dirs.iter().enumerate() {
        if cancelled.load(Ordering::Relaxed) {
            return Err("Export cancelled".to_string());
        }

        // Resolve relative path from DB
        let resolved_path = crate::paths::resolve_path(dir_path, storage_path);
        let display_name = label.as_deref().unwrap_or(
            resolved_path
                .file_name()
                .map(|n| n.to_str().unwrap_or(dir_path))
                .unwrap_or(dir_path),
        );
        let pct = 82 + ((i as u32 * 8) / total as u32);
        progress_fn(
            &format!("Checking repo {}/{} — {}", i + 1, total, display_name),
            pct,
        );

        if !resolved_path.exists() || !git::is_git_repo(&resolved_path) {
            continue;
        }

        if let Ok(dirty) = git::is_working_tree_dirty(&resolved_path) {
            if !dirty {
                continue;
            }
        }

        progress_fn(&format!("Capturing patch for {}...", display_name), pct);

        if let Ok(diff) = git::run_git(&resolved_path, &["diff", "HEAD"]) {
            if !diff.trim().is_empty() {
                let entry_name = format!("patches/{}.diff", dir_id);
                zip.start_file(&entry_name, zip_opts)
                    .map_err(|e| format!("Failed to add patch: {}", e))?;
                zip.write_all(diff.as_bytes())
                    .map_err(|e| format!("Failed to write patch: {}", e))?;
                any_patches = true;
            }
        }
    }

    Ok(any_patches)
}

fn collect_repo_directories(
    db_path: &Path,
    patch_ids: &HashSet<String>,
) -> Result<Vec<RepoDirectory>, String> {
    let conn = rusqlite::Connection::open(db_path)
        .map_err(|e| format!("Failed to open DB: {}", e))?;

    let mut stmt = conn
        .prepare(
            "SELECT d.id, d.feature_id, d.repo_url, d.label, f.title
             FROM directories d
             JOIN features f ON f.id = d.feature_id
             WHERE d.repo_url IS NOT NULL AND d.repo_url != ''",
        )
        .map_err(|e| format!("Failed to query: {}", e))?;

    let results = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .map_err(|e| format!("Failed to read: {}", e))?
        .filter_map(|r| r.ok())
        .map(|(id, feature_id, repo_url, label, feature_title)| {
            let has_patch = patch_ids.contains(&id);
            RepoDirectory {
                directory_id: id,
                feature_id,
                feature_title,
                repo_url,
                label,
                has_patch,
            }
        })
        .collect();

    Ok(results)
}

/// Extract `feature-hub.db` from a ZIP to a temp file and return its path.
fn extract_db_to_temp(zip_path: &Path) -> Result<PathBuf, String> {
    let file = fs::File::open(zip_path)
        .map_err(|e| format!("Failed to open archive: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Invalid ZIP archive: {}", e))?;
    let mut entry = archive
        .by_name("feature-hub.db")
        .map_err(|_| "Archive missing database — not a valid Feature Hub export".to_string())?;
    let temp_path = std::env::temp_dir()
        .join(format!("fh-import-{}.db", uuid::Uuid::new_v4()));
    let mut out =
        fs::File::create(&temp_path).map_err(|e| format!("Failed to create temp file: {}", e))?;
    std::io::copy(&mut entry, &mut out)
        .map_err(|e| format!("Failed to extract DB: {}", e))?;
    Ok(temp_path)
}

/// Read and parse `manifest.json` from a ZIP archive.
fn read_manifest_from_zip(zip_path: &Path) -> Result<ExportManifest, String> {
    let file = fs::File::open(zip_path)
        .map_err(|e| format!("Failed to open archive: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Invalid ZIP archive: {}", e))?;
    let mut entry = archive
        .by_name("manifest.json")
        .map_err(|_| "Archive missing manifest.json — not a valid Feature Hub export".to_string())?;
    let mut buf = String::new();
    entry
        .read_to_string(&mut buf)
        .map_err(|e| format!("Failed to read manifest: {}", e))?;
    serde_json::from_str(&buf).map_err(|e| format!("Invalid manifest: {}", e))
}

/// Import tags from `import.tags` into `main.tags`, returning a map of
/// import_tag_id → current_tag_id (handles same-name / different-ID divergence).
fn import_tags_with_map(
    current_conn: &rusqlite::Connection,
) -> Result<HashMap<String, String>, String> {
    let import_tags: Vec<(String, String, String)> = {
        let mut stmt = current_conn
            .prepare("SELECT id, name, color FROM import.tags")
            .map_err(|e| e.to_string())?;
        let v: Vec<(String, String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        v
    };

    let mut id_map = HashMap::new();
    for (import_id, name, color) in &import_tags {
        // Does the exact same ID exist in the current DB?
        let by_id: i64 = current_conn
            .query_row(
                "SELECT COUNT(*) FROM main.tags WHERE id = ?1",
                rusqlite::params![import_id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if by_id > 0 {
            id_map.insert(import_id.clone(), import_id.clone());
            continue;
        }

        // Does a tag with the same name exist (different ID)?
        let existing_id: Option<String> = {
            let mut stmt = current_conn
                .prepare("SELECT id FROM main.tags WHERE name = ?1")
                .map_err(|e| e.to_string())?;
            let v: Vec<String> = stmt
                .query_map(rusqlite::params![name], |r| r.get(0))
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok())
                .collect();
            v.into_iter().next()
        };

        if let Some(current_id) = existing_id {
            id_map.insert(import_id.clone(), current_id);
        } else {
            current_conn
                .execute(
                    "INSERT INTO main.tags (id, name, color) VALUES (?1, ?2, ?3)",
                    rusqlite::params![import_id, name, color],
                )
                .map_err(|e| format!("Failed to insert tag: {}", e))?;
            id_map.insert(import_id.clone(), import_id.clone());
        }
    }

    Ok(id_map)
}

/// Insert `import.feature_tags` into `main.feature_tags`, remapping tag IDs via `tag_id_map`.
fn import_feature_tags_mapped(
    current_conn: &rusqlite::Connection,
    tag_id_map: &HashMap<String, String>,
) -> Result<(), String> {
    let entries: Vec<(String, String)> = {
        let mut stmt = current_conn
            .prepare("SELECT feature_id, tag_id FROM import.feature_tags")
            .map_err(|e| e.to_string())?;
        let v: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        v
    };

    for (feature_id, import_tag_id) in &entries {
        let current_tag_id = tag_id_map.get(import_tag_id).unwrap_or(import_tag_id);
        let _ = current_conn.execute(
            "INSERT OR IGNORE INTO main.feature_tags (feature_id, tag_id) VALUES (?1, ?2)",
            rusqlite::params![feature_id, current_tag_id],
        );
    }

    Ok(())
}

/// Extract workspace files from the ZIP into `storage`, skipping DB, manifest, and patches.
fn extract_files_to_storage(zip_path: &Path, storage: &Path) -> Result<usize, String> {
    let file = fs::File::open(zip_path)
        .map_err(|e| format!("Failed to open archive: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Invalid ZIP archive: {}", e))?;
    let mut count = 0usize;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read archive entry: {}", e))?;
        let name = entry.name().to_string();

        if name == "feature-hub.db"
            || name == "manifest.json"
            || name.starts_with("patches/")
            || entry.is_dir()
        {
            continue;
        }

        let out_path = storage.join(&name);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create dir for {}: {}", name, e))?;
        }
        let mut out =
            fs::File::create(&out_path).map_err(|e| format!("Failed to create {}: {}", name, e))?;
        std::io::copy(&mut entry, &mut out)
            .map_err(|e| format!("Failed to extract {}: {}", name, e))?;
        count += 1;
    }

    Ok(count)
}

/// Like `collect_repo_directories` but takes an already-open connection.
fn collect_repo_directories_from_conn(
    conn: &rusqlite::Connection,
    patch_ids: &HashSet<String>,
) -> Result<Vec<RepoDirectory>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT d.id, d.feature_id, d.repo_url, d.label, f.title \
             FROM directories d \
             JOIN features f ON f.id = d.feature_id \
             WHERE d.repo_url IS NOT NULL AND d.repo_url != ''",
        )
        .map_err(|e| format!("Failed to query repo directories: {}", e))?;

    let results = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .map_err(|e| format!("Failed to read repo directories: {}", e))?
        .filter_map(|r| r.ok())
        .map(|(id, feature_id, repo_url, label, feature_title)| RepoDirectory {
            has_patch: patch_ids.contains(&id),
            directory_id: id,
            feature_id,
            feature_title,
            repo_url,
            label,
        })
        .collect();

    Ok(results)
}

fn archive_has_patches(zip_path: &Path) -> Result<HashSet<String>, String> {
    let file = fs::File::open(zip_path)
        .map_err(|e| format!("Failed to open archive: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Failed to read archive: {}", e))?;

    let mut ids = HashSet::new();
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index_raw(i) {
            let name = entry.name().to_string();
            if let Some(rest) = name.strip_prefix("patches/") {
                if let Some(id) = rest.strip_suffix(".diff") {
                    ids.insert(id.to_string());
                }
            }
        }
    }

    Ok(ids)
}
