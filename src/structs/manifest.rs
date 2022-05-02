use console::style;
use dialoguer::{Confirm, MultiSelect};
use git2::{Oid, Repository};
use serde::Deserialize;
use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    path::{Path, PathBuf},
};

use crate::{
    git::operations::{add_and_commit, get_repo_dir, push},
    utils::get_theme,
};

use super::{AggregatedDotfileMetadata, Config, Dotfile};

/// Represents an aggregation of [Dotfile]s, as found in the `jtd.yaml` file. This is done via a
/// mapping of `dotfile_name` to [Dotfile]
#[derive(Deserialize, Debug, Clone)]
pub struct Manifest {
    #[serde(default, rename = ".config")]
    config: Config,

    #[serde(flatten)]
    data: HashMap<String, Dotfile>,
}

impl Manifest {
    pub fn get(path: &Path) -> Result<Manifest, Box<dyn Error>> {
        let config: Manifest = serde_yaml::from_reader(File::open(path).map_err(|_| {
            format!(
                "Could not find manifest {} in repository.",
                path.file_name()
                    .map(|v| v.to_string_lossy())
                    .unwrap_or_else(|| "N/A".into())
            )
        })?)
        .map_err(|err| format!("Could not parse manifest: {}", err))?;
        Ok(config)
    }

