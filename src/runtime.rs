use std::{
    collections::HashMap,
    env,
    fs::{self, canonicalize},
    future::Future,
    hint, io,
    path::Path,
    process::{self, Stdio},
    sync::{
        atomic::{AtomicU8, Ordering},
        Mutex,
    },
    task::Poll,
    thread,
};

use crossbeam_utils::Backoff;
use dircpy::copy_dir;
use futures::{future::ready, stream::FuturesUnordered, StreamExt, TryStreamExt};
use thiserror::Error;
use tokio::task::block_in_place;
use which::which;

use crate::{
    notifier::Notifier, parser::Spanned, Command, EnvCommand, FsCommand, IoCommand, Module, Unit,
};

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("A dependency of this unit failed")]
    DependencyError(Spanned<String>),
    #[error("Dependency {1} failed preventing completion of {0}")]
    FailedDependency(String, Spanned<String>),
    #[error("Failed to execute {0:?}: {1}")]
    ExecutionFailure(Vec<Spanned<String>>, io::Error),
    #[error("{0}")]
    FsError(FsError),
    #[error("{0}")]
    JoinPathsError(env::JoinPathsError),
    #[error("Command {0:?} isn't supported on this runtime")]
    CommandUnsupported(Command),
}

#[derive(Debug)]
pub struct Runtime {
    module: Module,
    notifier: Notifier,
    once: HashMap<Spanned<String>, AtomicU8>,
}

const UOS_INCOMPLETE: u8 = 0;
const UOS_IN_PROGRESS: u8 = 1;
const UOS_COMPLETE: u8 = 2;
const UOS_FAILED: u8 = 3;

impl Runtime {
    pub fn new(module: Module, notifier: Notifier) -> Self {
        let mut once = HashMap::with_capacity(module.units.len());
        for name in module.units.keys() {
            once.insert(name.clone(), AtomicU8::new(UOS_INCOMPLETE));
        }

        Self {
            module,
            notifier,
            once,
        }
    }

    pub fn notifier(&self) -> &Notifier {
        &self.notifier
    }

    fn get_unit(&self, name: impl Into<String>) -> (&Spanned<String>, &Unit) {
        self.module
            .units
            .get_key_value(&Spanned::new(name.into()))
            .unwrap()
    }

    fn get_uos(&self, name: impl Into<String>) -> &AtomicU8 {
        self.once.get(&Spanned::new(name.into())).unwrap()
    }

    pub fn run(&self, unit_name: &str) -> Result<(), RuntimeError> {
        let (unit_span, unit) = self.get_unit(unit_name);
        let errors = Mutex::new(Vec::new());

        self.notifier.run(unit_name);

        if !unit.depends_on.is_empty() {
            thread::scope(|s| {
                let mut deps = unit
                    .depends_on
                    .iter()
                    .filter(|d| self.get_uos(d.inner()).load(Ordering::Acquire) == UOS_INCOMPLETE);
                let first = deps.next();

                #[allow(clippy::while_let_on_iterator)]
                while let Some(name) = deps.next() {
                    let uos_state = self.get_uos(name.inner());
                    let uos = uos_state.compare_exchange(
                        UOS_INCOMPLETE,
                        UOS_IN_PROGRESS,
                        Ordering::Acquire,
                        Ordering::Relaxed,
                    );
                    if uos.is_ok() {
                        s.spawn(|| {
                            self.notifier.start_dep(unit_name, name.inner());
                            if let Err(e) = self.run(name.inner()) {
                                uos_state.store(UOS_FAILED, Ordering::Release);
                                errors.lock().unwrap().push(e);
                            } else {
                                uos_state.store(UOS_COMPLETE, Ordering::Release);
                            }
                        });
                    } else if Err(UOS_FAILED) == uos {
                        errors.lock().unwrap().push(RuntimeError::FailedDependency(
                            unit_name.to_string(),
                            name.clone(),
                        ));
                    }
                }

                if let Some(first) = first {
                    let uos_state = self.get_uos(first.inner());
                    let uos = uos_state.compare_exchange(
                        UOS_INCOMPLETE,
                        UOS_IN_PROGRESS,
                        Ordering::Acquire,
                        Ordering::Relaxed,
                    );
                    if uos.is_ok() {
                        self.notifier.start_dep(unit_name, first.inner());
                        if let Err(e) = self.run(first.inner()) {
                            uos_state.store(UOS_FAILED, Ordering::Release);
                            errors.lock().unwrap().push(e);
                        } else {
                            uos_state.store(UOS_COMPLETE, Ordering::Release);
                        }
                    } else if Err(UOS_FAILED) == uos {
                        errors.lock().unwrap().push(RuntimeError::FailedDependency(
                            unit_name.to_string(),
                            first.clone(),
                        ));
                    }
                }
            });

            let mut errors = errors.into_inner().unwrap();
            unit.depends_on
                .iter()
                .filter_map(|d| {
                    let uos = self.get_uos(d.inner());
                    if uos.load(Ordering::Acquire) == UOS_IN_PROGRESS {
                        Some((d, uos))
                    } else {
                        None
                    }
                })
                .for_each(|(d, uos)| {
                    self.notifier.block_on_dep(unit_name, d.inner());
                    let backoff = Backoff::new();
                    while uos.load(Ordering::Acquire) < UOS_COMPLETE {
                        // WARN: Graphic depiction of long running spin loop
                        hint::spin_loop();
                        backoff.snooze();
                    }
                    if uos.load(Ordering::Relaxed) == UOS_FAILED {
                        errors.push(RuntimeError::FailedDependency(
                            unit_name.to_string(),
                            d.clone(),
                        ));
                    }
                });

            if !errors.is_empty() {
                self.notifier.err(&errors);
                return Err(RuntimeError::DependencyError(unit_span.clone()));
            }
        }

        for cmd in &unit.commands {
            cmd.call(self)?;
        }

        self.notifier.complete(unit_name);

        Ok(())
    }

