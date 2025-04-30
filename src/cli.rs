use clap::Parser;
use std::process;

use crate::error::Result;
use crate::repodiff::RepoDiff;
use crate::utils::git_operations::GitOperations;

/// Command-line arguments for RepoDiff
#[derive(Parser, Debug)]
#[command(author, version = env!("CARGO_PKG_VERSION"), about, long_about = None)]
pub struct Args {
    /// The file to output the combined diff
    #[arg(short, long)]
    pub output_file: Option<String>,

    /// The commit hash to compare (the 'newer' commit)
    #[arg(short = 'c', long = "commit")]
    pub commit: Option<String>,

    /// Compare the latest commit on the current branch to the latest common commit with another branch
    #[arg(short, long, conflicts_with_all = ["commit", "previous"])]
    pub branch: Option<String>,

    /// Compare the specified commit (--commit) with a previous commit.
    /// If a hash is provided, compare with that specific hash.
    /// If no hash is provided, compare with the parent of the commit specified by --commit.
    #[arg(short = 'p', long = "previous", value_name = "PREVIOUS_COMMIT_HASH", num_args = 0..=1, requires = "commit", conflicts_with = "branch")]
    pub previous: Option<Option<String>>,
}

/// Main entry point for the CLI
pub fn run() -> Result<()> {
    let args = Args::parse();
    
    // Initialize the RepoDiff tool and GitOperations
    let mut repodiff = RepoDiff::new("config.json")?;
    let git_ops = GitOperations::new();
    
    // Determine the commit hashes based on provided arguments
    let (commit1, commit2) = if let Some(branch) = args.branch {
        // Branch comparison logic
        let commit1 = git_ops.get_latest_common_commit_with_branch(&branch)?;
        let commit2 = git_ops.get_latest_commit()?;
        
        println!(
            "Comparing latest common commit with branch '{}' ({}) and the latest commit on the current branch ({}).",
            branch,
            &commit1[..12.min(commit1.len())],
            &commit2[..12.min(commit2.len())]
        );
        (commit1, commit2)

    } else if let Some(commit_to_compare) = args.commit {
        // Commit comparison logic (using --commit and --previous)
        match args.previous {
            Some(Some(prev_commit_hash)) => {
                // -p <hash> provided: Compare commit_to_compare with prev_commit_hash
                let commit1 = prev_commit_hash;
                let commit2 = commit_to_compare;
                println!(
                    "Comparing specified commit {} with previous commit {}.",
                    &commit2[..12.min(commit2.len())],
                    &commit1[..12.min(commit1.len())]
                );
                (commit1, commit2)
            }
            Some(None) => {
                // -p flag provided without value: Compare commit_to_compare with its parent
                let commit2 = commit_to_compare;
                let commit1 = git_ops.get_previous_commit(&commit2)?;
                println!(
                    "Comparing specified commit {} with its parent commit {}.",
                    &commit2[..12.min(commit2.len())],
                    &commit1[..12.min(commit1.len())]
                );
                (commit1, commit2)
            }
            None => {
                // Only -c provided, which is not enough for comparison.
                eprintln!("Missing comparison target. Use --previous (-p) to compare with a parent or specific commit when using --commit, or use --branch (-b) to compare with another branch.");
                process::exit(1);
            }
        }
    } else {
        // Neither --branch nor --commit specified.
        eprintln!("You must specify either a branch to compare (--branch) or a commit to compare (--commit) along with a comparison target (--previous).");
        process::exit(1);
    };
    
    // Set output file or default
    let output_file = args.output_file.unwrap_or_else(|| "repodiff_output.txt".to_string());
    
    // Process the diff and get the token count
    let token_count = repodiff.process_diff(&commit1, &commit2, &output_file)?;
    
    // Output results
    println!("Processed diff written to {}", output_file);
    println!("Total number of tokens: {}", token_count);
    
    Ok(())
}