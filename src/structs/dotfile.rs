use crate::git::operations::{
    add_and_commit, checkout_ref, get_commit, get_head_hash, get_repo_dir, normal_merge,
};
use crate::utils::run_command_vec;
use crate::MANIFEST_PATH;
use console::style;
use git2::Repository;
use sha1::{Digest, Sha1};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use std::error::Error;

use crate::utils::hash_command_vec;

use super::{Config, DotfileMetadata};

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Dotfile {
    pub file: String,
    pub target: PathBuf,
    pub pre_install: Option<Vec<String>>,
    pub post_install: Option<Vec<String>>,
}

impl Dotfile {
    fn hash_pre_install(&self) -> String {
        if let Some(pre_install) = &self.pre_install {
            hash_command_vec(pre_install)
        } else {
            "".to_string()
        }
    }

    fn hash_post_install(&self) -> String {
        if let Some(post_install) = &self.post_install {
            hash_command_vec(post_install)
        } else {
            "".to_string()
        }
    }

    /// Return whether this dotfile has run stages, i.e. pre_install or post_install is not `None`
    /// and the hash of the pre/post install stages are different to the one in the metadata
    pub fn has_unexecuted_run_stages(&self, maybe_metadata: &Option<&DotfileMetadata>) -> bool {
        if let Some(metadata) = maybe_metadata {
            // If metadata is available, don't return true if the steps have already
            // been executed
            (self.pre_install.is_some() && metadata.pre_install_hash != self.hash_pre_install())
                || (self.post_install.is_some()
                    && metadata.post_install_hash != self.hash_post_install())
        } else {
            // Otherwise just depend on the presence of the steps
            self.pre_install.is_some() || self.post_install.is_some()
        }
    }

    fn run_pre_install(
        &self,
        metadata: &Option<DotfileMetadata>,
    ) -> Result<String, Box<dyn Error>> {
        let mut hash = String::new();

        if let Some(pre_install) = &self.pre_install {
            let mut skip_pre_install = false;

            if let Some(metadata) = metadata {
                if self.hash_pre_install() == metadata.pre_install_hash {
                    info!("{}", style("Skipping pre install steps as they have been run in a previous install").blue());
                    skip_pre_install = true;
                }
            }

            if !skip_pre_install {
                success!("Running pre-install steps");
                run_command_vec(pre_install)?;
                hash = self.hash_pre_install();
            }
        }
        Ok(hash)
    }

    fn run_post_install(
        &self,
        metadata: &Option<DotfileMetadata>,
    ) -> Result<String, Box<dyn Error>> {
        let mut hash = String::new();

        if let Some(post_install) = &self.post_install {
            let mut skip_post_install = false;

            if let Some(metadata) = metadata {
                if self.hash_post_install() == metadata.post_install_hash {
                    success!(
                        "Skipping post install steps as they have been run in a previous install"
                    );
                    skip_post_install = true;
                }
            }

            if !skip_post_install {
                success!("Running post-install steps");
                run_command_vec(post_install)?;
                hash = self.hash_post_install();
            }
        }
        Ok(hash)
    }

