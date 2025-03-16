mod cli;
mod error;
mod repodiff;
mod utils {
    pub mod config_manager;
    pub mod diff_parser;
    pub mod git_operations;
    pub mod token_counter;
}
pub mod filters;

fn main() {
    if let Err(e) = cli::run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
