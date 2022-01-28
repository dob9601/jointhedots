pub mod cli;
pub mod git;
pub mod structs;
pub mod utils;

pub mod subcommands {
    mod install;
    mod interactive;
    mod sync;

    pub use install::install_subcommand_handler;
    pub use interactive::interactive_subcommand_handler;
    pub use sync::sync_subcommand_handler;
}
