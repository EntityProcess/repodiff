# Product Requirements Document (PRD) for RepoDiff: Single-Pass Diff + Post-Processing (Rust)

## 1. Overview

RepoDiff generates a simplified and context-aware unified diff of a Git repository, designed for code reviews and Large Language Model (LLM) analysis. It performs a **single** `git diff` command to capture all changed files and then post-processes the output in Rust to apply user-defined rules. This approach avoids multiple Git calls and complex exclude patterns. The key feature is flexible handling of C# files, offering both detailed change context and API surface views.

## 2. Key Objectives

1.  **Single-Pass Diff:** Generate one unified diff covering all changed files in a single `git diff` operation.
2.  **Post-Processing (Rust):** Apply user-defined rules to the diff output, focusing on two primary modes for C# files (with a combined mode option):
    *   **Change Context Mode:** Prioritizes detailed context *around* code changes within C# methods. Includes the full body of changed methods, surrounding signatures, and contextual lines.
    *   **API Signature Mode:** Focuses on extracting the API surface (namespace, class, method, and property signatures) of C# files. Method bodies are generally truncated or omitted.
    *   **Combined Mode:** Layers API Signature Mode *on top of* Change Context Mode, providing detailed change context *and* a broader API overview.
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
      "context_lines": 3,
      "api_signature_mode": true, // Enables combined mode
      "api_signature_method_body_lines": 10 // Lines to keep in API mode & surrounding methods
    },
    {
      "file_pattern": "*.cs", //Example of pure Change Context Mode
      "include_method_body": true,
      "include_signatures": true,
      "context_lines": 5
    },
    {
      "file_pattern": "*.cs", // Example of pure API Signature Mode
      "api_signature_mode": true,
      "api_signature_method_body_lines": 5
    },
    {
      "file_pattern": "*.py",
      "context_lines": 5 // Standard context lines for Python files
    }
  ]
}
```

## 4. Single-Pass Approach

1.  **Collect All Changes:** Execute a single `git diff` command for the specified commits or branches (e.g., `git diff commit1 commit2 --unified=999999`).  The large `--unified` value ensures we capture sufficient context initially.
2.  **Parse the Unified Diff:** Use a Rust library (likely with Tree-sitter for C# parsing) to parse the diff output into a structured representation (e.g., a dictionary of files and hunks).
3.  **Apply Filters:** Iterate through the parsed files and hunks. For each file, determine the matching configuration filter and apply the corresponding rules (Change Context, API Signature, or Combined).
4.  **Reconstruct the Diff:** Generate the final, modified unified diff output. Adjust hunk headers and line numbers to reflect any changes made during post-processing (e.g., replacing method bodies with `{ ... }`).

## 5. Handling C# Files - Modes

This section details the different modes for processing C# (`*.cs`) files.

### 5.1 API Signature Mode (`api_signature_mode: true`, `api_signature_method_body_lines`: Optional)

*   **Purpose:** To extract and present the API surface of C# files, focusing on structure and signatures.  Implementation details are minimized.

*   **Behavior:**

    1.  **Signatures Included:** Always include the *declaration lines* for namespaces, classes, methods, and properties.
    2.  **Method Body Handling:**
        *   Method bodies are generally truncated or omitted.
        *   The `api_signature_method_body_lines` setting (if present) controls the maximum number of lines to include from the *beginning* of a method body.  If a body is shorter than or equal to this value, it's included. If longer, it's truncated, and the rest of the body is replaced with `{ ... }`. If `api_signature_method_body_lines` is not provided, then omit the method bodies entirely.
    3.  **`context_lines`:**  Provides a *minimal* amount of context *around* the included signatures. It might ensure that at least `context_lines` are present around diff locations. It does *not* significantly expand the output.

*   **Use Case:** Ideal for high-level API reviews, understanding structural changes, or for LLMs that need API information without implementation details.

### 5.2 Change Context Mode (`include_method_body: true`, `include_signatures: true`)

*   **Purpose:** To provide comprehensive context for code changes *within* C# methods, enabling detailed code reviews.

*   **Key Terms:**

    *   **Changed Method:** A method whose body (between `{` and `}`) contains at least one line from a diff hunk.
    *   **Diff Hunk:** A section of the unified diff output indicating a change (starts with `@@ ... @@`).
    *   **Method Signature:** The line declaring a method (e.g., `public int Foo(int x)`).
    *   **Context Range:** A range of lines around a changed method: `method_signature_start_line - context_lines` to `method_body_end_line + context_lines`.
    *   **Contextual Method:** A method *other than* the changed method whose signature falls within the context range.
    *   **Nested Structures:** Methods within methods (lambdas, local functions), or classes within classes.

*   **Behavior:**

    1.  **Identify Changed Methods:** Use a C# parser (Tree-sitter with `tree-sitter-c-sharp` is strongly recommended) to identify methods containing changed lines.
    2.  **Include Full Changed Method Body:** For *each* changed method, include its *entire* body in the output. Do *not* truncate it.
    3.  **Include Namespace and Class Signatures:** Include the *declaration lines* of the namespace and class that *directly* enclose the changed method. Do *not* include other members of the class/namespace unless they fall within the context range (see point 7).
    4.  **Calculate the Context Range:**  For each changed method, calculate the context range (see "Key Terms").
    5.  **Include Contextual Method Signatures:** If `include_signatures` is true, include the *signatures* of other methods within the same class whose signatures fall within the calculated context range. Replace the *bodies* of these contextual methods with `{ ... }`.
    6.  **Handle Nested Structures:**
        *   Nested methods/lambdas: Treat as part of the enclosing method's body.
        *   Nested classes: Treat as separate classes.
    7.  **Handle Code Outside of Methods (Within the Context Range) - Point 7 (See Below):**  Address "other code" (comments, using directives, etc.) as described in the dedicated section.
    8. **Output Format**: Ensure that the result remains a valid unified diff.

*   **`context_lines` Role:** Defines the range for including contextual method signatures and "other code" around changed methods. It does *not* affect the inclusion of the full changed method body.

*   **Use Case:** Ideal for in-depth code reviews where understanding the full context of a change is paramount.

### 5.3 Combined Mode (`api_signature_mode: true`, `include_method_body: true`, `include_signatures: true`, `api_signature_method_body_lines`: Optional)

*   **Purpose:** Combines the features of Change Context Mode and API Signature Mode to provide both detailed change context *and* a broader view of the API surface.

*   **Behavior:** This mode layers API Signature Mode *on top of* Change Context Mode.

    1.  **Changed Methods:** The `include_method_body: true` setting takes the highest precedence.  If a method is changed, its *entire* body is included.
    2.  **Context Range:** The `context_lines` setting defines the region around changed methods to identify "surrounding methods."
    3.  **Surrounding Methods (Within `context_lines`):**
        *   Include the *signatures* of these methods (due to `include_signatures: true`).
        *   Include up to `api_signature_method_body_lines` lines from the *beginning* of their bodies. Truncate longer bodies and replace the remainder with `{ ... }`.
    4.  **Non-Surrounding Methods (Outside `context_lines`):**
         *   Include the *signatures* of these methods (due to `include_signatures: true` and `api_signature_mode: true`).
         *   Include up to `api_signature_method_body_lines` lines from the *beginning* of their bodies. Truncate longer bodies and replace the remainder with `{ ... }`.
    5.   **Namespace and Class Declarations:** Include the declaration lines for namespaces and classes (due to `api_signature_mode: true`).
    6.  **"Other Code" (Point 7):** Handle code outside of methods based on `context_lines` and standard diff rules.

* **Precedence**
    1. `include_method_body`
    2. `context_lines`
    3. Surrounding Methods
    4. Non-Surrounding Methods
    5. `api_signature_mode`
    6. Other Code

### 5.4 Edge Cases (for all C# modes)

*   **Namespace Change:** Include the changed namespace declaration line and surrounding `context_lines`.
*   **Abstract Method:** Include the full signature as the "method body" and apply `context_lines` around it.
*   **Nested Class:** Treat a nested class as a separate class. If it has a changed method, include the method's body and the nested class's signature.

## 6. Handle Code Outside of Methods (Within the Context Range) - Point 7

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

- In API Signature Mode (api_signature_mode: true), most method bodies are intentionally omitted or truncated, focusing on API signatures.
- In Change Context Mode (include_method_body: true), the entire method containing the diff is included.
- In Combined Mode, the entire method containing the diff is included.  Signatures of other methods are also included. Bodies of methods outside the changed method are truncated based on api_signature_method_body_lines.
- The "{ ... }" placeholder indicates that a method body has been omitted or truncated.
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
                if rule.api_signature_mode && rule.include_method_body && rule.include_signatures {
                    apply_combined_mode(hunks, rule); // Combined mode for C#
                } else if rule.api_signature_mode {
                    apply_api_signature_mode(hunks, rule); // Pure API Signature Mode
                } else if rule.include_method_body && rule.include_signatures {
                    apply_change_context_mode(hunks, rule); // Pure Change Context Mode
                } else {
					apply_context_filter(hunks, rule.context_lines) // for other file types.
				}
            }
        }
    }
}
fn apply_combined_mode(hunks: &mut Vec<Hunk>, rule: &Rule) {
    // 1. Find changed methods (highest precedence)
    let changed_methods = find_changed_methods(hunks);

    // 2. Use a parser (Tree-sitter) to find *all* methods in the file
    let all_methods = find_all_methods(hunks); // Get all methods from the AST

	// Include namespace and class declarations
    include_namespace_and_class_declarations(hunks);

    for method in &all_methods {
        if changed_methods.contains(method) {
            // 3. Include full body for changed methods
            include_full_method_body(hunks, method);
        } else {
            // 4. Check if it's a "surrounding method"
            let mut is_surrounding = false;
            for changed_method in &changed_methods {
                let context_range = calculate_context_range(changed_method, rule.context_lines);
                if is_method_within_context_range(method, &context_range) {
                    is_surrounding = true;
                    break; // Once we find it's surrounding one changed method, no need to check others
                }
            }

            if is_surrounding {
                // 5. "Surrounding" method: Signature + Truncated Body
                include_method_signature(hunks, method);
                include_truncated_method_body(hunks, method, rule.api_signature_method_body_lines);
            } else {
                // 6. Not surrounding, but still include signature (api_signature_mode) and truncated body.
                include_method_signature(hunks, method);
                include_truncated_method_body(hunks, method, rule.api_signature_method_body_lines)
            }
        }
    }

    // 7. Handle "other code" (Point 7)
    handle_other_code(hunks, rule.context_lines);

     // 8. Adjust Hunk Headers: VERY IMPORTANT. After all modifications, adjust hunk headers.
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
fn include_method_signature(hunks: &mut Vec<Hunk>, method: &MethodInfo) { /* ... */ }
fn include_truncated_method_body(hunks: &mut Vec<Hunk>, method: &MethodInfo, max_lines: usize) { /* ... */ }
fn include_namespace_and_class_declarations(hunks: &mut Vec<Hunk>) { /* ... */ }
fn handle_other_code(hunks: &mut Vec<Hunk>, context_lines: usize) { /* ... */ }
fn adjust_hunk_headers(hunks: &mut Vec<Hunk>) {/* ... */ }

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


