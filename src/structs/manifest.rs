use console::style;
use dialoguer::{Confirm, MultiSelect};
use git2::{Oid, Repository};
use serde::Deserialize;
use std::{collections::HashMap, error::Error, path::PathBuf};

use crate::{
    git::operations::{add_and_commit, checkout_ref, get_head_hash, push},
    utils::get_theme,
    MULTIPLE_DOTFILES_COMMIT_FORMAT,
};

use super::{AggregatedDotfileMetadata, Dotfile};

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

        aggregated_metadata.save()?;
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
        repo: &Repository,
        sync_all: bool,
        target_dotfiles: Vec<String>,
        commit_msg: Option<&str>,
        aggregated_metadata: Option<AggregatedDotfileMetadata>,
    ) -> Result<(), Box<dyn Error>> {
        let theme = get_theme();

        let dotfiles = self.get_target_dotfiles(target_dotfiles, sync_all);
        let mut commit_hashes = vec![];

        if aggregated_metadata.is_none() {
            println!(
                "{}",
                style(
                    "! Could not find any metadata on the currently installed dotfiles. Proceed with naive sync and overwrite remote files?"
                )
                .yellow()
            );
            if !Confirm::with_theme(&theme)
                .with_prompt("Use naive sync?")
                .default(false)
                .wait_for_newline(true)
                .interact()
                .unwrap()
            {
                return Err("Aborting due to lack of dotfile metadata".into());
            }
        }

        let mut aggregated_metadata = aggregated_metadata.unwrap_or_default();

        for (dotfile_name, dotfile) in dotfiles.iter() {
            println!("Syncing {}", dotfile_name);
            let new_metadata = dotfile.sync(
                repo,
                dotfile_name,
                aggregated_metadata.data.get(dotfile_name.as_str()),
            )?;

            commit_hashes.push(new_metadata.commit_hash.to_owned());
            aggregated_metadata
                .data
                .insert((*dotfile_name).to_string(), new_metadata);
        }

        // Commits[0] isn't necessarily the oldest commit, iterate through and get minimum by
        // time
        let first_commit = commit_hashes
            .iter()
            .filter_map(|hash| {
                let maybe_commit = repo.find_commit(Oid::from_str(hash).ok()?);
                maybe_commit.ok()
            })
            .min_by_key(|commit| commit.time());
        if let Some(first_commit) = first_commit {
            let target_commit = first_commit.parent(0)?;
            checkout_ref(repo, "HEAD")?;
            repo.reset(target_commit.as_object(), git2::ResetType::Soft, None)?;

            let commit_msg = if let Some(message) = commit_msg {
                message.to_string()
            } else {
                MULTIPLE_DOTFILES_COMMIT_FORMAT
                    .replace(
                        "{}",
                        &dotfiles
                            .iter()
                            .map(|(name, _)| name.as_str())
                            .collect::<Vec<&str>>()
                            .join(", "),
                    )
                    .chars() // Kinda cursed way to replace the last occurrence
                    .rev()
                    .collect::<String>()
                    .replacen(",", "dna ", 1)
                    .chars()
                    .rev()
                    .collect()
            };
            // FIXME: Don't commit if commit_hashes is empty
            let commit_hash = add_and_commit(repo, None, &commit_msg, None, Some("HEAD"))?
                .id()
                .to_string();
            for (dotfile_name, metadata) in aggregated_metadata.data.iter_mut() {
                if dotfiles
                    .iter()
                    .map(|(name, _dotfile)| name)
                    .any(|s| s == &dotfile_name)
                    || sync_all
                {
                    metadata.commit_hash = commit_hash.to_owned();
                }
            }
        }

        push(repo)?;

        println!("{}", style("✔ Successfully synced changes!").green());

        aggregated_metadata.save()?;
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
