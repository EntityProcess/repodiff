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

### Compare Latest Commit with Another Branch

To compare the latest commit in the current branch with the latest common commit in another branch:

```bash
repodiff -b main -o output.txt
```

### Compare Two Specific Commits

```bash
repodiff -c abc1234 -d 5678def -o output.txt
```

Parameters:
* `-b`, `--branch`: Branch to compare with (e.g., `main` or `master`)
* `-c`, `--commit1`: First commit hash
* `-d`, `--commit2`: Second commit hash
* `-o`, `--output_file`: (Optional) Path to the output file. If not provided, the diff will be written to a default file in the system's temporary directory.
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