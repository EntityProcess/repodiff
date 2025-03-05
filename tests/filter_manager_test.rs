use repodiff::filters::filter_manager::FilterManager;
use repodiff::utils::config_manager::FilterRule;
use std::collections::HashMap;
use repodiff::utils::diff_parser::Hunk;

#[test]
fn test_new_with_filters() {
    // Create filter rules
    let filters = vec![
        FilterRule {
            file_pattern: "*.cs".to_string(),
            context_lines: 10,
        },
        FilterRule {
            file_pattern: "*Test*.cs".to_string(),
            context_lines: 5,
        },
        FilterRule {
            file_pattern: "*.xml".to_string(),
            context_lines: 2,
        },
        FilterRule {
            file_pattern: "*".to_string(),
            context_lines: 3,
        },
    ];
    
    // Create the FilterManager
    let filter_manager = FilterManager::new(&filters);
    
    // Test post-processing with different file patterns
    let mut patch_dict = HashMap::new();
    
    // Create a test hunk for a .cs file
    let cs_hunk = create_test_hunk();
    patch_dict.insert("file.cs".to_string(), vec![cs_hunk.clone()]);
    
    // Create a test hunk for a test .cs file
    let test_cs_hunk = create_test_hunk();
    patch_dict.insert("MyTest.cs".to_string(), vec![test_cs_hunk.clone()]);
    
    // Create a test hunk for an .xml file
    let xml_hunk = create_test_hunk();
    patch_dict.insert("config.xml".to_string(), vec![xml_hunk.clone()]);
    
    // Create a test hunk for a .md file
    let md_hunk = create_test_hunk();
    patch_dict.insert("readme.md".to_string(), vec![md_hunk.clone()]);
    
    // Apply post-processing
    let processed = filter_manager.post_process_files(&patch_dict);
    
    // Check that all files are still present
    assert_eq!(processed.len(), 4);
    assert!(processed.contains_key("file.cs"));
    assert!(processed.contains_key("MyTest.cs"));
    assert!(processed.contains_key("config.xml"));
    assert!(processed.contains_key("readme.md"));
}

#[test]
fn test_new_with_empty_filters() {
    // Create the FilterManager with empty filters
    let filter_manager = FilterManager::new(&[]);
    
    // Test post-processing with different file patterns
    let mut patch_dict = HashMap::new();
    
    // Create a test hunk
    let hunk = create_test_hunk();
    patch_dict.insert("file.cs".to_string(), vec![hunk.clone()]);
    
    // Apply post-processing
    let processed = filter_manager.post_process_files(&patch_dict);
    
    // Check that the file is still present
    assert_eq!(processed.len(), 1);
    assert!(processed.contains_key("file.cs"));
}

#[test]
fn test_post_process_files_with_complex_patterns() {
    // Create filter rules with complex patterns
    let filters = vec![
        FilterRule {
            file_pattern: "src/*.rs".to_string(),
            context_lines: 10,
        },
        FilterRule {
            file_pattern: "tests/*_test.rs".to_string(),
            context_lines: 5,
        },
        FilterRule {
            file_pattern: "**/*.json".to_string(),
            context_lines: 2,
        },
        FilterRule {
            file_pattern: "*".to_string(),
            context_lines: 3,
        },
    ];
    
    // Create the FilterManager
    let filter_manager = FilterManager::new(&filters);
    
    // Test post-processing with different file patterns
    let mut patch_dict = HashMap::new();
    
    // Create test hunks for different file patterns
    let rs_hunk = create_test_hunk();
    patch_dict.insert("src/main.rs".to_string(), vec![rs_hunk.clone()]);
    
    let test_rs_hunk = create_test_hunk();
    patch_dict.insert("tests/config_test.rs".to_string(), vec![test_rs_hunk.clone()]);
    
    let json_hunk = create_test_hunk();
    patch_dict.insert("config/settings.json".to_string(), vec![json_hunk.clone()]);
    
    let md_hunk = create_test_hunk();
    patch_dict.insert("README.md".to_string(), vec![md_hunk.clone()]);
    
    // Apply post-processing
    let processed = filter_manager.post_process_files(&patch_dict);
    
    // Check that all files are still present
    assert_eq!(processed.len(), 4);
    assert!(processed.contains_key("src/main.rs"));
    assert!(processed.contains_key("tests/config_test.rs"));
    assert!(processed.contains_key("config/settings.json"));
    assert!(processed.contains_key("README.md"));
}

// Helper function to create a test hunk
fn create_test_hunk() -> Hunk {
    Hunk {
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
    }
} 