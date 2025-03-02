import tiktoken
from typing import Optional


class TokenCounter:
    """
    Handles token counting for LLM models using OpenAI's tiktoken.
    """
    
    def __init__(self, model: str = "gpt-4o"):
        """
        Initialize the TokenCounter with a specific model.
        
        Args:
            model: The name of the LLM model to use for token counting.
        """
        self.model = model
        self.encoding = tiktoken.encoding_for_model(model)
    
    def count_tokens(self, text: str) -> int:
        """
        Count the number of tokens in the given text.
        
        Args:
            text: The text to count tokens for.
            
        Returns:
            The number of tokens in the text.
        """
        return len(self.encoding.encode(text)) 