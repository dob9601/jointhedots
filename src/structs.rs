use std::error::Error;
use std::fs::{self, File};
use std::path::PathBuf;
use std::{collections::HashMap, path::Path};

use console::style;
use dialoguer::{Confirm, MultiSelect};
use git2::Repository;
use serde::{Deserialize, Serialize};

use crate::git::operations::get_head_hash;
use crate::utils::{
    get_theme, hash_command_vec, run_command_vec, INSTALLED_DOTFILES_MANIFEST_PATH,
};

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
        if self.has_run_stages(Some(target_dotfiles.iter().map(|v| v.as_str()).collect())) {
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

        let dotfiles: Vec<(&String, &Dotfile)> = if install_all {
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
        };

        // Safe to unwrap here, repo.path() points to .git folder. Path will always
        // have a component after parent.
        let repo_dir = repo.path().parent().unwrap().to_owned();

        let mut aggregated_metadata = AggregatedDotfileMetadata::get_current()?;
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

    /// Return whether this Manifest contains dotfiles containing potentially dangerous run stages.
    /// Optionally can take a vector of [Dotfile]s for testing a subset of the manifest.
    pub fn has_run_stages(&self, dotfile_names: Option<Vec<&str>>) -> bool {
        let dotfile_names =
            dotfile_names.unwrap_or_else(|| self.data.keys().map(|k| k.as_str()).collect());

        self.data
            .iter()
            .filter(|(dotfile_name, _)| dotfile_names.contains(&dotfile_name.as_str()))
            .any(|(_, dotfile)| dotfile.has_run_stages())
    }
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
            hash_command_vec(pre_install)
        } else {
            "".to_string()
        }
    }

    pub fn hash_post_install(&self) -> String {
        if let Some(post_install) = &self.post_install {
            // Unwrap is safe, hash will always be utf-8
            hash_command_vec(post_install)
        } else {
            "".to_string()
        }
    }

    /// Return whether this dotfile has run stages, i.e. pre_install or post_install is not `None`
    pub fn has_run_stages(&self) -> bool {
        self.pre_install.is_some() || self.post_install.is_some()
    }

    fn run_pre_install(&self, metadata: &Option<DotfileMetadata>) -> Result<(), Box<dyn Error>> {
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
            }
        }
        Ok(())
    }

    fn run_post_install(&self, metadata: &Option<DotfileMetadata>) -> Result<(), Box<dyn Error>> {
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
            }
        }
        Ok(())
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
            )).green()
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

/// Struct representing a `manifest.yaml` file, typically found in ~/.local/share/jointhedots.
/// Represents an aggregation of the metadata of all of the dotfiles in a Manifest via a mapping of
/// `dotfile_name` to [DotfileMetadata]
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AggregatedDotfileMetadata {
    #[serde(flatten)]
    pub data: HashMap<String, DotfileMetadata>,
}

impl AggregatedDotfileMetadata {
    pub fn new() -> Self {
        AggregatedDotfileMetadata::default()
    }

    /// Get the current AggregatedDotfileMetadata for this machine, or create one if it doesn't exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use jointhedots::structs::AggregatedDotfileMetadata;
    ///
    /// let manifest = AggregatedDotfileMetadata::get_current().unwrap();
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

/// Represent the metadata of an installed dotfile
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DotfileMetadata {
    /// The hash of the commit this dotfile was installed from
    pub commit_hash: String,

    /// The sha1 hash of the pre-install steps. Used to figure out whether pre-install should be
    /// run again on subsequent installations
    pub pre_install_hash: String,

    /// The sha1 hash of the post-install steps. Used to figure out whether post-install should be
    /// run again on subsequent installations
    pub post_install_hash: String,
}

impl DotfileMetadata {
    /// Extract the metadata from a [Dotfile] and the commit hash the dotfile was installed from
    pub fn new(commit_hash: &str, dotfile: &Dotfile) -> Self {
        DotfileMetadata {
            commit_hash: commit_hash.to_string(),
            pre_install_hash: dotfile.hash_pre_install(),
            post_install_hash: dotfile.hash_pre_install(),
        }
    }
}
