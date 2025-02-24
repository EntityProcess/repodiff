
# RepoDiff

**RepoDiff** is a tool designed to simplify code reviews by generating comprehensive git diffs between two commits or branches. It allows you to configure diff options based on file paths, optimizing the output for consumption by large language models (LLMs).

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
repodiff -b <branch> [-o /path/to/output_file.txt]
```

**Example:** 
Compare the latest commit in the current branch with the latest common commit in master, and write the result to a default file in the system's temporary directory

```bash
repodiff -b master
```

### Compare Two Commits

```bash
repodiff -c1 <commit1> -c2 <commit2> [-o /path/to/output_file.txt]
```

* `-c1`, `--commit1`: First commit hash.
* `-c2`, `--commit2`: Second commit hash.
* `-o`, `--output_file`: (Optional) Path to the output file. If not provided, the diff will be written to a default file in the system's temporary directory.

### Configuring Diff Options

You can customize the diff options using a `config.json` file. This allows you to apply different diff strategies depending on the file path.

For example:

```bash
{
  "tiktoken_model": "gpt-4o",
  "diffs": [
    ["-U50", "--ignore-all-space", "--", ":!*Test*"],
    ["-U20", "--ignore-all-space", "--", "*Test*"]
  ]
}
```

Explanation of the options:

* `tiktoken_model`: This specifies the language model you're using (for example, gpt-4o), which helps estimate how many tokens the output will contain.
* `diffs`: This is a list of different comparison rules. Each rule has settings that control how Git compares the files:
    * `-U50`: Show 50 lines of context around changes (default is 3 lines).
    * `--ignore-all-space`: Ignore spaces when comparing files (useful when whitespace changes don't matter).
    * `--`: Signals the end of options and the start of file patterns.
    * `:!*Test*`: Exclude files with Test in their path.
    * `*Test*`: Include only files with Test in their path.

This setup means:
* For most files, it shows a larger context (50 lines around each change) and ignores spaces.
* For test files (*Test*), it shows fewer lines of context (20 lines) and also ignores spaces.

## Prerequisites

- **PowerShell (Windows Only)**: If you're using Windows, you need to run the script in PowerShell. The pattern matching functionality in the script will not work properly in Command Prompt (`cmd`).
- **Python 3.x**: Ensure Python is installed on your system.

## Installation

### Option 1: Download the Executable

1. Go to the [Releases](https://github.com/EntityProcess/RepoDiff/releases) page.
2. Download the latest version of the `repodiff.exe` executable.
3. Move the `repodiff.exe` file to a directory included in your system's `PATH`.

### Option 2: Build the Executable Yourself

Clone the repository and navigate to the directory:

```bash
git clone https://github.com/EntityProcess/RepoDiff.git
cd RepoDiff
```

Install PyInstaller by running the following command:

```bash
pip install pyinstaller
```

Generate the executable by running `build.bat`.

Add `./RepoDiff/dist` to your `PATH` environmental variable.

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue for any bugs or feature requests.