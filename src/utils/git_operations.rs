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

    /// Find the merge base commit between the current branch (HEAD) and a target branch
    ///
    /// # Arguments
    ///
    /// * `branch` - The name of the target branch to find the common ancestor with
    pub fn find_merge_base(&self, branch: &str) -> Result<String> {
        let output = Command::new("git")
            .args(["merge-base", "HEAD", branch])
            .output()
            .map_err(|e| {
                RepoDiffError::GitError(format!(
                    "Failed to find merge base with branch '{}': {}",
                    branch, e
                ))
            })?;

        if !output.status.success() {
            return Err(RepoDiffError::GitError(format!(
                "Failed to find merge base with branch '{}': {}",
                branch,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Get the latest commit hash for a specific branch
    ///
    /// # Arguments
    ///
    /// * `branch_name` - The name of the branch
    pub fn get_branch_head(&self, branch_name: &str) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", branch_name])
            .output()
            .map_err(|e| RepoDiffError::GitError(format!("Failed to get HEAD for branch '{}': {}", branch_name, e)))?;

        if !output.status.success() {
            return Err(RepoDiffError::GitError(format!(
                "Failed to get HEAD for branch '{}': {}",
                branch_name,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Get the previous commit of a given commit hash
    ///
    /// # Arguments
    ///
    /// * `commit` - The commit hash to get the previous commit for
    ///
    /// # Returns
    ///
    /// The hash of the previous commit
    pub fn get_previous_commit(&self, commit: &str) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", &format!("{}^1", commit)])
            .output()
            .map_err(|e| RepoDiffError::GitError(format!("Failed to get previous commit for '{}': {}", commit, e)))?;

        if !output.status.success() {
            return Err(RepoDiffError::GitError(format!(
                "Failed to get previous commit for '{}': {}",
                commit,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
} 