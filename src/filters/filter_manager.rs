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
            return self.apply_context_filter(hunks, rule.context_lines);
        }

        let file_info = self.csharp_parser.parse_file(code, hunks);
        let mut processed_hunks = Vec::new();

        for hunk in hunks {
            let mut new_hunk = hunk.clone();
            let mut new_lines = Vec::new();
            let mut last_included_line = hunk.new_start - 1;

            // Step 1: Compute context_lines_set and identify changed lines
            let mut context_lines_set = std::collections::HashSet::new();
            let mut change_locations = Vec::new();
            let mut temp_line = hunk.new_start;
            for line in &hunk.lines {
                if line.starts_with('+') || line.starts_with('-') {
                    change_locations.push(temp_line);
                    let start = temp_line.saturating_sub(rule.context_lines);
                    let end = temp_line + rule.context_lines;
                    for i in start..=end {
                        context_lines_set.insert(i);
                    }
                }
                if !line.starts_with('-') {
                    temp_line += 1;
                }
            }

            // Step 2: Identify changed and contextual methods
            let changed_methods: Vec<&CSharpMethod> = file_info.methods.iter()
                .filter(|m| m.has_changes)
                .collect();
            
            let contextual_methods: Vec<&CSharpMethod> = if rule.include_signatures {
                file_info.methods.iter()
                    .filter(|m| !m.has_changes && (
                        // Method signature or any part of body falls within context range
                        context_lines_set.contains(&m.signature_line) ||
                        (m.start_line..=m.end_line).any(|l| context_lines_set.contains(&l))
                    ))
                    .collect()
            } else {
                Vec::new()
            };

            // Step 3: Process each line
            let mut line_counter = hunk.new_start;
            for line in &hunk.lines {
                let is_changed_line = line.starts_with('+') || line.starts_with('-');
                let is_context_line = context_lines_set.contains(&line_counter);

                // Check method membership
                let in_changed_method = changed_methods.iter()
                    .find(|m| line_counter >= m.start_line && line_counter <= m.end_line);
                let in_contextual_method = contextual_methods.iter()
                    .find(|m| line_counter >= m.start_line && line_counter <= m.end_line);

                // Determine if line should be included
                let mut should_include = is_changed_line;
                let mut should_add_placeholder = false;

                if let Some(method) = in_changed_method {
                    // Changed method logic - preserve existing behavior
                    if rule.include_method_body {
                        should_include = true;
                    } else if line_counter == method.signature_line {
                        should_include = true;
                        should_add_placeholder = true;
                    }
                } else if let Some(method) = in_contextual_method {
                    // Contextual method logic - new behavior
                    if line_counter == method.signature_line {
                        should_include = true;
                    } else if line_counter > method.signature_line && line_counter <= method.end_line {
                        // For body lines, only include if within context range
                        should_include = is_context_line;
                        // Add placeholder if we're skipping lines
                        if !should_include && !new_lines.last().map_or(false, |l: &String| l.ends_with("⋮----")) {
                            should_add_placeholder = true;
                        }
                    }
                } else {
                    // Other code: include if in context range or part of enclosing declaration
                    let in_enclosing_declaration = {
                        let mut found = false;
                        for &(start, end) in file_info.namespace_declarations.iter().chain(file_info.class_declarations.iter()) {
                            if line_counter == start && changed_methods.iter().any(|m| m.start_line >= start && m.end_line <= end) {
                                found = true;
                                break;
                            }
                        }
                        found
                    };
                    should_include = is_context_line || (in_enclosing_declaration && rule.include_signatures);
                }

                // Include the line or placeholder
                if should_include {
                    new_lines.push(line.clone());
                    last_included_line = line_counter;
                } else if should_add_placeholder && line_counter > last_included_line + 1 {
                    new_lines.push(" ⋮----".to_string());
                    last_included_line = line_counter;
                }

                if !line.starts_with('-') {
                    line_counter += 1;
                }
            }

            // Update hunk with filtered lines
            new_hunk.lines = new_lines;
            new_hunk.new_count = new_hunk.lines.iter().filter(|l| !l.starts_with('-')).count();
            new_hunk.old_count = new_hunk.lines.iter().filter(|l| !l.starts_with('+')).count();

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
        let mut result = HashMap::new();
        
        for (file_path, hunks) in patch_dict {
            let rule = self.find_matching_rule(file_path);
            
            // Special handling for C# files
            if file_path.ends_with(".cs") && (rule.include_method_body || rule.include_signatures) {
                // TODO: Get the full file content from Git
                // For now, we'll reconstruct it from the hunks
                let code = self.reconstruct_file_content(hunks);
                result.insert(file_path.clone(), self.process_csharp_file(hunks, &rule, &code));
            } else {
                result.insert(file_path.clone(), self.apply_context_filter(hunks, rule.context_lines));
            }
        }
        
        result
    }

    /// Reconstruct file content from hunks (temporary solution)
    ///
    /// # Arguments
    ///
    /// * `hunks` - List of hunks containing the file changes
    fn reconstruct_file_content(&self, hunks: &[Hunk]) -> String {
        let mut content = String::new();
        for line in hunks.iter().flat_map(|h| &h.lines) {
            if line.starts_with('-') {
                continue;
            }
            if line.starts_with('+') {
                content.push_str(&line[1..]);
            } else {
                content.push_str(line);
            }
            content.push('\n');
        }
        content
    }
} 