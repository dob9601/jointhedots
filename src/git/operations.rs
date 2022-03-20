use std::io::{stdin, stdout, Write};
use std::{error::Error, path::Path, sync::RwLock};

use console::style;
use dialoguer::{Input, Password};
use git2::build::CheckoutBuilder;
use git2::{Commit, Direction, PushOptions, RemoteCallbacks, Repository, Signature, Status};
use git2::{Error as Git2Error, IndexAddOption, MergeOptions};
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
    let (object, reference) = repo
        .revparse_ext(reference)
        .map_err(|err| format!("Ref not found: {}", err))?;

    if let Some(gref) = reference {
        repo.set_head(gref.name().unwrap())
    } else {
        repo.set_head_detached(object.id())
    }
    .map_err(|err| format!("Failed to set HEAD: {}", err).into())
}

pub fn get_commit<'a>(repo: &'a Repository, commit_hash: &str) -> Result<Commit<'a>, Git2Error> {
    let (object, _) = repo.revparse_ext(commit_hash)?;
    object.peel_to_commit()
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

pub fn generate_signature() -> Result<Signature<'static>, Git2Error> {
    Signature::now("Jointhedots Sync", "jtd@danielobr.ie")
}

pub fn has_changes(repo: &Repository) -> Result<bool, Box<dyn Error>> {
    Ok(repo.statuses(None)?.iter().map(|e| e.status()).collect::<Vec<Status>>().len() > 0)
}

/// Add and commit the specified files to the repository index.
///
/// # Arguments
///
/// * `repo` - The repository object
/// * `file_paths` - Optionally the paths of the files to commit. If `None`, all changes are
/// committed.
/// * `message` - The commit message to use
/// * `parents` - Optionally the parent commits for the new commit. If None, `HEAD` is used
/// * `update_head` - Optionally whether to update the commit the `HEAD` reference points at.
///
/// # Returns
///
/// The new commit in the repository
pub fn add_and_commit<'a>(
    repo: &'a Repository,
    file_paths: Option<Vec<&Path>>,
    message: &str,
    parents: Option<Vec<Commit>>,
    update_head: Option<&str>,
) -> Result<Commit<'a>, Box<dyn Error>> {
    let mut index = repo.index()?;

    if let Some(file_paths) = file_paths {
        for path in file_paths {
            index.add_path(path)?;
        }
    } else {
        index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
    }

    let oid = index.write_tree()?;
    let tree = repo.find_tree(oid)?;

    let signature = generate_signature()?;

    let parents = match parents {
        Some(parent_vec) => parent_vec,
        None => vec![get_head(repo)?],
    };
    let oid = repo.commit(
        update_head,
        &signature,
        &signature,
        message,
        &tree,
        &parents.iter().collect::<Vec<&Commit>>(),
    )?;

    repo.find_commit(oid)
        .map_err(|err| format!("Failed to commit to repo: {}", err.to_string()).into())
}

// Adapted from https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs
pub fn normal_merge<'a>(
    repo: &'a Repository,
    main_tip: &Commit,
    feature_tip: &Commit,
) -> Result<Commit<'a>, git2::Error> {
    let local_tree = repo.find_commit(main_tip.id())?.tree()?;
    let remote_tree = repo.find_commit(feature_tip.id())?.tree()?;
    let ancestor = repo
        .find_commit(repo.merge_base(main_tip.id(), feature_tip.id())?)?
        .tree()?;

    let mut options = MergeOptions::new();
    options
        .standard_style(true)
        .minimal(true)
        .fail_on_conflict(false);

    let mut idx = repo.merge_trees(&ancestor, &local_tree, &remote_tree, Some(&options))?;

    if idx.has_conflicts() {
        let repo_dir = repo.path().to_string_lossy().replace(".git/", "");
        repo.checkout_index(
            Some(&mut idx),
            Some(
                CheckoutBuilder::default()
                    .allow_conflicts(true)
                    .conflict_style_merge(true),
            ),
        )?;
        println!(
            "{}",
            style(format!(
                "⚠ Merge conficts detected. Resolve them manually with the following steps:\n\n  \
                1. Open the temporary repository (located in {}),\n  \
                2. Resolve any merge conflicts as you would with any other repository\n  \
                3. Adding the changed files but NOT committing them\n  \
                4. Returning to this terminal and pressing the \"Enter\" key\n",
                repo_dir
            ))
            .red()
        );

        loop {
            print!(
                "{}",
                style("Press ENTER when conflicts are resolved")
                    .blue()
                    .italic()
            );
            let _ = stdout().flush();

            let mut _newline = String::new();
            stdin().read_line(&mut _newline).unwrap_or(0);

            idx = repo.index()?;
            idx.read(false)?;

            if !idx.has_conflicts() {
                break;
            } else {
                println!("{}", style("Conflicts not resolved").red())
            }
        }
    }
    // SYNC FAILS FIRST TIME, ALWAYS SUCCEEDS SECOND?
    // LINES BELOW DONT SEEM TO BE NECESSARY
    // SOME WEIRD REFS BEING PUSHED TO THE GIT REPO? UNSURE HOW TO CHECK
    let result_tree = repo.find_tree(idx.write_tree_to(repo)?)?;
    // now create the merge commit
    let msg = format!("Merge: {} into {}", feature_tip.id(), main_tip.id());
    let sig = generate_signature()?;
    let local_commit = repo.find_commit(main_tip.id())?;
    let remote_commit = repo.find_commit(feature_tip.id())?;
    // Do our merge commit and set current branch head to that commit.
    let merge_commit = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &msg,
        &result_tree,
        &[&local_commit, &remote_commit],
    )?;
    // Set working tree to match head.
    repo.checkout_head(None)?;
    Ok(repo.find_commit(merge_commit)?)
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
