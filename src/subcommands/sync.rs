use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    cli::SyncSubcommandArgs,
    structs::Dotfile,
    utils::{get_repo_host_ssh_url, clone_repo, get_manifest},
};

pub fn sync_subcommand_handler(args: SyncSubcommandArgs) -> Result<(), Box<dyn Error>> {
    let url = get_repo_host_ssh_url(&args.source)?.to_string() + &args.repository;

    let repo = clone_repo(&url)?;

    let manifest = get_manifest()?;

    let dotfiles_iter = manifest.into_iter();
    let dotfiles: Vec<(String, Dotfile)> = if !args.target_dotfiles.is_empty() {
        dotfiles_iter
            .filter(|(dotfile_name, _)| args.target_dotfiles.contains(dotfile_name))
            .collect()
    } else {
        dotfiles_iter.collect()
    };

    for (dotfile_name, dotfile) in dotfiles {
        println!("Syncing {}", dotfile_name);

        let mut target_path_buf = PathBuf::from(repo.path());
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

    Command::new("git").arg("add").arg("-A").current_dir(repo.path()).status()?;
    Command::new("git").arg("commit").args(["-m", "JTD Sync"]).current_dir(repo.path()).status()?;

    Command::new("git")
        .arg("push")
        .current_dir(repo.path())
        .status()?;
    Ok(())
}
