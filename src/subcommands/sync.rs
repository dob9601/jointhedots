use std::{
    error::Error,
    fs::{self, File},
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    cli::SyncSubcommandArgs,
    structs::Config,
    utils::{get_repo_host_ssh_url, clone_repo},
    MANIFEST_PATH, REPO_DIR,
};

pub fn sync_subcommand_handler(args: SyncSubcommandArgs) -> Result<(), Box<dyn Error>> {
    let url = get_repo_host_ssh_url(&args.source)?.to_string() + &args.repository;

    clone_repo(Path::new(REPO_DIR), &url)?;

    let config: Config = serde_yaml::from_reader(File::open(MANIFEST_PATH)?).unwrap();

    for (dotfile_name, dotfile) in config.into_iter() {
        println!("Syncing {}", dotfile_name);

        let mut target_path_buf = PathBuf::from(REPO_DIR);
        target_path_buf.push(&dotfile.file);
        let target_path = target_path_buf.as_path();

        let origin_path_str = shellexpand::tilde(
            dotfile
                .target
                .to_str()
                .expect("Invalid unicode in target path"),
        );
        let origin_path = Path::new(origin_path_str.as_ref());

        fs::copy(origin_path, target_path)?;
    }

    Command::new("git").arg("add").arg("-A").current_dir(REPO_DIR).status()?;
    Command::new("git").arg("commit").args(["-m", "JTD Sync"]).current_dir(REPO_DIR).status()?;

    Command::new("git")
        .arg("push")
        .current_dir(REPO_DIR)
        .status()?;
    Ok(())
}
