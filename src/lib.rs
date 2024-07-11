use std::collections::{HashMap, HashSet};
use thiserror::Error;

use parser::{Spanned, AST};

pub mod colours;
pub mod error;
pub mod lexer;
pub mod notifier;
pub mod parser;
pub mod runtime;

#[derive(Debug, Clone)]
pub struct Module {
    expose: HashMap<Pragma, Spanned<String>>,
    units: HashMap<Spanned<String>, Unit>,
}

impl Module {
    pub fn lower(ast: Vec<AST>) -> Result<Self, Vec<LowerError>> {
        let mut errors = Vec::new();

        let mut unit_iter = ast.iter().filter(|a| matches!(a, AST::Unit(_, _)));
        let mut units: HashMap<Spanned<String>, Unit> =
            HashMap::with_capacity(unit_iter.size_hint().0);

        while let Some(AST::Unit(name, cmds)) = unit_iter.next() {
            let res = Unit::lower(cmds);
            let unit = match res {
                Ok(u) => u,
                Err(e) => {
                    errors.extend(e);
                    // Still need the unit to exist for name checks
                    // Lower will still error
                    Unit::default()
                }
            };

            if let Some((other, _)) = units.get_key_value(name) {
                errors.push(LowerError::DuplicateUnit(other.clone(), name.clone()));
            } else {
                units.insert(name.clone(), unit);
            }
        }

        let mut expose_iter = ast.into_iter().filter(|a| matches!(a, AST::Expose(_, _)));
        let mut expose = HashMap::with_capacity(expose_iter.size_hint().0);

        while let Some(AST::Expose(prag, unit)) = expose_iter.next() {
            let mut err = false;
            if expose.contains_key(&prag) {
                err = true;
                errors.push(LowerError::DuplicateExpose(prag, unit.clone()));
            }
            if !units.contains_key(&unit) {
                err = true;
                errors.push(LowerError::NameError(unit.clone()));
            }
            if err {
                continue;
            }

            expose.insert(prag, unit.clone());
        }

        for unit in units.values() {
            for u in &unit.depends_on {
                if !units.contains_key(u) {
                    errors.push(LowerError::NameError(u.clone()))
                }
            }

            let mut dos_iter = unit.commands.iter().filter(|c| matches!(c, Command::Do(_)));
            while let Some(Command::Do(dos)) = dos_iter.next() {
                for d in dos {
                    if !units.contains_key(d) {
                        errors.push(LowerError::NameError(d.clone()))
                    }
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(Self { expose, units })
    }

    pub fn units(&self) -> &HashMap<Spanned<String>, Unit> {
        &self.units
    }

    pub fn unit_exists(&self, name: impl Into<String>) -> bool {
        self.units.contains_key(&Spanned::new(name.into()))
    }

    pub fn unit_by_pragma(&self, pragma: Pragma) -> Option<String> {
        self.expose.get(&pragma).map(Spanned::inner_owned)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LowerError {
    #[error("Attempted to define another binding for pragma {0:?}")]
    DuplicateExpose(Pragma, Spanned<String>),
    #[error("Attempted to define multiple units with the name {1}")]
    DuplicateUnit(Spanned<String>, Spanned<String>),
    #[error("Dependency {1} defined multiple times in the same unit")]
    DuplicateDependency(Spanned<String>, Spanned<String>),
    #[error("Meta item {1} defined multiple times in the same unit")]
    DuplicateMetaItem(Spanned<String>, Spanned<String>),
    #[error("Refers to a unit {0} that doesn't exist")]
    NameError(Spanned<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Unit {
    depends_on: HashSet<Spanned<String>>,
    pub meta: HashMap<Spanned<String>, String>,
    commands: Vec<Command>,
}

impl Unit {
    pub fn lower(cmds: &[Command]) -> Result<Self, Vec<LowerError>> {
        let mut errors = Vec::new();

        let depends_iter = cmds
            .iter()
            .filter_map(|c| {
                if let Command::DependsOn(deps) = c {
                    Some(deps)
                } else {
                    None
                }
            })
            .flatten();
        let mut depends_on: HashSet<Spanned<String>> =
            HashSet::with_capacity(depends_iter.size_hint().0);

        for dep in depends_iter {
            if let Some(other) = depends_on.get(dep) {
                errors.push(LowerError::DuplicateDependency(other.clone(), dep.clone()))
            } else {
                depends_on.insert(dep.clone());
            }
        }

        let meta_iter = cmds.iter().filter_map(|c| {
            if let Command::Meta(meta) = c {
                Some(meta)
            } else {
                None
            }
        });
        let mut meta: HashMap<Spanned<String>, String> =
            HashMap::with_capacity(meta_iter.size_hint().0);

        for meta_items in meta_iter {
            for (var, val) in meta_items {
                if let Some((other, _)) = meta.get_key_value(var) {
                    errors.push(LowerError::DuplicateMetaItem(other.clone(), var.clone()))
                } else {
                    meta.insert(var.clone(), val.clone());
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(Self {
            depends_on,
            meta,
            commands: cmds
                .iter()
                .filter(|c| !matches!(c, Command::Meta(_) | Command::DependsOn(_)))
                .cloned()
                .collect(),
        })
    }

    pub fn deps(&self) -> &HashSet<Spanned<String>> {
        &self.depends_on
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pragma {
    Test,
    Build,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    DependsOn(Vec<Spanned<String>>),
    Meta(Vec<(Spanned<String>, String)>),
    Do(Vec<Spanned<String>>),
    Exec(Vec<Spanned<String>>),
    Concurrent(Vec<Box<Command>>),

    Fs(FsCommand),
    Io(IoCommand),
    Env(EnvCommand),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsCommand {
    Create(Spanned<String>),
    CreateDir(Spanned<String>),
    Remove(Spanned<String>),
    Move(Spanned<String>, Spanned<String>),
    MoveTo(
        Spanned<String>,
        Vec<(Spanned<String>, Option<Spanned<String>>)>,
    ),
    Copy(Spanned<String>, Spanned<String>),
    CopyTo(
        Spanned<String>,
        Vec<(Spanned<String>, Option<Spanned<String>>)>,
    ),
    PrintFile(Spanned<String>),
    EPrintFile(Spanned<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IoCommand {
    PrintLn(Spanned<String>),
    Print(Spanned<String>),
    EPrintLn(Spanned<String>),
    EPrint(Spanned<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvCommand {
    SetVar(Spanned<String>, Spanned<String>),
    RemoveVar(Spanned<String>),
    PathPush(Spanned<String>),
    PathRemove(Spanned<String>),
}
