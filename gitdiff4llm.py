import subprocess
import sys
import os
import json
import tiktoken
import argparse
import tempfile

# Tokenizer function using OpenAI's tiktoken for LLMs (GPT-3/4)
def count_tokens(text, model="gpt-4o"):
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
    # Check if running as a PyInstaller bundle (frozen)
    if getattr(sys, 'frozen', False):
        # If the application is run as a PyInstaller bundle, use the _MEIPASS directory
        script_dir = os.path.dirname(sys.executable)
    else:
        # If running in a normal Python environment, use the script directory
        script_dir = os.path.dirname(os.path.realpath(__file__))
    
    # Construct the path to config.json
    config_path = os.path.join(script_dir, config_file_name)
    
    if os.path.exists(config_path):
        try:
            with open(config_path, 'r', encoding='utf-8') as f:
                return json.load(f)
        except (FileNotFoundError, json.JSONDecodeError) as e:
            print(f"Error loading config file: {e}")
            sys.exit(1)
    else:
        print(f"Config file '{config_file_name}' not found in script directory '{script_dir}'.")
        sys.exit(1)

# Function to get the latest commit hash for the current branch
def get_latest_commit():
    try:
        result = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            capture_output=True, text=True, check=True, encoding='utf-8', errors='replace'
        )
        return result.stdout.strip()
    except subprocess.CalledProcessError as e:
        print(f"Error getting the latest commit: {e}")
        sys.exit(1)

# Function to get the latest common commit between the current branch and base branch
def get_latest_common_commit_with_branch(branch):
    try:
        result = subprocess.run(
            ["git", "merge-base", "HEAD", branch],
            capture_output=True, text=True, check=True, encoding='utf-8', errors='replace'
        )
        return result.stdout.strip()
    except subprocess.CalledProcessError as e:
        print(f"Error getting the latest common commit with '{branch}': {e}")
        sys.exit(1)

# Main function to generate the combined diff and calculate token count
def main(commit1, commit2, output_file):
    # Load the config from the default or specified path
    config = load_config()
    
    # Extract tiktoken model and diff configs from the config
    tiktoken_model = config.get("tiktoken_model", "gpt-4o")
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
    # Set up argument parser
    parser = argparse.ArgumentParser(description="Run git diff between two commits and analyze with LLM.")
    parser.add_argument("-o", "--output_file", help="The file to output the combined diff.")
    parser.add_argument("-c1", "--commit1", help="The first commit hash.")
    parser.add_argument("-c2", "--commit2", help="The second commit hash.")
    parser.add_argument("-b", "--branch", help="Compare the latest commit on the current branch to the latest common commit with another branch (e.g., master).")

    args = parser.parse_args()

    # Determine the commit hashes
    if args.branch:
        commit1 = get_latest_common_commit_with_branch(args.branch)
        commit2 = get_latest_commit()

        # Print the commits being used for the comparison
        print(f"Comparing latest common commit with branch '{args.branch}' ({commit1[:12]}) and the latest commit on the current branch ({commit2[:12]}).")
    else:
        if not args.commit1 or not args.commit2:
            print("You must either provide two commit hashes using --commit1 and --commit2, or use the -b option to compare against another branch.")
            sys.exit(1)
        commit1 = args.commit1
        commit2 = args.commit2

    # Set output file or default to the user's temporary directory
    if args.output_file:
        output_file = args.output_file
    else:
        temp_dir = tempfile.gettempdir()
        output_file = os.path.join(temp_dir, "gitdiff4llm", "gitdiff_output.txt")
        print(f"No output file specified. Using temporary directory: {output_file}")

    # Make sure the output directory exists
    os.makedirs(os.path.dirname(output_file), exist_ok=True)

    main(commit1, commit2, output_file)
