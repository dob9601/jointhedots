use std::{
    error::Error,
    fs::File,
    io::{self, Write},
    process::Command,
};

use console::style;
use dialoguer::{
    console::Style,
    theme::{ColorfulTheme, Theme},
};
use git2::{Cred, RemoteCallbacks};
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

pub fn clone_repo(url: &str) -> Result<git2::Repository, Box<dyn Error>> {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(SPINNER_RATE);
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(SPINNER_FRAMES)
            .template("{spinner:.blue} {msg}"),
    );
    pb.set_message("Attempting to clone repository");

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key_from_agent(username_from_url.unwrap())
    });

    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(callbacks);

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);

    // Clone the project.
    let repo_dir = tempdir()?;
    let repo = builder.clone(url, repo_dir.path())?;
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