    pub fn install(
        &self,
        repo: &Repository,
        install_all: bool,
        target_dotfiles: Vec<String>,
        force_install: bool,
        trust: bool,
    ) -> Result<(), Box<dyn Error>> {
        let theme = get_theme();

        let mut skip_install_commands = false;

        let dotfiles = self.get_target_dotfiles(target_dotfiles, install_all);
        let mut aggregated_metadata = AggregatedDotfileMetadata::get_or_create()?;

        if !trust
            && self.has_unexecuted_run_stages(
                Some(dotfiles.iter().map(|(v, _)| v.as_str()).collect()),
                &aggregated_metadata,
            )
        {
            warn!(
                "Some of the dotfiles being installed contain pre_install and/or post_install \
                steps. If you do not trust this manifest, you can skip running them."
            );
            skip_install_commands = Confirm::with_theme(&theme)
                .with_prompt("Skip running pre/post install?")
                .default(false)
                .wait_for_newline(true)
                .interact()
                .unwrap();
        }

        let repo_dir = get_repo_dir(&repo);

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
                dotfile.install(&repo, maybe_metadata, skip_install_commands, force_install)?;

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
        use_naive_sync: bool,
    ) -> Result<(), Box<dyn Error>> {
        let theme = get_theme();

        let dotfiles = self.get_target_dotfiles(target_dotfiles, sync_all);
        let mut commit_hashes = vec![];

        if aggregated_metadata.is_none() && !use_naive_sync {
            println!(
                "{}",
                style(
                    "Could not find any metadata on the currently installed dotfiles. Proceed with naive sync and overwrite remote files?"
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
                &self.config,
                aggregated_metadata.data.get(dotfile_name.as_str()),
            )?;

            commit_hashes.push(new_metadata.commit_hash.to_owned());
            aggregated_metadata
                .data
                .insert((*dotfile_name).to_string(), new_metadata);
        }

        if self.config.squash_commits {
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
                repo.reset(target_commit.as_object(), git2::ResetType::Soft, None)?;

                let commit_msg = if let Some(message) = commit_msg {
                    message.to_string()
                } else {
                    self.config.generate_commit_message(
                        dotfiles
                            .iter()
                            .map(|(name, _)| name.as_str())
                            .collect::<Vec<&str>>(),
                    )
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
        } else {
            info!("Not squashing commits");
        }

        push(repo)?;

        success!("Successfully synced changes!");

        aggregated_metadata.save()?;
        Ok(())
    }

    pub fn diff(&self, repo: &Repository, target_dotfile: &str) -> Result<(), Box<dyn Error>> {
        let dotfile = self.data.get(target_dotfile).ok_or_else(|| format!("Target dotfile \"{}\" was not found in the manifest", target_dotfile))?;
        dotfile.diff(&repo)
    }
}

impl IntoIterator for Manifest {
    type Item = (String, Dotfile);

    type IntoIter = std::collections::hash_map::IntoIter<String, Dotfile>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{read_to_string, File},
        io::Write,
        path::{Path, PathBuf},
    };

    use super::*;
    use tempfile::tempdir;

    const SAMPLE_MANIFEST: &str = r"
kitty:
  file: dotfile
  target: ~/some/path/here
        ";

    #[test]
    fn test_manifest_get() {
        let tempdir = tempdir().unwrap();

        let path = tempdir.path().join(Path::new("manifest.yaml"));
        let mut manifest_file = File::create(path.to_owned()).unwrap();
        manifest_file.write(SAMPLE_MANIFEST.as_bytes()).unwrap();

        let manifest = Manifest::get(&path).unwrap();

        let kitty_dotfile = Dotfile {
            file: "dotfile".to_string(),
            target: PathBuf::from("~/some/path/here"),
            pre_install: None,
            post_install: None,
        };

        assert_eq!(manifest.data["kitty"], kitty_dotfile);
    }

    #[test]
    fn test_manifest_install() {
        let repo_dir = tempdir().expect("Could not create temporary repo dir");
        let repo = Repository::init(&repo_dir).expect("Could not initialise repository");

        let dotfile_dir = tempdir().expect("Could not create temporary dotfile dir");
        let target_path = dotfile_dir.path().join("dotfile");

        // Create file in repo
        let filepath = repo_dir.path().to_owned().join("dotfile");
        File::create(filepath.to_owned()).expect("Could not create file in repo");
        let _commit = add_and_commit(
            &repo,
            Some(vec![&filepath]),
            "commit message",
            Some(vec![]),
            Some("HEAD"),
        )
        .expect("Failed to commit to repository");

        let manifest: Manifest = serde_yaml::from_str(
            &SAMPLE_MANIFEST.replace("~/some/path/here", &target_path.to_string_lossy()),
        )
        .unwrap();

        manifest
            .install(&repo, true, vec![], true, false)
            .expect("Failed to install manifest");
        assert!(Path::exists(&target_path));
    }

    #[test]
    fn test_manifest_sync() {
        let repo_dir = tempdir().expect("Could not create temporary repo dir");
        let repo = Repository::init(&repo_dir).expect("Could not initialise repository");

        let dotfile_dir = tempdir().expect("Could not create temporary dotfile dir");
        let target_path = dotfile_dir.path().join("dotfile");

        // Create file in repo
        let repo_dotfile_path = repo_dir.path().join("dotfile");
        File::create(repo_dotfile_path.to_owned()).expect("Could not create file in repo");
        let _commit = add_and_commit(
            &repo,
            Some(vec![&repo_dotfile_path]),
            "commit message",
            Some(vec![]),
            Some("HEAD"),
        )
        .expect("Failed to commit to repository");

        // Create dotfile "on the local system"
        let mut file =
            File::create(target_path.to_owned()).expect("Could not create file in tempdir");
        file.write_all(b"These are local changes on the system")
            .expect("Failed to write to dotfile");

        let manifest: Manifest = serde_yaml::from_str(
            &SAMPLE_MANIFEST.replace("~/some/path/here", &target_path.to_string_lossy()),
        )
        .unwrap();

        let err = manifest
            .sync(&repo, true, vec![], None, None, true)
            .unwrap_err();

        // FIXME: This is a very dodgy test, maybe setup a mock repo for pushing to?
        assert_eq!(
            err.to_string(),
            "remote 'origin' does not exist; class=Config (7); code=NotFound (-3)"
        );

        assert_eq!(
            read_to_string(&repo_dotfile_path).unwrap(),
            "These are local changes on the system"
        );
    }
}
