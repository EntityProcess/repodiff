Okay, let's break down the suggested refactoring for `csharp_parser.rs` and `filter_manager.rs` based on the guidelines and analysis.

## Refactoring `csharp_parser.rs`

The primary area for improvement is the `find_nodes` function, which is long and complex. We can refactor it by extracting logic for each node type into separate, smaller functions.

**Refactoring Steps for `csharp_parser.rs`:**

1.  **Extract Node Handling Functions:** Create separate functions for handling each node type within `find_nodes`. This will make `find_nodes` shorter and the logic for each node type more isolated and easier to understand.

    *   `fn handle_method_declaration(node: Node, code: &str, file: &mut CSharpFile)`
    *   `fn handle_property_declaration(node: Node, code: &str, file: &mut CSharpFile)`
    *   `fn handle_using_directive(node: Node, code: &str, file: &mut CSharpFile)`
    *   `fn handle_namespace_declaration(node: Node, code: &str, file: &mut CSharpFile)`
    *   `fn handle_class_declaration(node: Node, code: &str, file: &mut CSharpFile)`

2.  **Simplify `find_nodes`:**  Update `find_nodes` to use these new handler functions within the `match` statement.  `find_nodes` will primarily be responsible for traversing the tree and dispatching to the appropriate handler.

3.  **Review and Simplify Property Declaration Handling:** The logic for properties (especially with accessors and arrow expressions) is complex. Within `handle_property_declaration`, consider further breaking down the accessor handling into helper functions if it remains too long or complex.

4.  **Consider Unifying Change Detection Functions:** Evaluate if `node_contains_changes` and `method_contains_changes` can be unified or if `node_contains_changes` is truly necessary. If their logic is very similar, consolidation would reduce code duplication. If they are distinct, ensure their purpose is clear and they are named appropriately.

5.  **Improve Clarity and Comments:** Review variable names within the new handler functions for clarity. Add comments to explain complex logic within these functions.


**Example of Refactored `find_nodes` and extracted handler (Conceptual - may need adjustments based on exact logic):**

```rust
impl CSharpParser {
    // ... (rest of CSharpParser implementation) ...

    fn find_nodes(&self, node: Node, code: &str, file: &mut CSharpFile) {
        match node.kind() {
            "method_declaration" => self.handle_method_declaration(node, code, file),
            "property_declaration" => self.handle_property_declaration(node, code, file),
            "using_directive" => self.handle_using_directive(node, file),
            "namespace_declaration" => self.handle_namespace_declaration(node, file),
            "class_declaration" => self.handle_class_declaration(node, file),
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.find_nodes(child, code, file);
        }
    }

    fn handle_method_declaration(&self, node: Node, code: &str, file: &mut CSharpFile) {
        let start_line = node.start_position().row + 1;
        let end_line = node.end_position().row + 1;

        // Find the signature line
        let signature_line = node.child_by_field_name("header")
            .map(|n| n.start_position().row + 1)
            .unwrap_or(start_line);

        let text = node.utf8_text(code.as_bytes())
            .unwrap_or_default()
            .to_string();

        file.methods.push(CSharpMethod {
            start_line,
            end_line,
            signature_line,
            text,
            has_changes: false,
        });
    }

    fn handle_property_declaration(&self, node: Node, code: &str, file: &mut CSharpFile) {
        // ... (Extracted property declaration handling logic from original find_nodes) ...
    }

    fn handle_using_directive(&self, node: Node, file: &mut CSharpFile) {
        let start_line = node.start_position().row + 1;
        let end_line = node.end_position().row + 1;
        file.using_statements.push((start_line, end_line));
    }

    fn handle_namespace_declaration(&self, node: Node, file: &mut CSharpFile) {
        let start_line = node.start_position().row + 1;
        let end_line = node.end_position().row + 1;
        file.namespace_declarations.push((start_line, end_line));
    }

    fn handle_class_declaration(&self, node: Node, file: &mut CSharpFile) {
        let start_line = node.start_position().row + 1;
        let end_line = node.end_position().row + 1;
        file.class_declarations.push((start_line, end_line));
    }

    // ... (rest of CSharpParser implementation) ...
}
```


## Refactoring `filter_manager.rs`

The `process_csharp_file` function is the most complex part of `filter_manager.rs`. We need to break it down into smaller, more manageable functions.

**Refactoring Steps for `filter_manager.rs`:**

