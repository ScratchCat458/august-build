use std::{collections::HashMap, error::Error, ffi::OsStr, path::PathBuf};

use august_build::{
    parsing::parse_script,
    runtime::{
        cli::{CLICommand, CLI},
        ExecutionPool,
    },
    Command, CommandDefinition, Pragma, Task,
};
use clap::Parser;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Table};
use owo_colors::OwoColorize;
use walkdir::WalkDir;

const DEFAULT_SCRIPT_PATH: &str = "main.august";

fn main() -> Result<(), Box<dyn Error>> {
    let cli = CLI::parse();
    std::fs::create_dir_all({
        let mut home = dirs::home_dir().unwrap();
        home.push(".august");
        home.push("modules");
        home
    })
    .expect("Failed to create module directory");

    match cli.command {
        CLICommand::Info => {
            let global_module_dir = {
                let mut home = dirs::home_dir().unwrap();
                home.push(".august");
                home.push("modules");
                home
            };
            let global_module_count = {
                let mut count = 0;
                for i in WalkDir::new(global_module_dir) {
                    let i = match i {
                        Ok(i) => i,
                        Err(_) => {
                            continue;
                        }
                    };

                    if i.path().extension() == Some(OsStr::new("august")) {
                        count += 1;
                    }
                }
                count
            };
            let module_count = {
                let mut count = 0;
                for i in WalkDir::new(".") {
                    let i = match i {
                        Ok(i) => i,
                        Err(_) => {
                            continue;
                        }
                    };

                    if i.path().extension() == Some(OsStr::new("august")) {
                        count += 1;
                    }
                }
                count
            };

            let mut table = Table::new();

            table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .add_row(vec!["Package Name", env!("CARGO_PKG_NAME")])
                .add_row(vec!["Author(s)", env!("CARGO_PKG_AUTHORS")])
                .add_row(vec!["Version", env!("CARGO_PKG_VERSION")])
                .add_row(vec!["Documentation", env!("CARGO_PKG_HOMEPAGE")])
                .add_row(vec!["Repository", env!("CARGO_PKG_REPOSITORY")])
                .add_row(vec![
                    "Global Modules",
                    format!("{global_module_count} in ~/.august/modules/").as_str(),
                ])
                .add_row(vec![
                    "Local Modules",
                    format!("{module_count} in current directory").as_str(),
                ]);

            println!("{table}");
        }
        CLICommand::Test { script } => {
            let script = script
                .unwrap_or(PathBuf::from(DEFAULT_SCRIPT_PATH))
                .to_str()
                .unwrap_or(DEFAULT_SCRIPT_PATH)
                .to_string();

            let module = parse_script(script)?;
            let main_task = match module.pragmas.test {
                Some(t) => t,
                None => {
                    println!("{} No task was assigned to `august test`.\n      Try adding `#pragma test task_name` to your build script", " ERR ".black().on_red());
                    std::process::exit(1);
                }
            };
            ExecutionPool::new(module.tasks, module.cmd_defs).run(main_task);
        }
        CLICommand::Build { script } => {
            let script = script
                .unwrap_or(PathBuf::from(DEFAULT_SCRIPT_PATH))
                .to_str()
                .unwrap_or(DEFAULT_SCRIPT_PATH)
                .to_string();

            let module = parse_script(script)?;
            let main_task = match module.pragmas.build {
                Some(t) => t,
                None => {
                    println!("{} No task was assigned to `august build`.\n      Try adding `#pragma build task_name` to your build script", " ERR ".black().on_red());
                    std::process::exit(1);
                }
            };
            ExecutionPool::new(module.tasks, module.cmd_defs).run(main_task);
        }
        CLICommand::Run { script, task_name } => {
            let script = script
                .unwrap_or(PathBuf::from(DEFAULT_SCRIPT_PATH))
                .to_str()
                .unwrap_or(DEFAULT_SCRIPT_PATH)
                .to_string();

            let module = parse_script(script)?;
            ExecutionPool::new(module.tasks, module.cmd_defs).run(task_name);
        }
        CLICommand::Check { script } => {
            let script = script
                .unwrap_or(PathBuf::from(DEFAULT_SCRIPT_PATH))
                .to_str()
                .unwrap_or(DEFAULT_SCRIPT_PATH)
                .to_string();

            parse_script(script)?;
        }
        CLICommand::Inspect { script } => {
            let script = script
                .unwrap_or(PathBuf::from(DEFAULT_SCRIPT_PATH))
                .to_str()
                .unwrap_or(DEFAULT_SCRIPT_PATH)
                .to_string();

            let module = parse_script(script)?;

            let mut table = Table::new();

            table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_header(["Property", "Contents"])
                .add_row(["Namespace", &module.namespace])
                .add_row(["Pragma", &fmt_pragma(module.pragmas)])
                .add_row(["Tasks", &fmt_tasks(&module.tasks)])
                .add_row(["Command Definitions", &fmt_cmddefs(&module.cmd_defs)]);

            println!("{table}");
        }
    }
    Ok(())
}

fn fmt_pragma(pragma: Pragma) -> String {
    format!(
        "Test --> {}\nBuild --> {}",
        pragma.test.unwrap_or("None".to_string()),
        pragma.build.unwrap_or("None".to_string())
    )
}

fn fmt_tasks(hashmap: &HashMap<String, Task>) -> String {
    let mut string = String::new();
    for k in hashmap.keys() {
        string.push_str(&format!("- {k}\n"));
    }
    string
}

fn fmt_cmddefs(hashmap: &HashMap<Command, CommandDefinition>) -> String {
    let mut string = String::new();
    for k in hashmap.keys() {
        let cmd = match k {
            Command::Local(n) => n.to_string(),
            Command::External(ns, n) => format!("{n} from {ns}"),
            Command::Internal(_) => panic!(),
        };

        string.push_str(&format!("- {cmd}\n"));
    }
    string
}
