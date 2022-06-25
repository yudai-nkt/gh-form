use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "gh-form", about, version)]
pub struct Args {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Debug, Subcommand)]
pub enum Action {
    /// Start a local server to preview issue form
    Preview {
        /// Path to the directory where issue forms are located
        #[clap(short, long, default_value = ".github/ISSUE_TEMPLATE")]
        directory: PathBuf,
        #[clap(short, long, default_value = "8047")]
        /// Port to bind
        port: u16,
    },
}
