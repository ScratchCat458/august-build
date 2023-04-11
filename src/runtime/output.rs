use std::{collections::HashSet, fmt};

use owo_colors::OwoColorize;

use crate::Command;

#[derive(Debug, Clone)]
pub enum RuntimeError {
    NonExsistentCommand,
    ProcessFailure,
    MalformedShellCommand,
    FailedFileRead,
}

pub enum Notification {
    Start {
        build_goal: String,
    },
    TaskRun {
        task_name: String,
        dep_names: HashSet<String>,
    },
    DependencyRun {
        dep_name: String,
    },
    CommandRun {
        command: Command,
    },
    CommandDefinition {
        command: Command,
    },
    Completion,
    Fail {
        error: RuntimeError,
    },
}

impl Notification {
    pub fn print(&self) {
        println!("{self}");
    }
}

impl fmt::Display for Notification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let m = match self {
            Self::Start { build_goal } => {
                format!(
                    "{} Preparing to complete `{build_goal}`",
                    "    START    ".black().on_green()
                )
            }
            Self::TaskRun {
                task_name,
                dep_names,
            } => {
                format!(
                    "{} Running task `{task_name}` with dependencies `{dep_names:?}`",
                    "    TASK ↴   ".black().on_purple()
                )
            }
            Self::DependencyRun { dep_name } => {
                format!(
                    "{} Expanding dependency `{dep_name}` to task for execution",
                    " DEPDENDENCY ".black().on_yellow()
                )
            }
            Self::CommandRun { command } => {
                let ic = match command {
                    Command::Internal(ic) => ic,
                    _ => panic!(),
                };

                format!(
                    "{} Executing internal command: `{ic:?}`",
                    "   COMMAND   ".black().on_blue()
                )
            }
            Self::CommandDefinition { command } => {
                let c = match command {
                    Command::Local(l) => l.to_owned(),
                    Command::External(n, e) => format!("{n}.{e}"),
                    _ => panic!(),
                };

                format!(
                    "{} Expanding command definition `{c}`",
                    "   CMDDEF↴   ".black().on_bright_cyan()
                )
            }
            Self::Completion => {
                format!(
                    "{} All required tasks have been completed",
                    "    DONE✓    ".black().on_green()
                )
            }
            Self::Fail { error } => {
                format!(
                    "{} An error occurred during runtime: {error:?}",
                    "    FAIL    ".black().on_red()
                )
            }
        };

        write!(f, "{m}")
    }
}
