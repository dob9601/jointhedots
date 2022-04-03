use std::{error::Error, str::FromStr};

use clap::ArgEnum;
use strum_macros::{Display, EnumIter};

#[derive(ArgEnum, Clone, EnumIter, Display, Debug, PartialEq)]
pub enum ConnectionMethod {
    SSH,
    HTTPS,
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

#[derive(ArgEnum, Clone, EnumIter, Display, Debug, PartialEq)]
#[clap(rename_all = "PascalCase")]
pub enum RepoHostName {
    GitHub,
    GitLab,
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
    https_prefix: &'static str,
}

const GITLAB: RepoHost = RepoHost {
    ssh_prefix: "git@gitlab.com:",
    https_prefix: "https://gitlab.com/",
};

const GITHUB: RepoHost = RepoHost {
    ssh_prefix: "git@github.com:",
    https_prefix: "https://github.com/",
};

pub fn get_host_git_url(
    repository: &str,
    host: &RepoHostName,
    method: &ConnectionMethod,
) -> Result<String, Box<dyn Error>> {
    let repo_host = match *host {
        RepoHostName::GitHub => GITHUB,
        RepoHostName::GitLab => GITLAB,
    };

    match method {
        ConnectionMethod::SSH => Ok(format!("{}{}{}", repo_host.ssh_prefix, repository, ".git")),
        ConnectionMethod::HTTPS => Ok(format!(
            "{}{}{}",
            repo_host.https_prefix, repository, ".git"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_host_git_url_ssh_github() {
        let repo = "dob9601/dotfiles";
        let host = RepoHostName::GitHub;
        let method = ConnectionMethod::SSH;

        let host_url = get_host_git_url(repo, &host, &method).expect("Failed to get host url");
        assert_eq!(
            host_url,
            String::from("git@github.com:dob9601/dotfiles.git")
        )
    }

    #[test]
    fn test_get_host_git_url_https_gitlab() {
        let repo = "dob9601/dotfiles";
        let host = RepoHostName::GitLab;
        let method = ConnectionMethod::HTTPS;

        let host_url = get_host_git_url(repo, &host, &method).expect("Failed to get host url");
        assert_eq!(
            host_url,
            String::from("https://gitlab.com/dob9601/dotfiles.git")
        )
    }

    #[test]
    fn test_repo_host_name_from_str_github() {
        let hostname = "github";
        assert_eq!(
            <RepoHostName as std::str::FromStr>::from_str(hostname)
                .expect("Could not convert from str"),
            RepoHostName::GitHub
        )
    }

    #[test]
    fn test_repo_host_name_from_str_gitlab() {
        let hostname = "gitlab";
        assert_eq!(
            <RepoHostName as std::str::FromStr>::from_str(hostname)
                .expect("Could not convert from str"),
            RepoHostName::GitLab
        )
    }

    #[test]
    fn test_connection_method_from_str_ssh() {
        let method = "ssh";
        assert_eq!(
            <ConnectionMethod as std::str::FromStr>::from_str(method)
                .expect("Could not convert from str"),
            ConnectionMethod::SSH
        )
    }

    #[test]
    fn test_connection_method_from_str_https() {
        let method = "https";
        assert_eq!(
            <ConnectionMethod as std::str::FromStr>::from_str(method)
                .expect("Could not convert from str"),
            ConnectionMethod::HTTPS
        )
    }
}
