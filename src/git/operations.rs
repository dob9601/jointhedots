use std::io::{stdin, stdout, Write};
use std::{error::Error, path::Path, sync::RwLock};

use console::{style, StyledObject};
use dialoguer::{Input, Password};
use git2::build::CheckoutBuilder;
use git2::{
    AnnotatedCommit, Commit, DiffLine, Direction, PushOptions, RemoteCallbacks, Repository,
    Signature,
};
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

    repo.checkout_tree(&object, None)?;

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
                    .with_prompt("Password (hidden)")
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

    success!("Successfully cloned repository!");

    Ok(repo)
}

pub fn generate_signature() -> Result<Signature<'static>, Git2Error> {
    Signature::now("Jointhedots Sync", "jtd@danielobr.ie")
}

pub fn add_all(repo: &Repository, file_paths: Option<Vec<&Path>>) -> Result<(), Box<dyn Error>> {
    let mut index = repo.index()?;
    if let Some(file_paths) = file_paths {
        index.add_all(file_paths.iter(), IndexAddOption::DEFAULT, None)?;
    } else {
        index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
    }
    index.write()?;
    Ok(())
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
    maybe_parents: Option<Vec<&Commit>>,
    update_ref: Option<&str>,
) -> Result<Commit<'a>, Box<dyn Error>> {
    add_all(&repo, file_paths)?;

    let mut index = repo.index()?;
    let oid = index.write_tree()?;
    let tree = repo.find_tree(oid)?;
    let signature = generate_signature()?;

    let head;
    let parents = match maybe_parents {
        Some(parent_vec) => parent_vec,
        None => {
            head = get_head(repo)?;
            vec![&head]
        }
    };
    let oid = repo.commit(update_ref, &signature, &signature, message, &tree, &parents)?;

    repo.find_commit(oid)
        .map_err(|err| format!("Failed to commit to repo: {}", err.to_string()).into())
}

pub fn normal_merge<'a>(
    repo: &'a Repository,
    main_tip: &AnnotatedCommit,
    feature_tip: &AnnotatedCommit,
) -> Result<Commit<'a>, Box<dyn Error>> {
    let mut options = MergeOptions::new();
    options
        .standard_style(true)
        .minimal(true)
        .fail_on_conflict(false);
    repo.merge(&[feature_tip], Some(&mut options), None)?;

    let mut idx = repo.index()?;
    idx.read(false)?;
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
        error!(
            "Merge conficts detected. Resolve them manually with the following steps:\n\n  \
             1. Open the temporary repository (located in {}),\n  \
             2. Resolve any merge conflicts as you would with any other repository\n  \
             3. Adding the changed files but NOT committing them\n  \
             4. Returning to this terminal and pressing the \"Enter\" key\n",
            repo_dir
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

            idx.read(false)?;

            if !idx.has_conflicts() {
                break;
            } else {
                error!("Conflicts not resolved");
            }
        }
    }

    let tree = repo.find_tree(repo.index()?.write_tree()?)?;
    let signature = generate_signature()?;
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Merge",
        &tree,
        &[
            &repo.find_commit(main_tip.id())?,
            &repo.find_commit(feature_tip.id())?,
        ],
    )?;
    repo.cleanup_state()?;
    Ok(get_head(&repo)?)
}

pub fn colorize_diff_line<'a>(line: &'a DiffLine) -> Option<StyledObject<&'a str>> {
    let decoded_diff = std::str::from_utf8(line.content());

    if let Ok(diff) = decoded_diff {
        let styled_diff = style(diff);
        match line.origin() {
            '+' => Some(styled_diff.green()),
            '-' => Some(styled_diff.red()),
            '>' => Some(styled_diff.green()),
            '<' => Some(styled_diff.red()),
            'F' => Some(styled_diff.bold()),
            'H' => Some(styled_diff.cyan()),
            _ => Some(styled_diff)
        }
    } else {
        None
    }
}

