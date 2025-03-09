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
            include_method_body: false,
            include_signatures: false,
        },
        FilterRule {
            file_pattern: "*Test*.cs".to_string(),
            context_lines: 5,
            include_method_body: false,
            include_signatures: false,
        },
        FilterRule {
            file_pattern: "*.xml".to_string(),
            context_lines: 2,
            include_method_body: false,
            include_signatures: false,
        },
        FilterRule {
            file_pattern: "*".to_string(),
            context_lines: 3,
            include_method_body: false,
            include_signatures: false,
        },
    ];
    
    // Create the FilterManager
    let mut filter_manager = FilterManager::new(&filters);
    
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
    let mut filter_manager = FilterManager::new(&[]);
    
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
            include_method_body: false,
            include_signatures: false,
        },
        FilterRule {
            file_pattern: "tests/*_test.rs".to_string(),
            context_lines: 5,
            include_method_body: false,
            include_signatures: false,
        },
        FilterRule {
            file_pattern: "**/*.json".to_string(),
            context_lines: 2,
            include_method_body: false,
            include_signatures: false,
        },
        FilterRule {
            file_pattern: "*".to_string(),
            context_lines: 3,
            include_method_body: false,
            include_signatures: false,
        },
    ];
    
    // Create the FilterManager
    let mut filter_manager = FilterManager::new(&filters);
    
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

