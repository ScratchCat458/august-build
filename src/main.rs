use std::{
    env::set_current_dir,
    fs::{canonicalize, read_to_string},
    io::{stderr, stdout},
    path::{Path, PathBuf},
    process::exit,
};

use august_build::{lexer::lexer, parser::parser, runtime::Runtime, Module, Pragma};
use chumsky::{Parser, Stream};
use clap::CommandFactory;
use cli::Cli;
use comfy_table::{
    modifiers::{UTF8_ROUND_CORNERS, UTF8_SOLID_INNER_BORDERS},
    presets::UTF8_FULL,
    Row, Table,
};
use thiserror::Error;

use crate::{
    cli::{CLICommand, ColourSupport},
    colours::OwoColorizeStderrSupported,
    error::{LowerErrorFormatter, ParserErrorFormatter},
    notifier::{LogNotifier, SilentNotifier},
};

mod cli;
mod colours;
mod error;
mod notifier;

fn main() {
    if let Err(e) = do_main() {
        eprintln!("{} {e}", "[err]".red());
        exit(1);
    }
}

fn do_main() -> Result<(), CLIError> {
    use CLICommand::*;

    let cli = <Cli as clap::Parser>::parse();

    match cli.colour {
        ColourSupport::Always => {
            owo_colors::set_override(true);
        }
        ColourSupport::Never => owo_colors::set_override(false),
        _ => {}
    }

    match cli.subcommand {
        Check => {
            parse_file(&cli.script)?;
        }
        Inspect => {
            let (module, _) = parse_file(&cli.script)?;
            inspect(&module);
        }
        Build => {
            let (module, code) = parse_file(&cli.script)?;
            let this = module
                .unit_by_pragma(Pragma::Build)
                .ok_or(CLIError::NonExposedPragma(Pragma::Build))?
                .clone();
            run_unit_async(&cli, module, &code, &this)?
        }
        Test => {
            let (module, code) = parse_file(&cli.script)?;
            let this = module
                .unit_by_pragma(Pragma::Test)
                .ok_or(CLIError::NonExposedPragma(Pragma::Test))?
                .clone();
            run_unit_async(&cli, module, &code, &this)?
        }
        Run {
            ref unit,
            threads_runtime,
        } => {
            let (module, code) = parse_file(&cli.script)?;
            if module.unit_exists(unit) {
                if threads_runtime {
                    run_unit(&cli, module, &code, unit)?
                } else {
                    run_unit_async(&cli, module, &code, unit)?
                }
            } else {
                Err(CLIError::NonExistentUnit(unit.clone()))?;
            }
        }
        Completions { shell } => {
            clap_complete::generate(shell, &mut Cli::command(), "august", &mut stdout())
        }
        Info => {
            let mut table = Table::new();

            table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .apply_modifier(UTF8_SOLID_INNER_BORDERS)
                .add_row(vec!["Package Name", env!("CARGO_PKG_NAME")])
                .add_row(vec!["Author(s)", env!("CARGO_PKG_AUTHORS")])
                .add_row(vec!["Version", env!("CARGO_PKG_VERSION")])
                .add_row(vec!["Documentation", env!("CARGO_PKG_HOMEPAGE")])
                .add_row(vec!["Repository", env!("CARGO_PKG_REPOSITORY")]);

            println!("{table}");
        }
    }

    Ok(())
}

#[derive(Debug, Error)]
enum CLIError {
    #[error("An error occurred during lexing")]
    Lexing,
    #[error("An error occurred during parsing")]
    Parsing,
    #[error("An error occurred during lowering")]
    Lowering,
    #[error("An error occurred during runtime")]
    Runtime,
    #[error("No unit assigned to {0:?}")]
    NonExposedPragma(Pragma),
    #[error("Unit {0} does not exist")]
    NonExistentUnit(String),
    #[error("{0:?}: {1}")]
    IO(PathBuf, std::io::Error),
}

fn relative_to(path: impl AsRef<Path>) -> Result<(), CLIError> {
    set_current_dir(
        canonicalize(&path)
            .map_err(|_| {
                CLIError::IO(
                    path.as_ref().to_path_buf(),
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Path provided for script cannot be canonicalized",
                    ),
                )
            })?
            .parent()
            .ok_or_else(|| {
                CLIError::IO(
                    path.as_ref().to_path_buf(),
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Path provided for script doesn't have a parent directory",
                    ),
                )
            })?,
    )
    .map_err(|io| CLIError::IO(path.as_ref().to_path_buf(), io))?;

    Ok(())
}

