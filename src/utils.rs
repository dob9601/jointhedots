use std::{error::Error, process::Command};

use git2::{Repository, Commit, ObjectType, IndexAddOption};

pub const GITHUB_SSH_URL_PREFIX: &str = "git@github.com:";
pub const GITLAB_SSH_URL_PREFIX: &str = "git@gitlab.com:";

pub fn run_command_vec(command_vec: &[String]) -> Result<(), Box<dyn Error>> {
    for command in command_vec.iter() {
        let command_vec: Vec<&str> = command.split(' ').collect();
        Command::new(command_vec[0])
            .args(&command_vec[1..])
            .spawn()?;
    }
    Ok(())
}

pub fn get_repo_host_ssh_url(host: &str) -> Result<&str, Box<dyn Error>> {
    match host.to_lowercase().as_str() {
        "github" => Ok(GITHUB_SSH_URL_PREFIX),
        "gitlab" => Ok(GITLAB_SSH_URL_PREFIX),
        _ => Err("Provided host unknown".into())
    }
}

pub fn find_last_commit(repo: &Repository) -> Result<Commit, git2::Error> {
    let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
    obj.into_commit().map_err(|_| git2::Error::from_str("Couldn't find commit"))
}

pub fn add_and_commit_changes(repo: &Repository, msg: &str) -> Result<(), Box<dyn Error>> {
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;

    let oid = index.write_tree()?;
    let signature = repo.signature()?;
    let parent_commit = find_last_commit(repo)?;
    let tree = repo.find_tree(oid)?;
    repo.commit(Some("HEAD"), //  point HEAD to our new commit
                &signature, // author
                &signature, // committer
                msg, // commit message
                &tree, // tree
                &[&parent_commit])?; // parents

    Ok(())
}
