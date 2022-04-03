use serde::Deserialize;

const SINGLE_DOTFILE_COMMIT_FORMAT: &str = "Sync {} dotfile";
const MULTIPLE_DOTFILES_COMMIT_FORMAT: &str = "Sync dotfiles for {}";

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub commit_prefix: String,
    pub squash_commits: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            commit_prefix: "🔁 ".to_string(),
            squash_commits: true,
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
                    .collect::<String>(),
            );
        }

        commit_message
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_commit_message_single_dotfile() {
        let config = Config::default();

        let commit_message = config.generate_commit_message(vec!["neovim"]);

        assert_eq!("🔁 Sync neovim dotfile", commit_message.as_str());
    }

    #[test]
    fn test_generate_commit_message_multiple_dotfiles() {
        let config = Config::default();

        let commit_message = config.generate_commit_message(vec!["neovim", "kitty"]);

        assert_eq!(
            "🔁 Sync dotfiles for neovim and kitty",
            commit_message.as_str()
        );
    }
}
