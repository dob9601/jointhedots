use clap::Parser;
use jointhedots::{cli::JoinTheDots, subcommands};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    match JoinTheDots::parse() {
        JoinTheDots::Sync(args) => subcommands::sync_subcommand_handler(args),
        JoinTheDots::Install(args) => subcommands::install_subcommand_handler(args)
    }
}

