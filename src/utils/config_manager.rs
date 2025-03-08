use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::error::Result;

/// Filter rule for controlling context lines in git diffs
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilterRule {
    /// File pattern to match (glob pattern)
    pub file_pattern: String,
    /// Number of context lines to keep around changes
    pub context_lines: usize,
    /// Whether to include the full method body for changed methods (C# only)
    #[serde(default)]
    pub include_method_body: bool,
    /// Whether to include method signatures within context range (C# only)
    #[serde(default)]
    pub include_signatures: bool,
}

/// Configuration for the RepoDiff tool
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// The tiktoken model to use for token counting
    pub tiktoken_model: String,
    /// List of filter rules
    pub filters: Vec<FilterRule>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            tiktoken_model: "gpt-4o".to_string(),
            filters: vec![FilterRule {
                file_pattern: "*".to_string(),
                context_lines: 3,
                include_method_body: false,
                include_signatures: false,
            }],
        }
    }
}

/// Manages configuration loading and access for the RepoDiff tool
pub struct ConfigManager {
    config: Config,
}

impl ConfigManager {
    /// Initialize the ConfigManager with a specific config file
    ///
    /// # Arguments
    ///
    /// * `config_file_name` - The name of the configuration file to load
    pub fn new(config_file_name: &str) -> Result<Self> {
        let config = Self::load_config(config_file_name)?;
        Ok(ConfigManager { config })
    }

    /// Load configuration from the config file
    ///
    /// # Arguments
    ///
    /// * `config_file_name` - The name of the configuration file to load
    fn load_config(config_file_name: &str) -> Result<Config> {
        let config_path = Self::find_config_path(config_file_name)?;
        
        // Return default config if file doesn't exist
        if !config_path.exists() {
            return Ok(Config::default());
        }
        
        let config_str = fs::read_to_string(&config_path)?;
        let config: Config = serde_json::from_str(&config_str)?;
        
        Ok(config)
    }

    /// Find the path to the config file
    ///
    /// # Arguments
    ///
    /// * `config_file_name` - The name of the configuration file to find
    fn find_config_path(config_file_name: &str) -> Result<PathBuf> {
        // First, try the current directory
        let current_dir = std::env::current_dir()?;
        let config_path = current_dir.join(config_file_name);
        
        if config_path.exists() {
            return Ok(config_path);
        }
        
        // Then try the executable directory
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let config_path = exe_dir.join(config_file_name);
                if config_path.exists() {
                    return Ok(config_path);
                }
            }
        }
        
        // Return the current directory path
        Ok(current_dir.join(config_file_name))
    }

    /// Get the tiktoken model from the configuration
    pub fn get_tiktoken_model(&self) -> &str {
        &self.config.tiktoken_model
    }

    /// Get the filters from the configuration
    pub fn get_filters(&self) -> &[FilterRule] {
        &self.config.filters
    }
} 