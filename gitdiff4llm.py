import subprocess
import sys
import os
import tiktoken

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

# Main function to generate the combined diff and calculate token count
def main(commit1, commit2, output_file):
    # Run git diff with the first set of options
    diff1 = run_git_diff(commit1, commit2, ["-U100", "--ignore-all-space", "--", ":!*Test*"])
    
    # Run git diff with the second set of options for test files
    diff2 = run_git_diff(commit1, commit2, ["-U20", "--ignore-all-space", "--", "*Test*"])
    
    # Ensure both diffs are valid strings
    if diff1 is None:
        diff1 = ""
    if diff2 is None:
        diff2 = ""

    # Combine the two diffs
    combined_diff = diff1 + "\n" + diff2
    
    # Write the combined diff to the output file
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write(combined_diff)
    
    # Calculate token count using LLM tokenizer
    token_count = count_tokens(combined_diff)
    
    # Output results
    print(f"Combined diff written to {output_file}")
    print(f"Total number of tokens: {token_count}")

# Entry point of the script
if __name__ == "__main__":
    if len(sys.argv) != 4:
        print("Usage: python gitdiff4review.py <commit1> <commit2> <output_file>")
        sys.exit(1)

    commit1 = sys.argv[1]
    commit2 = sys.argv[2]
    output_file = sys.argv[3]

    # Make sure the output directory exists
    os.makedirs(os.path.dirname(output_file), exist_ok=True)

    main(commit1, commit2, output_file)
