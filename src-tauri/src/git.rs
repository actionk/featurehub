use std::path::Path;
use std::process::Command;

pub fn run_git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(format!("git {} failed: {}", args.join(" "), stderr))
    }
}

pub fn clone_repo(url: &str, target_dir: &Path) -> Result<(), String> {
    let output = Command::new("git")
        .args([
            "clone",
            "--depth", "1",
            "-c", "core.longpaths=true",
            url,
            &target_dir.to_string_lossy(),
        ])
        .output()
        .map_err(|e| format!("Failed to run git clone: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(format!("git clone failed: {}", stderr))
    }
}

pub fn is_git_repo(dir: &Path) -> bool {
    if !dir.exists() {
        return false;
    }
    run_git(dir, &["rev-parse", "--is-inside-work-tree"]).is_ok()
}

pub fn is_working_tree_dirty(dir: &Path) -> Result<bool, String> {
    let output = run_git(dir, &["status", "--porcelain"])?;
    Ok(!output.is_empty())
}

pub fn list_local_branches(dir: &Path) -> Result<Vec<String>, String> {
    let output = run_git(dir, &["branch", "--all", "--format=%(refname:short)"])?;
    let mut seen = std::collections::HashSet::new();
    let mut branches = Vec::new();
    for line in output.lines() {
        let name = line.strip_prefix("origin/").unwrap_or(line);
        if name == "HEAD" {
            continue;
        }
        if seen.insert(name.to_string()) {
            branches.push(name.to_string());
        }
    }

    // For shallow clones, also list remote branches via ls-remote
    if let Ok(remote_output) = run_git(dir, &["ls-remote", "--heads", "origin"]) {
        for line in remote_output.lines() {
            // Format: <sha>\trefs/heads/<branch>
            if let Some(refname) = line.split('\t').nth(1) {
                let name = refname.strip_prefix("refs/heads/").unwrap_or(refname);
                if seen.insert(name.to_string()) {
                    branches.push(name.to_string());
                }
            }
        }
    }

    branches.sort();
    Ok(branches)
}

pub fn get_current_branch(dir: &Path) -> Result<String, String> {
    run_git(dir, &["rev-parse", "--abbrev-ref", "HEAD"])
}

/// Returns a summary of the repo's working tree status:
/// (modified_count, untracked_count, staged_count)
pub fn get_status_counts(dir: &Path) -> Result<(usize, usize, usize), String> {
    let output = run_git(dir, &["status", "--porcelain"])?;
    if output.is_empty() {
        return Ok((0, 0, 0));
    }
    let mut modified = 0;
    let mut untracked = 0;
    let mut staged = 0;
    for line in output.lines() {
        if line.len() < 2 {
            continue;
        }
        let index = line.as_bytes()[0];
        let worktree = line.as_bytes()[1];
        if line.starts_with("??") {
            untracked += 1;
        } else {
            if index != b' ' && index != b'?' {
                staged += 1;
            }
            if worktree != b' ' && worktree != b'?' {
                modified += 1;
            }
        }
    }
    Ok((modified, untracked, staged))
}

pub fn create_branch(dir: &Path, name: &str) -> Result<(), String> {
    if !dir.exists() {
        return Err(format!("Directory does not exist: {}", dir.display()));
    }
    if !is_git_repo(dir) {
        return Err(format!("Not a git repository: {}", dir.display()));
    }
    if is_working_tree_dirty(dir)? {
        return Err("Cannot create branch: uncommitted changes. Commit or stash first.".to_string());
    }

    // Check branch doesn't already exist
    let branches = list_local_branches(dir)?;
    if branches.iter().any(|b| b == name) {
        return Err(format!("Branch '{}' already exists", name));
    }

    run_git(dir, &["checkout", "-b", name])?;
    Ok(())
}

/// Fetch from the remote (origin by default).
pub fn fetch(dir: &Path) -> Result<(), String> {
    if !dir.exists() {
        return Err(format!("Directory does not exist: {}", dir.display()));
    }
    if !is_git_repo(dir) {
        return Err(format!("Not a git repository: {}", dir.display()));
    }
    run_git(dir, &["fetch", "--prune"])?;
    Ok(())
}

/// Returns (ahead, behind) counts relative to the upstream tracking branch.
/// Returns None if there's no upstream configured or if it can't be determined.
pub fn get_ahead_behind(dir: &Path) -> Option<(usize, usize)> {
    // Get the upstream tracking branch
    let upstream = run_git(dir, &["rev-parse", "--abbrev-ref", "@{upstream}"]).ok()?;
    if upstream.is_empty() {
        return None;
    }

    // Get ahead/behind in one command: "ahead<TAB>behind"
    let output = run_git(
        dir,
        &[
            "rev-list",
            "--left-right",
            "--count",
            &format!("HEAD...{}", upstream),
        ],
    )
    .ok()?;

    let parts: Vec<&str> = output.split_whitespace().collect();
    if parts.len() == 2 {
        let ahead = parts[0].parse().ok()?;
        let behind = parts[1].parse().ok()?;
        Some((ahead, behind))
    } else {
        None
    }
}

pub fn checkout_branch(dir: &Path, name: &str) -> Result<(), String> {
    if !dir.exists() {
        return Err(format!("Directory does not exist: {}", dir.display()));
    }
    if !is_git_repo(dir) {
        return Err(format!("Not a git repository: {}", dir.display()));
    }
    if is_working_tree_dirty(dir)? {
        return Err(
            "Cannot switch branches: uncommitted changes. Commit or stash first.".to_string(),
        );
    }

    // Try direct checkout first (works for local branches)
    if run_git(dir, &["checkout", name]).is_ok() {
        return Ok(());
    }

    // For shallow clones: fetch the specific remote branch then checkout
    run_git(dir, &["fetch", "origin", &format!("{}:{}", name, name)])?;
    run_git(dir, &["checkout", name])?;
    Ok(())
}
