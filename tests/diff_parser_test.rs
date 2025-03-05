// Import the module to test
use repodiff::utils::diff_parser::DiffParser;

#[test]
fn test_parse_unified_diff_empty() {
    // Test parsing an empty diff
    let diff_output = "";
    let result = DiffParser::parse_unified_diff(diff_output).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_parse_unified_diff_single_file() {
    // Test parsing a diff with a single file
    let diff_output = "diff --git a/file1.txt b/file1.txt
--- a/file1.txt
+++ b/file1.txt
@@ -1,3 +1,3 @@
 line1
-line2
+line2_modified
 line3";
    
    let result = DiffParser::parse_unified_diff(diff_output).unwrap();
    
    assert_eq!(result.len(), 1);
    assert!(result.contains_key("file1.txt"));
    assert_eq!(result["file1.txt"].len(), 1);  // One hunk
    
    let hunk = &result["file1.txt"][0];
    assert_eq!(hunk.old_start, 1);
    assert_eq!(hunk.old_count, 3);
    assert_eq!(hunk.new_start, 1);
    assert_eq!(hunk.new_count, 3);
    assert_eq!(hunk.lines, vec![" line1", "-line2", "+line2_modified", " line3"]);
}

#[test]
fn test_parse_unified_diff_multiple_files() {
    // Test parsing a diff with multiple files
    let diff_output = "diff --git a/file1.txt b/file1.txt
--- a/file1.txt
+++ b/file1.txt
@@ -1,3 +1,3 @@
 line1
-line2
+line2_modified
 line3
diff --git a/file2.txt b/file2.txt
--- a/file2.txt
+++ b/file2.txt
@@ -1,2 +1,3 @@
 line1
+line2_added
 line3";
    
    let result = DiffParser::parse_unified_diff(diff_output).unwrap();
    
    assert_eq!(result.len(), 2);
    assert!(result.contains_key("file1.txt"));
    assert!(result.contains_key("file2.txt"));
    
    // Check file1.txt
    assert_eq!(result["file1.txt"].len(), 1);
    assert_eq!(result["file1.txt"][0].lines, vec![" line1", "-line2", "+line2_modified", " line3"]);
    
    // Check file2.txt
    assert_eq!(result["file2.txt"].len(), 1);
    assert_eq!(result["file2.txt"][0].lines, vec![" line1", "+line2_added", " line3"]);
}

#[test]
fn test_parse_unified_diff_multiple_hunks() {
    // Test parsing a diff with multiple hunks in a file
    let diff_output = "diff --git a/file1.txt b/file1.txt
--- a/file1.txt
+++ b/file1.txt
@@ -1,3 +1,3 @@
 line1
-line2
+line2_modified
 line3
@@ -10,2 +10,3 @@
 line10
+line11_added
 line12";
    
    let result = DiffParser::parse_unified_diff(diff_output).unwrap();
    
    assert_eq!(result.len(), 1);
    assert!(result.contains_key("file1.txt"));
    assert_eq!(result["file1.txt"].len(), 2);  // Two hunks
    
    // Check first hunk
    assert_eq!(result["file1.txt"][0].old_start, 1);
    assert_eq!(result["file1.txt"][0].old_count, 3);
    assert_eq!(result["file1.txt"][0].lines, vec![" line1", "-line2", "+line2_modified", " line3"]);
    
    // Check second hunk
    assert_eq!(result["file1.txt"][1].old_start, 10);
    assert_eq!(result["file1.txt"][1].old_count, 2);
    assert_eq!(result["file1.txt"][1].lines, vec![" line10", "+line11_added", " line12"]);
}

#[test]
fn test_parse_unified_diff_with_rename() {
    // Test parsing a diff with a renamed file
    let diff_output = "diff --git a/old_file.txt b/new_file.txt
similarity index 90%
rename from old_file.txt
rename to new_file.txt
--- a/old_file.txt
+++ b/new_file.txt
@@ -1,3 +1,3 @@
 line1
-line2
+line2_modified
 line3";
    
    let result = DiffParser::parse_unified_diff(diff_output).unwrap();
    
    assert_eq!(result.len(), 1);
    assert!(result.contains_key("new_file.txt"));
    
    let hunk = &result["new_file.txt"][0];
    assert!(hunk.is_rename);
    assert_eq!(hunk.rename_from.as_ref().unwrap(), "old_file.txt");
    assert_eq!(hunk.rename_to.as_ref().unwrap(), "new_file.txt");
    assert_eq!(hunk.similarity_index.as_ref().unwrap(), "similarity index 90%");
}

#[test]
fn test_reconstruct_patch_empty() {
    // Test reconstructing an empty patch
    let patch_dict = std::collections::HashMap::new();
    let result = DiffParser::reconstruct_patch(&patch_dict);
    assert_eq!(result, "");
}

#[test]
fn test_filter_hunk_context_lines() {
    // Create a sample hunk
    let hunk = repodiff::utils::diff_parser::Hunk {
        header: "@@ -1,10 +1,10 @@".to_string(),
        old_start: 1,
        old_count: 10,
        new_start: 1,
        new_count: 10,
        lines: vec![
            " line1".to_string(),
            " line2".to_string(),
            " line3".to_string(),
            "-line4".to_string(),
            "+line4_modified".to_string(),
            " line5".to_string(),
            " line6".to_string(),
            " line7".to_string(),
            " line8".to_string(),
            " line9".to_string(),
            " line10".to_string(),
        ],
        is_rename: false,
        rename_from: None,
        rename_to: None,
        similarity_index: None,
    };
    
    // Create a vector of hunks
    let hunks = vec![hunk];
    
    // Create a filter manager to filter the hunks
    let filter_rules = vec![
        repodiff::utils::config_manager::FilterRule {
            file_pattern: "*".to_string(),
            context_lines: 2,
        }
    ];
    let filter_manager = repodiff::filters::filter_manager::FilterManager::new(&filter_rules);
    
    // Apply filtering
    let filtered_hunks = filter_manager.post_process_files(&std::collections::HashMap::from([
        ("test.txt".to_string(), hunks)
    ]));
    
    // Check the result
    assert_eq!(filtered_hunks["test.txt"].len(), 1);
    assert_eq!(filtered_hunks["test.txt"][0].lines.len(), 6);
    assert_eq!(filtered_hunks["test.txt"][0].lines, vec![
        " line2".to_string(),
        " line3".to_string(),
        "-line4".to_string(),
        "+line4_modified".to_string(),
        " line5".to_string(),
        " line6".to_string(),
    ]);
} 