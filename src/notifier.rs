use std::io::{self, stderr};

use ariadne::{Color, Label, Report, ReportKind, Source};
use owo_colors::OwoColorize;

use crate::{parser::Spanned, runtime::RuntimeError, Command};

#[derive(Debug)]
pub struct Notifier {
    file_name: String,
    code: String,
    silent: bool,
    verbose: bool,
}

impl Notifier {
    pub fn new(file_name: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            file_name: file_name.into(),
            code: code.into(),
            silent: false,
            verbose: false,
        }
    }

    #[inline]
    pub fn silent(mut self) -> Self {
        self.silent = true;
        self
    }

    #[inline]
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    pub fn run(&self, name: &str) {
        if self.silent {
            return;
        }
        let _guard = stderr().lock();
        eprintln!("{} Begining work on unit {}", "[run]".green(), name.green())
    }

    pub fn complete(&self, name: &str) {
        if self.silent {
            return;
        }
        let _guard = stderr().lock();
        eprintln!("{} Completed unit {}", "[run]".green(), name.green())
    }

    pub fn start_dep(&self, parent: &str, child: &str) {
        if self.silent {
            return;
        }
        let _guard = stderr().lock();
        eprintln!(
            "{} Spawning unit {} to resolve dependency of {}",
            "[dep]".yellow(),
            child.yellow(),
            parent.yellow()
        )
    }

    pub fn block_on_dep(&self, parent: &str, child: &str) {
        if self.silent {
            return;
        }
        let _guard = stderr().lock();
        eprintln!(
            "{} Blocking {} until {} reaches completion",
            "[dep]".yellow(),
            parent.yellow(),
            child.yellow()
        );
    }

    pub fn call(&self, cmd: &Command) {
        if self.silent || !self.verbose {
            return;
        }
        use crate::EnvCommand::*;
        use crate::FsCommand::*;
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
            let _guard = stderr().lock();
            eprintln!("{} {inner}", "[cmd]".blue())
        }
    }

    pub fn err(&self, errors: &[RuntimeError]) {
        if self.silent {
            return;
        }
        use crate::runtime::FsError::*;
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