    pub async fn run_async(&self, unit_name: &str) -> Result<(), RuntimeError> {
        let (unit_span, unit) = self.get_unit(unit_name);

        self.notifier.run(unit_name);

        if !unit.depends_on.is_empty() {
            let futs = unit
                .depends_on
                .iter()
                .map(|dep| {
                    Box::pin(async move {
                        let uos_state = self.get_uos(dep.inner());
                        let uos = uos_state.compare_exchange(
                            UOS_INCOMPLETE,
                            UOS_IN_PROGRESS,
                            Ordering::Acquire,
                            Ordering::Relaxed,
                        );
                        match uos {
                            Ok(_) => {
                                self.notifier.start_dep(unit_name, dep.inner());
                                match self.run_async(dep.inner()).await {
                                    Err(e) => {
                                        uos_state.store(UOS_FAILED, Ordering::Release);
                                        Err(e)
                                    }
                                    Ok(o) => {
                                        uos_state.store(UOS_COMPLETE, Ordering::Release);
                                        Ok(o)
                                    }
                                }
                            }
                            Err(UOS_FAILED) => Err(RuntimeError::FailedDependency(
                                unit_name.to_owned(),
                                dep.clone(),
                            )),
                            Err(UOS_IN_PROGRESS) => BlockOnDepFuture { uos: uos_state }
                                .await
                                .map_err(|f| f(unit_name.to_owned(), dep.clone())),
                            _ => Ok(()),
                        }
                    })
                })
                .collect::<FuturesUnordered<_>>();

            let errors = futs
                .into_stream()
                .filter_map(|res| ready(res.err()))
                .collect::<Vec<_>>()
                .await;
            if !errors.is_empty() {
                self.notifier.err(&errors);
                return Err(RuntimeError::DependencyError(unit_span.clone()));
            }
        }
        for cmd in &unit.commands {
            cmd.call_async(self).await?;
        }

        self.notifier.complete(unit_name);

        Ok(())
    }
}

impl Command {
    pub fn call(&self, rt: &Runtime) -> Result<(), RuntimeError> {
        use Command::*;

        rt.notifier.call(self);

        match self {
            // no op, shouldn't be in Vec<Command>
            DependsOn(_) => Ok(()),
            Meta(_) => Ok(()),

            Do(units) => units.iter().try_for_each(|unit| rt.run(unit.inner())),
            Exec(cmd) => exec(cmd),

            Fs(cmd) => cmd.call(),
            Io(cmd) => cmd.call(),
            Env(cmd) => cmd.call(),

            cmd => Err(RuntimeError::CommandUnsupported(cmd.clone())),
        }
    }

    pub async fn call_async(&self, rt: &Runtime) -> Result<(), RuntimeError> {
        use Command::*;

        rt.notifier.call(self);

        match self {
            // no op, shouldn't be in Vec<Command>
            DependsOn(_) => Ok(()),
            Meta(_) => Ok(()),

            Do(units) => units.iter().try_for_each(|unit| rt.run(unit.inner())),
            Exec(cmd) => exec_async(cmd).await,
            Concurrent(cmds) => {
                let mut errors = cmds
                    .iter()
                    .map(|cmd| cmd.call_async(rt))
                    .collect::<FuturesUnordered<_>>()
                    .into_stream()
                    .filter_map(|res| ready(res.err()))
                    .collect::<Vec<_>>()
                    .await;

                // TODO: Make less jank
                if !errors.is_empty() {
                    Err(errors.pop().unwrap())
                } else {
                    Ok(())
                }
            }

            Fs(cmd) => cmd.call_async().await,
            Io(cmd) => cmd.call(),
            Env(cmd) => cmd.call(),
        }
    }
}

