use repodiff::utils::token_counter::TokenCounter;

#[test]
fn test_count_tokens() {
    // Create the TokenCounter with the default model
    let token_counter = TokenCounter::new("gpt-4o").unwrap();
    
    // Test counting tokens for a simple string
    let text = "Hello, world!";
    let token_count = token_counter.count_tokens(text);
    
    // The exact token count may vary depending on the tokenizer, but it should be positive
    assert!(token_count > 0);
    
    // For "Hello, world!" with gpt-4o, it should be around 4 tokens
    assert!(token_count >= 3 && token_count <= 5);
}

#[test]
fn test_count_tokens_empty_string() {
    // Create the TokenCounter with the default model
    let token_counter = TokenCounter::new("gpt-4o").unwrap();
    
    // Test counting tokens for an empty string
    let text = "";
    let token_count = token_counter.count_tokens(text);
    
    // An empty string should have 0 tokens
    assert_eq!(token_count, 0);
}

#[test]
fn test_count_tokens_long_text() {
    // Create the TokenCounter with the default model
    let token_counter = TokenCounter::new("gpt-4o").unwrap();
    
    // Test counting tokens for a longer text
    let text = "This is a longer text that should have more tokens. It includes some punctuation, numbers like 12345, and special characters like @#$%.";
    let token_count = token_counter.count_tokens(text);
    
    // The exact token count may vary, but it should be significantly more than the short text
    assert!(token_count > 10);
}

#[test]
fn test_count_tokens_with_code() {
    // Create the TokenCounter with the default model
    let token_counter = TokenCounter::new("gpt-4o").unwrap();
    
    // Test counting tokens for code
    let code = r#"
fn main() {
    println!("Hello, world!");
    let x = 42;
    let y = x * 2;
    println!("x = {}, y = {}", x, y);
}
"#;
    let token_count = token_counter.count_tokens(code);
    
    // The exact token count may vary, but it should be positive
    assert!(token_count > 0);
} 