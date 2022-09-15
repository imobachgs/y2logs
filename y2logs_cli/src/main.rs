mod commands;
use clap::Parser;
use commands::{Cli, Commands};
use commands::filter;

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Filter(args) => filter::run(args)
    };
}