fn exec(cmd: &[Spanned<String>]) -> Result<(), RuntimeError> {
    let prog = &cmd[0];
    let args = cmd[1..].iter().map(Spanned::inner);

    let exec = process::Command::new(which(prog.inner()).map_err(|e| {
        RuntimeError::ExecutionFailure(
            cmd.to_vec(),
            io::Error::new(io::ErrorKind::NotFound, e.to_string()),
        )
    })?)
    .args(args)
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .output()
    .map_err(|io| RuntimeError::ExecutionFailure(cmd.to_vec(), io))?;

    if !exec.status.success() {
        return Err(RuntimeError::ExecutionFailure(
            cmd.to_vec(),
            io::Error::new(
                io::ErrorKind::Other,
                format!("Process returned non-successfully with {}.", exec.status),
            ),
        ));
    }

    Ok(())
}

async fn exec_async(cmd: &[Spanned<String>]) -> Result<(), RuntimeError> {
    use tokio::process;

    let prog = &cmd[0];
    let args = cmd[1..].iter().map(Spanned::inner);

    let exec = process::Command::new(which(prog.inner()).map_err(|e| {
        RuntimeError::ExecutionFailure(
            cmd.to_vec(),
            io::Error::new(io::ErrorKind::NotFound, e.to_string()),
        )
    })?)
    .args(args)
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .output()
    .await
    .map_err(|io| RuntimeError::ExecutionFailure(cmd.to_vec(), io))?;

    if !exec.status.success() {
        return Err(RuntimeError::ExecutionFailure(
            cmd.to_vec(),
            io::Error::new(
                io::ErrorKind::Other,
                format!("Process returned non-successfully with {}.", exec.status),
            ),
        ));
    }

    Ok(())
}

impl FsCommand {
    pub fn call(&self) -> Result<(), RuntimeError> {
        use FsCommand::*;

        match self {
            Create(p) => fs::File::create(p.inner())
                .map(|_| ())
                .map_err(|io| FsError::CreateFileError(p.clone(), io)),
            CreateDir(p) => {
                fs::create_dir_all(p.inner()).map_err(|io| FsError::CreateDirError(p.clone(), io))
            }
            Remove(p) => {
                let path: &Path = p.inner().as_ref();
                if path.is_dir() {
                    fs::remove_dir_all(path)
                } else {
                    fs::remove_file(path)
                }
                .map_err(|io| FsError::RemoveError(p.clone(), io))
            }
            Copy(src, dst) => {
                let src_path: &Path = src.inner().as_ref();
                if src_path.is_dir() {
                    copy_dir(src_path, dst.inner())
                } else {
                    fs::copy(src_path, dst.inner()).map(|_| ())
                }
                .map_err(|io| FsError::CopyError(src.clone(), dst.clone(), io))
            }
            Move(src, dst) => {
                let src_path: &Path = src.inner().as_ref();
                if src_path.is_dir() {
                    copy_dir(src_path, dst.inner())
                } else {
                    fs::copy(src_path, dst.inner()).map(|_| ())
                }
                .map_err(|io| {
                    RuntimeError::FsError(FsError::CopyError(src.clone(), dst.clone(), io))
                })?;

                if src_path.is_dir() {
                    fs::remove_dir_all(src_path)
                } else {
                    fs::remove_file(src_path)
                }
                .map_err(|io| FsError::RemoveError(src.clone(), io))
            }
            PrintFile(p) => {
                let contents = fs::read_to_string(p.inner())
                    .map_err(|io| RuntimeError::FsError(FsError::FileAccessError(p.clone(), io)))?;
                println!("{contents}");
                Ok(())
            }
            EPrintFile(p) => {
                let contents = fs::read_to_string(p.inner())
                    .map_err(|io| RuntimeError::FsError(FsError::FileAccessError(p.clone(), io)))?;
                eprintln!("{contents}");
                Ok(())
            }
        }
        .map_err(RuntimeError::FsError)
    }

