use std::{collections::HashSet, fmt};

use owo_colors::OwoColorize;

use crate::Command;

#[derive(Debug)]
pub enum RuntimeError {
    NonExistentCommand(Command),
    ProcessFailure(String, i32),
    RunScriptError(run_script::ScriptError),
    IoError(std::io::Error),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let m = match self {
            Self::NonExistentCommand(c) => {
                let command = match c {
                    Command::Local(n) => n.to_string(),
                    Command::External(ns, n) => format!("{n} from {ns}"),
                    Command::Internal(_) => panic!(),
                };

                format!("Unable to find the command {command}")
            }
            Self::ProcessFailure(p, ec) => {
                format!("Process `{p}` failed with error code {ec}")
            }
            Self::RunScriptError(e) => {
                format!("An issue regarding `run_script` occurred: {e}")
            }
            Self::IoError(e) => format!("{e}"),
        };
        writeln!(f, "{m}")
    }
}

impl From<std::io::Error> for RuntimeError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<run_script::ScriptError> for RuntimeError {
    fn from(value: run_script::ScriptError) -> Self {
        Self::RunScriptError(value)
    }
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
        eprintln!("{self}");
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
                let Command::Internal(ic) = command else {
                    panic!("Not an internal command")
                };

                format!(
                    "{} Executing internal command: `{ic:?}`",
                    "   COMMAND   ".black().on_blue()
                )
            }
            Self::CommandDefinition { command } => {
                let c = match command {
                    Command::Local(l) => l.clone(),
                    Command::External(n, e) => format!("{n}.{e}"),
                    Command::Internal(_) => panic!("Not a CommandDefinition"),
                };

                format!(
                    "{} Expanding command definition `{c}`",
                    "   CMDDEF↴   ".black().on_bright_cyan()
                )
            }
            Self::Completion => {
                format!(
                    "{} All required tasks have been completed",
                    "    DONE ✓   ".black().on_green()
                )
            }
            Self::Fail { error } => {
                format!(
                    "{} An error occurred during runtime: {error:?}",
                    "    FAIL ✕   ".black().on_red()
                )
            }
        };

        write!(f, "{m}")
    }
}
