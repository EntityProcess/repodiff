use tree_sitter::{Parser, Node};
use crate::utils::diff_parser::Hunk;

/// Represents a C# method in the code
#[derive(Debug, PartialEq)]
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

/// Represents a C# file in the code
#[derive(Debug)]
pub struct CSharpFile {
    /// Methods in the file
    pub methods: Vec<CSharpMethod>,
    /// Using statements in the file
    pub using_statements: Vec<(usize, usize)>, // (start_line, end_line)
    /// Class declarations in the file
    pub class_declarations: Vec<(usize, usize)>, // (start_line, end_line)
    /// Namespace declarations in the file
    pub namespace_declarations: Vec<(usize, usize)>, // (start_line, end_line)
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
    pub fn parse_file(&mut self, code: &str, hunks: &[Hunk]) -> CSharpFile {
        let tree = self.parser.parse(code, None).expect("Failed to parse C# code");
        let root_node = tree.root_node();
        
        let mut file = CSharpFile {
            methods: Vec::new(),
            using_statements: Vec::new(),
            class_declarations: Vec::new(),
            namespace_declarations: Vec::new(),
        };

        self.find_nodes(root_node, code, &mut file);
        
        // Mark methods that contain changes or have changes in their body
        for method in &mut file.methods {
            method.has_changes = self.method_contains_changes(method, hunks);
        }
        
        file
    }
    
    /// Find all method declarations in the AST
    fn find_nodes(&self, node: Node, code: &str, file: &mut CSharpFile) {
        match node.kind() {
            "method_declaration" => {
                let start_line = node.start_position().row + 1;
                let end_line = node.end_position().row + 1;
                
                // Find the signature line by looking for the first child that's a method header
                let signature_line = node.child_by_field_name("header")
                    .map(|n| n.start_position().row + 1)
                    .unwrap_or(start_line);
                
                let text = node.utf8_text(code.as_bytes())
                    .unwrap_or_default()
                    .to_string();
                
                file.methods.push(CSharpMethod {
                    start_line,
                    end_line,
                    signature_line,
                    text,
                    has_changes: false,
                });
            },
            "property_declaration" => {
                let start_line = node.start_position().row + 1;
                let end_line = node.end_position().row + 1;
                let signature_line = start_line;

                // Check if this is an arrow expression property (=>)
                let is_arrow_expr = node.child_by_field_name("value")
                    .map(|n| n.kind() == "arrow_expression_clause")
                    .unwrap_or(false);

                if is_arrow_expr {
                    // For arrow expression properties, treat the whole thing as one method
                    let text = node.utf8_text(code.as_bytes())
                        .unwrap_or_default()
                        .to_string();
                    
                    file.methods.push(CSharpMethod {
                        start_line,
                        end_line,
                        signature_line,
                        text,
                        has_changes: false,
                    });
                } else {
                    // For regular properties, first add the property declaration itself
                    let text = node.utf8_text(code.as_bytes())
                        .unwrap_or_default()
                        .to_string();
                    
                    file.methods.push(CSharpMethod {
                        start_line,
                        end_line,
                        signature_line,
                        text,
                        has_changes: false,
                    });

                    // Then look for accessors within the property
                    let mut cursor: tree_sitter::TreeCursor<'_> = node.walk();
                    for child in node.children(&mut cursor) {
                        if child.kind() == "accessor_declaration" {
                            let accessor_start = child.start_position().row + 1;
                            let accessor_end = child.end_position().row + 1;
                            let accessor_text = child.utf8_text(code.as_bytes())
                                .unwrap_or_default()
                                .to_string();
                            
                            file.methods.push(CSharpMethod {
                                start_line: accessor_start,
                                end_line: accessor_end,
                                signature_line: accessor_start,
                                text: accessor_text,
                                has_changes: false,
                            });
                        }
                    }
                }
            },
            "using_directive" => {
                let start_line = node.start_position().row + 1;
                let end_line = node.end_position().row + 1;
                file.using_statements.push((start_line, end_line));
            },
            "namespace_declaration" => {
                let start_line = node.start_position().row + 1;
                let end_line = node.end_position().row + 1;
                file.namespace_declarations.push((start_line, end_line));
            },
            "class_declaration" => {
                let start_line = node.start_position().row + 1;
                let end_line = node.end_position().row + 1;
                file.class_declarations.push((start_line, end_line));
            },
            _ => {}
        }
        
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.find_nodes(child, code, file);
        }
    }

    /// Check if a method contains any changes from the diff hunks
    fn method_contains_changes(&self, method: &CSharpMethod, hunks: &[Hunk]) -> bool {
        for hunk in hunks {
            let mut current_line = hunk.new_start;
            
            // Check if any line in the hunk is within this method's body
            for line in &hunk.lines {
                if current_line >= method.start_line && current_line <= method.end_line {
                    // If it's a change line (+ or -) within the method body, mark the method as changed
                    if line.starts_with('+') || line.starts_with('-') {
                        return true;
                    }
                }
                
                // Only increment line count for non-deletion lines
                if !line.starts_with('-') {
                    current_line += 1;
                }
            }
        }
        false
    }

    /// Check if a node contains any changes from the diff hunks
    pub fn node_contains_changes(&self, start_line: usize, end_line: usize, hunks: &[Hunk]) -> bool {
        for hunk in hunks {
            let mut current_line = hunk.new_start;
            
            for line in &hunk.lines {
                if current_line >= start_line && current_line <= end_line {
                    if line.starts_with('+') || line.starts_with('-') {
                        return true;
                    }
                }
                
                if !line.starts_with('-') {
                    current_line += 1;
                }
            }
        }
        false
    }
} 