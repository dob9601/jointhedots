use std::error::Error;

use tempfile::tempdir;

use crate::cli::InstallSubcommandArgs;
use crate::git::operations::clone_repo;
use crate::git::remote::get_host_git_url;
use crate::utils::get_manifest;

pub fn install_subcommand_handler(args: InstallSubcommandArgs) -> Result<(), Box<dyn Error>> {
    let url = get_host_git_url(&args.repository, &args.source, &args.method)?;

    let target_dir = tempdir()?;
    let repo = clone_repo(&url, target_dir.path())?;
    let manifest = get_manifest(target_dir.path())?;

    manifest.install(repo, args.all, args.target_dotfiles, args.force)
}
