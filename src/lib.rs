pub const GITHUB_URL: &str = "https://github.com/";
pub const REPO_DIR: &str = "/tmp/jtd/";
pub const MANIFEST_PATH: &str = "/tmp/jtd/jtd.yaml";

pub mod structs;
pub mod cli;
pub mod utils;

pub mod subcommands {
    mod sync;
    mod install;

    pub use sync::sync_subcommand_handler;
    pub use install::install_subcommand_handler;
}
