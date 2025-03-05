use clap::Parser;
use std::process;

use crate::error::Result;
use crate::repodiff::RepoDiff;
use crate::utils::git_operations::GitOperations;

/// Version of the application
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Command-line arguments for RepoDiff
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The file to output the combined diff
    #[arg(short, long)]
    pub output_file: Option<String>,

    /// The first commit hash
    #[arg(short = 'c', long = "commit1")]
    pub commit1: Option<String>,

    /// The second commit hash
    #[arg(short = 'd', long = "commit2")]
    pub commit2: Option<String>,

    /// Compare the latest commit on the current branch to the latest common commit with another branch
    #[arg(short, long)]
    pub branch: Option<String>,

    /// Display the current version of RepoDiff
    #[arg(short, long)]
    pub version: bool,
}

/// Main entry point for the CLI
pub fn run() -> Result<()> {
    let args = Args::parse();
    
    // Check if version flag is set
    if args.version {
        println!("RepoDiff version {}", VERSION);
        return Ok(());
    }
    
    // Initialize the RepoDiff tool
    let repodiff = RepoDiff::new("config.json")?;
    let git_ops = GitOperations::new();
    
    // Determine the commit hashes
    let (commit1, commit2) = if let Some(branch) = args.branch {
        let commit1 = git_ops.get_latest_common_commit_with_branch(&branch)?;
        let commit2 = git_ops.get_latest_commit()?;
        
        // Print the commits being used for the comparison
        println!(
            "Comparing latest common commit with branch '{}' ({}) and the latest commit on the current branch ({}).",
            branch,
            &commit1[..12.min(commit1.len())],
            &commit2[..12.min(commit2.len())]
        );
        
        (commit1, commit2)
    } else {
        if args.commit1.is_none() || args.commit2.is_none() {
            eprintln!("You must either provide two commit hashes using --commit1 and --commit2, or use the -b option to compare against another branch.");
            process::exit(1);
        }
        
        (args.commit1.unwrap(), args.commit2.unwrap())
    };
    
    // Set output file or default to the user's temporary directory
    let output_file = if let Some(output_file) = args.output_file {
        output_file
    } else {
        let default_output = RepoDiff::get_default_output_file();
        println!("No output file specified. Using temporary directory: {}", default_output);
        default_output
    };
    
    // Process the diff and get the token count
    let token_count = repodiff.process_diff(&commit1, &commit2, &output_file)?;
    
    // Output results
    println!("Processed diff written to {}", output_file);
    println!("Total number of tokens: {}", token_count);
    
    Ok(())
} 