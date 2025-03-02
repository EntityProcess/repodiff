import pytest
from unittest.mock import patch, MagicMock
from repodiff.utils.token_counter import TokenCounter


class TestTokenCounter:
    """
    Tests for the TokenCounter class.
    """
    
    @patch('tiktoken.encoding_for_model')
    def test_init_with_default_model(self, mock_encoding_for_model):
        """Test initialization with the default model."""
        mock_encoding = MagicMock()
        mock_encoding_for_model.return_value = mock_encoding
        
        counter = TokenCounter()
        
        mock_encoding_for_model.assert_called_once_with("gpt-4o")
        assert counter.model == "gpt-4o"
        assert counter.encoding == mock_encoding
    
    @patch('tiktoken.encoding_for_model')
    def test_init_with_custom_model(self, mock_encoding_for_model):
        """Test initialization with a custom model."""
        mock_encoding = MagicMock()
        mock_encoding_for_model.return_value = mock_encoding
        
        counter = TokenCounter(model="gpt-3.5-turbo")
        
        mock_encoding_for_model.assert_called_once_with("gpt-3.5-turbo")
        assert counter.model == "gpt-3.5-turbo"
        assert counter.encoding == mock_encoding
    
    @patch('tiktoken.encoding_for_model')
    def test_count_tokens(self, mock_encoding_for_model):
        """Test counting tokens in text."""
        mock_encoding = MagicMock()
        mock_encoding.encode.return_value = [1, 2, 3, 4, 5]  # 5 tokens
        mock_encoding_for_model.return_value = mock_encoding
        
        counter = TokenCounter()
        token_count = counter.count_tokens("This is a test.")
        
        mock_encoding.encode.assert_called_once_with("This is a test.")
        assert token_count == 5
    
    @patch('tiktoken.encoding_for_model')
    def test_count_tokens_empty_string(self, mock_encoding_for_model):
        """Test counting tokens in an empty string."""
        mock_encoding = MagicMock()
        mock_encoding.encode.return_value = []  # 0 tokens
        mock_encoding_for_model.return_value = mock_encoding
        
        counter = TokenCounter()
        token_count = counter.count_tokens("")
        
        mock_encoding.encode.assert_called_once_with("")
        assert token_count == 0 