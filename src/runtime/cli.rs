use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone, PartialEq, Eq)]
#[command(author, version, about)]
pub struct CLI {
    #[command(subcommand)]
    pub command: CLICommand,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum CLICommand {
    /// Provides more specific information about the installation
    Info,
    /// Runs the task assigned to the test pragma
    Test {
        /// The file path of the build script. The default is `main.august`.
        #[arg(short, long)]
        script: Option<PathBuf>,
    },
    /// Runs the task assigned to the build pragma
    Build {
        /// The file path of the build script. The default is `main.august`.
        #[arg(short, long)]
        script: Option<PathBuf>,
    },
    /// Runs the task provided as an argument
    Run {
        /// The file path of the build script. The default is `main.august`.
        #[arg(short, long)]
        script: Option<PathBuf>,
        /// The name of the task to be run
        task_name: String,
    },
    /// Parses the build script and associated modules, but doesn't run anything
    Parse {
        /// The file path of the build script. The default is `main.august`.
        #[arg(short, long)]
        script: Option<PathBuf>,
        /// Displays a debug render of the fully parsed module
        #[arg(short, long)]
        display: bool,
    },
}
