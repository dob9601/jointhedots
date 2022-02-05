use std::error::Error;
use std::fs::{self, File};
use std::path::PathBuf;

use console::style;
use dialoguer::{Confirm, MultiSelect};
use tempfile::tempdir;

use crate::cli::InstallSubcommandArgs;
use crate::git::operations::{clone_repo, get_head_hash};
use crate::git::remote::get_host_git_url;
use crate::structs::{AggregatedDotfileMetadata, Dotfile};
use crate::utils::{get_manifest, get_theme};

pub fn install_subcommand_handler(args: InstallSubcommandArgs) -> Result<(), Box<dyn Error>> {
    let url = get_host_git_url(&args.repository, &args.source, &args.method)?;
    let theme = get_theme();
    let target_dir = tempdir()?;

    let repo = clone_repo(&url, target_dir.path())?;
    let head_hash = get_head_hash(&repo)?;

    let manifest = get_manifest(target_dir.path())?;

    let dotfiles: Vec<(String, Dotfile)> = if args.all {
        manifest.into_iter().collect()
    } else if !args.target_dotfiles.is_empty() {
        manifest
            .into_iter()
            .filter(|(dotfile_name, _)| args.target_dotfiles.contains(dotfile_name))
            .collect()
    } else {
        let dotfile_names = manifest
            .clone()
            .into_iter()
            .map(|pair| pair.0)
            .collect::<Vec<String>>();
        let selected = MultiSelect::with_theme(&theme)
            .with_prompt("Select the dotfiles you wish to install. Use \"SPACE\" to select and \"ENTER\" to proceed.")
            .items(&dotfile_names)
            .interact()
            .unwrap();

        manifest
            .into_iter()
            .enumerate()
            .filter(|(index, (_, _))| selected.contains(index))
            .map(|(_, (name, dotfile))| (name, dotfile))
            .collect()
    };

    let mut skip_install_commands = true;
    if dotfiles
        .iter()
        .any(|(_, dotfile)| dotfile.pre_install.is_some() || dotfile.post_install.is_some())
    {
        println!(
            "{}",
            style(
                "! This manifest contains pre_install and/or post_install \
                steps. If you do not trust this manifest, you can skip running them."
            )
            .yellow()
        );
        skip_install_commands = Confirm::with_theme(&theme)
            .with_prompt("Skip running pre/post install?")
            .default(false)
            .wait_for_newline(true)
            .interact()
            .unwrap();
    }

    // Safe to unwrap here, repo.path() points to .git folder. Path will always
    // have a component after parent.
    let repo_dir = repo.path().parent().unwrap().to_owned();

    let mut aggregated_metadata = AggregatedDotfileMetadata::get_current()?;
    for (dotfile_name, dotfile) in dotfiles {
        let mut origin_path_buf = PathBuf::from(&repo_dir);
        origin_path_buf.push(&dotfile.file);

        if dotfile.target.exists() && !args.force {
            let force = Confirm::with_theme(&theme)
                .with_prompt(format!(
                    "Dotfile \"{}\" already exists on disk. Overwrite?",
                    dotfile_name
                ))
                .default(false)
                .interact()
                .unwrap();
            if !force {
                continue;
            }
        }

        println!("Commencing install for {}", dotfile_name);

        let maybe_metadata = aggregated_metadata
            .data
            .get(&dotfile_name)
            .map(|d| (*d).clone());

        let metadata =
            dotfile.install(&repo_dir, maybe_metadata, &head_hash, skip_install_commands)?;

        aggregated_metadata
            .data
            .insert(dotfile_name.to_string(), metadata);
    }

    let data_path = shellexpand::tilde("~/.local/share/jointhedots/");
    fs::create_dir_all(data_path.as_ref())?;

    let output_manifest_file = File::create(data_path.as_ref().to_owned() + "manifest.yaml")?;
    serde_yaml::to_writer(output_manifest_file, &aggregated_metadata)?;

    Ok(())
}
