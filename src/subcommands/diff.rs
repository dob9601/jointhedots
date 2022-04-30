use std::error::Error;

use tempfile::tempdir;

use crate::{
    cli::DiffSubcommandArgs,
    git::{operations::clone_repo, remote::get_host_git_url},
    structs::Manifest,
};

pub fn diff_subcommand_handler(args: DiffSubcommandArgs) -> Result<(), Box<dyn Error>> {
    let url = get_host_git_url(&args.repository, &args.source, &args.method)?;
    let target_dir = tempdir()?;

    let repo = clone_repo(&url, target_dir.path())?;

    let mut manifest_path = target_dir.path().to_path_buf();
    manifest_path.push(args.manifest);

    let manifest = Manifest::get(&manifest_path)?;

    manifest.diff(&repo, &args.target_dotfile)
}
