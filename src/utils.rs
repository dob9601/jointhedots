use std::{
    error::Error,
    fs::File,
    io::{self, Write},
    process::Command, path::Path,
};

use console::style;
use dialoguer::{
    console::Style,
    theme::{ColorfulTheme, Theme}, Input, Password,
};
use git2_credentials::{CredentialHandler, CredentialUI};

use crate::structs::Manifest;

pub const GITHUB_SSH_URL_PREFIX: &str = "git@github.com:";
pub const GITLAB_SSH_URL_PREFIX: &str = "git@gitlab.com:";

pub const SPINNER_FRAMES: &[&str] = &[
    "⢀⠀", "⡀⠀", "⠄⠀", "⢂⠀", "⡂⠀", "⠅⠀", "⢃⠀", "⡃⠀", "⠍⠀", "⢋⠀", "⡋⠀", "⠍⠁", "⢋⠁", "⡋⠁", "⠍⠉", "⠋⠉",
    "⠋⠉", "⠉⠙", "⠉⠙", "⠉⠩", "⠈⢙", "⠈⡙", "⢈⠩", "⡀⢙", "⠄⡙", "⢂⠩", "⡂⢘", "⠅⡘", "⢃⠨", "⡃⢐", "⠍⡐", "⢋⠠",
    "⡋⢀", "⠍⡁", "⢋⠁", "⡋⠁", "⠍⠉", "⠋⠉", "⠋⠉", "⠉⠙", "⠉⠙", "⠉⠩", "⠈⢙", "⠈⡙", "⠈⠩", "⠀⢙", "⠀⡙", "⠀⠩",
    "⠀⢘", "⠀⡘", "⠀⠨", "⠀⢐", "⠀⡐", "⠀⠠", "⠀⢀", "⠀⡀", "  ", "  ",
];
pub const SPINNER_RATE: u64 = 48;

pub fn run_command_vec(command_vec: &[String]) -> Result<(), Box<dyn Error>> {
    for (stage, command) in command_vec.iter().enumerate() {
        println!("{} {}", style(format!("Step #{}:", stage)).cyan(), command);
        io::stdout().flush()?;

        let command_vec: Vec<&str> = command.split(' ').collect();
        Command::new(command_vec[0])
            .args(&command_vec[1..])
            .spawn()?
            .wait_with_output()?;
    }
    Ok(())
}

pub fn get_repo_host_ssh_url(host: &str) -> Result<&str, Box<dyn Error>> {
    match host.to_lowercase().as_str() {
        "github" => Ok(GITHUB_SSH_URL_PREFIX),
        "gitlab" => Ok(GITLAB_SSH_URL_PREFIX),
        _ => Err("Provided host unknown".into()),
    }
}
pub fn get_manifest(target_dir: &Path) -> Result<Manifest, Box<dyn Error>> {
    let mut path = target_dir.to_owned();
    path.push("jtd.yaml");

    let config: Manifest = serde_yaml::from_reader(
        File::open(path).map_err(|_| "Could not find manifest in repository.")?,
    )
    .map_err(|_| "Could not parse manifest.")?;
    Ok(config)
}

pub fn get_theme() -> impl Theme {
    ColorfulTheme {
        values_style: Style::new().yellow().dim(),
        ..ColorfulTheme::default()
    }
}
