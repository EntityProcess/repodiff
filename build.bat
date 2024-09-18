pyinstaller --onefile --hidden-import=tiktoken --hidden-import=tiktoken_ext.openai_public --hidden-import=tiktoken_ext --add-data "config.json;." gitdiff4llm.py
pause