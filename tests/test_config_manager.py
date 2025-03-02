import pytest
import os
import sys
import json
from unittest.mock import patch, mock_open, MagicMock
from repodiff.utils.config_manager import ConfigManager


class TestConfigManager:
    """
    Tests for the ConfigManager class.
    """
    
    @patch('os.path.exists')
    @patch('builtins.open', new_callable=mock_open, read_data='{"tiktoken_model": "test-model", "filters": [{"file_pattern": "*.test", "context_lines": 5}]}')
    def test_load_config_success(self, mock_file, mock_exists):
        """Test loading config successfully."""
        # Set up the mocks
        mock_exists.return_value = True
        
        # Create the ConfigManager
        config_manager = ConfigManager()
        
        # Check the loaded config
        assert config_manager.config == {
            "tiktoken_model": "test-model",
            "filters": [{"file_pattern": "*.test", "context_lines": 5}]
        }
    
    @patch('os.path.exists')
    @patch('sys.exit')
    def test_load_config_file_not_found(self, mock_exit, mock_exists):
        """Test loading config when the file doesn't exist."""
        # Set up the mocks
        mock_exists.return_value = False
        
        # Create the ConfigManager
        ConfigManager()
        
        # Verify sys.exit was called
        mock_exit.assert_called_once_with(1)
    
    @patch('os.path.exists')
    @patch('builtins.open', side_effect=json.JSONDecodeError("Expecting value", "", 0))
    @patch('sys.exit')
    def test_load_config_invalid_json(self, mock_exit, mock_open, mock_exists):
        """Test loading config with invalid JSON."""
        # Set up the mocks
        mock_exists.return_value = True
        
        # Create the ConfigManager
        ConfigManager()
        
        # Verify sys.exit was called
        mock_exit.assert_called_once_with(1)
    
    @patch('os.path.exists')
    @patch('builtins.open', new_callable=mock_open, read_data='{"tiktoken_model": "test-model", "filters": [{"file_pattern": "*.test", "context_lines": 5}]}')
    def test_get_tiktoken_model(self, mock_file, mock_exists):
        """Test getting the tiktoken model from config."""
        # Set up the mocks
        mock_exists.return_value = True
        
        # Create the ConfigManager
        config_manager = ConfigManager()
        
        # Check the tiktoken model
        assert config_manager.get_tiktoken_model() == "test-model"
    
    @patch('os.path.exists')
    @patch('builtins.open', new_callable=mock_open, read_data='{"filters": [{"file_pattern": "*.test", "context_lines": 5}]}')
    def test_get_tiktoken_model_default(self, mock_file, mock_exists):
        """Test getting the default tiktoken model when not in config."""
        # Set up the mocks
        mock_exists.return_value = True
        
        # Create the ConfigManager
        config_manager = ConfigManager()
        
        # Check the tiktoken model
        assert config_manager.get_tiktoken_model() == "gpt-4o"
    
    @patch('os.path.exists')
    @patch('builtins.open', new_callable=mock_open, read_data='{"tiktoken_model": "test-model", "filters": [{"file_pattern": "*.test", "context_lines": 5}]}')
    def test_get_filters(self, mock_file, mock_exists):
        """Test getting the filters from config."""
        # Set up the mocks
        mock_exists.return_value = True
        
        # Create the ConfigManager
        config_manager = ConfigManager()
        
        # Check the filters
        assert config_manager.get_filters() == [{"file_pattern": "*.test", "context_lines": 5}]
    
    @patch('os.path.exists')
    @patch('builtins.open', new_callable=mock_open, read_data='{"tiktoken_model": "test-model"}')
    def test_get_filters_default(self, mock_file, mock_exists):
        """Test getting the default filters when not in config."""
        # Set up the mocks
        mock_exists.return_value = True
        
        # Create the ConfigManager
        config_manager = ConfigManager()
        
        # Check the filters
        assert config_manager.get_filters() == [{'file_pattern': '*', 'context_lines': 3}] 