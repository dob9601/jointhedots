use serde::Deserialize;

const SINGLE_DOTFILE_COMMIT_FORMAT: &str = "Sync {} dotfile";
const MULTIPLE_DOTFILES_COMMIT_FORMAT: &str = "Sync dotfiles for {}";

fn default_commit_prefix() -> String {
    "🔁 ".to_string()
}

fn default_squash_commits() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_commit_prefix")]
    pub commit_prefix: String,
    
    #[serde(default = "default_squash_commits")]
    pub squash_commits: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            commit_prefix: default_commit_prefix(),
            squash_commits: default_squash_commits(),
        }
    }
}

impl Config {
    pub fn generate_commit_message(&self, dotfile_names: Vec<&str>) -> String {
        let mut commit_message = self.commit_prefix.to_string();

        if dotfile_names.len() == 1 {
            commit_message.push_str(&SINGLE_DOTFILE_COMMIT_FORMAT.replace("{}", &dotfile_names[0]));
        } else {
            commit_message.push_str(
                &MULTIPLE_DOTFILES_COMMIT_FORMAT
                    .replace("{}", &dotfile_names.join(", "))
                    .chars()
                    .rev()
                    .collect::<String>()
                    .replacen(",", "dna ", 1)
                    .chars()
                    .rev()
                    .collect::<String>()
            );
        }

        commit_message
    }
}
