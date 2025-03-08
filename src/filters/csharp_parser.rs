use tree_sitter::{Parser, Node};
use crate::utils::diff_parser::Hunk;

/// Represents a C# method in the code
#[derive(Debug)]
pub struct CSharpMethod {
    /// Start line of the method (1-indexed)
    pub start_line: usize,
    /// End line of the method (1-indexed)
    pub end_line: usize,
    /// Line containing the method signature
    pub signature_line: usize,
    /// Full method text
    pub text: String,
    /// Whether this method contains changes
    pub has_changes: bool,
}

/// Parser for C# code that extracts method information
pub struct CSharpParser {
    parser: Parser,
}

impl CSharpParser {
    /// Create a new C# parser
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_c_sharp::language()).expect("Error loading C# grammar");
        CSharpParser { parser }
    }

    /// Parse C# code and extract method information
    ///
    /// # Arguments
    ///
    /// * `code` - The C# code to parse
    /// * `hunks` - The diff hunks to identify changed methods
    pub fn parse_methods(&mut self, code: &str, hunks: &[Hunk]) -> Vec<CSharpMethod> {
        let tree = self.parser.parse(code, None).expect("Failed to parse C# code");
        let root_node = tree.root_node();
        
        let mut methods = Vec::new();
        self.find_methods(root_node, code, &mut methods);
        
        // Mark methods that contain changes
        for method in &mut methods {
            method.has_changes = self.method_contains_changes(method, hunks);
        }
        
        methods
    }
    
    /// Find all method declarations in the AST
    fn find_methods(&self, node: Node, code: &str, methods: &mut Vec<CSharpMethod>) {
        if node.kind() == "method_declaration" {
            let start_line = node.start_position().row + 1;
            let end_line = node.end_position().row + 1;
            
            // Find the signature line by looking for the first child that's a method header
            let signature_line = node.child_by_field_name("header")
                .map(|n| n.start_position().row + 1)
                .unwrap_or(start_line);
            
            let text = node.utf8_text(code.as_bytes())
                .unwrap_or_default()
                .to_string();
            
            methods.push(CSharpMethod {
                start_line,
                end_line,
                signature_line,
                text,
                has_changes: false,
            });
        }
        
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.find_methods(child, code, methods);
        }
    }
    
    /// Check if a method contains any changes from the diff hunks
    fn method_contains_changes(&self, method: &CSharpMethod, hunks: &[Hunk]) -> bool {
        for hunk in hunks {
            let hunk_start = hunk.new_start;
            let hunk_end = hunk.new_start + hunk.new_count;
            
            // Check if the hunk overlaps with the method
            if (hunk_start >= method.start_line && hunk_start <= method.end_line) ||
               (hunk_end >= method.start_line && hunk_end <= method.end_line) {
                // Check if there are actual changes in the overlapping region
                for line in &hunk.lines {
                    if line.starts_with('+') || line.starts_with('-') {
                        return true;
                    }
                }
            }
        }
        false
    }
} 