pub fn get_repo_dir(repo: &Repository) -> &Path {
    // Safe to unwrap here, repo.path() points to .git folder. Path will always
    // have a component before .git
    repo.path().parent().unwrap()
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

#[cfg(test)]
mod tests {
    use std::fs::File;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_get_head() {
        let repo_dir = tempdir().expect("Could not create temporary repo dir");
        let repo = Repository::init(&repo_dir).expect("Could not initialise repository");

        let commit = add_and_commit(&repo, None, "", Some(vec![]), Some("HEAD")).unwrap();

        assert_eq!(commit.id(), get_head(&repo).unwrap().id());
    }

    #[test]
    fn test_get_head_hash() {
        let repo_dir = tempdir().unwrap();
        let repo = Repository::init(&repo_dir).unwrap();

        let commit = add_and_commit(&repo, None, "", Some(vec![]), Some("HEAD")).unwrap();

        assert_eq!(commit.id().to_string(), get_head_hash(&repo).unwrap());
    }

    #[test]
    fn test_checkout_ref() {
        let repo_dir = tempdir().expect("Could not create temporary repo dir");
        let repo = Repository::init(&repo_dir).expect("Could not initialise repository");

        let first_commit = add_and_commit(&repo, None, "", Some(vec![]), Some("HEAD")).unwrap();
        let second_commit =
            add_and_commit(&repo, None, "", Some(vec![&first_commit]), Some("HEAD")).unwrap();

        assert_eq!(
            repo.head().unwrap().peel_to_commit().unwrap().id(),
            second_commit.id()
        );

        checkout_ref(&repo, &first_commit.id().to_string())
            .expect("Failed to checkout first commit");

        assert_eq!(get_head_hash(&repo).unwrap(), first_commit.id().to_string());
    }

    #[test]
    fn test_get_commit() {
        let repo_dir = tempdir().unwrap();
        let repo = Repository::init(&repo_dir).unwrap();

        let commit = add_and_commit(&repo, None, "", Some(vec![]), Some("HEAD")).unwrap();
        let hash = commit.id().to_string();

        assert_eq!(
            get_commit(&repo, &hash).unwrap().id().to_string(),
            commit.id().to_string()
        );
    }

    #[test]
    fn test_ask_user_password_with_cache() {
        {
            let mut credential_cache = CREDENTIAL_CACHE
                .write()
                .expect("Could not get write handle on credential cache");
            credential_cache.0 = Some("username".to_string());
            credential_cache.1 = Some("password".to_string());
        }

        let credential_ui = CredentialUIDialoguer;

        let credentials = credential_ui
            .ask_user_password("")
            .expect("Could not get user password");
        assert_eq!(
            ("username".to_string(), "password".to_string()),
            credentials
        );
    }

    #[test]
    fn test_ask_ssh_passphrase_with_cache() {
        {
            let mut credential_cache = CREDENTIAL_CACHE
                .write()
                .expect("Could not get write handle on credential cache");
            credential_cache.1 = Some("password".to_string());
        }

        let credential_ui = CredentialUIDialoguer;

        let credentials = credential_ui
            .ask_ssh_passphrase("")
            .expect("Could not get user password");
        assert_eq!("password".to_string(), credentials);
    }

    #[test]
    fn test_generate_callbacks() {
        let _callbacks = generate_callbacks().expect("Failed to generate callbacks");
        // FIXME: Find some way to assert the return type of callbacks
    }

    #[test]
    fn test_clone_repo() {
        let repo_dir = tempdir().expect("Failed to create tempdir");

        let _repo = clone_repo("https://github.com/dob9601/dotfiles.git", repo_dir.path())
            .expect("Failed to clone repo");

        assert!(Path::exists(
            &repo_dir.path().to_owned().join(Path::new("jtd.yaml"))
        ));
    }

    #[test]
    fn test_add_and_commit() {
        let repo_dir = tempdir().expect("Could not create temporary repo dir");
        let repo = Repository::init(&repo_dir).expect("Could not initialise repository");

        let mut filepath = repo_dir.path().to_owned();
        filepath.push(Path::new("file.rs"));
        File::create(filepath.to_owned()).expect("Could not create file in repo");

        add_and_commit(
            &repo,
            Some(vec![&filepath]),
            "commit message",
            Some(vec![]),
            Some("HEAD"),
        )
        .expect("Failed to commit to repository");
        assert_eq!(
            "commit message",
            get_head(&repo)
                .unwrap()
                .message()
                .expect("No commit message found")
        );
    }

    #[test]
    fn test_normal_merge() {
        let repo_dir = tempdir().expect("Could not create temporary repo dir");
        let repo = Repository::init(&repo_dir).expect("Could not initialise repository");

        let first_commit = add_and_commit(
            &repo,
            Some(vec![]),
            "1st commit",
            Some(vec![]),
            Some("HEAD"),
        )
        .expect("Failed to create 1st commit");

        let _second_commit = add_and_commit(
            &repo,
            Some(vec![]),
            "2nd commit",
            Some(vec![&first_commit]),
            Some("HEAD"),
        )
        .expect("Failed to create 2nd commit");

        let head_ref = &repo.head().unwrap();
        let head_ref_name = head_ref.name().unwrap();
        let annotated_main_head = repo.reference_to_annotated_commit(&head_ref).unwrap();

        let _branch = repo
            .branch("branch", &first_commit, true)
            .expect("Failed to create branch");
        checkout_ref(&repo, "branch").expect("Failed to checkout new branch");

        let annotated_branch_head = repo
            .reference_to_annotated_commit(&repo.head().unwrap())
            .unwrap();

        checkout_ref(&repo, head_ref_name).expect("Failed to checkout new branch");

        normal_merge(&repo, &annotated_main_head, &annotated_branch_head)
            .expect("Failed to merge branch");

        // FIXME: Some assertion on the repo state after this
    }

    #[test]
    fn test_generate_signature() {
        let signature = generate_signature().unwrap();

        assert_eq!(signature.email().unwrap(), "jtd@danielobr.ie");
        assert_eq!(signature.name().unwrap(), "Jointhedots Sync");
    }
}
