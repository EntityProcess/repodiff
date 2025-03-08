Okay, here's the revised PRD with `api_signature_mode` and `api_signature_method_body_lines` removed, and the logic streamlined to use only `include_method_body`, `include_signatures`, and `context_lines`.  I've also adjusted the pseudocode and explanations to reflect these changes.

--- START OF FILE core-prd-revised.txt ---

# Product Requirements Document (PRD) for RepoDiff: Single-Pass Diff + Post-Processing (Rust)

## 1. Overview

RepoDiff generates a simplified and context-aware unified diff of a Git repository, designed for code reviews and Large Language Model (LLM) analysis. It performs a **single** `git diff` command to capture all changed files and then post-processes the output in Rust to apply user-defined rules. This approach avoids multiple Git calls and complex exclude patterns.  The key feature is flexible handling of C# files, offering control over the inclusion of method bodies and surrounding signatures.

## 2. Key Objectives

1.  **Single-Pass Diff:** Generate one unified diff covering all changed files in a single `git diff` operation.
2.  **Post-Processing (Rust):** Apply user-defined rules to the diff output, focusing on configurable handling of C# files:
    *   **Include Method Body:**  Controls whether the *entire* body of a changed method is included.
    *   **Include Signatures:** Controls whether the signatures of methods surrounding a changed method are included (within the defined context lines).
    * **Context Lines** Defines the context for including the surrounding signatures.
3.  **User-Friendly Output:** Generate a valid unified diff with clear placeholders (e.g., `{ ... }`) indicating omitted code sections and an instructional header explaining the conventions.
4. **Configurable**: Allow users to set the config via a json file.

## 3. Example Configuration

```json
{
  "tiktoken_model": "gpt-4o",
  "filters": [
    {
      "file_pattern": "*.cs",
      "include_method_body": true,
      "include_signatures": true,
      "context_lines": 3
    },
    {
      "file_pattern": "*.cs",
      "include_method_body": false,
      "include_signatures": false, // Only show the diff hunks.
      "context_lines": 0,
    },
    {
      "file_pattern": "*.py",
      "context_lines": 5,  // Standard context lines for Python files
      "include_signatures": false //Include signatures is not applicable for non .cs files.
    }
  ]
}
```

## 4. Single-Pass Approach

