use std::fs;
use std::path::Path;

use crate::error::Result;
use crate::utils::config_manager::ConfigManager;
use crate::utils::git_operations::GitOperations;
use crate::utils::diff_parser::DiffParser;
use crate::utils::token_counter::TokenCounter;
use crate::filters::filter_manager::FilterManager;

/// Main class for the RepoDiff tool that handles the core functionality
pub struct RepoDiff {
    /// Configuration manager
    config_manager: ConfigManager,
    /// Token counter
    token_counter: TokenCounter,
    /// Filter manager
    filter_manager: FilterManager,
    /// Git operations
    git_operations: GitOperations,
}

impl RepoDiff {
    /// Initialize the RepoDiff tool
    ///
    /// # Arguments
    ///
    /// * `config_file_name` - The name of the configuration file to load
    pub fn new(config_file_name: &str) -> Result<Self> {
        let config_manager = ConfigManager::new(config_file_name)?;
        let token_counter = TokenCounter::new(config_manager.get_tiktoken_model())?;
        let filter_manager = FilterManager::new(config_manager.get_filters());
        let git_operations = GitOperations::new();
        
        Ok(RepoDiff {
            config_manager,
            token_counter,
            filter_manager,
            git_operations,
        })
    }
    
    /// Process the diff between two commits and write the result to a file
    ///
    /// # Arguments
    ///
    /// * `commit1` - The first commit hash to compare
    /// * `commit2` - The second commit hash to compare
    /// * `output_file` - The file to write the processed diff to
    ///
    /// # Returns
    ///
    /// The number of tokens in the processed diff
    pub fn process_diff(&self, commit1: &str, commit2: &str, output_file: &str) -> Result<usize> {
        // Get the raw diff output
        let raw_diff = self.git_operations.run_git_diff(commit1, commit2)?;
        
        // Parse and process the diff
        let patch_dict = DiffParser::parse_unified_diff(&raw_diff)?;
        let processed_dict = self.filter_manager.post_process_files(&patch_dict);
        let final_output = DiffParser::reconstruct_patch(&processed_dict);
        
        // Create output directory if it doesn't exist
        if let Some(parent) = Path::new(output_file).parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Write the processed diff to the output file
        fs::write(output_file, &final_output)?;
        
        // Calculate token count
        let token_count = self.token_counter.count_tokens(&final_output);
        
        Ok(token_count)
    }
    
    /// Get the default output file path in the temporary directory
    pub fn get_default_output_file() -> String {
        let temp_dir = std::env::temp_dir();
        let output_dir = temp_dir.join("repodiff");
        let output_file = output_dir.join("repodiff_output.txt");
        
        output_file.to_string_lossy().to_string()
    }
} 