fn parse_file(src: impl AsRef<Path>) -> Result<(Module, String), CLIError> {
    let code = read_to_string(&src).map_err(|io| CLIError::IO(src.as_ref().to_path_buf(), io))?;
    let len = code.len();

    let src_str = src.as_ref().to_string_lossy().to_string();

    let tokens = lexer()
        .parse(Stream::from_iter(
            len..len + 1,
            code.chars().enumerate().map(|(i, c)| (c, i..i + 1)),
        ))
        .map_err(|err| {
            ParserErrorFormatter::new(err, &src_str, &code)
                .write_reports(&mut stderr())
                .ok();
            CLIError::Lexing
        })?;

    let ast = parser()
        .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
        .map_err(|err| {
            ParserErrorFormatter::new(err, &src_str, &code)
                .write_reports(&mut stderr())
                .ok();
            CLIError::Parsing
        })?;

    Module::lower(ast)
        .map_err(|err| {
            LowerErrorFormatter::new(err, &src_str, &code)
                .write_reports(&mut stderr())
                .ok();
            CLIError::Lowering
        })
        .map(|module| (module, code))
}

fn run_unit(cli: &Cli, module: Module, code: &str, name: &str) -> Result<(), CLIError> {
    relative_to(&cli.script)?;

    let runtime = if cli.quiet {
        Runtime::new(module, SilentNotifier)
    } else {
        Runtime::new(module, {
            let mut n = LogNotifier::new(
                cli.script
                    .file_name()
                    .map(|p| p.to_string_lossy())
                    .unwrap_or_default(),
                code,
            );
            if cli.verbose {
                n = n.verbose();
            }
            n
        })
    };

    runtime.run(name).map_err(|e| {
        runtime.notifier().error(&[e]);
        CLIError::Runtime
    })
}

fn run_unit_async(cli: &Cli, module: Module, code: &str, name: &str) -> Result<(), CLIError> {
    relative_to(&cli.script)?;

    let runtime = if cli.quiet {
        Runtime::new(module, SilentNotifier)
    } else {
        Runtime::new(module, {
            let mut n = LogNotifier::new(
                cli.script
                    .file_name()
                    .map(|p| p.to_string_lossy())
                    .unwrap_or_default(),
                code,
            );
            if cli.verbose {
                n = n.verbose();
            }
            n
        })
    };

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(runtime.run_async(name))
        .map_err(|e| {
            runtime.notifier().error(&[e]);
            CLIError::Runtime
        })
}

fn inspect(module: &Module) {
    let is_none_meta = module.units().iter().all(|(_, v)| v.meta.is_empty());

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .apply_modifier(UTF8_SOLID_INNER_BORDERS);

    if is_none_meta {
        table.set_header(["Unit", "Dependencies"]);
        table.add_rows(module.units().iter().map(|(k, v)| {
            Row::from([
                k.inner(),
                v.deps()
                    .iter()
                    .fold(String::new(), |acc, d| acc + d.inner() + "\n")
                    .trim_end(),
            ])
        }));
    } else {
        table.set_header(["Unit", "Dependencies", "@meta", ""]);
        table.add_rows(module.units().iter().flat_map(|(k, v)| {
            // TODO: Rewrite using iterators
            let mut rows = Vec::with_capacity(v.meta.len().max(v.deps().len()));

            let mut meta_iter = v.meta.iter();
            let mut dep_iter = v.deps().iter();

            rows.push(if let Some((var, val)) = meta_iter.next() {
                Row::from([
                    k.inner(),
                    dep_iter.next().map(|d| &**d.inner()).unwrap_or_default(),
                    var.inner(),
                    val,
                ])
            } else {
                Row::from([
                    k.inner(),
                    dep_iter.next().map(|d| &**d.inner()).unwrap_or_default(),
                    "",
                    "",
                ])
            });

            for (var, val) in meta_iter {
                rows.push(Row::from([
                    "",
                    dep_iter.next().map(|d| &**d.inner()).unwrap_or_default(),
                    &**var.inner(),
                    val,
                ]));
            }

            for dep in dep_iter {
                rows.push(Row::from(["", dep.inner(), "", ""]));
            }

            rows
        }));
    }

    let mut expose_table = Table::new();
    expose_table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .apply_modifier(UTF8_SOLID_INNER_BORDERS);
    expose_table.set_header(["Pragma", "Unit"]);
    expose_table.add_row(Row::from([
        "Test",
        &module
            .unit_by_pragma(Pragma::Test)
            .unwrap_or("".to_string()),
    ]));
    expose_table.add_row(Row::from([
        "Build",
        &module
            .unit_by_pragma(Pragma::Build)
            .unwrap_or("".to_string()),
    ]));

    println!("{expose_table}\n{table}");
}