#[test]
fn test_csharp_method_body_inclusion() {
    let filters = vec![
        FilterRule {
            file_pattern: "*.cs".to_string(),
            context_lines: 3,
            include_method_body: true,
            include_signatures: false,
        },
    ];
    
    let mut filter_manager = FilterManager::new(&filters);
    let mut patch_dict = HashMap::new();
    
    // Test regular method
    let method_hunk = Hunk {
        header: "@@ -1,10 +1,10 @@".to_string(),
        old_start: 1,
        old_count: 10,
        new_start: 1,
        new_count: 10,
        lines: raw_to_lines(r#"
namespace Test {
    public class MyClass {
        public void MyMethod() {
            int x = 1;
-           Console.WriteLine(x);
+           Console.WriteLine(x + 1);
        }
    }
}"#),
        is_rename: false,
        rename_from: None,
        rename_to: None,
        similarity_index: None,
    };
    
    patch_dict.insert("Method.cs".to_string(), vec![method_hunk]);
    let processed = filter_manager.post_process_files(&patch_dict);
    
    // When include_method_body is true, we should see the entire method
    let method_result = &processed["Method.cs"][0];
    assert!(method_result.lines.iter().any(|l| l.contains("public void MyMethod()")));
    assert!(method_result.lines.iter().any(|l| l.contains("int x = 1")));
    assert!(method_result.lines.iter().any(|l| l.contains("Console.WriteLine(x + 1)")));
}

#[test]
fn test_csharp_property_body_inclusion() {
    let filters = vec![
        FilterRule {
            file_pattern: "*.cs".to_string(),
            context_lines: 3,
            include_method_body: true,
            include_signatures: false,
        },
    ];
    
    let mut filter_manager = FilterManager::new(&filters);
    let mut patch_dict = HashMap::new();
    
    // Test property with accessors where setter is changed
    let property_hunk = Hunk {
        header: "@@ -1,15 +1,15 @@".to_string(),
        old_start: 1,
        old_count: 15,
        new_start: 1,
        new_count: 15,
        lines: raw_to_lines(r#"
using System;

namespace Test {
    public class MyClass {
        private int myField;
        public int MyProperty
        {
            get { return myField; }
            set
            {
-               myField = value;
+               myField = value + 1;
            }
        }
    }
}"#),
        is_rename: false,
        rename_from: None,
        rename_to: None,
        similarity_index: None,
    };
    
    patch_dict.insert("Property.cs".to_string(), vec![property_hunk]);
    let processed = filter_manager.post_process_files(&patch_dict);
    
    // When include_method_body is true and a property accessor is changed,
    // we should see the entire property including its signature and all accessors
    let property_result = &processed["Property.cs"][0];
    assert!(property_result.lines.iter().any(|l| l.contains("public int MyProperty")));
    assert!(property_result.lines.iter().any(|l| l.contains("get { return myField; }")));
    assert!(property_result.lines.iter().any(|l| l.contains("set")));
    assert!(property_result.lines.iter().any(|l| l.contains("myField = value + 1")));
}

#[test]
fn test_csharp_arrow_property_inclusion() {
    let filters = vec![
        FilterRule {
            file_pattern: "*.cs".to_string(),
            context_lines: 3,
            include_method_body: true,
            include_signatures: false,
        },
    ];
    
    let mut filter_manager = FilterManager::new(&filters);
    let mut patch_dict = HashMap::new();
    
    // Test arrow expression property
    let arrow_property_hunk = Hunk {
        header: "@@ -1,10 +1,10 @@".to_string(),
        old_start: 1,
        old_count: 10,
        new_start: 1,
        new_count: 10,
        lines: raw_to_lines(r#"
namespace Test {
    public class MyClass {
-       public int QuickProperty => myField;
+       public int QuickProperty => myField + 1;
    }
}"#),
        is_rename: false,
        rename_from: None,
        rename_to: None,
        similarity_index: None,
    };
    
    patch_dict.insert("ArrowProperty.cs".to_string(), vec![arrow_property_hunk]);
    let processed = filter_manager.post_process_files(&patch_dict);
    
    // When include_method_body is true and an arrow property is changed,
    // we should see the entire property
    let arrow_result = &processed["ArrowProperty.cs"][0];
    assert!(arrow_result.lines.iter().any(|l| l.contains("public int QuickProperty =>")));
    assert!(arrow_result.lines.iter().any(|l| l.contains("myField + 1")));
}

// Helper function to convert a raw string to lines with proper indentation
fn raw_to_lines(s: &str) -> Vec<String> {
    s.lines()
        .skip(1) // Skip the initial empty line
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else if line.starts_with('-') || line.starts_with('+') {
                // For diff lines, preserve the marker and the indentation after it
                let marker = &line[0..1];
                let rest = &line[1..];
                format!("{}{}", marker, rest)
            } else {
                // For non-diff lines, add a space prefix to mark them as context lines
                format!(" {}", line)
            }
        })
        .collect()
}

#[test]
fn test_include_signatures_and_method_body() {
    let filters = vec![
        FilterRule {
            file_pattern: "*.cs".to_string(),
            context_lines: 4,
            include_method_body: true,
            include_signatures: true,
        },
    ];
    
    let mut filter_manager = FilterManager::new(&filters);
    let mut patch_dict = HashMap::new();
    
    let hunk = Hunk {
        header: "@@ -1,60 +1,60 @@".to_string(),
        old_start: 1,
        old_count: 60,
        new_start: 1,
        new_count: 60,
        lines: raw_to_lines(r#"
namespace Test {
    public class MyClass {
        public void Method1() {
            int x = 1;
-           Console.WriteLine(x);
+           Console.WriteLine(x + 1);
            int y = 2;
        }

        public void Method2() {
            // Initialize variables
            bool flag = true;
            int counter = 0;

            // Complex logic block
            if (flag) {
                for (int i = 0; i < 10; i++) {
                    counter++;
                }
            }

            // Final processing
            if (counter > 5) {
                return;
            }
        }

        public void Method3() {
            // Initial setup
            var setup = true;
            var items = new List<int>();

            // Some processing
            var result = Process(items);

            // Complex logic block
-           if (setup) {
+           if (setup && items.Any()) {
                for (int i = 0; i < 10; i++) {
                    counter++;
+                   items.Add(i);
                }
            }

            // Final cleanup
            items.Clear();
        }

        public void Method4() {
            // Initial setup
            var setup = true;
            var items = new List<int>();

            // Some processing
            var result = Process(items);

            // Complex logic block
            if (setup) {
                items.Add(42);
            }
        }
    }
}"#),
        is_rename: false,
        rename_from: None,
        rename_to: None,
        similarity_index: None,
    };
    
    patch_dict.insert("test.cs".to_string(), vec![hunk]);
    
    let processed = filter_manager.post_process_files(&patch_dict);
    let processed_hunks = &processed["test.cs"];
    
    let expected_lines = raw_to_lines(r#"
namespace Test {
    public class MyClass {
        public void Method1() {
            int x = 1;
-           Console.WriteLine(x);
+           Console.WriteLine(x + 1);
            int y = 2;
        }

        public void Method2() {
            // Initialize variables
            bool flag = true;
            int counter = 0;

            // Complex logic block
            if (flag) {
                for (int i = 0; i < 10; i++) {
                    counter++;
                }
            }

            // Final processing
            if (counter > 5) {
                return;
            }
        }

        public void Method3() {
            // Initial setup
            var setup = true;
            var items = new List<int>();

            // Some processing
            var result = Process(items);

            // Complex logic block
            if (setup && items.Any()) {
                for (int i = 0; i < 10; i++) {
                    counter++;
                    items.Add(i);
                }
            }

            // Final cleanup
            items.Clear();
        }

        public void Method4() {
            // Initial setup
            var setup = true;
            var items = new List<int>();

            // Some processing
            var result = Process(items);

            // Complex logic block
            if (setup) {
                items.Add(42);
            }
        }
    }
}"#);
    
    assert_eq!(processed_hunks[0].lines, expected_lines);
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