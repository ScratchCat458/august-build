use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

#[derive(Debug, Clone, Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// File path of the build script.
    #[arg(global(true), short, long, default_value = PathBuf::from("main.august").into_os_string())]
    pub script: PathBuf,
    /// Provides more specific logging output are command execution
    #[arg(global(true), short, long)]
    pub verbose: bool,
    /// Causes unit execution to not produce any logging output
    #[arg(global(true), short, long)]
    pub quiet: bool,

    #[arg(global(true), long, value_enum, default_value_t, alias("color"))]
    pub colour: ColourSupport,

    #[command(subcommand)]
    pub subcommand: CLICommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum CLICommand {
    /// Provides information about the CLI
    Info,
    /// Parses the build script and displays related information
    Inspect,
    /// Parses the build script to check for errors
    Check,
    /// Writes command line shell completions to stdout
    Completions { shell: Shell },
    /// Runs the unit exposed to `build`
    Build,
    /// Runs the unit exposed to `test`
    Test,
    /// Runs the unit provided as an argument
    Run { unit: String },
}

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum ColourSupport {
    Always,
    #[default]
    Auto,
    Never,
}
