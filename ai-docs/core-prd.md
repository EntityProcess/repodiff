# Product Requirements Document (PRD) for RepoDiff: Single-Pass Diff + Post-Processing (Rust)

## 1. Overview

RepoDiff generates a simplified and context-aware unified diff of a Git repository, designed for code reviews and Large Language Model (LLM) analysis. It performs a **single** `git diff` command to capture all changed files and then post-processes the output in Rust to apply user-defined rules.  This approach avoids multiple Git calls and complex exclude patterns.  The key feature is flexible handling of C# files, offering control over the inclusion of method bodies and surrounding signatures.

## 2. Key Objectives

1.  **Single-Pass Diff:** Generate one unified diff covering all changed files in a single `git diff` operation.
2.  **Post-Processing (Rust):** Apply user-defined rules to the diff output, focusing on configurable handling of C# files:
    *   **Include Method Body:**  Controls whether the *entire* body of a changed method is included.
    *   **Include Signatures:** Controls whether the signatures of methods surrounding a changed method are included (within the defined context lines).
    * **Context Lines** Defines the context for including the surrounding signatures.
3.  **User-Friendly Output:** Generate a valid unified diff with clear placeholders (e.g., `⋮----`) indicating omitted code sections and an instructional header explaining the conventions.
4. **Configurable**: Allow users to set the config via a json file.
5. **Flexible Commit Comparison**: Support various ways to compare commits, including comparing a specific commit with its parent commit.

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
4.  **Reconstruct the Diff:** Generate the final, modified unified diff output. Adjust hunk headers and line numbers to reflect any changes made during post-processing (e.g., replacing method bodies with `⋮----`).

## 5. Handling C# Files

This section details the handling of C# (`*.cs`) files.

*   **Key Terms:**

    *   **Changed Method:** A method whose body (between `{` and `}`) contains at least one line from a diff hunk.
    *   **Diff Hunk:** A section of the unified diff output indicating a change (starts with `@@ ... @@`).
    *   **Method Signature:** The line declaring a method (e.g., `public int Foo(int x)`).
    *   **Context Range:** A range of lines determined by `context_lines` above and below each changed line. For example, if a line is changed at line 100 and `context_lines` is 5, the context range is lines 95-105.
    *   **Contextual Method:** A method *other than* the changed method whose signature falls within the context range of a changed line.
    *   **Nested Structures:** Methods within methods (lambdas, local functions), or classes within classes.

*   **Configuration Options:**

    *   `include_method_body`: If `true`, the *entire* body of a *changed* method is included in the output. If `false`, the changed method's body is replaced with `⋮----`.
    *   `include_signatures`: If `true`, the *signatures* of contextual methods (methods within the context range of a changed line) are included.
    *   `context_lines`: Defines the number of lines before and after each changed line to include in the output.

*   **Behavior Summary:**

    1. **Changed Methods:**
       * If `include_method_body` is `true`, Include the entire body of each changed method
       * If `include_method_body` is `false`, Replace the body with `⋮----`

    2. **Contextual Methods:**
       * If `include_signatures` is `true`: 
         * Include the signature of each contextual method
         * If the method body is small (≤ 2 * context_lines lines), include the entire body
         * If the method body is large, include the first and last context_lines lines with a `⋮----` placeholder in between
       * If `include_signatures` is `false`: Omit the method entirely

    3. **Context Lines:**
       * Include `context_lines` lines above and below each changed line
       * If a line is within `context_lines` of multiple changed lines, include it once
       * Insert `⋮----` placeholders where code is omitted

    4. **Namespace and Class Declarations:**
       * Always include the namespace and class declarations that directly enclose changed methods

*   **Detailed Behavior:**

    1.  **Identify Changed Methods:** Use a C# parser to identify methods containing changed lines.
    2.  **Handle Changed Method Body:**
        *   If `include_method_body` is `true`, include the *entire* body of each changed method.
        *   If `include_method_body` is `false`, replace the *entire* body of each changed method with `⋮----`.
    3.  **Include Namespace and Class Signatures:** Include the *declaration lines* of the namespace and class that *directly* enclose the changed method.
    4.  **Calculate the Context Range:** For each changed line, calculate the context range (changed_line - context_lines to changed_line + context_lines).
    5.  **Include Contextual Method Signatures (if `include_signatures` is true):**
        *   For any method (other than the changed methods) that has any line (signature or body) within the context range of a changed line (i.e., within context_lines lines before or after a changed line), include the method signature.
        * Additionally, include the lines of that method's body that fall within the context range of any changed line.
        * If there are consecutive lines within the method body that are not within the context range, insert a ⋮---- placeholder to indicate omitted code sections.
        * All included lines for contextual methods are marked as context lines (prefixed with a space) in the unified diff output.

    6.  **Handle Nested Structures:**
        *   Nested methods/lambdas: Treat as part of the enclosing method's body.
        *   Nested classes: Treat as separate classes.
    7.  **Handle Code Outside of Methods:** Include any code that falls within the context range of a changed line, even if it's not part of a method.
    8.  **Output Format**: Ensure that the result remains a valid unified diff as described in Section 8.

*   **Use Cases:**

    *   **Detailed Code Review:**  Set `include_method_body: true` and `include_signatures: true` to get the full context of changes and surrounding methods.
    *   **High-Level Overview:** Set `include_method_body: false` and `include_signatures: true` to focus on structural changes and API signatures.
    *   **Minimal Diff:** Set `include_method_body: false` and `include_signatures: false` to see only the changed lines themselves, with minimal context.

## 6. Handle Code Outside of Methods (Within the Context Range)

*   **Purpose:** This rule describes how to handle code that's *not* part of a method body and *not* the main namespace/class declaration (e.g., comments, `using` directives, field declarations).

*   **How it Works:**

    1.  **Identify "Other Code":** Any lines *not* within a method's `{` and `}` and *not* namespace/class declaration lines.
    2.  **Generate standard unified diff hunks that include *all* changed lines in "other code" and `context_lines` unchanged lines before and after each group of changed lines.**
    3.  **These hunks are included *regardless* of the context range of changed methods**, ensuring all changes outside methods are visible with appropriate context. Increasing `context_lines` will include more unchanged lines around these changed lines, providing additional context outside the changed methods.

## 7. Additional Instructions in Output

Include a header in the final diff explaining placeholders:

```