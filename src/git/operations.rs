use std::{error::Error, path::Path, sync::RwLock};

use console::style;
use dialoguer::{Input, Password};
use git2::{Commit, Direction, Oid, PushOptions, RemoteCallbacks, Repository, Signature};
use git2_credentials::{CredentialHandler, CredentialUI};

use crate::utils::get_theme;
use lazy_static::lazy_static;

pub fn get_head(repo: &Repository) -> Result<Commit, Box<dyn Error>> {
    let commit = repo
        .head()?
        .resolve()?
        .peel(git2::ObjectType::Commit)?
        .into_commit()
        .unwrap();
    Ok(commit)
}

pub fn get_head_hash(repo: &Repository) -> Result<String, Box<dyn Error>> {
    Ok(get_head(repo)?.id().to_string())
}

pub fn checkout_ref(repo: &Repository, reference: &str) -> Result<(), Box<dyn Error>> {
    let (object, reference) = repo.revparse_ext(reference).map_err(|err| format!("Ref not found: {}", err))?;
    if let Some(gref) = reference {
        repo.set_head(gref.name().unwrap())
    } else {
        repo.set_head_detached(object.id())
    }.map_err(|err| format!("Failed to set HEAD: {}", err).into())
}

lazy_static! {
    static ref CREDENTIAL_CACHE: RwLock<(Option<String>, Option<String>)> =
        RwLock::new((None, None));
}

pub struct CredentialUIDialoguer;

impl CredentialUI for CredentialUIDialoguer {
    fn ask_user_password(&self, username: &str) -> Result<(String, String), Box<dyn Error>> {
        let theme = get_theme();

        let mut credential_cache = CREDENTIAL_CACHE.write()?;

        let user = match &credential_cache.0 {
            Some(username) => username.to_owned(),
            None => {
                let user = Input::with_theme(&theme)
                    .default(username.to_owned())
                    .with_prompt("Username")
                    .interact()?;
                credential_cache.0 = Some(user.to_owned());
                user
            }
        };

        let password = match &credential_cache.1 {
            Some(password) => password.to_owned(),
            None => {
                let pass = Password::with_theme(&theme)
                    .with_prompt("password (hidden)")
                    .allow_empty_password(true)
                    .interact()?;
                credential_cache.1 = Some(pass.to_owned());
                pass
            }
        };

        Ok((user, password))
    }

    fn ask_ssh_passphrase(&self, passphrase_prompt: &str) -> Result<String, Box<dyn Error>> {
        let mut credential_cache = CREDENTIAL_CACHE.write()?;

        let passphrase = match &credential_cache.1 {
            Some(passphrase) => passphrase.to_owned(),
            None => {
                let pass = Password::with_theme(&get_theme())
                    .with_prompt(format!(
                        "{} (leave blank for no password): ",
                        passphrase_prompt
                    ))
                    .allow_empty_password(true)
                    .interact()?;
                credential_cache.1 = Some(pass.to_owned());
                pass
            }
        };

        Ok(passphrase)
    }
}

pub fn generate_callbacks() -> Result<RemoteCallbacks<'static>, Box<dyn Error>> {
    let mut cb = git2::RemoteCallbacks::new();
    let git_config = git2::Config::open_default()
        .map_err(|err| format!("Could not open default git config: {}", err))?;
    let mut ch = CredentialHandler::new_with_ui(git_config, Box::new(CredentialUIDialoguer {}));
    cb.credentials(move |url, username, allowed| ch.try_next_credential(url, username, allowed));

    Ok(cb)
}

pub fn clone_repo(url: &str, target_dir: &Path) -> Result<git2::Repository, Box<dyn Error>> {
    // Clone the project.
    let cb = generate_callbacks()?;

    // clone a repository
    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(cb)
        .download_tags(git2::AutotagOption::All)
        .update_fetchhead(true);
    let repo = git2::build::RepoBuilder::new()
        .fetch_options(fo)
        .clone(url, target_dir)
        .map_err(|err| format!("Could not clone repo: {}", &err))?;

    println!("{}", style("✔ Successfully cloned repository!").green());

    Ok(repo)
}

pub fn add_and_commit(
    repo: &Repository,
    file_paths: Vec<&Path>,
    message: &str,
) -> Result<Oid, Box<dyn Error>> {
    let mut index = repo.index()?;

    for path in file_paths {
        index.add_path(path)?;
    }

    let oid = index.write_tree()?;
    let tree = repo.find_tree(oid)?;

    let signature = Signature::now("Jointhedots Sync", "jtd@danielobr.ie")?;

    let parent = get_head(repo)?;
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &[&parent],
    )
    .map_err(|err| format!("Failed to commit to repo: {}", err.to_string()).into())
}

pub fn is_in_past(repo: &Repository, commit_hash: &str) -> Result<bool, Box<dyn Error>> {
    let head_commit = repo.head()?.target().ok_or("Could not get HEAD commit")?;
    Ok(head_commit.to_string() == commit_hash)
}

pub fn push(repo: &Repository) -> Result<(), Box<dyn Error>> {
    let mut remote = repo.find_remote("origin")?;

    remote.connect_auth(Direction::Push, Some(generate_callbacks()?), None)?;
    let mut options = PushOptions::new();
    options.remote_callbacks(generate_callbacks()?);
    remote
        .push(&["refs/heads/master:refs/heads/master"], Some(&mut options))
        .map_err(|err| format!("Could not push to remote repo: {}", err).into())
}
