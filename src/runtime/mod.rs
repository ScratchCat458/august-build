use self::output::RuntimeError;
use crate::{runtime::output::Notification, Command, CommandDefinition, InternalCommand, Task};
use run_script::run_script;
use std::{collections::HashMap, path::PathBuf};

pub mod cli;
pub mod output;

pub struct ExecutionPool {
    pub tasks: HashMap<String, Task>,
    pub cmd_defs: HashMap<Command, CommandDefinition>,
    pub active_tasks: Vec<String>,
}

impl ExecutionPool {
    pub fn new(
        tasks: HashMap<String, Task>,
        cmd_defs: HashMap<Command, CommandDefinition>,
    ) -> Self {
        Self {
            tasks,
            cmd_defs,
            active_tasks: Vec::new(),
        }
    }

    pub fn run(&mut self, main_task: impl Into<String>) {
        let main_task = main_task.into();
        Notification::Start {
            build_goal: main_task.clone(),
        }
        .print();
        match self.deploy_task(main_task) {
            Ok(_) => {
                Notification::Completion.print();
            }
            Err(e) => {
                Notification::Fail { error: e }.print();
                std::process::exit(1);
            }
        }
    }

    pub fn purge_task_as_dependency(&mut self, task_name: impl Into<String>) {
        let task_name = task_name.into();

        self.tasks.iter_mut().for_each(|(_, v)| {
            v.dependencies.remove(&task_name);
        });
    }

    pub fn deploy_task(&mut self, task_name: impl Into<String>) -> Result<(), RuntimeError> {
        let task_name = task_name.into();
        let task = self.tasks.get(&task_name).expect("Bad developer").clone();
        self.active_tasks.push(task_name.clone());

        Notification::TaskRun {
            task_name: task_name.clone(),
            dep_names: task.dependencies.clone(),
        }
        .print();

        if !task.dependencies.is_empty() {
            for dep in &task.dependencies {
                if self.active_tasks.contains(dep) {
                    continue;
                }

                Notification::DependencyRun {
                    dep_name: dep.clone(),
                }
                .print();
                self.deploy_task(dep)?;
            }
        }

        for command in &task.commands {
            self.run_command(command)?;
        }

        self.purge_task_as_dependency(task_name);
        Ok(())
    }

    pub fn run_command(&self, command: &Command) -> Result<(), RuntimeError> {
        if let Command::Internal(ic) = command {
            Notification::CommandRun {
                command: command.clone(),
            }
            .print();

            match ic {
                InternalCommand::Exec(e) => {
                    let (code, _, _) = run_script!(e)?;
                    if code != 0 {
                        return Err(RuntimeError::ProcessFailure(e.to_string(), code));
                    }
                }
                InternalCommand::MakeFile(f) => {
                    std::fs::write(PathBuf::from(f), "")?;
                }
                InternalCommand::MakeDirectory(d) => {
                    std::fs::create_dir_all(PathBuf::from(d))?;
                }
                InternalCommand::SetEnvironmentVar(v, c) => {
                    std::env::set_var(v, c);
                }
                InternalCommand::PrintString(t) => {
                    println!("{t}");
                }
                InternalCommand::PrintFile(f) => {
                    let file = std::fs::read_to_string(f)?;
                    println!("{file}");
                }
                InternalCommand::RemoveDirectory(d) => {
                    std::fs::remove_dir_all(d)?;
                }
                InternalCommand::RemoveFile(f) => {
                    std::fs::remove_file(f)?;
                }
                InternalCommand::CopyFile(s, d) => {
                    std::fs::copy(s, d)?;
                }
                InternalCommand::MoveFile(s, d) => {
                    std::fs::copy(s, d)?;
                    std::fs::remove_file(s)?;
                }
            }
        } else {
            Notification::CommandDefinition {
                command: command.clone(),
            }
            .print();

            let Some(def) = self.cmd_defs.get(command) else {
                    return Err(RuntimeError::NonExistentCommand(command.clone()));
                };
            for cmd in &def.commands {
                self.run_command(cmd)?;
            }
        }
        Ok(())
    }
}
