import pytest
import sys
from unittest.mock import patch, MagicMock
from repodiff.utils.git_operations import GitOperations
import subprocess


class TestGitOperations:
    """
    Tests for the GitOperations class.
    """
    
    @patch('subprocess.run')
    def test_run_git_diff_success(self, mock_run):
        """Test running git diff successfully."""
        # Set up the mock
        mock_process = MagicMock()
        mock_process.stdout = "mock diff output"
        mock_run.return_value = mock_process
        
        # Call the method
        result = GitOperations.run_git_diff("commit1", "commit2")
        
        # Check the result
        assert result == "mock diff output"
        
        # Verify the subprocess.run call
        mock_run.assert_called_once_with(
            ["git", "diff", "commit1", "commit2", "--unified=999999", "--ignore-all-space", "--find-renames"],
            capture_output=True, text=True, check=True, encoding='utf-8', errors='replace'
        )
    
    @patch('subprocess.run')
    @patch('sys.exit')
    def test_run_git_diff_error(self, mock_exit, mock_run):
        """Test running git diff with an error."""
        # Set up the mock to raise an exception
        mock_run.side_effect = subprocess.CalledProcessError(1, "git diff")
        
        # Call the method
        GitOperations.run_git_diff("commit1", "commit2")
        
        # Verify sys.exit was called
        mock_exit.assert_called_once_with(1)
    
    @patch('subprocess.run')
    def test_get_latest_commit_success(self, mock_run):
        """Test getting the latest commit successfully."""
        # Set up the mock
        mock_process = MagicMock()
        mock_process.stdout = "abcdef1234567890\n"
        mock_run.return_value = mock_process
        
        # Call the method
        result = GitOperations.get_latest_commit()
        
        # Check the result
        assert result == "abcdef1234567890"
        
        # Verify the subprocess.run call
        mock_run.assert_called_once_with(
            ["git", "rev-parse", "HEAD"],
            capture_output=True, text=True, check=True, encoding='utf-8', errors='replace'
        )
    
    @patch('subprocess.run')
    @patch('sys.exit')
    def test_get_latest_commit_error(self, mock_exit, mock_run):
        """Test getting the latest commit with an error."""
        # Set up the mock to raise an exception
        mock_run.side_effect = subprocess.CalledProcessError(1, "git rev-parse")
        
        # Call the method
        GitOperations.get_latest_commit()
        
        # Verify sys.exit was called
        mock_exit.assert_called_once_with(1)
    
    @patch('subprocess.run')
    def test_get_latest_common_commit_with_branch_success(self, mock_run):
        """Test getting the latest common commit successfully."""
        # Set up the mock
        mock_process = MagicMock()
        mock_process.stdout = "abcdef1234567890\n"
        mock_run.return_value = mock_process
        
        # Call the method
        result = GitOperations.get_latest_common_commit_with_branch("master")
        
        # Check the result
        assert result == "abcdef1234567890"
        
        # Verify the subprocess.run call
        mock_run.assert_called_once_with(
            ["git", "merge-base", "HEAD", "master"],
            capture_output=True, text=True, check=True, encoding='utf-8', errors='replace'
        )
    
    @patch('subprocess.run')
    @patch('sys.exit')
    def test_get_latest_common_commit_with_branch_error(self, mock_exit, mock_run):
        """Test getting the latest common commit with an error."""
        # Set up the mock to raise an exception
        mock_run.side_effect = subprocess.CalledProcessError(1, "git merge-base")
        
        # Call the method
        GitOperations.get_latest_common_commit_with_branch("master")
        
        # Verify sys.exit was called
        mock_exit.assert_called_once_with(1) 