    pub async fn call_async(&self) -> Result<(), RuntimeError> {
        use tokio::fs;
        use FsCommand::*;

        match self {
            Create(p) => fs::File::create(p.inner())
                .await
                .map(|_| ())
                .map_err(|io| FsError::CreateFileError(p.clone(), io)),
            CreateDir(p) => fs::create_dir_all(p.inner())
                .await
                .map_err(|io| FsError::CreateDirError(p.clone(), io)),
            Remove(p) => {
                let path: &Path = p.inner().as_ref();
                if path.is_dir() {
                    fs::remove_dir_all(path).await
                } else {
                    fs::remove_file(path).await
                }
                .map_err(|io| FsError::RemoveError(p.clone(), io))
            }
            Copy(src, dst) => {
                let src_path: &Path = src.inner().as_ref();
                if src_path.is_dir() {
                    // No async variant for dircpy
                    block_in_place(|| copy_dir(src_path, dst.inner()))
                } else {
                    fs::copy(src_path, dst.inner()).await.map(|_| ())
                }
                .map_err(|io| FsError::CopyError(src.clone(), dst.clone(), io))
            }
            Move(src, dst) => {
                let src_path: &Path = src.inner().as_ref();
                if src_path.is_dir() {
                    // No async variant for dircpy
                    block_in_place(|| copy_dir(src_path, dst.inner()))
                } else {
                    fs::copy(src_path, dst.inner()).await.map(|_| ())
                }
                .map_err(|io| {
                    RuntimeError::FsError(FsError::CopyError(src.clone(), dst.clone(), io))
                })?;

                if src_path.is_dir() {
                    fs::remove_dir_all(src_path).await
                } else {
                    fs::remove_file(src_path).await
                }
                .map_err(|io| FsError::RemoveError(src.clone(), io))
            }
            PrintFile(p) => {
                let contents = fs::read_to_string(p.inner())
                    .await
                    .map_err(|io| RuntimeError::FsError(FsError::FileAccessError(p.clone(), io)))?;
                println!("{contents}");
                Ok(())
            }
            EPrintFile(p) => {
                let contents = fs::read_to_string(p.inner())
                    .await
                    .map_err(|io| RuntimeError::FsError(FsError::FileAccessError(p.clone(), io)))?;
                eprintln!("{contents}");
                Ok(())
            }
        }
        .map_err(RuntimeError::FsError)
    }
}

#[derive(Debug, Error)]
pub enum FsError {
    #[error("Failed to create file {0}")]
    CreateFileError(Spanned<String>, io::Error),
    #[error("Failed to create directory {0}")]
    CreateDirError(Spanned<String>, io::Error),
    #[error("Failed to remove {0}")]
    RemoveError(Spanned<String>, io::Error),
    #[error("Failed to get contents of file {0}")]
    FileAccessError(Spanned<String>, io::Error),
    #[error("Failed to copy {0} to {1}")]
    CopyError(Spanned<String>, Spanned<String>, io::Error),
}

impl IoCommand {
    pub fn call(&self) -> Result<(), RuntimeError> {
        use IoCommand::*;

        match self {
            PrintLn(t) => println!("{t}"),
            Print(t) => print!("{t}"),
            EPrintLn(t) => eprintln!("{t}"),
            EPrint(t) => eprint!("{t}"),
        };

        Ok(())
    }
}

impl EnvCommand {
    pub fn call(&self) -> Result<(), RuntimeError> {
        use EnvCommand::*;

        match self {
            SetVar(var, val) => {
                env::set_var(var.inner(), val.inner());
                Ok(())
            }
            RemoveVar(var) => {
                env::remove_var(var.inner());
                Ok(())
            }
            PathPush(p) => {
                let mut path_var: Vec<_> = env::var_os("PATH")
                    .map(|i| env::split_paths(&i).collect())
                    .unwrap_or_default();
                path_var.push(canonicalize(p.inner()).unwrap());

                let new_path = env::join_paths(path_var).map_err(RuntimeError::JoinPathsError)?;
                env::set_var("PATH", new_path);

                Ok(())
            }
            PathRemove(p) => {
                if let Some(i) = env::var_os("PATH") {
                    let mut path_var = env::split_paths(&i).collect::<Vec<_>>();
                    path_var.retain(|i| i != &canonicalize(p.inner()).unwrap());

                    let new_path =
                        env::join_paths(path_var).map_err(RuntimeError::JoinPathsError)?;
                    env::set_var("PATH", new_path);
                }

                Ok(())
            }
        }
    }
}

struct BlockOnDepFuture<'a> {
    uos: &'a AtomicU8,
}

impl Future for BlockOnDepFuture<'_> {
    type Output = Result<(), fn(String, Spanned<String>) -> RuntimeError>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        if self.uos.load(Ordering::Acquire) < UOS_COMPLETE {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }

        if self.uos.load(Ordering::Relaxed) == UOS_FAILED {
            Poll::Ready(Err(|unit_name, dep_name| {
                RuntimeError::FailedDependency(unit_name, dep_name)
            }))
        } else {
            Poll::Ready(Ok(()))
        }
    }
}
