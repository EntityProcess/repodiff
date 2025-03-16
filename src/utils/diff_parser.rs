use std::collections::HashMap;
use regex::Regex;
use crate::error::Result;

/// Represents a hunk in a git diff
#[derive(Debug, Clone)]
pub struct Hunk {
    /// The hunk header
    pub header: String,
    /// The starting line number in the old file
    pub old_start: usize,
    /// The number of lines in the old file
    pub old_count: usize,
    /// The starting line number in the new file
    pub new_start: usize,
    /// The number of lines in the new file
    pub new_count: usize,
    /// The lines in the hunk
    pub lines: Vec<String>,
    /// Whether this is a rename
    pub is_rename: bool,
    /// The original filename (for renames)
    pub rename_from: Option<String>,
    /// The new filename (for renames)
    pub rename_to: Option<String>,
    /// The similarity index (for renames)
    pub similarity_index: Option<String>,
}

/// Parser for git diff output that converts it to a structured format
pub struct DiffParser;

impl DiffParser {
    /// Parse the unified diff output into a dictionary of files and their hunks
    ///
    /// # Arguments
    ///
    /// * `diff_output` - The raw output from git diff command
    pub fn parse_unified_diff(diff_output: &str) -> Result<HashMap<String, Vec<Hunk>>> {
        let mut files = HashMap::new();
        let mut current_file = None;
        let mut current_hunks = Vec::new();
        let mut is_rename = false;
        let mut rename_from = None;
        let mut rename_to = None;
        let mut similarity_index = None;
        
        let hunk_header_re = Regex::new(r"@@ -(\d+),?(\d+)? \+(\d+),?(\d+)? @@")?;
        
        let lines: Vec<&str> = diff_output.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            let line = lines[i];
            
            if line.starts_with("diff --git") {
                // Save previous file data if exists
                if let Some(file) = current_file.take() {
                    files.insert(file, current_hunks);
                    current_hunks = Vec::new();
                }
                
                is_rename = false;
                rename_from = None;
                rename_to = None;
                similarity_index = None;
                
                // Check for rename by looking ahead
                let mut j = i + 1;
                while j < lines.len() && !lines[j].starts_with("diff --git") {
                    if lines[j].starts_with("similarity index ") {
                        similarity_index = Some(lines[j].to_string());
                        is_rename = true;
                    } else if lines[j].starts_with("rename from ") {
                        rename_from = Some(lines[j][12..].to_string());
                    } else if lines[j].starts_with("rename to ") {
                        rename_to = Some(lines[j][10..].to_string());
                    }
                    j += 1;
                }
            } else if line.starts_with("--- a/") {
                // For renames, we need to handle this differently
                if !is_rename {
                    i += 1;
                    continue;
                }
            } else if line.starts_with("+++ b/") {
                if is_rename && rename_from.is_some() && rename_to.is_some() {
                    current_file = rename_to.clone();
                } else {
                    current_file = Some(line[6..].to_string());
                }
            } else if line.starts_with("@@") {
                // Parse hunk header
                if let Some(caps) = hunk_header_re.captures(line) {
                    let old_start = caps.get(1).unwrap().as_str().parse::<usize>().unwrap();
                    let old_count = caps.get(2)
                        .map_or(1, |m| m.as_str().parse::<usize>().unwrap_or(1));
                    let new_start = caps.get(3).unwrap().as_str().parse::<usize>().unwrap();
                    let new_count = caps.get(4)
                        .map_or(1, |m| m.as_str().parse::<usize>().unwrap_or(1));
                    
                    current_hunks.push(Hunk {
                        header: line.to_string(),
                        old_start,
                        old_count,
                        new_start,
                        new_count,
                        lines: Vec::new(),
                        is_rename,
                        rename_from: rename_from.clone(),
                        rename_to: rename_to.clone(),
                        similarity_index: similarity_index.clone(),
                    });
                }
            } else if current_file.is_some() && !current_hunks.is_empty() {
                current_hunks.last_mut().unwrap().lines.push(line.to_string());
            }
            
            i += 1;
        }
        
        // Save the last file
        if let Some(file) = current_file {
            files.insert(file, current_hunks);
        }
        
