use tiktoken_rs::CoreBPE;
use crate::error::{RepoDiffError, Result};

/// Handles token counting for LLM models using tiktoken
pub struct TokenCounter {
    /// The tiktoken encoding
    bpe: CoreBPE,
}

impl TokenCounter {
    /// Initialize the TokenCounter with a specific model
    ///
    /// # Arguments
    ///
    /// * `model` - The name of the LLM model to use for token counting
    pub fn new(model: &str) -> Result<Self> {
        let bpe = tiktoken_rs::get_bpe_from_model(model)
            .map_err(|e| RepoDiffError::TiktokenError(format!("Failed to get BPE for model {}: {}", model, e)))?;
        Ok(Self { bpe })
    }

    /// Count the number of tokens in the given text
    ///
    /// # Arguments
    ///
    /// * `text` - The text to count tokens for
    pub fn count_tokens(&self, text: &str) -> usize {
        self.bpe.encode_ordinary(text).len()
    }
} 