import os
import tempfile
from typing import Optional

from repodiff.utils.config_manager import ConfigManager
from repodiff.utils.git_operations import GitOperations
from repodiff.utils.diff_parser import DiffParser
from repodiff.utils.token_counter import TokenCounter
from repodiff.filters.filter_manager import FilterManager


class RepoDiff:
    """
    Main class for the RepoDiff tool that handles the core functionality.
    """
    
    def __init__(self, config_file_name: str = "config.json"):
        """
        Initialize the RepoDiff tool.
        
        Args:
            config_file_name: The name of the configuration file to load.
        """
        self.config_manager = ConfigManager(config_file_name)
        self.token_counter = TokenCounter(self.config_manager.get_tiktoken_model())
        self.filter_manager = FilterManager(self.config_manager.get_filters())
        self.git_operations = GitOperations()
    
    def process_diff(self, commit1: str, commit2: str, output_file: str) -> int:
        """
        Process the diff between two commits and write the result to a file.
        
        Args:
            commit1: The first commit hash to compare.
            commit2: The second commit hash to compare.
            output_file: The file to write the processed diff to.
            
        Returns:
            The number of tokens in the processed diff.
        """
        # Get the raw diff output
        raw_diff = self.git_operations.run_git_diff(commit1, commit2)
        
        # Parse and process the diff
        patch_dict = DiffParser.parse_unified_diff(raw_diff)
        processed_dict = self.filter_manager.post_process_files(patch_dict)
        final_output = DiffParser.reconstruct_patch(processed_dict)
        
        # Create output directory if it doesn't exist
        os.makedirs(os.path.dirname(output_file), exist_ok=True)
        
        # Write the processed diff to the output file
        with open(output_file, 'w', encoding='utf-8') as f:
            f.write(final_output)
        
        # Calculate token count
        token_count = self.token_counter.count_tokens(final_output)
        
        return token_count
    
    @staticmethod
    def get_default_output_file() -> str:
        """
        Get the default output file path in the temporary directory.
        
        Returns:
            The default output file path.
        """
        temp_dir = tempfile.gettempdir()
        return os.path.join(temp_dir, "repodiff", "repodiff_output.txt") 