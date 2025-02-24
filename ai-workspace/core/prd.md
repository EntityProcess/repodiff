# Product Requirements Document (PRD) for RepoDiff: Single-Pass Diff + Post-Processing

## 1. Overview
The goal is to **simplify** RepoDiff by performing a **single** `git diff` command that captures all changed files. Post-processing will then apply user-defined rules (like ignoring bodies of methods far from the changed lines). This approach eliminates the need for multiple Git calls or complex exclude patterns.

## 2. Key Objectives
1. Generate one **unified diff** that covers all changed files in a **single pass**.  
2. **Post-process** that diff in Python to apply user-defined patterns, such as:
   - Number of context lines around changes.
   - Inclusion of entire files with method body removal outside a specified threshold.
3. Provide **user-friendly** output. For instance, when removing method bodies, either keep the braces or replace them with a comment so that an LLM or human reviewer knows code is intentionally omitted.

## 3. Example Configuration
```json
{
  "tiktoken_model": "gpt-4o",
  "filters": [
    {
      "file_pattern": "*.cs",
      "include_entire_file_with_signatures": true,
      "method_body_threshold": 10
    },
    {
      "file_pattern": "*.py",
      "context_lines": 5
    }
  ]
}
```

## 4. Single-Pass Approach
1. **Collect All Changes**: Perform one `git diff` for the relevant commits or branches, capturing all changed files (e.g., `git diff commit1 commit2 --unified=999999`).
2. **Parse the Unified Diff**: Use Python to parse each file’s changed lines, hunks, and contextual lines.
3. **Apply Filters**: For each file, determine which config pattern applies and apply the appropriate rule.
4. **Finalize Output**: Reconstruct a single unified diff in memory or produce a custom output where method bodies are replaced with `{ ... }`.

## 5. Handling Method Body Removal in C#
When **`include_entire_file_with_signatures`** is true for a `.cs` file:
1. **Preserve** the **method signature** (e.g., `public void Foo(int x) { ... }`).
2. Keep an **opening** and **closing** brace for that method, replacing the body with `{ ... }`.

```csharp
public void Foo(int x)
{
    { ... }
}
```

The `{ ... }` placeholder is defined in an **instructional header** at the beginning of the diff output to explain that method bodies were intentionally omitted.

## 6. Additional Instructions in Output
Include a **header** in the final diff explaining placeholders:

```
NOTE: Some method bodies have been replaced with "{ ... }" to improve clarity for code reviews and LLM analysis.
```

## 7. Pseudocode
```python
def generate_single_pass_diff(commit1, commit2):
    cmd = ["git", "diff", commit1, commit2, "--unified=999999", "--ignore-all-space"]
    return run_command(cmd)

def parse_unified_diff(diff_output):
    return parse_patch(diff_output)

def post_process_files(patch_dict, config):
    for filename, hunks in patch_dict.items():
        rule = find_matching_rule(filename, config['filters'])
        if rule.get('include_entire_file_with_signatures'):
            threshold = rule.get('method_body_threshold', 10)
            patch_dict[filename] = apply_signature_removal(hunks, threshold)
        else:
            c = rule.get('context_lines', 3)
            patch_dict[filename] = apply_context_filter(hunks, c)

def main():
    config = load_config('config.json')
    raw_diff = generate_single_pass_diff("commit1", "commit2")
    patch_dict = parse_unified_diff(raw_diff)
    post_process_files(patch_dict, config)
    final_output = reconstruct_patch(patch_dict)
    add_explanatory_comments(final_output)
    print(final_output)
```

## 8. Performance
- **Single Git Command**: Works efficiently for 20-100 changed files.
- **In-Memory Post-Processing**: Minimal overhead, simple to maintain.

## 9. Summary
By **capturing all changes in one pass** and then **applying** user-defined rules in Python, RepoDiff remains simple, fast, and predictable. Including an **instructional header** helps both humans and LLMs understand the placeholders used for omitted code sections.
