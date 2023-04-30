use std::collections::{HashMap, HashSet};

pub mod parsing;
pub mod runtime;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Module {
    pub namespace: String,
    pub pragmas: Pragma,
    pub links: HashSet<String>,
    pub tasks: HashMap<String, Task>,
    pub cmd_defs: HashMap<Command, CommandDefinition>,
}

impl Module {
    pub fn namespace(&mut self, namespace: impl Into<String>) -> &mut Self {
        self.namespace = namespace.into();
        self
    }

    pub fn pragma(&mut self, pragma: Pragma) -> &mut Self {
        self.pragmas = pragma;
        self
    }

    pub fn link(&mut self, link: impl Into<String>) -> &mut Self {
        self.links.insert(link.into());
        self
    }

    pub fn task(&mut self, task_name: impl Into<String>, task: Task) -> &mut Self {
        self.tasks.insert(task_name.into(), task);
        self
    }

    pub fn cmd_def(&mut self, name: Command, definition: CommandDefinition) -> &mut Self {
        if let Command::Internal(_) = name {
            panic!("Developer did a no no!");
        }

        self.cmd_defs.insert(name, definition);
        self
    }

    pub fn link_module(&mut self, ext_module: Module) -> &mut Self {
        if !self.links.contains(&ext_module.namespace) {
            panic!(
                "Attempted to link an already linked or non specified module.
Likely a developer end issue but double-check your build scripts and dependencies just in case :)"
            );
        }
        self.links.remove(&ext_module.namespace);

        let mut cmd_defs = HashMap::new();
        for (k, v) in ext_module.cmd_defs {
            let name = match k {
                Command::Local(n) => n,
                _ => unreachable!(),
            };

            cmd_defs.insert(Command::External(ext_module.namespace.to_owned(), name), v);
        }

        self.cmd_defs.extend(cmd_defs);

        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Pragma {
    pub test: Option<String>,
    pub build: Option<String>,
}

impl Pragma {
    pub fn test(&mut self, job_name: impl Into<String>) -> &mut Self {
        self.test = Some(job_name.into());
        self
    }

    pub fn build(&mut self, job_name: impl Into<String>) -> &mut Self {
        self.build = Some(job_name.into());
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Task {
    pub dependencies: HashSet<String>,
    pub commands: Vec<Command>,
}

impl Task {
    pub fn dependency(&mut self, dep_name: impl Into<String>) -> &mut Self {
        self.dependencies.insert(dep_name.into());
        self
    }

    pub fn command(&mut self, command: Command) -> &mut Self {
        self.commands.push(command);
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandDefinition {
    pub commands: Vec<Command>,
}

impl CommandDefinition {
    pub fn command(&mut self, command: Command) -> &mut Self {
        self.commands.push(command);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Command {
    Internal(InternalCommand),
    Local(String),
    External(String, String),
}

// Execution implemented as a part of the runtime
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InternalCommand {
    Exec(String),
    SetEnvironmentVar(String, String),
    PrintString(String),
    PrintFile(String),
    MakeDirectory(String),
    MakeFile(String),
    RemoveDirectory(String),
    RemoveFile(String),
    CopyFile(String, String),
    MoveFile(String, String),
}
