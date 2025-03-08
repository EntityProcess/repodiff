use std::collections::HashMap;
use fnmatch_regex::glob_to_regex;
use crate::utils::config_manager::FilterRule;
use crate::utils::diff_parser::Hunk;
use crate::filters::csharp_parser::{CSharpParser, CSharpMethod};

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

        let file_info = self.csharp_parser.parse_file(code, hunks);
        let mut processed_hunks = Vec::new();

        // Pre-compute which nodes have changes to avoid repeated checks
        let changed_using_statements: Vec<_> = file_info.using_statements.iter()
            .filter(|(start, end)| self.csharp_parser.node_contains_changes(*start, *end, hunks))
            .collect();

        let changed_class_declarations: Vec<_> = file_info.class_declarations.iter()
            .filter(|(start, end)| self.csharp_parser.node_contains_changes(*start, *end, hunks))
            .collect();

        // Pre-compute context ranges for changed nodes
        let mut context_ranges = Vec::new();
        
        // Add context ranges for using statements
        for &(start, end) in &changed_using_statements {
            let context_start = start.saturating_sub(rule.context_lines);
            let context_end = end + rule.context_lines;
            context_ranges.push((context_start, context_end));
        }

        // Add context ranges for class declarations
        for &(start, end) in &changed_class_declarations {
            let context_start = start.saturating_sub(rule.context_lines);
            let context_end = end + rule.context_lines;
            context_ranges.push((context_start, context_end));
        }

        for hunk in hunks {
            let mut new_hunk = hunk.clone();
            let mut new_lines = Vec::new();
            let mut current_line = hunk.new_start;

            // Find changed methods in this hunk
            let mut changed_methods = Vec::new();
            if rule.include_method_body {
                let mut temp_line = current_line;
                for line in &hunk.lines {
                    if line.starts_with('+') || line.starts_with('-') {
                        // Find any methods that contain this line
                        for method in &file_info.methods {
                            if temp_line >= method.start_line && temp_line <= method.end_line {
                                if !changed_methods.contains(&method) {
                                    changed_methods.push(method);
                                    // Add context range for this method
                                    let context_start = method.start_line.saturating_sub(rule.context_lines);
                                    let context_end = method.end_line + rule.context_lines;
                                    context_ranges.push((context_start, context_end));
                                }
                            }
                        }
                    }
                    if !line.starts_with('-') {
                        temp_line += 1;
                    }
                }
            }

            // Now process each line
            for line in &hunk.lines {
                let mut should_include = false;

                // Always include changed lines
                if line.starts_with('+') || line.starts_with('-') {
                    should_include = true;
                } else {
                    // For context lines, check various conditions
                    
                    // Check if line is part of a changed using statement
                    let in_changed_using = changed_using_statements.iter()
                        .any(|(start, end)| current_line >= *start && current_line <= *end);

                    // Check if line is part of a changed class declaration
                    let in_changed_class_decl = changed_class_declarations.iter()
                        .any(|(start, end)| current_line >= *start && current_line <= *end);

                    // Check if line is part of a changed method
                    let in_changed_method = changed_methods.iter()
                        .any(|m| current_line >= m.start_line && current_line <= m.end_line);

                    // Check if line is a method signature
                    let is_signature = if rule.include_signatures {
                        file_info.methods.iter()
                            .any(|m| current_line == m.signature_line)
                    } else {
                        false
                    };

                    // Check if line is within any context range
                    let in_context_range = context_ranges.iter()
                        .any(|(start, end)| current_line >= *start && current_line <= *end);

                    should_include = in_changed_using || 
                                   in_changed_class_decl || 
                                   in_changed_method || 
                                   is_signature || 
                                   in_context_range;
                }

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