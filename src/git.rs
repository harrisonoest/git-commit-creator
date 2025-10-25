use anyhow::{Context, Result};
use git2::{Repository, Status, StatusOptions};
use std::collections::HashMap;
use std::process::Command;

type FileStatusMap = HashMap<String, FileStatus>;

/// Verifies current directory is a git repository and returns Repository handle
pub fn ensure_git_repository() -> Result<Repository> {
    Repository::discover(".").context("This directory is not inside a git repository")
}

/// Checks if there are any unstaged changes in the repository
pub fn has_changes(repo: &Repository) -> Result<bool> {
    let mut opts = StatusOptions::new();
    opts.include_ignored(false);

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

    Ok(true)
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
    opts.include_ignored(false);

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
    let output = Command::new("git").args(["branch", branch_name]).output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to create branch: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output = Command::new("git")
        .args(["checkout", branch_name])
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to checkout branch: {}",
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
            let lines: Vec<&str> = content.lines().take(100).collect();
            let preview = lines
                .iter()
                .map(|line| format!("+ {}", line))
                .collect::<Vec<_>>()
                .join("\n");

            let total_lines = content.lines().count();
            if total_lines > 100 {
                return Ok(format!(
                    "{}\n... ({} more lines)",
                    preview,
                    total_lines - 100
                ));
            }
            return Ok(preview);
        }
        return Ok("No changes to display".to_string());
    }

    // Truncate large diffs
    let lines: Vec<&str> = diff.lines().collect();
    if lines.len() > 100 {
        let truncated = lines[..100].join("\n");
        Ok(format!(
            "{}\n... (diff truncated, {} more lines)",
            truncated,
            lines.len() - 100
        ))
    } else {
        Ok(diff)
    }
}
