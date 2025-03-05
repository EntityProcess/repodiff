use std::collections::HashMap;
use fnmatch_regex::glob_to_regex;
use crate::utils::config_manager::FilterRule;
use crate::utils::diff_parser::Hunk;

/// Manages file pattern filters for controlling context lines in git diffs
pub struct FilterManager {
    /// List of filter rules
    filters: Vec<FilterRule>,
}

impl FilterManager {
    /// Initialize the FilterManager with a list of filter rules
    ///
    /// # Arguments
    ///
    /// * `filters` - List of filter dictionaries with 'file_pattern' and 'context_lines' keys
    pub fn new(filters: &[FilterRule]) -> Self {
        let filters = if filters.is_empty() {
            vec![FilterRule {
                file_pattern: "*".to_string(),
                context_lines: 3,
            }]
        } else {
            filters.to_vec()
        };
        
        FilterManager { filters }
    }
    
    /// Find the first matching filter rule for a filename
    ///
    /// # Arguments
    ///
    /// * `filename` - The filename to match against filter patterns
    fn find_matching_rule(&self, filename: &str) -> FilterRule {
        for filter_rule in &self.filters {
            if let Ok(pattern) = glob_to_regex(&filter_rule.file_pattern) {
                if pattern.is_match(filename) {
                    return filter_rule.clone();
                }
            }
        }
        
        // Default rule
        FilterRule {
            file_pattern: "*".to_string(),
            context_lines: 3,
        }
    }
    
    /// Adjust the context lines in hunks to match the specified number
    ///
    /// # Arguments
    ///
    /// * `hunks` - List of hunk dictionaries containing diff information
    /// * `context_lines` - Number of context lines to keep around changes
    fn apply_context_filter(&self, hunks: &[Hunk], context_lines: usize) -> Vec<Hunk> {
        let mut filtered_hunks = Vec::new();
        
        for hunk in hunks {
            let lines = &hunk.lines;
            let mut filtered_lines = Vec::new();
            let mut change_indices = Vec::new();
            
            // First, find all the changed lines (+ or -)
            for (i, line) in lines.iter().enumerate() {
                if line.starts_with('+') || line.starts_with('-') {
                    change_indices.push(i);
                }
            }
            
            if change_indices.is_empty() {
                continue;
            }
            
            // Now determine which context lines to keep
            let mut lines_to_keep = std::collections::HashSet::new();
            for &change_idx in &change_indices {
                // Add the changed line
                lines_to_keep.insert(change_idx);
                // Add context lines before
                for i in change_idx.saturating_sub(context_lines)..change_idx {
                    lines_to_keep.insert(i);
                }
                // Add context lines after
                for i in change_idx + 1..std::cmp::min(lines.len(), change_idx + context_lines + 1) {
                    lines_to_keep.insert(i);
                }
            }
            
            // Keep lines in their original order
            for (i, line) in lines.iter().enumerate() {
                if lines_to_keep.contains(&i) {
                    filtered_lines.push(line.clone());
                }
            }
            
            if !filtered_lines.is_empty() {
                // Create a new hunk with all metadata preserved
                let mut new_hunk = hunk.clone();
                new_hunk.lines = filtered_lines;
                filtered_hunks.push(new_hunk);
            }
        }
        
        filtered_hunks
    }
    
    /// Apply post-processing filters to each file in the patch
    ///
    /// # Arguments
    ///
    /// * `patch_dict` - Dictionary mapping filenames to lists of hunks
    pub fn post_process_files(&self, patch_dict: &HashMap<String, Vec<Hunk>>) -> HashMap<String, Vec<Hunk>> {
        let mut processed_dict = HashMap::new();
        
        for (filename, hunks) in patch_dict {
            let rule = self.find_matching_rule(filename);
            
            // Check if this is a renamed file
            let is_rename = hunks.iter().any(|hunk| hunk.is_rename);
            
            // Apply context line filtering
            let context_lines = rule.context_lines;
            let processed_hunks = self.apply_context_filter(hunks, context_lines);
            
            // Preserve rename information in processed hunks
            if is_rename && !processed_hunks.is_empty() && !hunks.is_empty() {
                // We already preserve rename info when cloning hunks
                processed_dict.insert(filename.clone(), processed_hunks);
            } else {
                processed_dict.insert(filename.clone(), processed_hunks);
            }
        }
        
        processed_dict
    }
} 