use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use dialoguer::Confirm;

use crate::cli::InstallSubcommandArgs;
use crate::structs::Dotfile;
use crate::utils::{clone_repo, get_manifest, get_repo_host_ssh_url, run_command_vec, get_theme};
use crate::REPO_DIR;

pub fn install_subcommand_handler(args: InstallSubcommandArgs) -> Result<(), Box<dyn Error>> {
    let url = get_repo_host_ssh_url(&args.source)?.to_string() + &args.repository;
    let theme = get_theme();

    clone_repo(Path::new(REPO_DIR), &url)?;

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
        let mut origin_path_buf = PathBuf::from(REPO_DIR);
        origin_path_buf.push(&dotfile.file);
        let origin_path = origin_path_buf.as_path();

        let target_path_str = shellexpand::tilde(
            dotfile
                .target
                .to_str()
                .expect("Invalid unicode in target path"),
        );
        let target_path = Path::new(target_path_str.as_ref());

        if target_path.exists() && !args.force {
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

        if let Some(pre_install) = &dotfile.pre_install {
            println!("Running pre-install steps");
            run_command_vec(pre_install)?;
        }

        println!(
            "Installing config file {} to location {}",
            &dotfile.file,
            target_path.to_str().expect("Invalid unicode in path")
        );

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|_| "Unable to create parent directories".to_string())?;
        }
        fs::copy(origin_path, target_path).expect("Failed to copy target file");

        if let Some(post_install) = &dotfile.post_install {
            println!("Running post-install steps");
            run_command_vec(post_install)?;
        }
    }
    Ok(())
}
