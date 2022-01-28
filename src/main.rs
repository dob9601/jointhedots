use clap::Parser;
use console::style;
use jointhedots::{cli::JoinTheDots, subcommands};
use std::process::exit;

fn main() {
    let result = match JoinTheDots::parse() {
        JoinTheDots::Sync(args) => subcommands::sync_subcommand_handler(args),
        JoinTheDots::Install(args) => subcommands::install_subcommand_handler(args),
        JoinTheDots::Interactive(_) => subcommands::interactive_subcommand_handler(),
    };
    if let Err(error) = result {
        println!(
            "{} {}",
            style("Error:").red().dim(),
            error.to_string().replace("\n", "\n       ")
        );
        exit(1);
    }
}
