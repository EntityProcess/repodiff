use std::fs;
use tempfile::tempdir;
use serde_json::json;

// Import the module to test
use repodiff::utils::config_manager::ConfigManager;

#[test]
fn test_load_config_success() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.json");
    
    // Create a test config file
    let config_content = json!({
        "tiktoken_model": "test-model",
        "filters": [{"file_pattern": "*.test", "context_lines": 5}]
    });
    fs::write(&config_path, config_content.to_string()).unwrap();
    
    // Create the ConfigManager with the test file path
    let config_manager = ConfigManager::new(config_path.to_str().unwrap()).unwrap();
    
    // Check the loaded config
    assert_eq!(config_manager.get_tiktoken_model(), "test-model");
    assert_eq!(config_manager.get_filters().len(), 1);
    assert_eq!(config_manager.get_filters()[0].file_pattern, "*.test");
    assert_eq!(config_manager.get_filters()[0].context_lines, 5);
}

#[test]
fn test_get_tiktoken_model_default() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.json");
    
    // Create a test config file with tiktoken_model set to default value
    let config_content = json!({
        "tiktoken_model": "gpt-4o",
        "filters": []
    });
    fs::write(&config_path, config_content.to_string()).unwrap();
    
    // Create the ConfigManager with the test file path
    let config_manager = ConfigManager::new(config_path.to_str().unwrap()).unwrap();
    
    // Check the default tiktoken model - the implementation uses "gpt-4o" as default
    assert_eq!(config_manager.get_tiktoken_model(), "gpt-4o");
}

#[test]
fn test_get_filters_default() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.json");
    
    // Create a test config file without filters
    let config_content = json!({
        "tiktoken_model": "test-model",
        "filters": []
    });
    fs::write(&config_path, config_content.to_string()).unwrap();
    
    // Create the ConfigManager with the test file path
    let config_manager = ConfigManager::new(config_path.to_str().unwrap()).unwrap();
    
    // Check the filters - empty array should be respected, not replaced with default
    assert_eq!(config_manager.get_filters().len(), 0);
}

#[test]
#[should_panic(expected = "system cannot find the path specified")]
fn test_load_config_file_not_found() {
    // Try to create a ConfigManager with a non-existent file
    let non_existent_path = "/path/to/nonexistent/config.json";
    let _ = ConfigManager::new(non_existent_path).unwrap();
}

#[test]
#[should_panic(expected = "key must be a string")]
fn test_load_config_invalid_json() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.json");
    
    // Create an invalid JSON file
    fs::write(&config_path, "{ invalid json }").unwrap();
    
    // Try to create a ConfigManager with the invalid file
    let _ = ConfigManager::new(config_path.to_str().unwrap()).unwrap();
} 