use std::{
    error::Error,
    fs::File,
    io::{self, Write},
    path::Path,
    process::Command,
};

use console::style;
use dialoguer::{
    console::Style,
    theme::{ColorfulTheme, Theme},
};

use crate::structs::Manifest;

pub enum ConnectionMethod {
    SSH,
    HTTPS
}

struct RepoHost {
    ssh_prefix: &'static str,
    https_prefix: &'static str
}

const GITLAB: RepoHost = RepoHost {
    ssh_prefix: "git@gitlab.com:",
    https_prefix: "https://gitlab.com/"
};

const GITHUB: RepoHost = RepoHost {
    ssh_prefix: "git@github.com:",
    https_prefix: "https://github.com/"
};

pub fn get_host_git_url(host: &str, method: ConnectionMethod) -> Result<&str, Box<dyn Error>> {
    let repo_host = match host.to_lowercase().as_str() {
        "github" => GITHUB,
        "gitlab" => GITLAB,
        _ => Err("Provided host unknown")?,
    };

    match method {
        ConnectionMethod::SSH => Ok(repo_host.ssh_prefix),
        ConnectionMethod::HTTPS => Ok(repo_host.https_prefix),
    }
}

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
