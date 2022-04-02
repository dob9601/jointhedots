use serde::Deserialize;

const SINGLE_DOTFILE_COMMIT_FORMAT: &str = "Sync {} dotfile";
const MULTIPLE_DOTFILES_COMMIT_FORMAT: &str = "Sync dotfiles for {}";

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub commit_prefix: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            commit_prefix: "🔁 ".to_string(),
        }
    }
}

impl Config {
    pub fn generate_commit_message(&self, dotfile_names: Vec<&str>) -> String {
        let mut commit_message = String::new();

        if dotfile_names.len() == 1 {
            commit_message.push_str(&SINGLE_DOTFILE_COMMIT_FORMAT.replace("", &dotfile_names[0]));
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
