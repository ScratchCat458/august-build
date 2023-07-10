use std::{fs::read_to_string, process::exit};

use ariadne::Source;
use std::path::PathBuf;

use crate::Module;
use walkdir::WalkDir;

use self::{error::ParserError, parse::parse_module, token::tokenise};

pub mod error;
pub mod parse;
pub mod token;

pub fn handle_parser_error(err: ParserError, file_name: String, content: String) {
    err.report_gen(file_name.clone())
        .eprint((file_name, Source::from(content)))
        .unwrap();
}

pub fn parse_script(main_script_name: impl Into<String>) -> Result<Module, ParserError> {
    let main_script_name = main_script_name.into();
    let input = read_to_string(main_script_name.clone())?;
    let tokens = tokenise(input.clone());
    let mut module = match parse_module(tokens) {
        Ok(m) => m,
        Err(e) => {
            handle_parser_error(e, main_script_name, input);
            exit(1)
        }
    };

    for i in module.links.clone() {
        let Some(ext_path) = resolve_module_path(i) else {
            continue;
        };
        let ext_input = read_to_string(ext_path)?;
        let ext_tokens = tokenise(ext_input);
        let ext_module = match parse_module(ext_tokens) {
            Ok(m) => m,
            Err(e) => {
                handle_parser_error(e, main_script_name, input);
                exit(1)
            }
        };
        module.link_module(ext_module);
    }

    Ok(module)
}

const FILE_EXTENSION: &str = ".august";

pub fn resolve_module_path(namespace: impl Into<String>) -> Option<PathBuf> {
    let namespace = namespace.into();

    // Check locals
    let local_dirs = WalkDir::new(".").into_iter();
    for entry in local_dirs {
        let Ok(entry) = entry else {
            continue;
        };

        if entry
            .file_name()
            .eq_ignore_ascii_case(format!("{namespace}{FILE_EXTENSION}"))
        {
            return Some(entry.path().to_owned());
        }
    }

    let module_dir = {
        let Some(mut home) = dirs::home_dir() else {
            return None;
        };
        home.push(".august");
        home.push("modules");
        home
    };
    // Check globals
    let global_dirs = WalkDir::new(module_dir).into_iter();
    for entry in global_dirs {
        let Ok(entry) = entry else {
            continue;
        };
        if entry
            .file_name()
            .eq_ignore_ascii_case(format!("{namespace}{FILE_EXTENSION}"))
        {
            return Some(entry.path().to_owned());
        }
    }

    None
}
