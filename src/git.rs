use anyhow::{Context, Result};
use git2::{Repository, Status, StatusOptions};
use std::collections::HashMap;
use std::process::Command;

type FileStatusMap = HashMap<String, FileStatus>;

/// Verifies current directory is a git repository and returns Repository handle
pub fn ensure_git_repository() -> Result<Repository> {
    Repository::discover(".").context("This directory is not inside a git repository")
}

/// Checks if there are any changes (including untracked files) in the repository
pub fn has_changes(repo: &Repository) -> Result<bool> {
    let mut opts = StatusOptions::new();
    opts.include_ignored(false).include_untracked(true);

    let statuses = repo.statuses(Some(&mut opts))?;

    for entry in statuses.iter() {
        if entry.status().intersects(
            Status::WT_NEW
                | Status::WT_MODIFIED
                | Status::WT_DELETED
                | Status::INDEX_NEW
                | Status::INDEX_MODIFIED
                | Status::INDEX_DELETED,
        ) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// File status indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
}

impl FileStatus {
    pub fn as_str(&self) -> &str {
        match self {
            FileStatus::Added => "A",
            FileStatus::Modified => "M",
            FileStatus::Deleted => "D",
        }
    }
}

/// Returns all changed files (staged and unstaged), list of staged files, and file statuses
pub fn get_all_changed_files(
    repo: &Repository,
) -> Result<(Vec<String>, Vec<String>, FileStatusMap)> {
    let mut opts = StatusOptions::new();
    opts.include_ignored(false).include_untracked(true);

    let statuses = repo.statuses(Some(&mut opts))?;
    let mut all_files = Vec::new();
    let mut staged_files = Vec::new();
    let mut file_statuses = std::collections::HashMap::new();

    for entry in statuses.iter() {
        if let Some(path) = entry.path() {
            let path_str = path.to_string();
            let status = entry.status();

            if status.intersects(
                Status::WT_NEW
                    | Status::WT_MODIFIED
                    | Status::WT_DELETED
                    | Status::INDEX_NEW
                    | Status::INDEX_MODIFIED
                    | Status::INDEX_DELETED,
            ) {
                all_files.push(path_str.clone());

                let file_status = if status.intersects(Status::WT_NEW | Status::INDEX_NEW) {
                    FileStatus::Added
                } else if status.intersects(Status::WT_DELETED | Status::INDEX_DELETED) {
                    FileStatus::Deleted
                } else {
                    FileStatus::Modified
                };
                file_statuses.insert(path_str.clone(), file_status);
            }

            if status.intersects(Status::INDEX_NEW | Status::INDEX_MODIFIED | Status::INDEX_DELETED)
            {
                staged_files.push(path_str);
            }
        }
    }

    Ok((all_files, staged_files, file_statuses))
}

/// Stages files based on extensions and/or directory filters
pub fn stage_files(extensions: Option<String>, directory: Option<String>) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.arg("add");

    if let Some(exts) = extensions {
        let extensions: Vec<&str> = exts.split(',').map(|s| s.trim()).collect();
        if let Some(dir) = directory {
            for ext in extensions {
                cmd.arg(format!(
                    "{}/**/*.{}",
                    dir.trim_end_matches('/'),
                    ext.trim_start_matches('.')
                ));
            }
        } else {
            for ext in extensions {
                cmd.arg(format!("*.{}", ext.trim_start_matches('.')));
            }
        }
    } else if let Some(dir) = directory {
        cmd.arg(dir.trim_end_matches('/'));
    } else {
        cmd.arg(".");
    }

    let output = cmd.output()?;
    if !output.status.success() {
        anyhow::bail!(
            "Failed to stage files: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

/// Builds final commit message
pub fn build_commit_message(prefix: &str, message: &str) -> Result<String> {
    Ok(format!("{prefix} {message}"))
}

/// Creates commit with message and optionally pushes to remote
pub fn commit_and_push(commit_msg: &str, no_push: bool) -> Result<()> {
    let output = Command::new("git")
        .args(["commit", "-m", commit_msg])
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to commit: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    println!("✅ Committed: {commit_msg}");

    if !no_push {
        let output = Command::new("git").arg("push").output()?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to push: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        println!("🚀 Pushed to remote");
    }

    Ok(())
}

/// Stages a single file
pub fn stage_file(file_path: &str) -> Result<()> {
    let output = Command::new("git").args(["add", file_path]).output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to stage file: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

/// Unstages a single file
pub fn unstage_file(file_path: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["reset", "HEAD", "--", file_path])
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to unstage file: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

/// Unstages all currently staged files
pub fn unstage_all() -> Result<()> {
    let output = Command::new("git").args(["reset", "HEAD", "--"]).output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to unstage: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

/// Builds branch name in format: prefix/{story_prefix}{story}/{name} or prefix/{name}
pub fn build_branch_name(
    prefix: &str,
    story: Option<&str>,
    name: &str,
    story_prefix: Option<&str>,
) -> Result<String> {
    let branch_name = if let Some(story_num) = story {
        let prefix_str = story_prefix.unwrap_or("");
        format!("{prefix}/{prefix_str}{story_num}/{name}")
    } else {
        format!("{prefix}/{name}")
    };
    Ok(branch_name)
}

/// Creates a new branch and checks it out
pub fn create_and_checkout_branch(branch_name: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["checkout", "-b", branch_name])
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to create branch: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    println!("✅ Created and checked out branch: {branch_name}");
    Ok(())
}

/// Gets the diff for a specific file
pub fn get_file_diff(file_path: &str, is_staged: bool) -> Result<String> {
    // Check if file is binary
    let file_output = Command::new("git")
        .args(["diff", "--numstat", "--", file_path])
        .output()?;

    if file_output.status.success() {
        let output_str = String::from_utf8_lossy(&file_output.stdout);
        if output_str.starts_with('-') && output_str.contains('-') {
            return Ok("Binary file - no preview available".to_string());
        }
    }

    let mut cmd = Command::new("git");
    cmd.arg("diff");

    if is_staged {
        cmd.arg("--cached");
    }

    cmd.args(["--", file_path]);

    let output = cmd.output()?;

    if !output.status.success() {
        return Ok("Error fetching diff".to_string());
    }

    let diff = String::from_utf8_lossy(&output.stdout).to_string();

    if diff.trim().is_empty() {
        // For new files, show the file content
        if let Ok(content) = std::fs::read_to_string(file_path) {
            return Ok(content
                .lines()
                .map(|line| format!("+ {line}"))
                .collect::<Vec<_>>()
                .join("\n"));
        }
        return Ok("No changes to display".to_string());
    }

    Ok(diff)
}

/// Updates remote branches using git remote update
#[allow(dead_code)]
pub fn update_remote_branches() -> Result<()> {
    Command::new("git")
        .args(["remote", "update", "origin", "--prune"])
        .output()
        .context("Failed to update remote branches")?;
    Ok(())
}

/// Gets all branches (local and remote)
#[allow(dead_code)]
pub fn get_all_branches() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["branch", "-a"])
        .output()
        .context("Failed to get branches")?;

    let branches = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| {
            let branch = line.trim();
            if let Some(stripped) = branch.strip_prefix('*') {
                stripped.trim().to_string()
            } else {
                branch.to_string()
            }
        })
        .collect();

    Ok(branches)
}

/// Searches branches by substring (case-insensitive)
#[allow(dead_code)]
pub fn search_branches(query: &str, branches: &[String]) -> Vec<String> {
    let query_lower = query.to_lowercase();
    branches
        .iter()
        .filter(|branch| {
            if !branch.starts_with("origin/") || query_lower.starts_with("origin/") {
                branch.to_lowercase().contains(&query_lower)
            } else {
                false
            }
        })
        .cloned()
        .collect()
}

/// Checks out a branch, handling remote branches by creating local tracking branches
#[allow(dead_code)]
pub fn checkout_branch(branch: &str) -> Result<()> {
    let is_remote = branch.starts_with("remotes/origin/") || branch.starts_with("origin/");

    if is_remote {
        let local_branch = branch
            .strip_prefix("remotes/origin/")
            .or_else(|| branch.strip_prefix("origin/"))
            .unwrap();

        let check_exists = Command::new("git")
            .args(["show-ref", "--verify", &format!("refs/heads/{local_branch}")])
            .output()?;

        if check_exists.status.success() {
            Command::new("git")
                .args(["checkout", local_branch])
                .output()
                .context("Failed to checkout branch")?;
        } else {
            Command::new("git")
                .args(["checkout", "-b", local_branch, "--track", branch])
                .output()
                .context("Failed to create tracking branch")?;
        }
    } else {
        Command::new("git")
            .args(["checkout", branch])
            .output()
            .context("Failed to checkout branch")?;
    }

    Ok(())
}
