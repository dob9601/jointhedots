use std::error::Error;

use tempfile::tempdir;

use crate::{
    cli::SyncSubcommandArgs,
    git::{operations::clone_repo, remote::get_host_git_url},
    structs::AggregatedDotfileMetadata,
    utils::get_manifest,
};

pub fn sync_subcommand_handler(args: SyncSubcommandArgs) -> Result<(), Box<dyn Error>> {
    let url = get_host_git_url(&args.repository, &args.source, &args.method)?;
    let target_dir = tempdir()?;

    let repo = clone_repo(&url, target_dir.path())?;

    let mut manifest_path = target_dir.path().to_path_buf();
    manifest_path.push(args.manifest);

    let manifest = get_manifest(&manifest_path)?;

    manifest.sync(
        repo,
        args.all,
        args.target_dotfiles,
        args.commit_msg.as_deref(),
        AggregatedDotfileMetadata::get()?,
    )
}