1.  **Extract Hunk Processing Stages into Functions:**  Divide `process_csharp_file` into functions representing logical stages of processing.

    *   `fn calculate_context_lines_and_changes(hunk: &Hunk, rule: &FilterRule) -> (HashSet<usize>, Vec<usize>)`:  Extracts the logic to compute `context_lines_set` and `change_locations`.
    *   `fn identify_changed_and_contextual_methods(file_info: &CSharpFile, context_lines_set: &HashSet<usize>, rule: &FilterRule) -> (Vec<&CSharpMethod>, Vec<&CSharpMethod>)`: Extracts the logic to identify `changed_methods` and `contextual_methods`.
    *   `fn process_hunk_lines(hunk: &Hunk, file_info: &CSharpFile, context_lines_set: &HashSet<usize>, changed_methods: &[&CSharpMethod], contextual_methods: &[&CSharpMethod], rule: &FilterRule) -> Vec<String>`: Extracts the line-by-line processing logic within the hunk. This function will handle the conditional inclusion of lines based on method type (changed, contextual, other).

2.  **Simplify Logic within Extracted Functions:**  Within each extracted function (especially `process_hunk_lines`), review and simplify the conditional logic for determining `should_include` and `should_add_placeholder`.  Consider using helper functions for specific checks if the logic remains complex.

3.  **Improve Variable Naming:**  Ensure variable names in the extracted functions and `process_csharp_file` are descriptive and easy to understand.

4.  **Replace `reconstruct_file_content` (Future Enhancement):** As a separate, but important step, plan to replace `reconstruct_file_content` with a proper mechanism to retrieve file content from Git using `GitOperations`. This might involve adding a method to `GitOperations` to get file content at a specific commit. This is crucial for long-term correctness and reliability.

5.  **Consider C#-Specific Logic Module (Optional):** If `filter_manager.rs` becomes too large and complex due to the C#-specific filtering, think about moving the `process_csharp_file` function and related helper functions into a separate module (e.g., `csharp_filter.rs`) within the `filters` directory. This would further separate concerns.


**Example of Refactored `process_csharp_file` (Conceptual - may need adjustments):**

```rust
impl FilterManager {
    // ... (rest of FilterManager implementation) ...

    fn process_csharp_file(&mut self, hunks: &[Hunk], rule: &FilterRule, code: &str) -> Vec<Hunk> {
        if !rule.include_method_body && !rule.include_signatures {
            return self.apply_context_filter(hunks, rule.context_lines);
        }

        let file_info = self.csharp_parser.parse_file(code, hunks);
        let mut processed_hunks = Vec::new();

        for hunk in hunks {
            let mut new_hunk = hunk.clone();

            let (context_lines_set, _change_locations) = self.calculate_context_lines_and_changes(hunk, rule);
            let (changed_methods, contextual_methods) = self.identify_changed_and_contextual_methods(&file_info, &context_lines_set, rule);
            let new_lines = self.process_hunk_lines(hunk, &file_info, &context_lines_set, &changed_methods, &contextual_methods, rule);

            new_hunk.lines = new_lines;
            new_hunk.new_count = new_hunk.lines.iter().filter(|l| !l.starts_with('-')).count();
            new_hunk.old_count = new_hunk.lines.iter().filter(|l| !l.starts_with('+')).count();

            if !new_hunk.lines.is_empty() {
                processed_hunks.push(new_hunk);
            }
        }

        processed_hunks
    }

    fn calculate_context_lines_and_changes(&self, hunk: &Hunk, rule: &FilterRule) -> (HashSet<usize>, Vec<usize>) {
        // ... (Extracted context lines and changes calculation logic from original process_csharp_file) ...
    }

    fn identify_changed_and_contextual_methods(&self, file_info: &CSharpFile, context_lines_set: &HashSet<usize>, rule: &FilterRule) -> (Vec<&CSharpMethod>, Vec<&CSharpMethod>) {
        // ... (Extracted method identification logic from original process_csharp_file) ...
    }

    fn process_hunk_lines(&self, hunk: &Hunk, file_info: &CSharpFile, context_lines_set: &HashSet<usize>, changed_methods: &[&CSharpMethod], contextual_methods: &[&CSharpMethod], rule: &FilterRule) -> Vec<String> {
        // ... (Extracted line processing logic from original process_csharp_file) ...
    }

    // ... (rest of FilterManager implementation) ...
}
```


**Testing:**

After each refactoring step, especially after breaking down functions, ensure you run the existing tests to check for regressions.  You should also add new unit tests specifically targeting the refactored functions and covering different scenarios of C# filtering (various combinations of `include_method_body`, `include_signatures`, and different code structures in C# files). This is crucial to ensure the refactoring maintains the original functionality and improves code quality.

By following these steps, you'll make the `csharp_parser.rs` and `filter_manager.rs` code more modular, easier to understand, and more maintainable, aligning with the principles of "Building Maintainable Software". Remember to proceed incrementally, testing after each step to ensure you don't introduce regressions.