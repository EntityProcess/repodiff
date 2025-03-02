import subprocess
import sys
from typing import Optional


class GitOperations:
    """
    Handles git operations for the RepoDiff tool.
    """
    
    @staticmethod
    def run_git_diff(commit1: str, commit2: str) -> str:
        """
        Execute the git diff command and return the result.
        
        Args:
            commit1: The first commit hash to compare.
            commit2: The second commit hash to compare.
            
        Returns:
            The output of the git diff command as a string.
            
        Raises:
            SystemExit: If the git diff command fails.
        """
        try:
            result = subprocess.run(
                ["git", "diff", commit1, commit2, "--unified=999999", "--ignore-all-space", "--find-renames"],
                capture_output=True, text=True, check=True, encoding='utf-8', errors='replace'
            )
            return result.stdout
        except subprocess.CalledProcessError as e:
            print(f"Error running git diff: {e}")
            sys.exit(1)
    
    @staticmethod
    def get_latest_commit() -> str:
        """
        Get the latest commit hash for the current branch.
        
        Returns:
            The latest commit hash as a string.
            
        Raises:
            SystemExit: If the git command fails.
        """
        try:
            result = subprocess.run(
                ["git", "rev-parse", "HEAD"],
                capture_output=True, text=True, check=True, encoding='utf-8', errors='replace'
            )
            return result.stdout.strip()
        except subprocess.CalledProcessError as e:
            print(f"Error getting the latest commit: {e}")
            sys.exit(1)
    
    @staticmethod
    def get_latest_common_commit_with_branch(branch: str) -> str:
        """
        Get the latest common commit between the current branch and base branch.
        
        Args:
            branch: The name of the base branch to compare with.
            
        Returns:
            The latest common commit hash as a string.
            
        Raises:
            SystemExit: If the git command fails.
        """
        try:
            result = subprocess.run(
                ["git", "merge-base", "HEAD", branch],
                capture_output=True, text=True, check=True, encoding='utf-8', errors='replace'
            )
            return result.stdout.strip()
        except subprocess.CalledProcessError as e:
            print(f"Error getting the latest common commit with '{branch}': {e}")
            sys.exit(1) 