use std::{error::Error, process::Command};

pub fn run_command_vec(command_vec: &[String]) -> Result<(), Box<dyn Error>> {
    for command in command_vec.iter() {
        let command_vec: Vec<&str> = command.split(' ').collect();
        Command::new(command_vec[0])
            .args(&command_vec[1..])
            .spawn()?;
    }
    Ok(())
}
