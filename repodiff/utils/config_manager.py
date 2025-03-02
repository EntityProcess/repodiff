import json
import os
import sys
from typing import Dict, Any, Optional


class ConfigManager:
    """
    Manages configuration loading and access for the RepoDiff tool.
    """
    
    def __init__(self, config_file_name: str = "config.json"):
        """
        Initialize the ConfigManager with a specific config file.
        
        Args:
            config_file_name: The name of the configuration file to load.
        """
        self.config_file_name = config_file_name
        self.config = self.load_config()
    
    def load_config(self) -> Dict[str, Any]:
        """
        Load configuration from the config file.
        
        Returns:
            The loaded configuration as a dictionary.
            
        Raises:
            SystemExit: If the config file cannot be loaded.
        """
        # Check if running as a PyInstaller bundle (frozen)
        if getattr(sys, 'frozen', False):
            # If the application is run as a PyInstaller bundle, use the _MEIPASS directory
            script_dir = os.path.dirname(sys.executable)
        else:
            # If running in a normal Python environment, use the script directory
            script_dir = os.path.dirname(os.path.realpath(__file__))
            # Go up two levels to reach the root directory
            script_dir = os.path.dirname(os.path.dirname(script_dir))
        
        # Construct the path to config.json
        config_path = os.path.join(script_dir, self.config_file_name)
        
        if os.path.exists(config_path):
            try:
                with open(config_path, 'r', encoding='utf-8') as f:
                    return json.load(f)
            except (FileNotFoundError, json.JSONDecodeError) as e:
                print(f"Error loading config file: {e}")
                sys.exit(1)
        else:
            print(f"Config file '{self.config_file_name}' not found in script directory '{script_dir}'.")
            sys.exit(1)
    
    def get_tiktoken_model(self) -> str:
        """
        Get the tiktoken model from the configuration.
        
        Returns:
            The tiktoken model name as a string.
        """
        return self.config.get("tiktoken_model", "gpt-4o")
    
    def get_filters(self) -> list:
        """
        Get the filters from the configuration.
        
        Returns:
            The list of filter dictionaries.
        """
        return self.config.get("filters", [{'file_pattern': '*', 'context_lines': 3}]) 