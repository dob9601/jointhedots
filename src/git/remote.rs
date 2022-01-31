use std::{error::Error, str::FromStr};

use clap::ArgEnum;
use strum_macros::{EnumIter, Display};

#[derive(ArgEnum, Clone, EnumIter, Display, Debug)]
pub enum ConnectionMethod {
    SSH,
    HTTPS
}

impl FromStr for ConnectionMethod {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ssh" => Ok(ConnectionMethod::SSH),
            "https" => Ok(ConnectionMethod::HTTPS),
            v => Err(format!("Failed to convert: '{}' is not a known variant.", v).into()),
        }
    }
}

#[derive(ArgEnum, Clone, EnumIter, Display, Debug)]
pub enum RepoHostName {
    GitHub,
    GitLab
}

impl FromStr for RepoHostName {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "github" => Ok(RepoHostName::GitHub),
            "gitlab" => Ok(RepoHostName::GitLab),
            v => Err(format!("Failed to convert: '{}' is not a known variant.", v).into()),
        }
    }
}

pub struct RepoHost {
    ssh_prefix: &'static str,
    https_prefix: &'static str
}

const GITLAB: RepoHost = RepoHost {
    ssh_prefix: "git@gitlab.com:",
    https_prefix: "https://gitlab.com/"
};

const GITHUB: RepoHost = RepoHost {
    ssh_prefix: "git@github.com:",
    https_prefix: "https://github.com/"
};

pub fn get_host_git_url(repository: &str, host: &RepoHostName, method: &ConnectionMethod) -> Result<String, Box<dyn Error>> {
    let repo_host = match *host {
        RepoHostName::GitHub => GITHUB,
        RepoHostName::GitLab => GITLAB,
    };

    match method {
        ConnectionMethod::SSH => Ok(format!("{}{}{}", repo_host.ssh_prefix, repository, ".git")),
        ConnectionMethod::HTTPS => Ok(format!("{}{}{}", repo_host.https_prefix, repository, ".git")),
    }
}

