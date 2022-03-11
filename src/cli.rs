use clap::Parser;

use crate::git::remote::{ConnectionMethod, RepoHostName};

#[derive(Parser, Debug)]
#[clap(name = "jointhedots", bin_name = "jtd", about, version)]
pub enum JoinTheDots {
    Install(InstallSubcommandArgs),
    Sync(SyncSubcommandArgs),
    Interactive(InteractiveSubcommandArgs),
}

#[derive(clap::Args, Debug)]
#[clap(about = "Install a specified JTD repository", version)]
pub struct InstallSubcommandArgs {
    #[clap(help = "The location of the repository in the form USERNAME/REPONAME")]
    pub repository: String,

    #[clap(
        arg_enum,
        long = "method",
        short = 'm',
        help = "The method to use for cloning/pushing the repository",
        default_value = "https"
    )]
    pub method: ConnectionMethod,

    #[clap(
        long = "manifest",
        short = 'n',
        help = "The manifest to use in the repository",
        default_value = "jtd.yaml"
    )]
    pub manifest: String,

    #[clap(
        help = "The dotfiles to install. If unspecified, install all of them",
        conflicts_with = "all"
    )]
    pub target_dotfiles: Vec<String>,

    #[clap(
        arg_enum,
        default_value = "GitHub",
        help = "Whether to source the repo from GitHub or GitLab",
        long = "source",
        short = 's',
        ignore_case = true
    )]
    pub source: RepoHostName,

    #[clap(
        help = "Whether to overwrite existing configs without prompt",
        long = "force",
        short = 'f'
    )]
    pub force: bool,

    #[clap(
        help = "Whether to run any pre_install/post_install commands without prompting",
        long = "trust",
        short = 't'
    )]
    pub trust: bool,

    #[clap(
        help = "Whether to install all dotfiles in the config",
        long = "all",
        short = 'a'
    )]
    pub all: bool,
}

#[derive(clap::Args, Debug)]
#[clap(
    about = "Sync the currently installed JTD repository with the provided remote repo.",
    version
)]
pub struct SyncSubcommandArgs {
    #[clap(help = "The location of the repository in the form USERNAME/REPONAME")]
    pub repository: String,

    #[clap(
        help = "The dotfiles to sync. If unspecified, sync all of them",
        conflicts_with = "all"
    )]
    pub target_dotfiles: Vec<String>,

    #[clap(
        help = "Whether to install all dotfiles in the config",
        long = "all",
        short = 'a'
    )]
    pub all: bool,

    #[clap(
        arg_enum,
        long = "method",
        short = 'm',
        help = "The method to use for cloning/pushing the repository",
        default_value = "ssh"
    )]
    pub method: ConnectionMethod,

    #[clap(
        long = "manifest",
        short = 'n',
        help = "The manifest to use in the repository",
        default_value = "jtd.yaml"
    )]
    pub manifest: String,

    #[clap(
        arg_enum,
        default_value = "GitHub",
        help = "Whether to source the repo from GitHub or GitLab",
        long = "source"
    )]
    pub source: RepoHostName,

    #[clap(
        help = "The message to use for the commit",
        long = "commit-msg",
        short = 'c'
    )]
    pub commit_msg: Option<String>,
}

#[derive(clap::Args, Debug)]
#[clap(about = "Interactively install dotfiles", version)]
pub struct InteractiveSubcommandArgs {}
