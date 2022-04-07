use std::{
    error::Error,
    io::{self, Write},
    process::Command,
};

use console::style;
use dialoguer::{
    console::Style,
    theme::{ColorfulTheme, Theme},
};
use sha1::{Digest, Sha1};

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

        let command_vec: Vec<String> = command
            .split(' ')
            .map(|component| shellexpand::tilde(component).to_string())
            .collect();
        Command::new(command_vec[0].as_str())
            .args(&command_vec[1..])
            .spawn()?
            .wait_with_output()?;
    }
    Ok(())
}

#[cfg(not(tarpaulin_include))]
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
    use std::path::Path;

    use super::*;

    #[test]
    fn test_run_command_vec() {
        let path = Path::new("/tmp/test-jtd");
        let command_vec = vec![format!("touch {}", path.to_string_lossy())];
        run_command_vec(&command_vec).expect("Could not run command vec");
        assert!(Path::new("/tmp/test-jtd").exists());
    }

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
