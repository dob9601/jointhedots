use std::{
    error::Error,
    fs::File,
    io::{self, Write},
    process::Command,
};

use console::style;
use dialoguer::{
    console::Style,
    theme::{ColorfulTheme, Theme}, Input, Password,
};
use git2_credentials::{CredentialHandler, CredentialUI};
use indicatif::{ProgressBar, ProgressStyle};
use tempfile::tempdir;

use crate::{structs::Manifest, MANIFEST_PATH};

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
pub fn clone_repo(url: &str) -> Result<git2::Repository, Box<dyn Error>> {
    // Clone the project.
    let repo_dir = tempdir()?;
    let mut cb = git2::RemoteCallbacks::new();
    let git_config = git2::Config::open_default().map_err(|err| format!("Could not open default git config: {}", err))?;
    let mut ch = CredentialHandler::new_with_ui(git_config, Box::new(CredentialUIDialoguer {}));
    cb.credentials(move |url, username, allowed| ch.try_next_credential(url, username, allowed));

    // clone a repository
    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(cb)
        .download_tags(git2::AutotagOption::All)
        .update_fetchhead(true);
    let dst = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(&dst.as_ref()).unwrap();
    let repo = git2::build::RepoBuilder::new()
        .fetch_options(fo)
        .clone(url, repo_dir.path()).map_err(|err| format!("Could not clone repo: {}", &err))?;

    println!("{}", style("✔ Successfully cloned repository!").green());

    Ok(repo)
}

pub fn get_manifest() -> Result<Manifest, Box<dyn Error>> {
    let config: Manifest = serde_yaml::from_reader(
        File::open(MANIFEST_PATH).map_err(|_| "Could not find manifest in repository.")?,
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

pub fn get_head_hash(target_dir: &str) -> Result<String, Box<dyn Error>> {
    let bytes = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(target_dir)
        .output()?
        .stdout;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

pub fn is_in_past(commit_hash: &str) -> Result<bool, Box<dyn Error>> {
    let command = Command::new("git")
        .args(["merge-base", "--is-ancestor", commit_hash, "HEAD"])
        .output()?;
    Ok(command.status.success())
}
