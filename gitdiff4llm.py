import subprocess
import sys
import os
import json
import tiktoken

# Tokenizer function using OpenAI's tiktoken for LLMs (GPT-3/4)
def count_tokens(text, model):
    encoding = tiktoken.encoding_for_model(model)
    return len(encoding.encode(text))

# Function to execute the git diff command and return the result
def run_git_diff(commit1, commit2, diff_options):
    try:
        result = subprocess.run(
            ["git", "diff", commit1, commit2] + diff_options,
            capture_output=True, text=True, check=True, encoding='utf-8', errors='replace'
        )
        return result.stdout
    except subprocess.CalledProcessError as e:
        print(f"Error running git diff: {e}")
        sys.exit(1)

# Function to load config (diff options and tiktoken model) from JSON config file
def load_config(config_file_name="config.json"):
    # First try to find the config file in the current working directory
    config_path = os.path.join(os.getcwd(), config_file_name)
    
    # If not found in the working directory, try to find it in the directory of the script or executable
    if not os.path.exists(config_path):
        script_dir = os.path.dirname(os.path.realpath(__file__))
        config_path = os.path.join(script_dir, config_file_name)
    
    if os.path.exists(config_path):
        try:
            with open(config_path, 'r', encoding='utf-8') as f:
                return json.load(f)
        except (FileNotFoundError, json.JSONDecodeError) as e:
            print(f"Error loading config file: {e}")
            sys.exit(1)
    else:
        print(f"Config file '{config_file_name}' not found in working directory or script directory.")
        sys.exit(1)

# Main function to generate the combined diff and calculate token count
def main(commit1, commit2, output_file):
    # Load the config from the default or specified path
    config = load_config()
    
    # Extract tiktoken model and diff configs from the config
    tiktoken_model = config.get("tiktoken_model", "gpt-4")
    diff_configs = config["diffs"]
    
    combined_diff = ""
    
    # Run git diff for each set of options and combine the results
    for diff_options in diff_configs:
        diff_output = run_git_diff(commit1, commit2, diff_options)
        if diff_output:
            combined_diff += diff_output + "\n"
    
    # Write the combined diff to the output file
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write(combined_diff)
    
    # Calculate token count using the tiktoken model
    token_count = count_tokens(combined_diff, tiktoken_model)
    
    # Output results
    print(f"Combined diff written to {output_file}")
    print(f"Total number of tokens: {token_count}")

# Entry point of the script
if __name__ == "__main__":
    if len(sys.argv) != 4:
        print("Usage: python gitdiff4llm.py <commit1> <commit2> <output_file>")
        sys.exit(1)

    commit1 = sys.argv[1]
    commit2 = sys.argv[2]
    output_file = sys.argv[3]

    # Make sure the output directory exists
    os.makedirs(os.path.dirname(output_file), exist_ok=True)

    main(commit1, commit2, output_file)
