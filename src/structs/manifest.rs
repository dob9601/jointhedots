use console::style;
use git2::Repository;
use serde::Deserialize;
use std::{error::Error, collections::HashMap, path::PathBuf, fs::{self, File}};
use dialoguer::{Confirm, MultiSelect};

use crate::{git::operations::{push, get_head_hash}, utils::get_theme};

use super::{Dotfile, AggregatedDotfileMetadata};

/// Represents an aggregation of [Dotfile]s, as found in the `jtd.yaml` file. This is done via a
/// mapping of `dotfile_name` to [Dotfile]
#[derive(Deserialize, Debug, Clone)]
pub struct Manifest {
    #[serde(flatten)]
    data: HashMap<String, Dotfile>,
}

impl Manifest {
    pub fn install(
        &self,
        repo: Repository,
        install_all: bool,
        target_dotfiles: Vec<String>,
        force_install: bool,
    ) -> Result<(), Box<dyn Error>> {
        let theme = get_theme();
        let head_hash = get_head_hash(&repo)?;

        let mut skip_install_commands = false;

        let dotfiles = self.get_target_dotfiles(target_dotfiles, install_all);
        let mut aggregated_metadata = AggregatedDotfileMetadata::get_or_create()?;

        if self.has_unexecuted_run_stages(
            Some(dotfiles.iter().map(|(v, _)| v.as_str()).collect()),
            &aggregated_metadata,
        ) {
            println!(
                "{}",
                style(
                    "! Some of the dotfiles being installed contain pre_install and/or post_install \
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

        for (dotfile_name, dotfile) in dotfiles {
            let mut origin_path_buf = PathBuf::from(&repo_dir);
            origin_path_buf.push(&dotfile.file);

            if dotfile.target.exists() && !force_install {
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
                .get(dotfile_name)
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

    fn get_target_dotfiles(
        &self,
        target_dotfiles: Vec<String>,
        all: bool,
    ) -> Vec<(&String, &Dotfile)> {
        let theme = get_theme();

        if all {
            self.data.iter().collect()
        } else if !target_dotfiles.is_empty() {
            self.data
                .iter()
                .filter(|(dotfile_name, _)| target_dotfiles.contains(dotfile_name))
                .collect()
        } else {
            let dotfile_names = &self
                .clone()
                .into_iter()
                .map(|pair| pair.0)
                .collect::<Vec<String>>();
            let selected = MultiSelect::with_theme(&theme)
                .with_prompt("Select the dotfiles you wish to install. Use \"SPACE\" to select and \"ENTER\" to proceed.")
                .items(dotfile_names)
                .interact()
                .unwrap();

            self.data
                .iter()
                .enumerate()
                .filter(|(index, (_, _))| selected.contains(index))
                .map(|(_, (name, dotfile))| (name, dotfile))
                .collect()
        }
    }

    /// Return whether this Manifest contains dotfiles containing unexecuted, potentially dangerous
    /// run stages. Optionally can take a vector of [Dotfile]s for testing a subset of the manifest.
    pub fn has_unexecuted_run_stages(
        &self,
        dotfile_names: Option<Vec<&str>>,
        metadata: &AggregatedDotfileMetadata,
    ) -> bool {
        let dotfile_names =
            dotfile_names.unwrap_or_else(|| self.data.keys().map(|k| k.as_str()).collect());

        self.data
            .iter()
            .filter(|(dotfile_name, _)| dotfile_names.contains(&dotfile_name.as_str()))
            .any(|(dotfile_name, dotfile)| {
                dotfile.has_unexecuted_run_stages(&metadata.data.get(dotfile_name))
            })
    }

    pub fn sync(
        &self,
        repo: Repository,
        sync_all: bool,
        target_dotfiles: Vec<String>,
        commit_msg: Option<&str>,
        aggregated_metadata: Option<AggregatedDotfileMetadata>,
    ) -> Result<(), Box<dyn Error>> {
        let theme = get_theme();

        let dotfiles = self.get_target_dotfiles(target_dotfiles, sync_all);
        let mut commits = vec![];

        // TODO: Sync should return commit objects as opposed to paths so that a vector can be
        // constructed from them and all commits can be squashed in 1 go
        if let Some(aggregated_metadata) = aggregated_metadata {

            for (dotfile_name, dotfile) in dotfiles.iter() {
                println!("Syncing {}", dotfile_name);
                let commit = dotfile.sync(
                    &repo,
                    dotfile_name,
                    aggregated_metadata.data.get(dotfile_name.as_str()),
                )?;

                commits.push(commit);
            }
        } else {
            println!(
                "{}",
                style(
                    "! Could not find any metadata on the currently installed dotfiles. Proceed with naive sync and overwrite remote files?"
                )
                .yellow()
            );
            if Confirm::with_theme(&theme)
                .with_prompt("Use naive sync?")
                .default(false)
                .wait_for_newline(true)
                .interact()
                .unwrap()
            {
                for (dotfile_name, dotfile) in dotfiles.iter() {
                    println!("Syncing {} naively", dotfile_name);
                    commits.push(dotfile.sync(&repo, dotfile_name, None)?);
                }
            } else {
                return Err("Aborting due to lack of dotfile metadata".into());
            }
        }

        // TODO: Squash commits
        let commit_msg = if let Some(message) = commit_msg {
            message.to_string()
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

        // add_and_commit(&repo, relative_paths, &commit_msg, None)?;

        push(&repo)?;

        println!("{}", style("✔ Successfully synced changes!").green());

        Ok(())
    }
}

impl IntoIterator for Manifest {
    type Item = (String, Dotfile);

    type IntoIter = std::collections::hash_map::IntoIter<String, Dotfile>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

