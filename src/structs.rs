use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct InstalledDotfilesManifest {
    #[serde(flatten)]
    pub data: HashMap<String, InstalledDotfile>,
}

impl InstalledDotfilesManifest {
    pub fn new() -> Self {
        InstalledDotfilesManifest::default()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstalledDotfile {
    pub commit_hash: String,
}

impl InstalledDotfile {
    pub fn new(commit_hash: &str) -> Self {
        InstalledDotfile {
            commit_hash: commit_hash.to_string(),
        }
    }
}
