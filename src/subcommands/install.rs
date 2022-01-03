use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use console::style;
use dialoguer::{Confirm, MultiSelect};

use crate::cli::InstallSubcommandArgs;
use crate::structs::Dotfile;
use crate::utils::{clone_repo, get_manifest, get_repo_host_ssh_url, get_theme, run_command_vec, get_head_hash};
use crate::REPO_DIR;

pub fn install_subcommand_handler(args: InstallSubcommandArgs) -> Result<(), Box<dyn Error>> {
    let url = get_repo_host_ssh_url(&args.source)?.to_string() + &args.repository;
    let theme = get_theme();

    clone_repo(Path::new(REPO_DIR), &url)?;

    let manifest = get_manifest()?;

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

    if dotfiles
        .iter()
        .any(|(_, dotfile)| dotfile.pre_install.is_some() || dotfile.post_install.is_some())
    {
        println!(
            "{}",
            style(
                "! This manifest contains pre_install and/or post_install \
                steps, are you sure you trust this manifest?"
            )
            .yellow()
        );
        let trust = Confirm::with_theme(&theme)
            .with_prompt("Trust this manifest?")
            .default(false)
            .wait_for_newline(true)
            .interact()
            .unwrap();

        if !trust {
            return Err("Aborting due to untrusted dotfile manifest".into());
        }
    }

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
            let (stdout, stderr) = run_command_vec(pre_install)?;
            println!("{}:\n| {}", style("Stdout").cyan(), stdout.replace("\n", "\n| "));
            println!("{}:\n| {}", style("Stderr").red(), stderr.replace("\n", "\n| "));
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
            let (stdout, stderr) = run_command_vec(post_install)?;
            println!("STDOUT:\n| {}", stdout.replace("\n", "\n| "));
            println!("STDERR:\n| {}", stderr.replace("\n", "\n| "));
        }
    }
    let data_path = shellexpand::tilde("~/.local/share/jointhedots/");
    fs::create_dir_all(data_path.as_ref())?;

    let mut head_file = File::create(data_path.to_string() + "HEAD")?;
    head_file.write_all(get_head_hash(REPO_DIR)?.as_bytes())?;

    Ok(())
}
