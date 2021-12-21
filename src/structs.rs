use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(flatten)]
    data: HashMap<String, Dotfile>,
}

impl IntoIterator for Config {
    type Item = (String, Dotfile);

    type IntoIter = std::collections::hash_map::IntoIter<String, Dotfile>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Dotfile {
    pub file: String,
    pub target: Box<Path>,
    pub pre_install: Option<Vec<String>>,
    pub post_install: Option<Vec<String>>,
}

