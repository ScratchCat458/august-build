use std::{
    env,
    ffi::{OsStr, OsString},
    fs::canonicalize,
    future::Future,
    io,
    path::Path,
    process::Output,
    sync::atomic::{AtomicU8, Ordering},
    task::Poll,
};

use arc_swap::ArcSwap;
use dircpy::copy_dir;
use futures::{future::ready, stream::FuturesUnordered, StreamExt, TryStreamExt};
use thiserror::Error;
use tokio::task::block_in_place;

use crate::{parser::Spanned, Command, EnvCommand, FsCommand, HashMap, IoCommand, Module, Unit};

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

pub struct Runtime {
    module: Module,
    notifier: Box<dyn Notifier + Sync>,
    once: HashMap<Spanned<String>, AtomicU8>,
    env_vars: ArcSwap<HashMap<OsString, OsString>>,
}

const UOS_INCOMPLETE: u8 = 0;
const UOS_IN_PROGRESS: u8 = 1;
const UOS_COMPLETE: u8 = 2;
const UOS_FAILED: u8 = 3;

impl Runtime {
    pub fn new(module: Module, notifier: impl Notifier + Sync + 'static) -> Self {
        let once = module
            .units
            .keys()
            .map(|name| (name.clone(), AtomicU8::new(UOS_INCOMPLETE)))
            .collect();

        let env_vars = ArcSwap::from_pointee(env::vars_os().collect());

        Self {
            module,
            notifier: Box::new(notifier),
            once,
            env_vars,
        }
    }

    pub fn notifier(&self) -> &dyn Notifier {
        &*self.notifier
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

    pub async fn run(&self, unit_name: &str) -> Result<(), RuntimeError> {
        // Box::pin because recursive generators are hard
        Box::pin(async {
            let (unit_span, unit) = self.get_unit(unit_name);

            self.notifier.start(unit_name);

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
                                    self.notifier.dependency(unit_name, dep.inner());
                                    match self.run(dep.inner()).await {
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
                    self.notifier.error(&errors);
                    return Err(RuntimeError::DependencyError(unit_span.clone()));
                }
            }
            for cmd in &unit.commands {
                cmd.call(self).await?;
            }

            self.notifier.complete(unit_name);

            Ok(())
        })
        .await
    }
}

impl Command {
    pub async fn call(&self, rt: &Runtime) -> Result<(), RuntimeError> {
        use Command::{Concurrent, DependsOn, Do, Env, Exec, Fs, Io, Meta};

        rt.notifier.call(self);

        match self {
            // no op, shouldn't be in Vec<Command>
            DependsOn(_) | Meta(_) => Ok(()),

            Do(units) => {
                for unit in units {
                    rt.run(unit.inner()).await?;
                }
                Ok(())
            }
            Exec(cmd) => {
                let args = cmd[1..].iter().map(Spanned::inner);
                let handle = duct::cmd(cmd[0].inner(), args)
                    .full_env(rt.env_vars.load().iter())
                    .unchecked()
                    .start()
                    .map_err(|io| RuntimeError::ExecutionFailure(cmd.to_vec(), io))?;
                let fut = HandleFuture { handle };
                let exec = fut
                    .await
                    .map_err(|io| RuntimeError::ExecutionFailure(cmd.to_vec(), io))?;
                if exec.status.success() {
                    Ok(())
                } else {
                    Err(RuntimeError::ExecutionFailure(
                        cmd.to_vec(),
                        io::Error::new(
                            io::ErrorKind::Other,
                            format!("Process returned non-successfully with {}.", exec.status),
                        ),
                    ))
                }
            }
            Concurrent(cmds) => {
                let mut errors = cmds
                    .iter()
                    .map(|cmd| cmd.call(rt))
                    .collect::<FuturesUnordered<_>>()
                    .into_stream()
                    .filter_map(|res| ready(res.err()))
                    .collect::<Vec<_>>()
                    .await;

                // TODO: Make less jank
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors.pop().unwrap())
                }
            }

            Fs(cmd) => cmd.call().await,
            Io(cmd) => cmd.call(),
            Env(cmd) => cmd.call(rt),
        }
    }
}

struct HandleFuture {
    handle: duct::Handle,
}

