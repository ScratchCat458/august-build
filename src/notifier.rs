use std::io;

use crate::colours::OwoColorizeStderrSupported;
use ariadne::{Color, Label, Report, ReportKind, Source};

use august_build::{
    parser::Spanned,
    runtime::{Notifier, NotifierEvent, RuntimeError},
    Command,
};

#[derive(Debug)]
pub struct SilentNotifier;

impl Notifier for SilentNotifier {
    fn on_event(&self, _event: NotifierEvent<'_>) {}
}

#[derive(Debug)]
pub struct LogNotifier {
    file_name: String,
    code: String,
    verbose: bool,
}

impl LogNotifier {
    pub fn new(file_name: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            file_name: file_name.into(),
            code: code.into(),
            verbose: false,
        }
    }

    #[inline]
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    fn cmd_call(&self, cmd: &Command) {
        if !self.verbose {
            return;
        }
        use august_build::EnvCommand::*;
        use august_build::FsCommand::*;
        use Command::*;

        let text = match cmd {
            Exec(args) => Some(
                args.iter()
                    .map(Spanned::inner)
                    .fold(format!("{} ", "[exec]".bright_blue()), |agg, s| {
                        agg + s + " "
                    }),
            ),
            Env(SetVar(k, v)) => Some(format!(
                "{} Setting {k} to {v}",
                "[env::set_var]".bright_blue()
            )),
            Env(RemoveVar(k)) => Some(format!(
                "{} Clearing the variable {k}",
                "[env::remove_var]".bright_blue()
            )),
            Env(PathPush(p)) => Some(format!(
                "{} Adding {p} to PATH",
                "[env::path_push]".bright_blue()
            )),
            Env(PathRemove(p)) => Some(format!(
                "{} Removing {p} from PATH",
                "[env::path_remove]".bright_blue()
            )),
            Fs(Create(p)) => Some(format!(
                "{} Creating empty file {p}",
                "[fs::create]".bright_blue()
            )),
            Fs(CreateDir(p)) => Some(format!(
                "{} Creating directory {p}",
                "[fs::create_dir]".bright_blue()
            )),
            Fs(Remove(p)) => Some(format!("{} Removing {p}", "[fs::remove]".bright_blue())),
            Fs(Copy(src, dst)) => Some(format!(
                "{} Copying {src} to {dst}",
                "[fs::copy]".bright_blue()
            )),
            Fs(Move(src, dst)) => Some(format!(
                "{} Moving {src} to {dst}",
                "[fs::move]".bright_blue()
            )),
            _ => None,
        };

        if let Some(inner) = text {
            eprintln!("{} {inner}", "[cmd]".blue())
        }
    }

    fn err(&self, errors: &[RuntimeError]) {
        use august_build::runtime::FsError::*;
        use RuntimeError::*;

        let fs_single = |p: &Spanned<String>, io: &io::Error, message: &str| {
            Report::build(
                ReportKind::Custom("[err]", Color::Red),
                self.file_name.clone(),
                p.span().start,
            )
            .with_message(message)
            .with_note(io.to_string())
            .with_label(Label::new((self.file_name.clone(), p.span())).with_color(Color::Red))
            .finish()
            .eprint((self.file_name.clone(), Source::from(self.code.clone())))
            .ok();
        };

        for err in errors {
            match err {
                DependencyError(span) => {
                    eprintln!(
                        "{} Failed to complete {} due to other errors",
                        "[err]".red(),
                        span.red()
                    )
                }
                FailedDependency(parent, child) => {
                    eprintln!(
                        "{} Unable to complete unit {} due to {} failing",
                        "[err]".red(),
                        parent.cyan(),
                        child.red()
                    )
                }
                ExecutionFailure(args, io) => {
                    if !args.is_empty() {
                        let start = args.first().unwrap().span().start;
                        let end = args.last().unwrap().span().end;

                        Report::build(ReportKind::Error, self.file_name.clone(), start)
                            .with_label(
                                Label::new((self.file_name.clone(), start..end))
                                    .with_color(Color::Red),
                            )
                            .with_message("Failed to execute process")
                            .with_note(io.to_string())
                            .finish()
                            .eprint((self.file_name.clone(), Source::from(self.code.clone())))
                            .ok();
                    }
                }
                FsError(CreateFileError(p, io)) => fs_single(p, io, "Failed to create file"),
                FsError(CreateDirError(p, io)) => fs_single(p, io, "Failed to create directory"),
                FsError(RemoveError(p, io)) => fs_single(p, io, "Failed to remove file/directory"),
                FsError(FileAccessError(p, io)) => {
                    fs_single(p, io, "Unable to read the file contents")
                }
                FsError(CopyError(src, dst, io)) => {
                    Report::build(
                        ReportKind::Custom("[err]", Color::Red),
                        self.file_name.clone(),
                        src.span().start,
                    )
                    .with_message(format!("Unable copy {} to {}", src.cyan(), dst.cyan()))
                    .with_note(io.to_string())
                    .with_label(
                        Label::new((self.file_name.clone(), src.span())).with_color(Color::Red),
                    )
                    .with_label(
                        Label::new((self.file_name.clone(), dst.span())).with_color(Color::Red),
                    )
                    .finish()
                    .eprint((self.file_name.clone(), Source::from(self.code.clone())))
                    .ok();
                }
                JoinPathsError(e) => {
                    eprintln!("{} Error occured when join to PATH: {e}", "[err]".red())
                }
                CommandUnsupported(cmd) => {
                    eprintln!(
                        "{} Command {cmd:?} is unsupported on the current runtime",
                        "[err]".red()
                    )
                }
            }
        }
    }
}

impl Notifier for LogNotifier {
    fn on_event(&self, event: NotifierEvent<'_>) {
        match event {
            NotifierEvent::Call(cmd) => self.cmd_call(cmd),
            NotifierEvent::Start(name) => {
                eprintln!("{} Begining work on unit {}", "[run]".green(), name.green())
            }
            NotifierEvent::Complete(name) => {
                eprintln!("{} Completed unit {}", "[run]".green(), name.green())
            }
            NotifierEvent::Error(err) => self.err(err),
            NotifierEvent::Dependency { parent, name } => eprintln!(
                "{} Spawning unit {} to resolve dependency of {}",
                "[dep]".yellow(),
                name.yellow(),
                parent.yellow()
            ),
            NotifierEvent::BlockOn { parent, name } => eprintln!(
                "{} Blocking {} until {} reaches completion",
                "[dep]".yellow(),
                parent.yellow(),
                name.yellow()
            ),
        }
    }
}
