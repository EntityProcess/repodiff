
# GitDiff4LLM

**GitDiff4LLM** is a tool designed to simplify code reviews by generating comprehensive git diffs between two commits, branches, or pull requests. It combines diffs into a single output file optimized for consumption by large language models (LLMs).

## Features

- Generate diffs between two commits or branches with customizable options.
- Supports customized diff options depending on file type.
- Automatically retrieves diffs for base and target branches from pull requests.
- Combines diffs into a single file
- Calculates token counts for estimating the query cost for LLMs.

## Usage

To use GitDiff4LLM, run the following command:

```bash
python gitdiff4llm.py <commit1> <commit2> /path/to/output_file.txt
```

### Example

```bash
python gitdiff4llm.py abc123 def456 combined_diff.txt
```

## Prerequisites

- **PowerShell (Windows Only)**: If you're using Windows, you need to run the script in PowerShell. The pattern matching functionality in the script will not work properly in Command Prompt (`cmd`).
- **Python 3.x**: Ensure Python is installed on your system.

## Installation

Clone the repository and navigate to the directory:

```bash
git clone https://github.com/EntityProcess/GitDiff4LLM.git
cd GitDiff4LLM
```

Make sure you have Python installed on your system, then you can run the script as outlined in the usage section.

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue for any bugs or feature requests.