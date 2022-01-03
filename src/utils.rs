use std::{
    error::Error,
    fs::{self, File},
    path::Path,
    process::Command,
};

use console::style;
use dialoguer::{
    console::Style,
    theme::{ColorfulTheme, Theme},
};
use indicatif::{ProgressStyle, ProgressBar};

use crate::{structs::Manifest, MANIFEST_PATH};

pub const GITHUB_SSH_URL_PREFIX: &str = "git@github.com:";
pub const GITLAB_SSH_URL_PREFIX: &str = "git@gitlab.com:";

pub const SPINNER_FRAMES: &[&str] = &[
    "⢀⠀", "⡀⠀", "⠄⠀", "⢂⠀", "⡂⠀", "⠅⠀", "⢃⠀", "⡃⠀", "⠍⠀", "⢋⠀", "⡋⠀", "⠍⠁", "⢋⠁", "⡋⠁", "⠍⠉", "⠋⠉",
    "⠋⠉", "⠉⠙", "⠉⠙", "⠉⠩", "⠈⢙", "⠈⡙", "⢈⠩", "⡀⢙", "⠄⡙", "⢂⠩", "⡂⢘", "⠅⡘", "⢃⠨", "⡃⢐", "⠍⡐", "⢋⠠",
    "⡋⢀", "⠍⡁", "⢋⠁", "⡋⠁", "⠍⠉", "⠋⠉", "⠋⠉", "⠉⠙", "⠉⠙", "⠉⠩", "⠈⢙", "⠈⡙", "⠈⠩", "⠀⢙", "⠀⡙", "⠀⠩",
    "⠀⢘", "⠀⡘", "⠀⠨", "⠀⢐", "⠀⡐", "⠀⠠", "⠀⢀", "⠀⡀", "  ", "  "
];
pub const SPINNER_RATE: u64 = 48;

pub fn run_command_vec(command_vec: &[String]) -> Result<(String, String), Box<dyn Error>> {
    let mut stdout = String::new();
    let mut stderr = String::new();

    for (stage, command) in command_vec.iter().enumerate() {
        println!("{} {}", style(format!("Running stage #{}:", stage)).cyan(), command);
        let command_vec: Vec<&str> = command.split(' ').collect();
        let output = Command::new(command_vec[0])
            .args(&command_vec[1..])
            .output()?;
        stdout.push_str(&String::from_utf8_lossy(&output.stdout));
        stderr.push_str(&String::from_utf8_lossy(&output.stderr));
    }
    Ok((stdout, stderr))
}

pub fn get_repo_host_ssh_url(host: &str) -> Result<&str, Box<dyn Error>> {
    match host.to_lowercase().as_str() {
        "github" => Ok(GITHUB_SSH_URL_PREFIX),
        "gitlab" => Ok(GITLAB_SSH_URL_PREFIX),
        _ => Err("Provided host unknown".into()),
    }
}

pub fn clone_repo(target_dir: &Path, url: &str) -> Result<(), Box<dyn Error>> {
    if Path::new(target_dir).exists() {
        fs::remove_dir_all(target_dir).map_err(|err| format!("Could not clear temporary directory: {}", err))?;
    }
    fs::create_dir_all(target_dir).map_err(|err| format!("Could not create temporary directory: {}", err))?;

    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(SPINNER_RATE);
    pb.set_style(
        ProgressStyle::default_spinner()
            // For more spinners check out the cli-spinners project:
            // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
            .tick_strings(SPINNER_FRAMES)
            .template("{spinner:.blue} {msg}"),
    );
    pb.set_message("Attempting to clone repository");
    let output = Command::new("git")
        .arg("clone")
        .arg(url)
        .arg(".")
        .current_dir(target_dir)
        .output()?;

    if !output.status.success() {
        pb.finish();
        let output = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to clone repository: {}", output).into());
    }
    pb.finish_and_clear();
    println!("{}", style("✔ Successfully cloned repository!").green());
    Ok(())
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
    let bytes = Command::new("git").args(["rev-parse", "HEAD"])
        .current_dir(target_dir)
        .output()?.stdout;
    Ok(String::from_utf8_lossy(&bytes).to_string())

}

pub fn is_in_past(commit_hash: &str) -> Result<bool, Box<dyn Error>> {
    let command = Command::new("git").args(["merge-base", "--is-ancestor", commit_hash, "HEAD"]).output()?;
    Ok(command.status.success())
}
