
# GitDiff4LLM

**GitDiff4LLM** is a tool designed to simplify code reviews by generating comprehensive git diffs between two commits, branches, or pull requests. It combines diffs into a single output file optimized for consumption by large language models (LLMs).

## Features

- Generate diffs between two commits or branches with customizable options.
- Supports customized diff options depending on file type.
- Combines diffs into a single file.
- Calculates token counts for estimating the query cost for LLMs.

## Usage

You can either provide commit hashes to compare directly, or use the -b option to compare the latest commit in the current branch with the latest common commit in another branch (e.g., `master`). You can also specify an output file, or let the script default to the system's temporary directory if no output file is provided.

### Compare Latest Commit with Another Branch

To compare the latest commit in the current branch with the latest common commit in another branch (e.g., `master`), use the `-b` option:
```bash
gitdiff4llm -b <branch> [-o /path/to/output_file.txt]
```

**Example:** 
Compare the latest commit in the current branch with the latest common commit in master, and write the result to a default file in the system's temporary directory

```bash
gitdiff4llm -b master
```

### Compare Two Commits

```bash
gitdiff4llm -c1 <commit1> -c2 <commit2> [-o /path/to/output_file.txt]
```

* `-c1`, `--commit1`: First commit hash.
* `-c2`, `--commit2`: Second commit hash.
* `-o`, `--output_file`: (Optional) Path to the output file. If not provided, the diff will be written to a default file in the system's temporary directory.

## Prerequisites

- **PowerShell (Windows Only)**: If you're using Windows, you need to run the script in PowerShell. The pattern matching functionality in the script will not work properly in Command Prompt (`cmd`).
- **Python 3.x**: Ensure Python is installed on your system.

## Installation

Clone the repository and navigate to the directory:

```bash
git clone https://github.com/EntityProcess/GitDiff4LLM.git
cd GitDiff4LLM
```

Install PyInstaller by running the following command:

```bash
pip install pyinstaller
```

Generate the executable by running `build.bat`.

Add `./GitDiff4LLM/dist` to your `PATH` environmental variable.

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue for any bugs or feature requests.