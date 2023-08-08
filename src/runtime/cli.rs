use std::{collections::HashMap, error::Error, ffi::OsStr, io::stdout, path::PathBuf};

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generator, Shell};
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Table};
use owo_colors::OwoColorize;
use walkdir::WalkDir;

use crate::{
    parsing::{error::ParserErrorHandler, parse_script},
    Command, CommandDefinition, Pragma, Task,
};

use super::{output::RuntimeError, ExecutionPool};

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
        /// Silents extra output lines for the task runner
        #[arg(short, long)]
        quiet: bool,
    },
    /// Runs the task assigned to the build pragma
    Build {
        /// The file path of the build script. The default is `main.august`.
        #[arg(short, long)]
        script: Option<PathBuf>,
        /// Silents extra output lines for the task runner
        #[arg(short, long)]
        quiet: bool,
    },
    /// Runs the task provided as an argument
    Run {
        /// The file path of the build script. The default is `main.august`.
        #[arg(short, long)]
        script: Option<PathBuf>,
        /// The name of the task to be run
        task_name: String,
        /// Silents extra output lines for the task runner
        #[arg(short, long)]
        quiet: bool,
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

const DEFAULT_SCRIPT_PATH: &str = "main.august";

pub fn run(cli: CLI) -> Result<(), CLIError> {
    match cli.command {
        CLICommand::Info => {
            let global_module_dir = {
                let mut home = dirs::home_dir().unwrap();
                home.push(".august");
                home.push("modules");
                home
            };
            let global_module_count = {
                let mut count = 0;
                for i in WalkDir::new(global_module_dir) {
                    let Ok(i) =  i else {
                        continue;
                    };

                    if i.path().extension() == Some(OsStr::new("august")) {
                        count += 1;
                    }
                }
                count
            };
            let module_count = {
                let mut count = 0;
                for i in WalkDir::new(".") {
                    let Ok(i) = i else {
                        continue;
                    };

                    if i.path().extension() == Some(OsStr::new("august")) {
                        count += 1;
                    }
                }
                count
            };

            let mut table = Table::new();

            table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .add_row(vec!["Package Name", env!("CARGO_PKG_NAME")])
                .add_row(vec!["Author(s)", env!("CARGO_PKG_AUTHORS")])
                .add_row(vec!["Version", env!("CARGO_PKG_VERSION")])
                .add_row(vec!["Documentation", env!("CARGO_PKG_HOMEPAGE")])
                .add_row(vec!["Repository", env!("CARGO_PKG_REPOSITORY")])
                .add_row(vec![
                    "Global Modules",
                    format!("{global_module_count} in ~/.august/modules/").as_str(),
                ])
                .add_row(vec![
                    "Local Modules",
                    format!("{module_count} in current directory").as_str(),
                ]);

            println!("{table}");
        }
        CLICommand::Test { script, quiet } => {
            let script = script
                .unwrap_or(PathBuf::from(DEFAULT_SCRIPT_PATH))
                .to_str()
                .unwrap_or(DEFAULT_SCRIPT_PATH)
                .to_string();

            let module = parse_script(script)?;
            let Some(main_task) = module.pragmas.test else {
                println!("{} No task was assigned to `august test`.
        Try adding `#pragma test task_name` to your build script",
                        " ERR ".black().on_red()
                );
                std::process::exit(1);
            };
            ExecutionPool::new(module.tasks, module.cmd_defs)
                .with_notifications(quiet)
                .run(main_task);
        }
        CLICommand::Build { script, quiet } => {
            let script = script
                .unwrap_or(PathBuf::from(DEFAULT_SCRIPT_PATH))
                .to_str()
                .unwrap_or(DEFAULT_SCRIPT_PATH)
                .to_string();

            let module = parse_script(script)?;
            let Some(main_task) = module.pragmas.build else {
                println!("{} No task was assigned to `august build`.
        Try adding `#pragma build task_name` to your build script", " ERR ".black().on_red());
                std::process::exit(1);
            };
            ExecutionPool::new(module.tasks, module.cmd_defs)
                .with_notifications(quiet)
                .run(main_task);
        }
        CLICommand::Run {
            script,
            task_name,
            quiet,
        } => {
            let script = script
                .unwrap_or(PathBuf::from(DEFAULT_SCRIPT_PATH))
                .to_str()
                .unwrap_or(DEFAULT_SCRIPT_PATH)
                .to_string();

            let module = parse_script(script)?;
            ExecutionPool::new(module.tasks, module.cmd_defs)
                .with_notifications(quiet)
                .run(task_name);
        }
        CLICommand::Check { script } => {
            let script = script
                .unwrap_or(PathBuf::from(DEFAULT_SCRIPT_PATH))
                .to_str()
                .unwrap_or(DEFAULT_SCRIPT_PATH)
                .to_string();

            parse_script(script)?;
        }
        CLICommand::Inspect { script } => {
            let script = script
                .unwrap_or(PathBuf::from(DEFAULT_SCRIPT_PATH))
                .to_str()
                .unwrap_or(DEFAULT_SCRIPT_PATH)
                .to_string();

            let module = parse_script(script)?;

            let mut table = Table::new();

            table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_header(["Property", "Contents"])
                .add_row(["Namespace", &module.namespace])
                .add_row(["Pragma", &fmt_pragma(module.pragmas)])
                .add_row(["Tasks", &fmt_tasks(&module.tasks)])
                .add_row(["Command Definitions", &fmt_cmddefs(&module.cmd_defs)]);

            println!("{table}");
        }
        CLICommand::Completions { shell } => {
            let mut cmd = CLI::command();
            generator::generate(shell, &mut cmd, "august", &mut stdout());
        }
    };

    Ok(())
}

#[derive(Debug)]
pub enum CLIError {
    ParserError(ParserErrorHandler),
    RuntimeError(RuntimeError),
}

impl From<ParserErrorHandler> for CLIError {
    fn from(value: ParserErrorHandler) -> Self {
        Self::ParserError(value)
    }
}

impl From<RuntimeError> for CLIError {
    fn from(value: RuntimeError) -> Self {
        Self::RuntimeError(value)
    }
}

impl std::fmt::Display for CLIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let m = match self {
            Self::ParserError(e) => format!("{e}"),
            Self::RuntimeError(e) => format!("{e}"),
        };

        write!(f, "{m}")
    }
}

impl Error for CLIError {}

fn fmt_pragma(pragma: Pragma) -> String {
    format!(
        "Test --> {}\nBuild --> {}",
        pragma.test.unwrap_or("None".to_string()),
        pragma.build.unwrap_or("None".to_string())
    )
}

fn fmt_tasks(hashmap: &HashMap<String, Task>) -> String {
    let mut string = String::new();
    for k in hashmap.keys() {
        string.push_str(&format!("- {k}\n"));
    }
    string
}

fn fmt_cmddefs(hashmap: &HashMap<Command, CommandDefinition>) -> String {
    let mut string = String::new();
    for k in hashmap.keys() {
        let cmd = match k {
            Command::Local(n) => n.to_string(),
            Command::External(ns, n) => format!("{n} from {ns}"),
            Command::Internal(_) => panic!(),
        };

        string.push_str(&format!("- {cmd}\n"));
    }
    string
}
