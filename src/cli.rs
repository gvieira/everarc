use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "everarc", version, about = "Everarc command-line interface")]
pub struct Cli {
    /// Path to the Everarc configuration file.
    #[arg(
        short,
        long,
        global = true,
        default_value = "everarc.toml",
        value_name = "PATH"
    )]
    pub config: PathBuf,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Build HTML from the project configuration.
    Build(crate::commands::build::BuildArgs),

    /// Check the project.
    Check,

    /// Show the Everarc guide.
    Guide,
}
