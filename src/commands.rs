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
        /// YAML-formatted issue form
        file: String,
    },
}