    fn install_dotfile(&self, repo_dir: &Path) -> Result<(), Box<dyn Error>> {
        let mut origin_path = repo_dir.to_path_buf();
        origin_path.push(&self.file);

        let unexpanded_target_path = &self.target.to_string_lossy();

        let target_path_str = shellexpand::tilde(unexpanded_target_path);

        let target_path = Path::new(target_path_str.as_ref());

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|_| "Unable to create parent directories".to_string())?;
        }
        fs::copy(origin_path, target_path).expect("Failed to copy target file");

        success!(
            "Installed config file {} to location {}",
            &self.file,
            target_path.to_str().expect("Invalid unicode in path")
        );

        Ok(())
    }

    /// Return whether this dotfile has changed since it was last synchronised
    ///
    /// This is performed by loading the current dotfile on the system, loading the dotfile as of
    /// the specified commit and comparing them byte by byte.
    ///
    /// # Arguments
    ///
    /// * `repo` - The repository object
    /// * `metadata` - The metadata associated to this dotfile
    ///
    /// # Returns
    ///
    /// A boolean signifying whether the dotfile on the local system differs to how it looked when
    /// last synced
    pub fn has_changed(
        &self,
        repo: &Repository,
        metadata: &DotfileMetadata,
    ) -> Result<bool, Box<dyn Error>> {
        let head_ref = repo.head()?;
        let head_ref_name = head_ref.name().unwrap();

        let unexpanded_target_path = &self.target.to_string_lossy();
        let local_dotfile_path = shellexpand::tilde(unexpanded_target_path).to_string();
        let dotfile_contents = fs::read_to_string(local_dotfile_path)?;
        let local_dotfile_hash = Sha1::digest(dotfile_contents.as_bytes());

        checkout_ref(repo, &metadata.sync_hash)?;

        let repo_dir = get_repo_dir(repo);
        let repo_dotfile_path = &repo_dir.join(&self.file);
        let dotfile_contents = fs::read_to_string(repo_dotfile_path)?;
        let repo_dotfile_hash = Sha1::digest(dotfile_contents.as_bytes());

        if local_dotfile_hash != repo_dotfile_hash {
            checkout_ref(repo, head_ref_name)?;
            Ok(true)
        } else {
            checkout_ref(repo, head_ref_name)?;
            Ok(false)
        }
    }

    /// Install the dotfile to the specified location.
    ///
    /// Refuse to do so if a local dotfile exists that has changes since the last sync, unless
    /// `force` is true.
    ///
    /// # Arguments
    ///
    /// * `repo` - The repository object
    /// * `maybe_metadata` - Optionally, this dotfiles metadata. If not passed, a naive install will
    /// be performed, meaning:
    ///   * No idempotency checks will be performed for pre/post steps
    ///   * No check can be made as to whether the dotfile has changed since last sync so it will
    ///   be overwritten no matter what
    /// * `skip_install_steps` - Whether to skip pre/post install steps
    /// * `force` - Whether to force the install, even if the local dotfile has changed since the
    /// last sync
    pub fn install(
        &self,
        repo: &Repository,
        maybe_metadata: Option<DotfileMetadata>,
        skip_install_steps: bool,
        force: bool,
    ) -> Result<DotfileMetadata, Box<dyn Error>> {
        let commit_hash = get_head_hash(repo)?;
        if !force {
            if let Some(ref metadata) = maybe_metadata {
                if self.has_changed(repo, metadata)? {
                    return Err("Refusing to install dotfile. Changes have been made since last sync. \
                            either run \"jtd sync\" for this dotfile or call install again with the \
                            \"--force\" flag".into());
                }
            }
        }

        let pre_install_hash = if !skip_install_steps {
            self.run_pre_install(&maybe_metadata)?
        } else {
            String::new()
        };

        let repo_dir = get_repo_dir(repo);
        self.install_dotfile(repo_dir)?;

        let post_install_hash = if !skip_install_steps {
            self.run_post_install(&maybe_metadata)?
        } else {
            String::new()
        };

        let new_metadata = DotfileMetadata::new(&commit_hash, &commit_hash, pre_install_hash, post_install_hash);

        Ok(new_metadata)
    }

    pub fn sync(
        &self,
        repo: &Repository,
        dotfile_name: &str,
        config: &Config,
        metadata: Option<&DotfileMetadata>,
    ) -> Result<DotfileMetadata, Box<dyn Error>> {
        let mut target_path_buf = get_repo_dir(repo).to_owned();
        target_path_buf.push(&self.file);
        let target_path = target_path_buf.as_path();

        let origin_path_unexpanded = &self.target.to_string_lossy();
        let origin_path_str = shellexpand::tilde(origin_path_unexpanded);
        let origin_path = Path::new(origin_path_str.as_ref());

        if let Some(metadata) = metadata {
            let mut new_metadata = metadata.clone();

            if self.has_changed(repo, metadata)? {
                let parent_commit = get_commit(repo, &metadata.install_hash).map_err(
                    |_| format!("Could not find last sync'd commit for {}, manifest is corrupt. Try fresh-installing \
                                this dotfile or manually correcting the commit hash in {}", dotfile_name, MANIFEST_PATH))?;

                let head_ref = repo.head()?;
                let head_ref_name = head_ref.name().unwrap();
                let merge_target_commit = repo.reference_to_annotated_commit(&head_ref)?;

                checkout_ref(repo, &parent_commit.id().to_string())?;
                fs::copy(origin_path, target_path)?;

                let new_branch_name = format!("merge-{}-dotfile", dotfile_name);
                let _new_branch = repo.branch(&new_branch_name, &parent_commit, true)?;
                checkout_ref(repo, &new_branch_name)?;

                let _new_commit = add_and_commit(
                    repo,
                    Some(vec![Path::new(&self.file)]),
                    &config.generate_commit_message(vec![dotfile_name]),
                    Some(vec![&parent_commit]),
                    Some("HEAD"),
                )?;

                let new_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
                checkout_ref(repo, head_ref_name)?;

                let merge_commit = normal_merge(repo, &merge_target_commit, &new_commit)
                    .map_err(|err| format!("Could not merge commits: {}", err))?;

                new_metadata.install_hash = merge_commit.id().to_string();
            } else {
                info!("Skipping syncing {} as no changes made", dotfile_name);
            }
            Ok(new_metadata)
        } else {
            fs::copy(origin_path, target_path)?;
            let new_commit = add_and_commit(
                repo,
                Some(vec![Path::new(&self.file)]),
                &config.generate_commit_message(vec![dotfile_name]),
                None,
                Some("HEAD"),
            )?;
            Ok(DotfileMetadata::new(
                &new_commit.id().to_string(),
                &new_commit.id().to_string(),
                self.hash_pre_install(),
                self.hash_post_install(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write, path::PathBuf};
    use tempfile::tempdir;

    use crate::git::operations::get_head;

    use super::*;

    #[test]
    fn test_hash_empty_pre_install() {
        let dotfile = Dotfile {
            file: "".to_string(),
            target: PathBuf::new(),
            pre_install: None,
            post_install: None,
        };

        assert_eq!("", dotfile.hash_pre_install());
    }

    #[test]
    fn test_hash_pre_install() {
        let dotfile = Dotfile {
            file: "".to_string(),
            target: PathBuf::new(),
            pre_install: Some(vec![
                "echo".to_string(),
                "ls".to_string(),
                "cat".to_string(),
            ]),
            post_install: None,
        };

        assert_eq!(
            "1ef98a8d0946d6512ca5da8242eb7a52a506de54",
            dotfile.hash_pre_install()
        );
    }

    #[test]
    fn test_hash_empty_post_install() {
        let dotfile = Dotfile {
            file: "".to_string(),
            target: PathBuf::new(),
            pre_install: None,
            post_install: None,
        };

        assert_eq!("", dotfile.hash_post_install());
    }

    #[test]
    fn test_hash_post_install() {
        let dotfile = Dotfile {
            file: "".to_string(),
            target: PathBuf::new(),
            pre_install: None,
            post_install: Some(vec![
                "echo".to_string(),
                "ls".to_string(),
                "cat".to_string(),
            ]),
        };

        assert_eq!(
            "1ef98a8d0946d6512ca5da8242eb7a52a506de54",
            dotfile.hash_post_install()
        );
    }

    #[test]
    fn test_has_unexecuted_run_stages_no_metadata() {
        let dotfile = Dotfile {
            file: "".to_string(),
            target: PathBuf::new(),
            pre_install: None,
            post_install: None,
        };

        assert!(!dotfile.has_unexecuted_run_stages(&None));
    }

    #[test]
    fn test_has_unexecuted_run_stages_with_metadata_no_install_steps() {
        let dotfile = Dotfile {
            file: "".to_string(),
            target: PathBuf::new(),
            pre_install: None,
            post_install: None,
        };

        let metadata = DotfileMetadata {
            install_hash: "".to_string(),
            sync_hash: "".to_string(),
            pre_install_hash: "".to_string(),
            post_install_hash: "".to_string(),
        };

        assert!(!dotfile.has_unexecuted_run_stages(&Some(&metadata)));
    }

    #[test]
    fn test_has_unexecuted_run_stages_with_metadata_with_install_steps_true() {
        let dotfile = Dotfile {
            file: "".to_string(),
            target: PathBuf::new(),
            pre_install: Some(vec![
                "echo".to_string(),
                "ls".to_string(),
                "cat".to_string(),
            ]),
            post_install: Some(vec![
                "echo".to_string(),
                "ls".to_string(),
                "cat".to_string(),
            ]),
        };

        let metadata = DotfileMetadata {
            install_hash: "".to_string(),
            sync_hash: "".to_string(),
            pre_install_hash: "".to_string(),
            post_install_hash: "".to_string(),
        };

        assert!(dotfile.has_unexecuted_run_stages(&Some(&metadata)));
    }

    #[test]
    fn test_has_unexecuted_run_stages_with_metadata_with_install_steps_false() {
        let dotfile = Dotfile {
            file: "".to_string(),
            target: PathBuf::new(),
            pre_install: Some(vec![
                "echo".to_string(),
                "ls".to_string(),
                "cat".to_string(),
            ]),
            post_install: Some(vec![
                "echo".to_string(),
                "ls".to_string(),
                "cat".to_string(),
            ]),
        };

        let metadata = DotfileMetadata {
            install_hash: "".to_string(),
            sync_hash: "".to_string(),
            pre_install_hash: "1ef98a8d0946d6512ca5da8242eb7a52a506de54".to_string(),
            post_install_hash: "1ef98a8d0946d6512ca5da8242eb7a52a506de54".to_string(),
        };

        assert!(!dotfile.has_unexecuted_run_stages(&Some(&metadata)));
    }

    #[test]
    fn test_has_changed_false() {
        let repo_dir = tempdir().expect("Could not create temporary repo dir");
        let dotfile_dir = tempdir().expect("Could not create temporary dotfile dir");
        let repo = Repository::init(&repo_dir).expect("Could not initialise repository");

        // Create file in repo
        let repo_filepath = repo_dir.path().to_owned().join("dotfile");
        File::create(repo_filepath.to_owned()).expect("Could not create file in repo");

        // Create dotfile "on the local system"
        let local_filepath = dotfile_dir.path().to_owned().join("dotfile");
        File::create(local_filepath).expect("Could not create file in tempdir");

        let commit = add_and_commit(
            &repo,
            Some(vec![&repo_filepath]),
            "commit message",
            Some(vec![]),
            Some("HEAD"),
        )
        .expect("Failed to commit to repository");

        let dotfile = Dotfile {
            file: "dotfile".to_string(),
            target: dotfile_dir.path().join("dotfile"),
            pre_install: None,
            post_install: None,
        };

        let metadata = DotfileMetadata {
            install_hash: commit.id().to_string(),
            sync_hash: commit.id().to_string(),
            pre_install_hash: "".to_string(),
            post_install_hash: "".to_string(),
        };

        assert!(!dotfile.has_changed(&repo, &metadata).unwrap());
    }

    #[test]
    fn test_has_changed_true() {
        let repo_dir = tempdir().expect("Could not create temporary repo dir");
        let dotfile_dir = tempdir().expect("Could not create temporary dotfile dir");
        let repo = Repository::init(&repo_dir).expect("Could not initialise repository");

        // Create file in repo
        let filepath = repo_dir.path().to_owned().join("dotfile");
        File::create(filepath).expect("Could not create file in repo");

        // Create dotfile "on the local system" with different contents
        let filepath = dotfile_dir.path().to_owned().join("dotfile");
        let mut dotfile_file =
            File::create(filepath.to_owned()).expect("Could not create file in tempdir");
        dotfile_file
            .write_all("This file has changes".as_bytes())
            .unwrap();

        let commit = add_and_commit(
            &repo,
            Some(vec![&filepath]),
            "commit message",
            Some(vec![]),
            Some("HEAD"),
        )
        .expect("Failed to commit to repository");

        let dotfile = Dotfile {
            file: "dotfile".to_string(),
            target: dotfile_dir.path().join("dotfile"),
            pre_install: None,
            post_install: None,
        };

        let metadata = DotfileMetadata {
            install_hash: commit.id().to_string(),
            sync_hash: commit.id().to_string(),
            pre_install_hash: "".to_string(),
            post_install_hash: "".to_string(),
        };

        assert!(dotfile.has_changed(&repo, &metadata).unwrap());
    }

    #[test]
    fn test_install_no_metadata() {
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

        let dotfile = Dotfile {
            file: "dotfile".to_string(),
            target: target_path.clone(),
            pre_install: None,
            post_install: None,
        };

        dotfile
            .install(&repo, None, true, true)
            .expect("Failed to install dotfile");

        assert!(Path::exists(&target_path));
    }

    #[test]
    fn test_install_commands() {
        let repo_dir = tempdir().expect("Could not create temporary repo dir");
        let repo = Repository::init(&repo_dir).expect("Could not initialise repository");

        let dotfile_dir = tempdir().expect("Could not create temporary dotfile dir");
        let target_path = dotfile_dir.path().join("dotfile");

        let target_touch_pre_install = dotfile_dir.path().join("pre_install");
        let target_touch_post_install = dotfile_dir.path().join("post_install");

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

        let dotfile = Dotfile {
            file: "dotfile".to_string(),
            target: target_path.clone(),
            pre_install: Some(vec![format!(
                "touch {}",
                target_touch_pre_install.to_string_lossy()
            )]),
            post_install: Some(vec![format!(
                "touch {}",
                target_touch_post_install.to_string_lossy()
            )]),
        };

        dotfile
            .install(&repo, None, false, true)
            .expect("Failed to install dotfile");

        assert!(Path::exists(&target_path));
        assert!(Path::exists(&target_touch_pre_install));
        assert!(Path::exists(&target_touch_post_install));
    }

    #[test]
    fn test_abort_install_if_local_changes() {
        let repo_dir = tempdir().expect("Could not create temporary repo dir");
        let repo = Repository::init(&repo_dir).expect("Could not initialise repository");

        let dotfile_dir = tempdir().expect("Could not create temporary dotfile dir");
        let target_path = dotfile_dir.path().join("dotfile");

        // Create file in repo
        let filepath = repo_dir.path().to_owned().join("dotfile");
        File::create(filepath.to_owned()).expect("Could not create file in repo");

        // Create dotfile "on the local system"
        let local_filepath = dotfile_dir.path().to_owned().join("dotfile");
        let mut file =
            File::create(local_filepath).expect("Could not create file in tempdir");
        file.write_all(b"These are local changes on the system")
            .expect("Failed to write to dotfile");

        let commit = add_and_commit(
            &repo,
            Some(vec![&filepath]),
            "commit message",
            Some(vec![]),
            Some("HEAD"),
        )
        .expect("Failed to commit to repository");

        let dotfile = Dotfile {
            file: "dotfile".to_string(),
            target: target_path,
            pre_install: None,
            post_install: None,
        };

        let metadata = DotfileMetadata {
            install_hash: commit.id().to_string(),
            sync_hash: commit.id().to_string(),
            pre_install_hash: "".to_string(),
            post_install_hash: "".to_string(),
        };

        assert!(dotfile.install(&repo, Some(metadata), true, false).is_err());
    }

    #[test]
    fn test_sync_naive() {
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

        // Create dotfile "on the local system"
        let local_filepath = dotfile_dir.path().to_owned().join("dotfile");
        let mut file =
            File::create(local_filepath).expect("Could not create file in tempdir");
        file.write_all(b"These are local changes on the system")
            .expect("Failed to write to dotfile");

        let dotfile = Dotfile {
            file: "dotfile".to_string(),
            target: target_path,
            pre_install: None,
            post_install: None,
        };

        let config = Config::default();

        dotfile
            .sync(&repo, "dotfile", &config, None)
            .expect("Failed to sync dotfile");
        assert_eq!(
            fs::read_to_string(filepath).unwrap(),
            "These are local changes on the system"
        );
    }

    #[test]
    fn test_sync_with_metadata() {
        let repo_dir = tempdir().expect("Could not create temporary repo dir");
        let repo = Repository::init(&repo_dir).expect("Could not initialise repository");

        let dotfile_dir = tempdir().expect("Could not create temporary dotfile dir");
        let target_path = dotfile_dir.path().join("dotfile");

        // Create file in repo
        let filepath = repo_dir.path().to_owned().join("dotfile");
        File::create(filepath.to_owned()).expect("Could not create file in repo");
        let commit = add_and_commit(
            &repo,
            Some(vec![&filepath]),
            "commit message",
            Some(vec![]),
            Some("HEAD"),
        )
        .expect("Failed to commit to repository");

        // Create dotfile "on the local system"
        let local_filepath = dotfile_dir.path().to_owned().join("dotfile");
        let mut file =
            File::create(local_filepath).expect("Could not create file in tempdir");
        file.write_all(b"These are local changes on the system")
            .expect("Failed to write to dotfile");

        let dotfile = Dotfile {
            file: "dotfile".to_string(),
            target: target_path,
            pre_install: None,
            post_install: None,
        };

        let metadata = DotfileMetadata {
            install_hash: commit.id().to_string(),
            sync_hash: commit.id().to_string(),
            pre_install_hash: "".to_string(),
            post_install_hash: "".to_string(),
        };

        let config = Config::default();

        dotfile
            .sync(&repo, "dotfile", &config, Some(&metadata))
            .expect("Failed to sync dotfile");
        assert_eq!(
            fs::read_to_string(filepath).unwrap(),
            "These are local changes on the system"
        );
    }

    #[test]
    fn test_sync_with_metadata_skip_if_no_changes() {
        let repo_dir = tempdir().expect("Could not create temporary repo dir");
        let repo = Repository::init(&repo_dir).expect("Could not initialise repository");

        let dotfile_dir = tempdir().expect("Could not create temporary dotfile dir");
        let target_path = dotfile_dir.path().join("dotfile");

        // Create file in repo
        let filepath = repo_dir.path().to_owned().join("dotfile");
        File::create(filepath.to_owned()).expect("Could not create file in repo");
        let commit = add_and_commit(
            &repo,
            Some(vec![&filepath]),
            "commit message",
            Some(vec![]),
            Some("HEAD"),
        )
        .expect("Failed to commit to repository");

        // Create dotfile "on the local system"
        let local_filepath = dotfile_dir.path().to_owned().join("dotfile");
        File::create(local_filepath).expect("Could not create file in tempdir");

        let dotfile = Dotfile {
            file: "dotfile".to_string(),
            target: target_path,
            pre_install: None,
            post_install: None,
        };

        let metadata = DotfileMetadata {
            install_hash: commit.id().to_string(),
            sync_hash: commit.id().to_string(),
            pre_install_hash: "".to_string(),
            post_install_hash: "".to_string(),
        };

        let config = Config::default();

        dotfile
            .sync(&repo, "dotfile", &config, Some(&metadata))
            .expect("Failed to sync dotfile");

        // Check that the head commit of the repo is still the initial commit - i.e. no changes
        // have been committed
        assert_eq!(commit.id(), get_head(&repo).unwrap().id());
    }
}
