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

        let expanded_command = shellexpand::tilde(command);
        let command_vec: Vec<&str> = expanded_command.split(' ').collect();
        Command::new(command_vec[0])
            .args(&command_vec[1..])
            .spawn()?
            .wait_with_output()?;
    }
    Ok(())
}

pub fn get_manifest(manifest_path: &Path) -> Result<Manifest, Box<dyn Error>> {
    let config: Manifest = serde_yaml::from_reader(File::open(manifest_path).map_err(|_| {
        format!(
            "Could not find manifest {} in repository.",
            manifest_path
                .file_name()
                .map(|v| v.to_string_lossy())
                .unwrap_or_else(|| "N/A".into())
        )
    })?)
    .map_err(|_| "Could not parse manifest.")?;
    Ok(config)
}

pub(crate) fn get_theme() -> impl Theme {
    ColorfulTheme {
        values_style: Style::new().yellow().dim(),
        ..ColorfulTheme::default()
    }
}

pub(crate) fn hash_command_vec(command_vec: &[String]) -> String {
    let mut hasher = Sha1::new();
    let bytes: Vec<u8> = command_vec.iter().map(|s| s.bytes()).flatten().collect();

    hasher.update(bytes);
    hex::encode(&hasher.finalize()[..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_command_vec() {
        let command_vec = vec![
            String::from("echo \"Hi!\""),
            String::from("echo \"This is a vector of shell commands!\""),
            String::from("echo \"Farewell!\""),
        ];

        assert_eq!(
            hash_command_vec(&command_vec),
            "b51a85b8eeee922159d23463ffc057ab25fbaf9b".to_string()
        );
    }
}
