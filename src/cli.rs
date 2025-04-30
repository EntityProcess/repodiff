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

    /// Compare the current branch HEAD with the target branch HEAD.
    /// Use --merge-base to compare with the common ancestor instead.
    #[arg(short, long, conflicts_with_all = ["commit", "previous"])]
    pub branch: Option<String>,

    /// Compare with the common ancestor (merge-base) of the current and target branch
    #[arg(short = 'a', long, requires = "branch")]
    pub merge_base: bool,

    /// Compare the specified commit (--commit) with a previous commit.
    /// If a hash is provided, compare with that specific hash.
    /// If no hash is provided, compare with the parent of the commit specified by --commit.
    #[arg(short = 'p', long = "previous", value_name = "PREVIOUS_COMMIT_HASH", num_args = 0..=1, requires = "commit", conflicts_with = "branch")]
    pub previous: Option<Option<String>>,
}

/// Abbreviate commit hash for cleaner output
fn short_hash(hash: &str) -> &str {
    &hash[..12.min(hash.len())]
}

/// Main entry point for the CLI
pub fn run() -> Result<()> {
    let args = Args::parse();
    
    // Initialize the RepoDiff tool and GitOperations
    let mut repodiff = RepoDiff::new("config.json")?;
    let git_ops = GitOperations::new();
    
    // Validate arguments first
    if args.branch.is_none() && args.commit.is_none() {
        eprintln!("You must specify either a branch to compare (--branch [-a]) or a commit to compare (--commit -p).");
        process::exit(1);
    }
    if args.commit.is_some() && args.previous.is_none() {
        // This specific combination (--commit without --previous) is invalid
         eprintln!("Missing comparison target. Use --previous (-p) to compare with a parent or specific commit when using --commit, or use --branch (-b) to compare with another branch.");
         process::exit(1);
    }

    // Determine the commit hashes based on validated arguments
    let (commit1, commit2): (String, String) = if let Some(branch) = args.branch {
        // Branch comparison logic
        let head_commit = git_ops.get_latest_commit()?;

        if args.merge_base {
            // Compare HEAD with merge-base
            let base_commit = git_ops.find_merge_base(&branch)?;
            println!(
                "Comparing merge-base with branch '{}' ({}) and current HEAD ({}).",
                branch,
                short_hash(&base_commit),
                short_hash(&head_commit)
            );
            (base_commit, head_commit)
        } else {
            // Compare HEAD with target branch HEAD
            let branch_head_commit = git_ops.get_branch_head(&branch)?;
            println!(
                "Comparing target branch '{}' HEAD ({}) and current HEAD ({}).",
                branch,
                short_hash(&branch_head_commit),
                short_hash(&head_commit)
            );
            (branch_head_commit, head_commit)
        }
    } else {
        // Commit comparison logic (--commit is guaranteed to be Some here, 
        // and --previous is also guaranteed to be Some due to the check above)
        let commit_to_compare = args.commit.unwrap(); // Safe due to initial check
        match args.previous.unwrap() { // Safe due to initial check
            Some(prev_commit_hash) => {
                // -p <hash> provided: Compare commit_to_compare with prev_commit_hash
                println!(
                    "Comparing specified commit {} with previous commit {}.",
                    short_hash(&commit_to_compare),
                    short_hash(&prev_commit_hash)
                );
                (prev_commit_hash, commit_to_compare)
            }
            None => {
                // -p flag provided without value: Compare commit_to_compare with its parent
                let parent_commit = git_ops.get_previous_commit(&commit_to_compare)?;
                println!(
                    "Comparing specified commit {} with its parent commit {}.",
                    short_hash(&commit_to_compare),
                    short_hash(&parent_commit)
                );
                (parent_commit, commit_to_compare)
            }
        }
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