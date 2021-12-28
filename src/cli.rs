use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "jointhedots", bin_name = "jtd", about)]
pub enum JoinTheDots {
    Install(InstallSubcommandArgs),
    Sync(SyncSubcommandArgs),
    Interactive(InteractiveSubcommandArgs),
}

#[derive(clap::Args, Debug)]
#[clap(about = "Install a specified JTD repository", author, version)]
pub struct InstallSubcommandArgs {
    #[clap(help = "The location of the repository in the form USERNAME/REPONAME")]
    pub repository: String,

    #[clap(help = "The dotfiles to install. If unspecified, install all of them")]
    pub target_dotfiles: Vec<String>,

    #[clap(
        default_value = "GitHub",
        help = "Whether to source the repo from GitHub or GitLab",
        long = "source"
    )]
    pub source: String,

    #[clap(
        help = "Whether to overwrite existing configs without prompt",
        long = "force"
    )]
    pub force: bool,

    #[clap(
        help = "Whether to run any pre_install/post_install commands without prompting",
        long = "trust"
    )]
    pub trust: bool,
}

#[derive(clap::Args, Debug)]
#[clap(
    about = "Sync the currently installed JTD repository with the provided remote repo.",
    author,
    version
)]
pub struct SyncSubcommandArgs {
    #[clap(help = "The location of the repository in the form USERNAME/REPONAME")]
    pub repository: String,

    #[clap(help = "The dotfiles to sync. If unspecified, sync all of them")]
    pub target_dotfiles: Vec<String>,

    #[clap(
        default_value = "GitHub",
        help = "Whether to source the repo from GitHub or GitLab",
        long = "source"
    )]
    pub source: String,
}

#[derive(clap::Args, Debug)]
#[clap(about = "Interactively install dotfiles", author, version)]
pub struct InteractiveSubcommandArgs {}
