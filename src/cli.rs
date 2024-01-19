use std::path::PathBuf;

use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Debug, Clone, Parser)]
#[command(author, version, about)]
pub struct CLI {
    #[arg(short, long, default_value = PathBuf::from("main.august").into_os_string())]
    pub script: PathBuf,
    #[arg(short, long)]
    pub verbose: bool,
    #[arg(short, long)]
    pub quiet: bool,

    #[command(subcommand)]
    pub subcommand: CLICommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum CLICommand {
    Info,
    Inspect,
    Check,
    Completions { shell: Shell },
    Build,
    Test,
    Run { unit: String },
}
