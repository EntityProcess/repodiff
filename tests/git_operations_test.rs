use repodiff::utils::git_operations::GitOperations;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

// Helper function to set up a test git repository
fn setup_test_repo() -> tempfile::TempDir {
    let temp_dir = tempdir().unwrap();
    let repo_path = temp_dir.path();
    
    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to initialize git repo");
    
    // Configure git user for commits
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to configure git user name");
    
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to configure git user email");
    
    // Create a file and commit it
    let file_path = repo_path.join("file1.txt");
    fs::write(&file_path, "Initial content").expect("Failed to write file");
    
    Command::new("git")
        .args(["add", "file1.txt"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to add file");
    
    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to commit");
    
    temp_dir
}

#[test]
#[ignore] // Ignore by default as it requires git to be installed
fn test_run_git_diff() {
    let temp_dir = setup_test_repo();
    let repo_path = temp_dir.path();
    
    // Get the initial commit hash
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to get commit hash");
    
    let commit1 = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    // Modify the file and create a new commit
    let file_path = repo_path.join("file1.txt");
    fs::write(&file_path, "Modified content").expect("Failed to modify file");
    
    Command::new("git")
        .args(["add", "file1.txt"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to add modified file");
    
    Command::new("git")
        .args(["commit", "-m", "Second commit"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to commit modified file");
    
    // Get the second commit hash
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to get second commit hash");
    
    let commit2 = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    // Test the run_git_diff function
    let git_operations = GitOperations::new();
    
    // Change to the repo directory for the test
    let current_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(repo_path).unwrap();
    
    let diff = git_operations.run_git_diff(&commit1, &commit2).unwrap();
    
    // Change back to the original directory
    std::env::set_current_dir(current_dir).unwrap();
    
    // The diff should contain the file name and the content change
    assert!(diff.contains("file1.txt"));
    assert!(diff.contains("-Initial content"));
    assert!(diff.contains("+Modified content"));
}

#[test]
#[ignore] // Ignore by default as it requires git to be installed
fn test_get_latest_commit() {
    let temp_dir = setup_test_repo();
    let repo_path = temp_dir.path();
    
    // Get the commit hash using git command
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to get commit hash");
    
    let expected_commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    // Test the get_latest_commit function
    let git_operations = GitOperations::new();
    
    // Change to the repo directory for the test
    let current_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(repo_path).unwrap();
    
    let commit = git_operations.get_latest_commit().unwrap();
    
    // Change back to the original directory
    std::env::set_current_dir(current_dir).unwrap();
    
    // The commit should match the expected commit
    assert_eq!(commit, expected_commit);
}

#[test]
#[ignore] // Ignore by default as it requires git to be installed
fn test_get_latest_common_commit_with_branch() {
    let temp_dir = setup_test_repo();
    let repo_path = temp_dir.path();
    
    // Get the initial commit hash
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to get commit hash");
    
    let initial_commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    // Create a new branch
    Command::new("git")
        .args(["checkout", "-b", "test-branch"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to create branch");
    
    // Modify the file and commit on the new branch
    let file_path = repo_path.join("file1.txt");
    fs::write(&file_path, "Branch content").expect("Failed to modify file on branch");
    
    Command::new("git")
        .args(["add", "file1.txt"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to add modified file on branch");
    
    Command::new("git")
        .args(["commit", "-m", "Branch commit"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to commit on branch");
    
    // Switch back to main and make another commit
    Command::new("git")
        .args(["checkout", "main"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to switch to main");
    
    let file_path = repo_path.join("file2.txt");
    fs::write(&file_path, "New file content").expect("Failed to create new file");
    
    Command::new("git")
        .args(["add", "file2.txt"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to add new file");
    
    Command::new("git")
        .args(["commit", "-m", "Main commit"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to commit on main");
    
    // Test the get_latest_common_commit_with_branch function
    let git_operations = GitOperations::new();
    
    // Change to the repo directory for the test
    let current_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(repo_path).unwrap();
    
    let ancestor = git_operations.get_latest_common_commit_with_branch("test-branch").unwrap();
    
    // Change back to the original directory
    std::env::set_current_dir(current_dir).unwrap();
    
    // The common ancestor should be the initial commit
    assert_eq!(ancestor, initial_commit);
}

#[test]
#[ignore] // Ignore by default as it requires git to be installed
fn test_get_previous_commit() {
    let temp_dir = setup_test_repo();
    let repo_path = temp_dir.path();
    
    // Get the initial commit hash
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to get commit hash");
    
    let initial_commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    // Modify the file and create a new commit
    let file_path = repo_path.join("file1.txt");
    fs::write(&file_path, "Modified content").expect("Failed to modify file");
    
    Command::new("git")
        .args(["add", "file1.txt"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to add modified file");
    
    Command::new("git")
        .args(["commit", "-m", "Second commit"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to commit modified file");
    
    // Get the second commit hash
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to get second commit hash");
    
    let second_commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    // Test the get_previous_commit function
    let git_operations = GitOperations::new();
    
    // Change to the repo directory for the test
    let current_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(repo_path).unwrap();
    
    let previous_commit = git_operations.get_previous_commit(&second_commit).unwrap();
    
    // Change back to the original directory
    std::env::set_current_dir(current_dir).unwrap();
    
    // The previous commit should be the initial commit
    assert_eq!(previous_commit, initial_commit);
} 