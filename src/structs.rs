use std::error::Error;
use std::fs::{File, self};
use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};

use crate::utils::{hash_command_vec, INSTALLED_DOTFILES_MANIFEST_PATH, run_command_vec};

#[derive(Deserialize, Debug, Clone)]
pub struct Manifest {
    #[serde(flatten)]
    data: HashMap<String, Dotfile>,
}

impl IntoIterator for Manifest {
    type Item = (String, Dotfile);

    type IntoIter = std::collections::hash_map::IntoIter<String, Dotfile>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Dotfile {
    pub file: String,
    pub target: Box<Path>,
    pub pre_install: Option<Vec<String>>,
    pub post_install: Option<Vec<String>>,
}

impl Dotfile {
    pub fn hash_pre_install(&self) -> String {
        if let Some(pre_install) = &self.pre_install {
            // Unwrap is safe, hash will always be utf-8
            hash_command_vec(pre_install).unwrap()
        } else {
            "".to_string()
        }
    }

    pub fn hash_post_install(&self) -> String {
        if let Some(post_install) = &self.post_install {
            // Unwrap is safe, hash will always be utf-8
            hash_command_vec(post_install).unwrap()
        } else {
            "".to_string()
        }
    }

    fn run_pre_install(&self, metadata: &Option<DotfileMetadata>) -> Result<(), Box<dyn Error>> {
        if let Some(pre_install) = &self.pre_install {
            let mut skip_pre_install = false;

            if let Some(metadata) = metadata {
                if self.hash_pre_install() == metadata.pre_install_hash {
                    skip_pre_install = true;
                }
            }

            if !skip_pre_install {
                println!("Running pre-install steps");
                run_command_vec(pre_install)?;
            }
        }
        Ok(())
    }

    fn run_post_install(&self, metadata: &Option<DotfileMetadata>) -> Result<(), Box<dyn Error>> {
        if let Some(post_install) = &self.post_install {
            let mut skip_post_install = false;

            if let Some(metadata) = metadata {
                if self.hash_post_install() == metadata.post_install_hash {
                    skip_post_install = true;
                }
            }

            if !skip_post_install {
                println!("Running post-install steps");
                run_command_vec(post_install)?;
            }
        }
        Ok(())
    }

    fn install_dotfile(&self, repo_dir: &Path) -> Result<(), Box<dyn Error>> {
        let mut origin_path = repo_dir.to_path_buf();
        origin_path.push(&self.file);

        let unexpanded_target_path = &self.target.to_string_lossy();

        let target_path_str = shellexpand::tilde(
            unexpanded_target_path
        );

        let target_path = Path::new(target_path_str.as_ref());

        println!(
            "Installing config file {} to location {}",
            &self.file,
            target_path.to_str().expect("Invalid unicode in path")
        );

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|_| "Unable to create parent directories".to_string())?;
        }
        fs::copy(origin_path, target_path).expect("Failed to copy target file");

        Ok(())
    }

    pub fn install(&self, repo_dir: &Path, metadata: Option<DotfileMetadata>, commit_hash: &str, skip_install_commands: bool) -> Result<DotfileMetadata, Box<dyn Error>> {
        if !skip_install_commands {
            self.run_pre_install(&metadata)?;
        }

        self.install_dotfile(repo_dir)?;

        if !skip_install_commands {
            self.run_post_install(&metadata)?;
        }

        let metadata = metadata.unwrap_or_else(|| DotfileMetadata::new(commit_hash, self));

        Ok(metadata)
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AggregatedDotfileMetadata {
    #[serde(flatten)]
    pub data: HashMap<String, DotfileMetadata>,
}

impl AggregatedDotfileMetadata {
    pub fn new() -> Self {
        AggregatedDotfileMetadata::default()
    }

    /// Get the current InstalledDotfilesManifest for this machine, or create one if it doesn't exist.
    ///
    /// # Examples
    ///
    /// ```
    /// let manifest = InstalledDotfilesManifest::get_current()?;
    /// ```
    pub fn get_current() -> Result<AggregatedDotfileMetadata, Box<dyn Error>> {
        let path = shellexpand::tilde(INSTALLED_DOTFILES_MANIFEST_PATH);
        let reader = File::open(path.as_ref()).ok();

        if let Some(file) = reader {
            let config: AggregatedDotfileMetadata = serde_yaml::from_reader(file).map_err(|_| {
            "Could not parse manifest. Check ~/.local/share/jointhedots/manifest.yaml for issues"
        })?;
            Ok(config)
        } else {
            Ok(AggregatedDotfileMetadata::new())
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DotfileMetadata {
    pub commit_hash: String,
    pub pre_install_hash: String,
    pub post_install_hash: String,
}

impl DotfileMetadata {
    /// Construct metadata
    pub fn new(
        commit_hash: &str,
        dotfile: &Dotfile
    ) -> Self {
        DotfileMetadata {
            commit_hash: commit_hash.to_string(),
            pre_install_hash: dotfile.hash_pre_install(),
            post_install_hash: dotfile.hash_pre_install(),
        }
    }
}
