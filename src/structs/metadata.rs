use std::collections::HashMap;
use std::fs;
use std::{error::Error, fs::File};

use serde::{Deserialize, Serialize};

use crate::MANIFEST_PATH;

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

    /// Get the current AggregatedDotfileMetadata for this machine, or return None if it doesn't exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use jointhedots::structs::AggregatedDotfileMetadata;
    ///
    /// let manifest = AggregatedDotfileMetadata::get().unwrap();
    /// ```
    pub fn get() -> Result<Option<AggregatedDotfileMetadata>, Box<dyn Error>> {
        let path = shellexpand::tilde(MANIFEST_PATH);
        let reader = File::open(path.as_ref()).ok();

        if let Some(file) = reader {
            let config: AggregatedDotfileMetadata =
                serde_yaml::from_reader(file).map_err(|_| {
                    format!(
                        "Could not parse manifest. Check {} for issues",
                        MANIFEST_PATH
                    )
                })?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

    /// Get the current AggregatedDotfileMetadata for this machine, or create one if it doesn't exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use jointhedots::structs::AggregatedDotfileMetadata;
    ///
    /// let manifest = AggregatedDotfileMetadata::get_or_create().unwrap();
    /// ```
    pub fn get_or_create() -> Result<AggregatedDotfileMetadata, Box<dyn Error>> {
        Ok(AggregatedDotfileMetadata::get()?.unwrap_or_else(AggregatedDotfileMetadata::new))
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let data_path = shellexpand::tilde(MANIFEST_PATH);
        fs::create_dir_all(data_path.as_ref())?;

        let output_manifest_file = File::create(data_path.to_string())?;
        Ok(serde_yaml::to_writer(output_manifest_file, &self)?)
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
    pub fn new(commit_hash: &str, pre_install_hash: String, post_install_hash: String) -> Self {
        DotfileMetadata {
            commit_hash: commit_hash.to_string(),
            pre_install_hash,
            post_install_hash,
        }
    }
}
