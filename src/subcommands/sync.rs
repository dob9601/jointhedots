use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use console::style;
use tempfile::tempdir;

use crate::{
    cli::SyncSubcommandArgs,
    git::{
        operations::{add_and_commit, clone_repo, push},
        remote::get_host_git_url,
    },
    structs::Dotfile,
    utils::get_manifest,
};

pub fn sync_subcommand_handler(args: SyncSubcommandArgs) -> Result<(), Box<dyn Error>> {
    let url = get_host_git_url(&args.repository, &args.source, &args.method)?;
    let target_dir = tempdir()?;

    let repo = clone_repo(&url, target_dir.path())?;

    let manifest = get_manifest(target_dir.path())?;

    let dotfiles_iter = manifest.into_iter();
    let dotfiles: Vec<(String, Dotfile)> = if !args.target_dotfiles.is_empty() {
        dotfiles_iter
            .filter(|(dotfile_name, _)| args.target_dotfiles.contains(dotfile_name))
            .collect()
    } else {
        dotfiles_iter.collect()
    };

    let mut relative_paths = vec![];
    for (dotfile_name, dotfile) in dotfiles.iter() {
        println!("Syncing {}", dotfile_name);

        let mut target_path_buf = PathBuf::from(repo.path());
        target_path_buf.push(&dotfile.file);
        let target_path = target_path_buf.as_path();

        let origin_path_unexpanded = &dotfile.target.to_string_lossy();
        let origin_path_str = shellexpand::tilde(origin_path_unexpanded);
        let origin_path = Path::new(origin_path_str.as_ref());

        relative_paths.push(Path::new(&dotfile.file));

        fs::copy(origin_path, target_path)?;
    }

    let commit_msg = if let Some(message) = args.commit_msg {
        message
    } else {
        format!(
            "Sync dotfiles for {}",
            dotfiles
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<&str>>()
                .join(", ")
        )
    };

    add_and_commit(&repo, relative_paths, &commit_msg)?;

    push(&repo)?;

    println!("{}", style("✔ Successfully synced changes!").green());
    Ok(())
}
