use clap::Parser;

#[derive(Parser)]
#[clap(name = "jointhedots")]
#[clap(bin_name = "jtd")]
pub enum JoinTheDots {
    Install(InstallSubcommandArgs),
    Sync(SyncSubcommandArgs)
}

#[derive(clap::Args)]
#[clap(about, author, version)]
pub struct InstallSubcommandArgs {
    pub repository: String,
}

#[derive(clap::Args)]
#[clap(about, author, version)]
pub struct SyncSubcommandArgs {
    pub repository: String,
}
