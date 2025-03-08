use std::collections::HashMap;
use fnmatch_regex::glob_to_regex;
use crate::utils::config_manager::FilterRule;
use crate::utils::diff_parser::Hunk;
use crate::filters::csharp_parser::CSharpParser;

/// Manages file pattern filters for controlling context lines in git diffs
pub struct FilterManager {
    /// List of filter rules
    filters: Vec<FilterRule>,
    /// C# parser
    csharp_parser: CSharpParser,
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
                include_method_body: false,
                include_signatures: false,
            }]
        } else {
            filters.to_vec()
        };
        
        FilterManager { 
            filters,
            csharp_parser: CSharpParser::new(),
        }
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
            include_method_body: false,
            include_signatures: false,
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
    
    /// Process C# file with method-aware filtering
    ///
    /// # Arguments
    ///
    /// * `hunks` - List of hunk dictionaries containing diff information
    /// * `rule` - The filter rule to apply
    /// * `code` - The full C# file content
    fn process_csharp_file(&mut self, hunks: &[Hunk], rule: &FilterRule, code: &str) -> Vec<Hunk> {
        if !rule.include_method_body && !rule.include_signatures {
            // If neither C# specific option is enabled, fall back to standard context filtering
            return self.apply_context_filter(hunks, rule.context_lines);
        }

        let methods = self.csharp_parser.parse_methods(code, hunks);
        let mut processed_hunks = Vec::new();

        for hunk in hunks {
            let mut new_hunk = hunk.clone();
            let mut new_lines = Vec::new();
            let mut current_line = hunk.new_start;

            for line in &hunk.lines {
                let should_include = if line.starts_with('+') || line.starts_with('-') {
                    // Always include changed lines
                    true
                } else {
                    // For context lines, check if they're part of a method we want to keep
                    let in_changed_method = methods.iter()
                        .any(|m| m.has_changes && 
                             current_line >= m.start_line && 
                             current_line <= m.end_line);

                    let in_context_range = methods.iter()
                        .any(|m| {
                            let context_start = m.start_line.saturating_sub(rule.context_lines);
                            let context_end = m.end_line + rule.context_lines;
                            current_line >= context_start && current_line <= context_end
                        });

                    let is_signature = methods.iter()
                        .any(|m| current_line == m.signature_line);

                    (rule.include_method_body && in_changed_method) ||
                    (rule.include_signatures && is_signature) ||
                    (!in_changed_method && in_context_range)
                };

                if should_include {
                    new_lines.push(line.clone());
                }
                
                if !line.starts_with('-') {
                    current_line += 1;
                }
            }

            // Update hunk with filtered lines
            new_hunk.lines = new_lines;
            new_hunk.new_count = new_hunk.lines.iter()
                .filter(|l| !l.starts_with('-'))
                .count();
            new_hunk.old_count = new_hunk.lines.iter()
                .filter(|l| !l.starts_with('+'))
                .count();

            if !new_hunk.lines.is_empty() {
                processed_hunks.push(new_hunk);
            }
        }

        processed_hunks
    }

    /// Post-process files according to their matching filter rules
    ///
    /// # Arguments
    ///
    /// * `patch_dict` - Dictionary mapping filenames to lists of hunks
    pub fn post_process_files(&mut self, patch_dict: &HashMap<String, Vec<Hunk>>) -> HashMap<String, Vec<Hunk>> {
        let mut processed_dict = HashMap::new();
        
        for (filename, hunks) in patch_dict {
            let rule = self.find_matching_rule(filename);
            
            // Special handling for C# files
            if filename.ends_with(".cs") && (rule.include_method_body || rule.include_signatures) {
                // TODO: Get the full file content from Git
                // For now, we'll reconstruct it from the hunks
                let code = self.reconstruct_file_content(hunks);
                processed_dict.insert(filename.clone(), self.process_csharp_file(hunks, &rule, &code));
            } else {
                processed_dict.insert(filename.clone(), self.apply_context_filter(hunks, rule.context_lines));
            }
        }
        
        processed_dict
    }

    /// Reconstruct file content from hunks (temporary solution)
    ///
    /// # Arguments
    ///
    /// * `hunks` - List of hunks containing the file changes
    fn reconstruct_file_content(&self, hunks: &[Hunk]) -> String {
        let mut content = String::new();
        let mut current_line = 1;

        for hunk in hunks {
            // Add any missing lines between hunks as empty lines
            while current_line < hunk.new_start {
                content.push_str("\n");
                current_line += 1;
            }

            for line in &hunk.lines {
                if !line.starts_with('-') {
                    content.push_str(&line[1..]);  // Skip the first character (space or +)
                    content.push('\n');
                    current_line += 1;
                }
            }
        }

        content
    }
} 