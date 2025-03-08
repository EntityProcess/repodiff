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

*   **Purpose:** To provide comprehensive context for code changes within C# methods, enabling detailed code reviews. This mode shows the *entire* body of any method containing a change, along with surrounding structural information.

*   **Key Terms:**

    *   **Changed Method:** A method whose body (the code between the opening `{` and closing `}`) contains at least one line that appears in a diff hunk.
    *   **Diff Hunk:** A section of the unified diff output that indicates a change.  It starts with a line like `@@ -10,5 +10,6 @@` and includes lines that were added, removed, or are context lines.
    *   **Method Signature:** The line that declares a method, including access modifiers (e.g., `public`, `private`), return type, method name, and parameters.  Example: `public int CalculateSum(int a, int b)`.
    *   **Context Range:**  A range of lines calculated around a changed method.  It's used to determine which other method signatures (and potentially other lines of code) should be included for context.
    *  **Contextual Method:**  A method *other than* the changed method, whose signature falls within the context range of the changed method.
    *   **Nested Structures:**  Structures like methods within methods (lambdas or local functions), or classes within classes.

*   **Behavior (Step-by-Step Instructions):**

    1.  **Identify Changed Methods:**
        *   Parse the unified diff output (from `git diff`) to find all diff hunks.
        *   For each diff hunk, examine the changed lines (those starting with `+` or `-`).
        *   Use a C# parser (strongly recommended: Tree-sitter with the `tree-sitter-c-sharp` grammar) to determine if these changed lines fall within the body of a C# method.  If a parser is unavailable, approximate method starts by looking for lines matching patterns like `[access_modifier] [return_type] method_name {`.  This fallback is less reliable.  Focus on *structural* parsing (identifying namespaces, classes, methods, and their boundaries); you don't need to fully parse the *contents* of method bodies for this mode.
        *   If a diff hunk touches a method's body, that method is a "changed method."

    2.  **Include Full Changed Method Body:**
        *   For *each* changed method:
            *   Use the parser to find the *entire* body of the method (from the opening `{` to the closing `}`).
            *   Include the *entire* method body in the output, *regardless* of the `context_lines` setting.  Do *not* truncate it.
            *   If a method contains *multiple* diff hunks, include the full body *only once*.

    3.  **Include Namespace and Class Signatures:**
        *   Use the parser to find the namespace and class *declaration lines* that *directly* enclose the changed method. This means the lines that *declare* the namespace and class, typically starting with `namespace` and `class` (or `struct`, `interface`, `enum`, etc., if applicable).
        *   Include *only these declaration lines* in the output.  You *do not* need to include other members of the class or namespace (such as fields, properties, or methods that are *not* within the calculated context range of the changed method).  We are only interested in the lines that *define* the enclosing namespace and class.

    4.  **Calculate the Context Range:**
        *   Use the parser to find the line number of the *start* of the changed method's signature (the method declaration line) and the line number of the *end* of the method's body (the closing `}`).
        *   The context range is calculated as: `method_start_line - context_lines` to `method_end_line + context_lines`.  This includes the entire changed method and extends `context_lines` above and below.

    5.  **Include Contextual Method Signatures:**
        *   Examine all *other* methods within the *same* class as the changed method.
        *   For each of these other methods:
            *   Use the parser to get the line number of the method's *signature*.
            *   If the signature's line number falls *within* the calculated context range (from step 4), include that method's *signature* in the output.
            *   Replace the *body* of these contextual methods with `{ ... }`.  This is a placeholder; it's *not* valid C# code.

    6.  **Handle Nested Structures:**
        *   **Nested Methods/Lambdas:** Treat any code inside a method (including lambdas, local functions, or anonymous methods) as part of the *enclosing* method's body. Do *not* treat them as separate methods for the purpose of this mode.
        * **Nested Classes:** A class *within* a class *is treated as a separate class*. If a nested class contain a changed method, the full body of the method and *signature* of the *nested* class are included in the output.

    7.   **Handle Other Code Within the Context Range:**
        * Any other code in the files (besides method bodies) should be treated as per a normal "context diff," following the `context_lines` parameter.

    8.  **Output Format (Valid Unified Diff):**
        *   The final output *must* be a valid unified diff.  This means:
            *   The file header (`--- a/file.cs` and `+++ b/file.cs`) must be correct.
            *   Hunk headers (e.g., `@@ -10,5 +12,3 @@`) must be adjusted to reflect the changes you've made (including lines, removing lines, and replacing bodies with `{ ... }`).  The line numbers in the hunk headers must be accurate for the *modified* file content.
            *   Lines that are unchanged (context lines) should start with a space.
            *   Lines that are added should start with a `+`.
            *   Lines that are removed should start with a `-`.

*   **`context_lines` Role:**

    *   Defines the context range around the changed method.
    *   Determines which *other* method signatures are included.
    *   Provides additional lines of code *outside* of methods (within the class or namespace) that fall within the context range.
    *   **Crucially, `context_lines` does *not* truncate the body of the method containing the diff when `include_method_body: true`.**

*   **Use Case:** Ideal for detailed code reviews where understanding the full context of a method change is crucial. Provides enough surrounding code (signatures and some contextual lines) to understand the change's impact and purpose.

### 5.2.4 Edge Cases
To ensure consistent behavior, handle these scenarios as follows:

*   **Namespace Change:**
    *   If the diff is in a namespace declaration (e.g., a rename), include the *changed line* and the surrounding `context_lines`.  Do *not* include all affected classes—just provide enough context to understand the rename.
    *   Example: For `namespace OldName` -> `namespace NewName` with `context_lines: 3`, show the namespace line and 3 lines after (e.g., the class declaration).

*   **Abstract Method:**
    *   If the diff is in an abstract method (e.g., `public abstract void DoWork();`), include the full signature (the single line) as the "method body". Apply `context_lines` around this signature.
    *   Example: For a renamed abstract method with `context_lines: 2`, show the signature and 2 lines above/below.

* **Nested Class:**
  * If the change occurs within a method of a nested class, include the full body of that method along with the signature of the nested class itself.
  * The outer (containing) class's signature should only be included if it falls within the calculated `context_lines` range relative to the changed method within the nested class.
  * *Example*: If you have a change in a method inside a nested class and context_lines is set to 2, the output will include: the full body of the changed method, the signature of the nested class, and up to 2 lines before the start of the method signature and up to 2 lines after the end of the method body (from the outer context).

## 6. Additional Instructions in Output
Include a **header** in the final diff explaining placeholders:

```
NOTE: Some method bodies have been replaced with "{ ... }" to improve clarity for code reviews and LLM analysis.

- In API Signature Mode (api_signature_mode: true), most method bodies are intentionally omitted or truncated, focusing on API signatures.
- In Change Context Mode (include_method_body: true), the entire method containing the diff is included to provide full context.  Signatures of other methods within the context range are also included, but their bodies are replaced with "{ ... }". The "{ ... }" placeholder indicates that a method body has been omitted for brevity.
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