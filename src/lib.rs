pub mod structs;
pub mod cli;
pub mod utils;
pub mod git;

pub mod subcommands {
    mod sync;
    mod install;
    mod interactive;

    pub use sync::sync_subcommand_handler;
    pub use install::install_subcommand_handler;
    pub use interactive::interactive_subcommand_handler;
}
