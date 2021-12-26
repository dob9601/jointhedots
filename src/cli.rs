use clap::Parser;

#[derive(Parser)]
#[clap(name = "jointhedots", bin_name = "jtd", about)]
pub enum JoinTheDots {
    Install(InstallSubcommandArgs),
    Sync(SyncSubcommandArgs)
}

#[derive(clap::Args)]
#[clap(about = "Install a specified JTD repository", author, version)]
pub struct InstallSubcommandArgs {
    #[clap(help = "The location of the repository in the form USERNAME/REPONAME")]
    pub repository: String,

    #[clap(help = "The dotfiles to install. If unspecified, install all of them")]
    pub target_dotfiles: Vec<String>,

    #[clap(default_value = "GitHub", help = "Whether to source the repo from GitHub or GitLab", long = "source")]
    pub source: String,

    #[clap(help = "whether to overwrite existing configs without prompt", long = "force")]
    pub force: bool
}

#[derive(clap::Args)]
#[clap(about = "Sync the currently installed JTD repository with the provided remote repo.", author, version)]
pub struct SyncSubcommandArgs {
    #[clap(help = "The location of the repository in the form USERNAME/REPONAME")]
    pub repository: String,

    #[clap(help = "The dotfiles to sync. If unspecified, sync all of them")]
    pub target_dotfiles: Vec<String>,

    #[clap(default_value = "GitHub", help = "Whether to source the repo from GitHub or GitLab", long = "source")]
    pub source: String
}
