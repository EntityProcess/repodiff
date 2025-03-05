use std::process::Command;
use crate::error::{RepoDiffError, Result};

/// Handles git operations for the RepoDiff tool
pub struct GitOperations;

impl GitOperations {
    /// Create a new GitOperations instance
    pub fn new() -> Self {
        GitOperations
    }

    /// Execute the git diff command and return the result
    ///
    /// # Arguments
    ///
    /// * `commit1` - The first commit hash to compare
    /// * `commit2` - The second commit hash to compare
    ///
    /// # Returns
    ///
    /// The output of the git diff command as a string
    pub fn run_git_diff(&self, commit1: &str, commit2: &str) -> Result<String> {
        let output = Command::new("git")
            .args([
                "diff",
                commit1,
                commit2,
                "--unified=999999",
                "--ignore-all-space",
                "--find-renames",
            ])
            .output()
            .map_err(|e| RepoDiffError::GitError(format!("Failed to execute git diff: {}", e)))?;

        if !output.status.success() {
            return Err(RepoDiffError::GitError(format!(
                "Git diff command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Get the latest commit hash for the current branch
    pub fn get_latest_commit(&self) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .map_err(|e| RepoDiffError::GitError(format!("Failed to get latest commit: {}", e)))?;

        if !output.status.success() {
            return Err(RepoDiffError::GitError(format!(
                "Failed to get latest commit: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Get the latest common commit between the current branch and base branch
    ///
    /// # Arguments
    ///
    /// * `branch` - The name of the base branch to compare with
    pub fn get_latest_common_commit_with_branch(&self, branch: &str) -> Result<String> {
        let output = Command::new("git")
            .args(["merge-base", "HEAD", branch])
            .output()
            .map_err(|e| {
                RepoDiffError::GitError(format!(
                    "Failed to get latest common commit with '{}': {}",
                    branch, e
                ))
            })?;

        if !output.status.success() {
            return Err(RepoDiffError::GitError(format!(
                "Failed to get latest common commit with '{}': {}",
                branch,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
} 