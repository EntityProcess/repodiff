# RepoDiff

**RepoDiff** is a tool designed to simplify code reviews by generating dynamic git diffs between two commits or branches. It optimizes diffs for analysis by large language models (LLMs) with features like context line adjustment.

## Features

- Generate diffs between two commits or branches with a single pass
- Configurable file pattern matching for different file types
- Adjustable context lines per file pattern
- Token counting for estimating LLM query costs
- Combines all changes into a single, well-formatted output
- High performance Rust implementation

## Installation

### Option 1: Download the Executable

1. Go to the [Releases](https://github.com/EntityProcess/RepoDiff/releases) page.
2. Download the latest version of the `repodiff.exe` executable.
3. Move the `repodiff.exe` file to a directory included in your system's `PATH`.

### Option 2: Build from Source

Clone the repository and navigate to the directory:

```bash
git clone https://github.com/EntityProcess/RepoDiff.git
cd RepoDiff
```

Build the Rust executable:

```bash
# On Windows, run:
cd repodiff
cargo build --release
```

The compiled binary will be available at `target/release/repodiff.exe` (Windows) or `target/release/repodiff` (Linux/macOS).

## Usage

### Compare Branches

To compare the latest commit on the current branch (`HEAD`) with the latest commit on another branch (`<target_branch>`):

```bash
# Default: Compare HEAD with <target_branch>'s HEAD
repodiff -b <target_branch> -o output.txt
```

This shows all changes on the current branch since it diverged from the target branch, *plus* any changes that occurred on the target branch after the divergence point.

To compare the latest commit on the current branch (`HEAD`) with the *common ancestor* (merge-base) commit between the current branch and the target branch:

```bash
# Use --merge-base: Compare HEAD with the merge-base of HEAD and <target_branch>
repodiff -b <target_branch> --merge-base -o output.txt
# Short flag equivalent:
repodiff -b <target_branch> -a -o output.txt 
```

This is often more useful for reviewing changes specific to the current branch, as it excludes changes made on the target branch after the branches diverged.

### Compare Two Specific Commits

To compare a specific commit with an earlier commit:

```bash
repodiff -c <newer_commit_hash> -p <earlier_commit_hash> -o output.txt
```

### Compare a Commit with its Parent (Previous) Commit

To compare a specific commit with its direct parent:

```bash
repodiff -c <commit_hash> -p -o output.txt
```

Parameters:
* `-b`, `--branch <target_branch>`: Specify the target branch for comparison. By default, compares the current branch's `HEAD` with the `<target_branch>`'s `HEAD`. Use with `--merge-base` to compare against the common ancestor instead.
* `-a`, `--merge-base`: When used with `-b`, compare the current branch's `HEAD` against the common ancestor (merge-base) commit of the current branch and the `<target_branch>`, instead of the `<target_branch>`'s `HEAD`.
* `-c`, `--commit <commit_hash>`: The newer commit hash to include in the comparison.
* `-p`, `--previous [PREVIOUS_COMMIT_HASH]`: Compare the commit specified by `-c` with a previous commit. If `PREVIOUS_COMMIT_HASH` is provided, compare against that specific hash. If omitted, compare against the parent of the commit specified by `-c`.
* `-o`, `--output_file <path>`: (Optional) Path to the output file. If not provided, the diff will be written to `repodiff_output.txt` in the current directory.
* `-v`, `--version`: Display the current version of RepoDiff
* `-h`, `--help`: Print help information

## Configuration

RepoDiff uses a `config.json` file in the project root directory. Example configuration:

```json
{
  "tiktoken_model": "gpt-4o",
  "filters": [
    {
      "file_pattern": "*Test*.cs",
      "context_lines": 1
    },
    {
      "file_pattern": "*.cs",
      "context_lines": 10,
      "include_method_body": true,
      "include_signatures": true
    },
    {
      "file_pattern": "*.xml",
      "context_lines": 10
    },
    {
      "file_pattern": "*",
      "context_lines": 3
    }
  ]
}
```

Configuration options:

* `tiktoken_model`: Specifies the language model for token counting (e.g., "gpt-4o").
* `filters`: An array of filter rules that determine how different files are processed.
  * `file_pattern`: Glob pattern to match files (e.g., "*.cs", "*Test*.cs").
  * `context_lines`: Number of context lines to show around changes (default: 3).
  * `include_method_body`: When true, includes the entire method body in the diff output when a change is detected within a method. This helps provide complete context for method-level changes.
  * `include_signatures`: When true, includes method signatures and class declarations in the diff output even if they haven't changed. This helps maintain readability by showing the structural context of the changes.

Filter rules are applied in order, with the first matching pattern being used.

## Output Format

The tool generates a unified diff format with some enhancements:

1. Standard git diff headers for each file.
2. Modified hunks based on the applied filters:
   - Adjusted context lines based on file patterns
   - Original line numbers preserved

Example output:

```diff
diff --git a/src/MyClass.cs b/src/MyClass.cs
--- a/src/MyClass.cs
+++ b/src/MyClass.cs
@@ -10,7 +10,7 @@ public class MyClass
     public void ProcessData(int value)
     {
-        var result = value * 2;
+        var result = value * 3;
         return result;
    }
```

## Prerequisites

- **Rust**: If building from source, you need Rust installed on your system.
- **Git**: Required for generating diffs.

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue for any bugs or feature requests.