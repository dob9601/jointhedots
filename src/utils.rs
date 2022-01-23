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
pub struct CredentialUIDialoguer;

impl CredentialUI for CredentialUIDialoguer {
    fn ask_user_password(&self, username: &str) -> Result<(String, String), Box<dyn Error>> {
        let theme = get_theme();
        let user: String = Input::with_theme(&theme)
            .default(username.to_owned())
            .with_prompt("Username")
            .interact()?;
        let password: String = Password::with_theme(&theme)
            .with_prompt("password (hidden)")
            .allow_empty_password(true)
            .interact()?;
        Ok((user, password))
    }

    fn ask_ssh_passphrase(&self, passphrase_prompt: &str) -> Result<String, Box<dyn Error>> {
        let passphrase: String = Password::with_theme(&get_theme())
            .with_prompt(format!("{} (leave blank for no password): ", passphrase_prompt))
            .allow_empty_password(true)
            .interact()?;
        Ok(passphrase)
    }
}
pub fn clone_repo(url: &str, target_dir: &Path) -> Result<git2::Repository, Box<dyn Error>> {
    // Clone the project.
    let mut cb = git2::RemoteCallbacks::new();
    let git_config = git2::Config::open_default().map_err(|err| format!("Could not open default git config: {}", err))?;
    let mut ch = CredentialHandler::new_with_ui(git_config, Box::new(CredentialUIDialoguer {}));
    cb.credentials(move |url, username, allowed| ch.try_next_credential(url, username, allowed));

    // clone a repository
    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(cb)
        .download_tags(git2::AutotagOption::All)
        .update_fetchhead(true);
    let repo = git2::build::RepoBuilder::new()
        .fetch_options(fo)
        .clone(url, target_dir).map_err(|err| format!("Could not clone repo: {}", &err))?;

    println!("{}", style("✔ Successfully cloned repository!").green());

    Ok(repo)
}
// is this italic

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

pub fn is_in_past(commit_hash: &str) -> Result<bool, Box<dyn Error>> {
    let command = Command::new("git")
        .args(["merge-base", "--is-ancestor", commit_hash, "HEAD"])
        .output()?;
    Ok(command.status.success())
}
