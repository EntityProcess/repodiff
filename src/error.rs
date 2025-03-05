use thiserror::Error;

/// Custom error types for the RepoDiff application
#[derive(Error, Debug)]
pub enum RepoDiffError {
    /// Error running git command
    #[error("Git error: {0}")]
    GitError(String),

    /// Error reading or writing files
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Error parsing JSON
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Error parsing regex
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    /// Error with tiktoken
    #[error("Tiktoken error: {0}")]
    TiktokenError(String),

    /// General error
    #[error("{0}")]
    GeneralError(String),
}

/// Result type for RepoDiff operations
pub type Result<T> = std::result::Result<T, RepoDiffError>; 