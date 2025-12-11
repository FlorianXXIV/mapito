use clap::{Args, Parser, Subcommand};

use crate::pack::pack::PackAction;

#[derive(Parser)]
#[command(version, about)]
pub struct Arguments {
    /// Search for mods on the Modrinth Database
    #[arg(short, long, group = "mod_actions")]
    pub search: Option<String>,

    /// Download a mod from modrinth to the mod folder defined in the configuration.
    #[arg(short, long, value_name = "SLUG|ID", group = "mod_actions")]
    pub download: Option<String>,

    /// Use the staging API instead of the regular API (for development)
    #[arg(short = 'S', long)]
    pub staging: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Perform multiple different Pack actions
    Pack(PackArgs),
    /// access config file in editor
    Config {
        /// Print current config
        #[arg(short, long)]
        info: bool,
    },
}

#[derive(Args)]
pub struct PackArgs {
    #[command(subcommand)]
    pub pack_action: PackAction,
}
