use std::error::Error;

use git2::Repository;


pub fn get_head_hash(repo: &Repository) -> Result<String, Box<dyn Error>> {
    let commit = repo.head()?.target().ok_or("Could not get HEAD commit")?;
    Ok(commit.to_string())
}