1.  **Collect All Changes:** Execute a single `git diff` command for the specified commits or branches (e.g., `git diff commit1 commit2 --unified=999999`).  The large `--unified` value ensures we capture sufficient context initially.
2.  **Parse the Unified Diff:** Use a Rust library (likely with Tree-sitter for C# parsing) to parse the diff output into a structured representation (e.g., a dictionary of files and hunks).
3.  **Apply Filters:** Iterate through the parsed files and hunks. For each file, determine the matching configuration filter and apply the corresponding rules.
4.  **Reconstruct the Diff:** Generate the final, modified unified diff output. Adjust hunk headers and line numbers to reflect any changes made during post-processing (e.g., replacing method bodies with `{ ... }`).

## 5. Handling C# Files

This section details the handling of C# (`*.cs`) files.

*   **Key Terms:**

    *   **Changed Method:** A method whose body (between `{` and `}`) contains at least one line from a diff hunk.
    *   **Diff Hunk:** A section of the unified diff output indicating a change (starts with `@@ ... @@`).
    *   **Method Signature:** The line declaring a method (e.g., `public int Foo(int x)`).
    *   **Context Range:** A range of lines around a changed method: `method_signature_start_line - context_lines` to `method_body_end_line + context_lines`.
    *   **Contextual Method:** A method *other than* the changed method whose signature falls within the context range.
    *   **Nested Structures:** Methods within methods (lambdas, local functions), or classes within classes.

*   **Configuration Options:**

    *   `include_method_body`:  If `true`, the *entire* body of a *changed* method is included in the output. If `false`, the changed method's body is replaced with `{ ... }`.
    *   `include_signatures`: If `true`, the *signatures* of contextual methods (methods within the context range of a changed method) are included.  The *bodies* of these contextual methods are always replaced with `{ ... }`.
    *    `context_lines`: Defines the number of lines before and after a changed method's signature and closing brace to consider for including contextual method signatures and "other code" (see Section 6).

*   **Behavior:**

    1.  **Identify Changed Methods:** Use a C# parser (Tree-sitter with `tree-sitter-c-sharp` is strongly recommended) to identify methods containing changed lines.
    2.  **Handle Changed Method Body:**
        *   If `include_method_body` is `true`, include the *entire* body of each changed method.
        *   If `include_method_body` is `false`, replace the *entire* body of each changed method with `{ ... }`.
    3.  **Include Namespace and Class Signatures:** Include the *declaration lines* of the namespace and class that *directly* enclose the changed method.
    4.  **Calculate the Context Range:** For each changed method, calculate the context range.
    5.  **Include Contextual Method Signatures (if `include_signatures` is true):**
        *   If `include_signatures` is `true`, include the *signatures* of contextual methods (those whose signatures fall within the context range).
        *   Replace the *bodies* of these contextual methods with `{ ... }`.
    6.  **Handle Nested Structures:**
        *   Nested methods/lambdas: Treat as part of the enclosing method's body.
        *   Nested classes: Treat as separate classes.
    7.  **Handle Code Outside of Methods (Within the Context Range):** Address "other code" (comments, using directives, etc.) as described in the dedicated section (Section 6).
    8. **Output Format**: Ensure that the result remains a valid unified diff.

*   **Use Cases:**

    *   **Detailed Code Review:**  Set `include_method_body: true` and `include_signatures: true` to get the full context of changes and surrounding methods.
    *   **High-Level Overview:** Set `include_method_body: false` and `include_signatures: true` to focus on structural changes and API signatures.
    *   **Minimal Diff:** Set `include_method_body: false` and `include_signatures: false` to see only the changed lines themselves, with minimal context.

## 6. Handle Code Outside of Methods (Within the Context Range)

*   **Purpose:** This rule describes how to handle code that's *not* part of a method body and *not* the main namespace/class declaration (e.g., comments, `using` directives, field declarations).

*   **How it Works:**

    1.  **Identify "Other Code":** Any lines *not* within a method's `{` and `}` and *not* namespace/class declaration lines.
    2.  **Check Context Range:** Determine if the line number of the "other code" falls within the context range of a *changed method*.
    3.  **Apply Standard Diff Context Rules:**
        *   **Changed Lines:** Include if changed (marked with `+` or `-`).
        *   **Unchanged Lines (Within Context):** Include as a context line (prefixed with a space) if within the context range.
        *   **Unchanged Lines (Outside Context):** Omit.

## 7. Additional Instructions in Output

Include a header in the final diff explaining placeholders:

```
NOTE: Some method bodies have been replaced with "{ ... }" to improve clarity for code reviews and LLM analysis.

- The "{ ... }" placeholder indicates that a method body has been omitted or truncated based on the configuration settings.
- include_method_body: true, will cause the entire method to be included.
- include_signatures: true, will include the signatures of methods that surround the changed method.
```

## 8. Pseudocode (Rust - Illustrative)

```rust
// Simplified and illustrative - not a complete implementation

fn generate_single_pass_diff(commit1: &str, commit2: &str) -> Result<String> {
    GitOperations::run_git_diff(commit1, commit2, 999999) // Large unified context
}

fn parse_unified_diff(diff_output: &str) -> Result<HashMap<String, Vec<Hunk>>> {
    DiffParser::parse(diff_output) // Parses into a structured representation
}

fn post_process_files(patch_dict: &mut HashMap<String, Vec<Hunk>>, config: &Config) {
    for (filename, hunks) in patch_dict.iter_mut() {
        if let Some(rule) = config.find_matching_rule(filename) {
            if rule.file_pattern.ends_with(".cs") {
                apply_csharp_rules(hunks, rule);
            } else {
                apply_context_filter(hunks, rule.context_lines); // For other file types
            }
        }
    }
}

fn apply_csharp_rules(hunks: &mut Vec<Hunk>, rule: &Rule) {
    // 1. Find changed methods
    let changed_methods = find_changed_methods(hunks);

    // 2. Use a parser (Tree-sitter) to find *all* methods in the file
    let all_methods = find_all_methods(hunks);

	// Include namespace and class declarations
    include_namespace_and_class_declarations(hunks);

    for method in &all_methods {
        if changed_methods.contains(method) {
            // 3. Handle changed method body based on include_method_body
            if rule.include_method_body {
                include_full_method_body(hunks, method);
            } else {
                replace_method_body_with_placeholder(hunks, method);
            }
        } else if rule.include_signatures {
           // Check if this is a contextual method
            let mut is_contextual = false;
            for changed_method in &changed_methods{
                let context_range = calculate_context_range(changed_method, rule.context_lines);
                if is_method_within_context_range(method, &context_range) {
                    is_contextual = true;
                    break;
                }
            }
            if is_contextual {
                include_method_signature(hunks, method);
                replace_method_body_with_placeholder(hunks, method); // Always replace body for contextual methods
            }
        }
    }

    // 4. Handle "other code" (Point 7)
    handle_other_code(hunks, rule.context_lines);

    // 5. Adjust Hunk Headers: VERY IMPORTANT. After all modifications, adjust hunk headers.
    adjust_hunk_headers(hunks);
}

fn main() -> Result<()> {
    let config = Config::load("config.json")?;
    let raw_diff = generate_single_pass_diff("commit1", "commit2")?;
    let mut patch_dict = parse_unified_diff(&raw_diff)?;
    post_process_files(&mut patch_dict, &config);

    // Reconstruct the final unified diff output
    let final_output = DiffParser::reconstruct_patch(&patch_dict);
    println!("{}", final_output);
    Ok(())
}

// Helper functions (placeholders - would need full implementation)
fn find_changed_methods(hunks: &Vec<Hunk>) -> Vec<MethodInfo> { /* ... */ }
fn find_all_methods(hunks: &Vec<Hunk>) -> Vec<MethodInfo>  { /* ... */ } // Uses parser
fn calculate_context_range(method: &MethodInfo, context_lines: usize) -> (usize, usize) { /* ... */ }
fn is_method_within_context_range(method: &MethodInfo, range: &(usize, usize)) -> bool { /* ... */ }
fn include_full_method_body(hunks: &mut Vec<Hunk>, method: &MethodInfo) { /* ... */ }
fn replace_method_body_with_placeholder(hunks: &mut Vec<Hunk>, method: &MethodInfo) { /* ... */ }
fn include_method_signature(hunks: &mut Vec<Hunk>, method: &MethodInfo) { /* ... */ }
fn include_namespace_and_class_declarations(hunks: &mut Vec<Hunk>) { /* ... */ }
fn handle_other_code(hunks: &mut Vec<Hunk>, context_lines: usize) { /* ... */ }
fn adjust_hunk_headers(hunks: &mut Vec<Hunk>) {/* ... */ }
fn apply_context_filter(hunks: &mut Vec<Hunk>, context_lines: usize) {/* ... */ } //Helper for non .cs files

// Struct to represent method information (example)
struct MethodInfo {
    start_line: usize,
    end_line: usize,
    signature_line: usize,
    name: String,
    // ... other relevant data ...
}
```

## 9. Performance

*   **Single Git Command:** Efficient for a moderate number of changed files (e.g., 20-100).
*   **In-Memory Post-Processing:** The Rust implementation should be performant, with the most significant overhead likely coming from parsing (especially with Tree-sitter).  Efficient data structures and algorithms should be used.