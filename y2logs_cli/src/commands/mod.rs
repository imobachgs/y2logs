pub mod filter;

use clap::{Parser, Subcommand};
use filter::FilterArgs;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Filter YaST2 log entries
    Filter(FilterArgs)
}