impl Future for HandleFuture {
    type Output = io::Result<Output>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        if let Some(output) = self.handle.try_wait()? {
            Poll::Ready(Ok(output.clone()))
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

impl FsCommand {
    pub async fn call(&self) -> Result<(), RuntimeError> {
        use tokio::fs;
        use FsCommand::{
            Copy, CopyTo, Create, CreateDir, EPrintFile, Move, MoveTo, PrintFile, Remove,
        };

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
            Copy(src, dst) => Ok(fs_copy(src, dst).await?),
            CopyTo(head, map) => {
                for (src, dst) in expand_binary_map(head, map) {
                    fs_copy(src, &dst).await?;
                }
                Ok(())
            }
            Move(src, dst) => Ok(fs_move(src, dst).await?),
            MoveTo(head, map) => {
                for (src, dst) in expand_binary_map(head, map) {
                    fs_move(src, &dst).await?;
                }
                Ok(())
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

fn expand_binary_map<'a>(
    head: &'a Spanned<String>,
    map: &'a [(Spanned<String>, Option<Spanned<String>>)],
) -> impl Iterator<Item = (&'a Spanned<String>, Spanned<String>)> + 'a {
    map.iter().map(|(src, dst)| {
        let new_dst = dst.as_ref().map_or_else(
            || head.clone().map(extend_path(src.inner())),
            |dst| head.clone().map(extend_path(dst.inner())),
        );
        (src, new_dst)
    })
}

fn extend_path(end: &str) -> impl Fn(String) -> String + '_ {
    |mut head| {
        if !head.ends_with('/') && !head.ends_with('\\') {
            head.push('/');
        }
        head.push_str(end);
        head
    }
}

async fn fs_move(src: &Spanned<String>, dst: &Spanned<String>) -> Result<(), RuntimeError> {
    use tokio::fs;
    fs_copy(src, dst).await?;

    let src_path: &Path = src.inner().as_ref();
    if src_path.is_dir() {
        fs::remove_dir_all(src_path).await
    } else {
        fs::remove_file(src_path).await
    }
    .map_err(|io| RuntimeError::FsError(FsError::RemoveError(src.clone(), io)))
}

async fn fs_copy(src: &Spanned<String>, dst: &Spanned<String>) -> Result<(), RuntimeError> {
    use tokio::fs;
    let src_path: &Path = src.inner().as_ref();
    if src_path.is_dir() {
        // No async variant for dircpy
        block_in_place(|| copy_dir(src_path, dst.inner()))
    } else {
        fs::copy(src_path, dst.inner()).await.map(|_| ())
    }
    .map_err(|io| RuntimeError::FsError(FsError::CopyError(src.clone(), dst.clone(), io)))
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
        use IoCommand::{EPrint, EPrintLn, Print, PrintLn};

        match self {
            PrintLn(t) => {
                println!("{t}");
            }
            Print(t) => {
                print!("{t}");
            }
            EPrintLn(t) => {
                eprintln!("{t}");
            }
            EPrint(t) => {
                eprint!("{t}");
            }
        };

        Ok(())
    }
}

impl EnvCommand {
    pub fn call(&self, rt: &Runtime) -> Result<(), RuntimeError> {
        use EnvCommand::{PathPush, PathRemove, RemoveVar, SetVar};

        // FIX: Find a way to make manipulating ENV vars safe
        match self {
            SetVar(var, val) => {
                rt.env_vars.rcu(|envs| {
                    let mut envs = HashMap::clone(envs);
                    envs.insert(var.inner().into(), val.inner().into());
                    envs
                });
                Ok(())
            }
            RemoveVar(var) => {
                rt.env_vars.rcu(|envs| {
                    let mut envs = HashMap::clone(envs);
                    envs.swap_remove(OsStr::new(var.inner()));
                    envs
                });
                Ok(())
            }
            PathPush(p) => {
                rt.env_vars.rcu(|envs| {
                    let mut envs = HashMap::clone(envs);

                    let mut path_var: Vec<_> = envs
                        .get(OsStr::new("PATH"))
                        .map(|i| env::split_paths(&i).collect())
                        .unwrap_or_default();
                    path_var.push(canonicalize(p.inner()).unwrap());

                    env::join_paths(path_var)
                        .ok()
                        .map(|new_path| envs.insert("PATH".into(), new_path));
                    envs
                });

                Ok(())
            }
            PathRemove(p) => {
                rt.env_vars.rcu(|envs| {
                    let mut envs = HashMap::clone(envs);
                    if let Some(i) = envs.get(OsStr::new("PATH")) {
                        let mut path_var = env::split_paths(&i).collect::<Vec<_>>();
                        path_var.retain(|i| i != &canonicalize(p.inner()).unwrap());

                        env::join_paths(path_var)
                            .ok()
                            .map(|new_path| envs.insert("PATH".into(), new_path));
                    }
                    envs
                });
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

pub enum NotifierEvent<'a> {
    Call(&'a Command),
    Start(&'a str),
    Complete(&'a str),
    Error(&'a [RuntimeError]),
    Dependency { parent: &'a str, name: &'a str },
    BlockOn { parent: &'a str, name: &'a str },
}

/// Trait to hook into runtime events, usually for logging.
pub trait Notifier {
    fn on_event(&self, event: NotifierEvent<'_>);
}

/// Convenience trait for using [`Notifier`]
pub trait NotifierExt: Notifier {
    fn call(&self, command: &Command) {
        self.on_event(NotifierEvent::Call(command));
    }

    fn start(&self, name: &str) {
        self.on_event(NotifierEvent::Start(name));
    }

    fn complete(&self, name: &str) {
        self.on_event(NotifierEvent::Complete(name));
    }

    fn error(&self, errors: &[RuntimeError]) {
        self.on_event(NotifierEvent::Error(errors));
    }

    fn dependency(&self, parent: &str, name: &str) {
        self.on_event(NotifierEvent::Dependency { parent, name });
    }

    fn block_on(&self, parent: &str, name: &str) {
        self.on_event(NotifierEvent::BlockOn { parent, name });
    }
}

impl<T: Notifier + ?Sized> NotifierExt for T {}
