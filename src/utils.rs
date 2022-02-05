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
use sha1::{Digest, Sha1};

use crate::structs::Manifest;

pub const SPINNER_FRAMES: &[&str] = &[
    "⢀⠀", "⡀⠀", "⠄⠀", "⢂⠀", "⡂⠀", "⠅⠀", "⢃⠀", "⡃⠀", "⠍⠀", "⢋⠀", "⡋⠀", "⠍⠁", "⢋⠁", "⡋⠁", "⠍⠉", "⠋⠉",
    "⠋⠉", "⠉⠙", "⠉⠙", "⠉⠩", "⠈⢙", "⠈⡙", "⢈⠩", "⡀⢙", "⠄⡙", "⢂⠩", "⡂⢘", "⠅⡘", "⢃⠨", "⡃⢐", "⠍⡐", "⢋⠠",
    "⡋⢀", "⠍⡁", "⢋⠁", "⡋⠁", "⠍⠉", "⠋⠉", "⠋⠉", "⠉⠙", "⠉⠙", "⠉⠩", "⠈⢙", "⠈⡙", "⠈⠩", "⠀⢙", "⠀⡙", "⠀⠩",
    "⠀⢘", "⠀⡘", "⠀⠨", "⠀⢐", "⠀⡐", "⠀⠠", "⠀⢀", "⠀⡀", "  ", "  ",
];
pub const SPINNER_RATE: u64 = 48;
pub const INSTALLED_DOTFILES_MANIFEST_PATH: &str = "~/.local/share/jointhedots/manifest.yaml";

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

pub(crate) fn get_theme() -> impl Theme {
    ColorfulTheme {
        values_style: Style::new().yellow().dim(),
        ..ColorfulTheme::default()
    }
}

pub(crate) fn hash_command_vec(command_vec: &[String]) -> Result<String, Box<dyn Error>> {
    let mut hasher = Sha1::new();
    let bytes: Vec<u8> = command_vec.iter().map(|s| s.bytes()).flatten().collect();

    hasher.update(bytes);
    std::str::from_utf8(&hasher.finalize()[..])
        .map(|s| s.to_string())
        .map_err(|err| format!("Failed to convert hash to string: {}", err.to_string()).into())
}
