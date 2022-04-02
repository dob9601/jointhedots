use crate::git::operations::{add_and_commit, get_commit, get_head, has_changes, normal_merge};
use crate::utils::run_command_vec;
use crate::MANIFEST_PATH;
use console::style;
use git2::Repository;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use std::error::Error;

use crate::utils::hash_command_vec;

use super::{Config, DotfileMetadata};

#[derive(Deserialize, Debug, Clone)]
pub struct Dotfile {
    pub file: String,
    pub target: PathBuf,
    pub pre_install: Option<Vec<String>>,
    pub post_install: Option<Vec<String>>,
}

impl Dotfile {
    fn hash_pre_install(&self) -> String {
        if let Some(pre_install) = &self.pre_install {
            // Unwrap is safe, hash will always be utf-8
            hash_command_vec(pre_install)
        } else {
            "".to_string()
        }
    }

    fn hash_post_install(&self) -> String {
        if let Some(post_install) = &self.post_install {
            // Unwrap is safe, hash will always be utf-8
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
                    println!("{}", style("  🛈 Skipping pre install steps as they have been run in a previous install").blue());
                    skip_pre_install = true;
                }
            }

            if !skip_pre_install {
                println!("{}", style("  ✔ Running pre-install steps").green());
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
                    println!("{}", style("  🛈 Skipping post install steps as they have been run in a previous install").blue());
                    skip_post_install = true;
                }
            }

            if !skip_post_install {
                println!("{}", style("  ✔ Running post-install steps").green());
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

        println!(
            "{}",
            style(format!(
                "  ✔ Installed config file {} to location {}",
                &self.file,
                target_path.to_str().expect("Invalid unicode in path")
            ))
            .green()
        );

        Ok(())
    }

    pub fn install(
        &self,
        repo_dir: &Path,
        metadata: Option<DotfileMetadata>,
        commit_hash: &str,
        skip_install_commands: bool,
    ) -> Result<DotfileMetadata, Box<dyn Error>> {
        let pre_install_hash = if !skip_install_commands {
            self.run_pre_install(&metadata)?
        } else {
            String::new()
        };

        self.install_dotfile(repo_dir)?;

        let post_install_hash = if !skip_install_commands {
            self.run_post_install(&metadata)?
        } else {
            String::new()
        };

        let new_metadata = DotfileMetadata::new(commit_hash, pre_install_hash, post_install_hash);

        Ok(new_metadata)
    }

    pub fn sync(
        &self,
        repo: &Repository,
        dotfile_name: &str,
        config: &Config,
        metadata: Option<&DotfileMetadata>,
    ) -> Result<DotfileMetadata, Box<dyn Error>> {
        // Safe to unwrap here, repo.path() points to .git folder. Path will always
        // have a component after parent.
        let mut target_path_buf = repo.path().parent().unwrap().to_owned();
        target_path_buf.push(&self.file);
        let target_path = target_path_buf.as_path();

        let origin_path_unexpanded = &self.target.to_string_lossy();
        let origin_path_str = shellexpand::tilde(origin_path_unexpanded);
        let origin_path = Path::new(origin_path_str.as_ref());

        if let Some(metadata) = metadata {
            let mut new_metadata = metadata.clone();
            let parent_commit = get_commit(repo, &metadata.commit_hash).map_err(
                |_| format!("Could not find last sync'd commit for {}, manifest is corrupt. Try fresh-installing \
                            this dotfile or manually correcting the commit hash in {}", dotfile_name, MANIFEST_PATH))?;
            let merge_target = get_head(repo)?;

            fs::copy(origin_path, target_path)?;
            if has_changes(repo)? {
                let new_commit = add_and_commit(
                    repo,
                    Some(vec![Path::new(&self.file)]),
                    &config.generate_commit_message(vec![dotfile_name]),
                    Some(vec![parent_commit]),
                    None,
                )?;

                let merge_commit = normal_merge(repo, &merge_target, &new_commit)?;

                new_metadata.commit_hash = merge_commit.id().to_string();
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
                self.hash_pre_install(),
                self.hash_post_install(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

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

        assert_eq!("1ef98a8d0946d6512ca5da8242eb7a52a506de54", dotfile.hash_pre_install());
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

        assert_eq!("1ef98a8d0946d6512ca5da8242eb7a52a506de54", dotfile.hash_post_install());
    }

    #[test]
    fn test_has_unexecuted_run_stages_no_metadata() {
        let dotfile = Dotfile {
            file: "".to_string(),
            target: PathBuf::new(),
            pre_install: None,
            post_install: None,
        };

        assert_eq!(false, dotfile.has_unexecuted_run_stages(&None));
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
            commit_hash: "".to_string(),
            pre_install_hash: "".to_string(),
            post_install_hash: "".to_string(),
        };

        assert_eq!(false, dotfile.has_unexecuted_run_stages(&Some(&metadata)));
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
            commit_hash: "".to_string(),
            pre_install_hash: "".to_string(),
            post_install_hash: "".to_string(),
        };

        assert_eq!(true, dotfile.has_unexecuted_run_stages(&Some(&metadata)));
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
            commit_hash: "".to_string(),
            pre_install_hash: "1ef98a8d0946d6512ca5da8242eb7a52a506de54".to_string(),
            post_install_hash: "1ef98a8d0946d6512ca5da8242eb7a52a506de54".to_string(),
        };

        assert_eq!(false, dotfile.has_unexecuted_run_stages(&Some(&metadata)));
    }
}
