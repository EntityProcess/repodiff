// Export modules for testing
pub mod utils {
    pub mod config_manager;
    pub mod diff_parser;
    pub mod token_counter;
    pub mod git_operations;
}

pub mod filters {
    pub mod filter_manager;
    pub mod csharp_parser;
}

pub mod error;
pub mod repodiff;
pub mod cli; 