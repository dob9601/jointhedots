use std::error::Error;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use crate::cli::InstallSubcommandArgs;
use crate::structs::Config;
use crate::utils::{run_command_vec, get_repo_host_ssh_url, clone_repo};
use crate::{REPO_DIR, MANIFEST_PATH};

pub fn install_subcommand_handler(args: InstallSubcommandArgs) -> Result<(), Box<dyn Error>>{
    let url = get_repo_host_ssh_url(&args.source)?.to_string() + &args.repository;

    clone_repo(Path::new(REPO_DIR), &url)?;

    let config: Config = serde_yaml::from_reader(File::open(MANIFEST_PATH)?).unwrap();

    for (dotfile_name, dotfile) in config.into_iter() {
        println!("Commencing install for {}", dotfile_name);

        if let Some(pre_install) = &dotfile.pre_install {
            println!("Running pre-install steps");
            run_command_vec(pre_install)?;
        }

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