        Ok(files)
    }
    
    /// Get the instructions for interpreting git diff output
    fn get_diff_instructions() -> Vec<String> {
        let instructions = r#"**Instructions for Interpreting Git Diff Output**

This document provides a guide to understanding the diff output generated by RepoDiff.

**Important Note:** The diff output in `repodiff_output.txt` has been sanitized to focus what's relevant for understanding the diffs.
Real-world Git diff output may contain more details.

**1. Basic Structure:**

A Git diff file describes the *differences* between two versions of a file. It's structured into *hunks*, which represent contiguous regions of change.

*   `diff --git a/<path> b/<path>`:  Indicates the file being compared. `a/` refers to the "old" version, and `b/` refers to the "new" version. (Note that paths always use forward slashes in Git diff output, even on Windows systems.)
*   `--- a/<path>`:  Marks the beginning of the original file content.
*   `+++ b/<path>`: Marks the beginning of the modified file content.
*   `@@ -<start_line_old>,<num_lines_old> +<start_line_new>,<num_lines_new> @@ <section_header>`: This is the *hunk header*. (Optional in simplified output, but common in real diffs).
    *   `-<start_line_old>,<num_lines_old>`: Indicates the starting line number and number of lines in the *old* version of the file that this hunk represents.  If only one line is affected, `,<num_lines_old>` will be omitted.
    *   `+<start_line_new>,<num_lines_new>`: Indicates the starting line number and number of lines in the *new* version of the file that this hunk represents.  If only one line is affected, `,<num_lines_new>` will be omitted.
    *   `<section_header>`:  (Optional) This is often a function or method name, providing context for the change.
*   Hunk Content:  Lines within a hunk are marked with a prefix:
    *   ` ` (space):  Unchanged line (context).
    *   `-`: Line removed from the old version.
    *   `+`: Line added to the new version.

**2. Simplified Example:**

```
diff --git a/MyFile.cs b/MyFile.cs
--- a/MyFile.cs
+++ b/MyFile.cs 
 // Some code
 string oldValue = "old";
-// Removed line
+string newValue = "new";
 // More code
```

**Explanation of the Example:**

*   The file being changed is `MyFile.cs`.
*   `" string oldValue = "old";"`:  This line is present in both versions.
*   `-// Removed line`: This line was removed from the old version.
*   `+string newValue = "new";`: This line was added to the new version.
*   `" // More code"`: This line is present in both versions.

**3. Key LLM Considerations:**

*   **Focus on Content Lines:** The most important part for understanding changes is the content prefixed with ` `, `-`, or `+`.
*   **Context is Crucial:** Use the surrounding unchanged lines to understand the *purpose* of the change.
*   **File Paths:**  Pay attention to the file paths (`a/<path>`, `b/<path>`) to understand which files are being modified.

**4. Application to your File:**

*   **".cs" Files:**  Changes to C# source code.  Focus on the addition (`+`) and removal (`-`) of code lines to understand logic changes.
*   **"Test*.cs" Files:** Changes to unit test files.  These are often important for understanding how the functionality is being tested and whether the changes are robust.
*   **".xml" Files:**  Changes to configuration or data files. Look for added, removed, or modified XML elements and attributes. Focus is usually on changes to properties.

**5. Special Instructions for File Types based on the given filters:**

* `.cs` code is assumed to not contain test code
* `*Test*.cs` contain test code, which should be helpful for understanding functionality.
* `*.xml` contains configuration.

By focusing on these key elements, you can effectively extract meaningful information from Git diff output and summarize the changes made in a software project.

---
"#;
        instructions.lines().map(|s| s.to_string()).collect()
    }

    /// Reconstruct a unified diff from the processed patch dictionary
    ///
    /// # Arguments
    ///
    /// * `patch_dict` - Dictionary mapping filenames to lists of hunks
    pub fn reconstruct_patch(patch_dict: &HashMap<String, Vec<Hunk>>) -> String {
        let mut output = Vec::new();
        
        // Only add instructions if the patch dictionary is not empty
        if !patch_dict.is_empty() {
            output.extend(Self::get_diff_instructions());
        }
        
        for (filename, hunks) in patch_dict {
            // Check if any hunks have rename information
            let is_rename = hunks.iter().any(|hunk| hunk.is_rename);
            
            if is_rename && !hunks.is_empty() {
                // Get rename information from the first hunk
                let first_hunk = &hunks[0];
                let rename_from = first_hunk.rename_from.as_ref();
                let rename_to = first_hunk.rename_to.as_ref();
                let similarity_index = first_hunk.similarity_index.as_ref();
                
                // Construct the rename diff header
                if let (Some(from), Some(to)) = (rename_from, rename_to) {
                    output.push(format!("diff --git a/{} b/{}", from, to));
                    if let Some(sim_idx) = similarity_index {
                        output.push(sim_idx.clone());
                    }
                    output.push(format!("rename from {}", from));
                    output.push(format!("rename to {}", to));
                    output.push(format!("--- a/{}", from));
                    output.push(format!("+++ b/{}", to));
                }
            } else {
                // Regular file diff
                output.push(format!("diff --git a/{} b/{}", filename, filename));
                output.push(format!("--- a/{}", filename));
                output.push(format!("+++ b/{}", filename));
            }
            
            for hunk in hunks {
                // Skip the hunk header as it's not necessary for understanding changes
                // output.push(hunk.header.clone());
                output.extend(hunk.lines.clone());
            }
        }
        
        output.join("\n")
    }
} 