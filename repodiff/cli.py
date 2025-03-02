import argparse
import sys
from typing import List, Optional

from repodiff.repodiff import RepoDiff
from repodiff.utils.git_operations import GitOperations


# Define version
__version__ = "0.2.0"


def parse_args(args: Optional[List[str]] = None) -> argparse.Namespace:
    """
    Parse command-line arguments.
    
    Args:
        args: Command-line arguments to parse. If None, sys.argv[1:] is used.
        
    Returns:
        Parsed arguments as a Namespace object.
    """
    parser = argparse.ArgumentParser(description="Run git diff between two commits and analyze with LLM.")
    parser.add_argument("-o", "--output_file", help="The file to output the combined diff.")
    parser.add_argument("-c1", "--commit1", help="The first commit hash.")
    parser.add_argument("-c2", "--commit2", help="The second commit hash.")
    parser.add_argument("-b", "--branch", help="Compare the latest commit on the current branch to the latest common commit with another branch (e.g., master).")
    parser.add_argument("-v", "--version", action="store_true", help="Display the current version of RepoDiff.")
    
    return parser.parse_args(args)


def main(args: Optional[List[str]] = None) -> None:
    """
    Main entry point for the CLI.
    
    Args:
        args: Command-line arguments to parse. If None, sys.argv[1:] is used.
    """
    parsed_args = parse_args(args)
    
    # Check if version flag is set
    if parsed_args.version:
        print(f"RepoDiff version {__version__}")
        sys.exit(0)
    
    # Initialize the RepoDiff tool
    repodiff = RepoDiff()
    git_ops = GitOperations()
    
    # Determine the commit hashes
    if parsed_args.branch:
        commit1 = git_ops.get_latest_common_commit_with_branch(parsed_args.branch)
        commit2 = git_ops.get_latest_commit()
        
        # Print the commits being used for the comparison
        print(f"Comparing latest common commit with branch '{parsed_args.branch}' ({commit1[:12]}) and the latest commit on the current branch ({commit2[:12]}).")
    else:
        if not parsed_args.commit1 or not parsed_args.commit2:
            print("You must either provide two commit hashes using --commit1 and --commit2, or use the -b option to compare against another branch.")
            sys.exit(1)
        commit1 = parsed_args.commit1
        commit2 = parsed_args.commit2
    
    # Set output file or default to the user's temporary directory
    if parsed_args.output_file:
        output_file = parsed_args.output_file
    else:
        output_file = RepoDiff.get_default_output_file()
        print(f"No output file specified. Using temporary directory: {output_file}")
    
    # Process the diff and get the token count
    token_count = repodiff.process_diff(commit1, commit2, output_file)
    
    # Output results
    print(f"Processed diff written to {output_file}")
    print(f"Total number of tokens: {token_count}")


if __name__ == "__main__":
    main() 