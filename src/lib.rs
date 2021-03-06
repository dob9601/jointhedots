#[macro_use]
pub mod log;

pub mod cli;
pub mod structs;
pub mod utils;

pub(crate) const MANIFEST_PATH: &str = "~/.local/share/jointhedots/manifest.yaml";

pub(crate) mod git {
    pub mod operations;
    pub mod remote;
}

pub mod subcommands {
    mod install;
    mod interactive;
    mod sync;

    pub use install::install_subcommand_handler;
    pub use interactive::interactive_subcommand_handler;
    pub use sync::sync_subcommand_handler;
}
