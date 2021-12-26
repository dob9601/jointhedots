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

    pub target_configs: Vec<String>,

    #[clap(default_value = "GitHub", help = "Whether to source the repo from GitHub or GitLab", long = "source")]
    pub source: String
}

#[derive(clap::Args)]
#[clap(about = "Sync the currently installed JTD repository with the provided remote repo.", author, version)]
pub struct SyncSubcommandArgs {
    #[clap(help = "The location of the repository in the form USERNAME/REPONAME")]
    pub repository: String,

    pub target_configs: Vec<String>,

    #[clap(default_value = "GitHub", help = "Whether to source the repo from GitHub or GitLab", long = "source")]
    pub source: String
}
