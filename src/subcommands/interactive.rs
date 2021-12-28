use console::style;
use dialoguer::{Confirm, Input, Select};
use regex::Regex;
use std::error::Error;

use crate::{cli::InstallSubcommandArgs, utils::get_theme};

use super::install_subcommand_handler;

pub fn interactive_subcommand_handler() -> Result<(), Box<dyn Error>> {
    println!("\
        Welcome to JTD! \n\
        This wizard will guide you through installing your preconfigured dotfiles repo. \n\
        If you haven't yet added a manifest to your dotfile repo, view the README for instructions on how to do so \n\n\
        \t{} https://github.com/dob9601/jointhedots \n\
        \t{} https://github.com/dob9601/dotfiles/blob/master/jtd.yaml
    ", style("README:").cyan(), style("Example Manifest:").cyan());

    let theme = get_theme();

    let repo_regex = Regex::new("[A-Za-z0-9]+/[A-Za-z0-9]+").unwrap();
    let repository = Input::with_theme(&theme)
        .with_prompt("Target Repository: ")
        .validate_with(|input: &String| {
            if repo_regex.is_match(input) {
                Ok(())
            } else {
                Err("Invalid repository passed, name should follow the format of owner/repo")
            }
        })
        .interact_text()
        .unwrap();

    let repo_sources = ["GitHub", "GitLab"];
    let source_index = Select::with_theme(&theme)
        .with_prompt("Repository Source: ")
        .default(0)
        .items(&repo_sources)
        .interact()
        .unwrap();

    let force = Confirm::with_theme(&theme)
        .with_prompt("Overwrite existing dotfiles")
        .default(false)
        .wait_for_newline(true)
        .interact()
        .unwrap();

    let install_args = InstallSubcommandArgs {
        repository,
        target_dotfiles: vec![],
        source: repo_sources[source_index].to_string(),
        force,
        trust: false
    };

    install_subcommand_handler(install_args)?;
    Ok(())
}
