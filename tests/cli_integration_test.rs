#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::process::{Command, Stdio};
    use tempfile::{tempdir, TempDir};

    struct TestRepo {
        dir: TempDir,
        main_head: String,
        feature_head: String,
        merge_base: String,
    }

    /// Sets up a temporary Git repository with a main and feature branch.
    ///
    /// Structure:
    /// M1 -> M2 (main)
    ///  \-> F1 -> F2 (feature)
    ///
    /// Returns commit hashes for main HEAD, feature HEAD, and the merge base.
    fn setup_git_repo() -> Result<TestRepo, Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let repo_path = dir.path();

        // Helper function to run git commands
        let run_git = |args: &[&str]| -> Result<String, Box<dyn std::error::Error>> {
            let output = Command::new("git")
                .current_dir(repo_path)
                .args(args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?;
            if !output.status.success() {
                return Err(format!(
                    "Git command failed: {:?}\nStderr: {}",
                    args,
                    String::from_utf8_lossy(&output.stderr)
                )
                .into());
            }
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        };
        
        // Helper function to create a commit
        let create_commit = |msg: &str, file_name: &str, content: &str| -> Result<String, Box<dyn std::error::Error>> {
            fs::write(repo_path.join(file_name), content)?;
            run_git(&["add", file_name])?;
            run_git(&["commit", "-m", msg])?;
            run_git(&["rev-parse", "HEAD"]) 
        };

        // Initialize repo and make first commit (M1)
        run_git(&["init"])?;
        run_git(&["config", "user.email", "test@example.com"])?;
        run_git(&["config", "user.name", "Test User"])?;
        let commit_m1 = create_commit("Initial commit", "file1.txt", "Content 1")?;
        
        // Ensure the primary branch is named 'main'
        let current_branch = run_git(&["branch", "--show-current"])?;
        if current_branch != "main" {
            run_git(&["branch", "-M", &current_branch, "main"])?;
        }
        
        // Create feature branch from main (M1) and make commits (F1, F2)
        run_git(&["checkout", "-b", "feature"])?;
        create_commit("Feature commit 1", "file2.txt", "Feature content A")?;
        let commit_f2 = create_commit("Feature commit 2", "file1.txt", "Content 1 modified by feature")?;
        
        // Switch back to main and make commit (M2)
        run_git(&["checkout", "main"])?;
        let commit_m2 = create_commit("Main commit 2", "file3.txt", "Main content B")?;
        
        // Verify HEAD is now feature branch F2 for subsequent tests
        run_git(&["checkout", "feature"])?; 

        Ok(TestRepo {
            dir,
            main_head: commit_m2,
            feature_head: commit_f2,
            merge_base: commit_m1, // M1 is the merge base
        })
    }

    fn run_repodiff(args: &[&str], cwd: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
        let repodiff_path = env!("CARGO_BIN_EXE_repodiff");
        let output = Command::new(repodiff_path)
            .current_dir(cwd)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "repodiff command failed: {:?}\nStderr: {}",
                args,
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Abbreviate commit hash for cleaner output in assertions
    fn short_hash(hash: &str) -> &str {
        &hash[..12.min(hash.len())]
    }
    
    #[test]
    fn test_compare_branch_head() -> Result<(), Box<dyn std::error::Error>> {
        let repo = setup_git_repo()?;
        
        let output = run_repodiff(&["-b", "main"], &repo.dir.path().to_path_buf())?;

        let expected_output = format!(
            "Comparing target branch 'main' HEAD ({}) and current HEAD ({})",
            short_hash(&repo.main_head),
            short_hash(&repo.feature_head)
        );
        
        assert!(output.contains(&expected_output), "Output did not contain expected branch head comparison message.\nExpected: {}\nGot: {}", expected_output, output);
        assert!(repo.dir.path().join("repodiff_output.txt").exists(), "Output file was not created.");

        Ok(())
    }

    #[test]
    fn test_compare_merge_base() -> Result<(), Box<dyn std::error::Error>> {
        let repo = setup_git_repo()?;
        
        let output = run_repodiff(&["-b", "main", "-a"], &repo.dir.path().to_path_buf())?;

        let expected_output = format!(
            "Comparing merge-base with branch 'main' ({}) and current HEAD ({})",
            short_hash(&repo.merge_base),
            short_hash(&repo.feature_head)
        );
        
        assert!(output.contains(&expected_output), "Output did not contain expected merge-base comparison message.\nExpected: {}\nGot: {}", expected_output, output);
        assert!(repo.dir.path().join("repodiff_output.txt").exists(), "Output file was not created.");

        Ok(())
    }
} 