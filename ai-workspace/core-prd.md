# Product Requirements Document (PRD) for RepoDiff: Single-Pass Diff + Post-Processing (Rust)

## 1. Overview

The goal is to **simplify** RepoDiff by performing a **single** `git diff` command that captures all changed files. Post-processing will then apply user-defined rules (like ignoring bodies of methods far from the changed lines). This approach eliminates the need for multiple Git calls or complex exclude patterns.

## 2. Key Objectives

1. Generate one **unified diff** that covers all changed files in a **single pass**.
2. **Post-process** that diff in Rust to apply user-defined patterns, supporting two main modes for C# (`*.cs`) files:
   - **API Signature Mode:** Focus on capturing the API surface. Include namespace, class, method, and property signatures for C# files. Method bodies are generally omitted or truncated (controlled by `method_body_threshold`). Useful for high-level API reviews. Configured using `api_signature_mode: true`.
   - **Change Context Mode:** Focus on providing detailed context for code changes within C# methods. Include the full body of the C# method containing a diff, along with surrounding namespace and class signatures and signatures of other methods within the `context_lines` range. Useful for in-depth code review of specific changes. Configured using `include_method_body: true` and `include_signatures: true`.
3. Provide **user-friendly** output with placeholders (e.g., `{ ... }`) to indicate omitted method bodies and instructional headers explaining these conventions.

## 3. Example Configuration
```json
{
  "tiktoken_model": "gpt-4o",
  "filters": [
    // Example of API Signature Mode (Rule 1)
    {
      "file_pattern": "*.cs",
      "api_signature_mode": true,
      "method_body_threshold": 10 // Include method bodies up to 10 lines, truncate longer ones
      // context_lines is less relevant in this mode, but can be included for minimal surrounding context
    },
    // Example of Change Context Mode (Rule 2)
    {
      "file_pattern": "*.cs",
      "include_method_body": true,
      "include_signatures": true,
      "context_lines": 3 // Provide context around the changed method and include nearby signatures
    },
    {
      "file_pattern": "*.py",
      "context_lines": 5 // Standard context lines for Python files
    }
  ]
}
```

## 4. Single-Pass Approach
1. **Collect All Changes**: Perform one `git diff` for the relevant commits or branches, capturing all changed files (e.g., `git diff commit1 commit2 --unified=999999`).
2. **Parse the Unified Diff**: Use Rust to parse each file's changed lines, hunks, and contextual lines.
3. **Apply Filters**: For each file, determine which config pattern applies and apply the appropriate rule.
4. **Finalize Output**: Reconstruct a single unified diff in memory or produce a custom output where method bodies are replaced with `{ ... }`.

## 5. Handling C# Files - Two Modes

This feature provides two distinct modes for handling C# (`*.cs`) files, selectable via configuration filters:

### 5.1 API Signature Mode (`api_signature_mode: true`) - Rule 1

* **Purpose:** To extract and present the API surface of C# files, focusing on signatures and structure.
* **Behavior:**
    1. **Signatures Included:** Always includes namespace, class, method, and property signatures.
    2. **Method Body Handling:**
       - Method bodies are generally truncated or omitted to emphasize signatures.
       - The `method_body_threshold` setting controls the maximum number of lines to include from a method body. If a body is shorter than or equal to the threshold, it's included in full. If longer, it's truncated, and the body is replaced with `{ ... }` after the initial lines (if any are kept).
    3. **`context_lines`:** If specified, `context_lines` provides a minimal amount of context *around* the namespace, class, and method signatures in the output. It might ensure at least `context_lines` are included around diff locations, potentially extending beyond just signatures.
    4. **Use Case:**  Ideal for high-level reviews, understanding API changes, or feeding LLMs that need to understand API structure without excessive implementation details.

### 5.2 Change Context Mode (`include_method_body: true`, `include_signatures: true`) - Rule 2

* **Purpose:** To provide comprehensive context for code changes within C# methods, enabling detailed code reviews.
* **Behavior:**
    1. **Full Changed Method Body:**  If a diff is found within a C# method, the *entire body* of that method is always included in the output, regardless of its length or the `context_lines` setting.
    2. **Namespace and Class Signatures:** The namespace and class declarations enclosing the changed method are always included to provide structural context.
    3. **Signatures of Other Methods (Contextual Inclusion):**
       - The `context_lines` setting determines the range of lines around the changed method that are considered "context."
       - If the *signature* of another method (within the same class) falls at least partially within this `context_lines` range, then that method's *signature* is included in the output.
       - The *body* of these other "contextual" methods is *replaced* with `{ ... }`.
    4. **`context_lines` Role:**  `context_lines` primarily serves to:
       - Define the context range around the changed method.
       - Determine which *other* method signatures are included for context.
       - Provide additional lines of code *outside* of methods (within the class or namespace) that fall within the context range.
       - **Crucially, `context_lines` does *not* truncate the body of the method containing the diff when `include_method_body: true`.**
    5. **Use Case:** Ideal for detailed code reviews where understanding the full context of a method change is crucial. Provides enough surrounding code (signatures and some contextual lines) to understand the change's impact and purpose.

## 6. Additional Instructions in Output
Include a **header** in the final diff explaining placeholders:

```
NOTE: Some method bodies have been replaced with "{ ... }" to improve clarity for code reviews and LLM analysis.

- In API Signature Mode (api_signature_mode: true), most method bodies are intentionally omitted or truncated, focusing on API signatures.
- In Change Context Mode (include_method_body: true), the entire method containing the diff is included to provide full context. Signatures of other methods within the context range are also included, but their bodies are replaced with "{ ... }".
```

## 7. Pseudocode (Rust)
```rust
fn generate_single_pass_diff(commit1: &str, commit2: &str) -> Result<String> {
    GitOperations::run_git_diff(commit1, commit2, 999999)
}

fn parse_unified_diff(diff_output: &str) -> Result<HashMap<String, Vec<Hunk>>> {
    DiffParser::parse(diff_output)
}

fn post_process_files(patch_dict: &mut HashMap<String, Vec<Hunk>>, config: &Config) {
    for (filename, hunks) in patch_dict.iter_mut() {
        let rule = config.find_matching_rule(filename);
        if rule.api_signature_mode {
            apply_api_signature_mode(hunks, rule.method_body_threshold);
        } else if rule.include_method_body && rule.include_signatures {
            apply_change_context_mode(hunks, rule.context_lines);
        } else {
            apply_context_filter(hunks, rule.context_lines);
        }
    }
}

fn main() -> Result<()> {
    let config = Config::load("config.json")?;
    let raw_diff = generate_single_pass_diff("commit1", "commit2")?;
    let mut patch_dict = parse_unified_diff(&raw_diff)?;
    post_process_files(&mut patch_dict, &config);
    let final_output = DiffParser::reconstruct_patch(&patch_dict);
    println!("{}", final_output);
    Ok(())
}
```

## 8. Performance
- **Single Git Command**: Efficient for 20-100 changed files.
- **In-Memory Post-Processing**: Minimal overhead, simple to maintain.