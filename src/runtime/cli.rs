use std::path::PathBuf;

use clap::{Command, CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

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
    /// Parses the build script and associated modules to check for errors, but doesn't run anything
    Check {
        /// The file path of the build script. The default is `main.august`.
        #[arg(short, long)]
        script: Option<PathBuf>,
    },
    /// Parses the build script and associated modules and displays some details about it.
    /// Doesn't run anything.
    Inspect {
        /// The file path of the build script. The default is `main.august`.
        #[arg(short, long)]
        script: Option<PathBuf>,
    },
    /// Generate command line autocompletions based on shell
    Completions {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

pub fn build_cmd() -> Command {
    CLI::command()
}
