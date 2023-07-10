use std::error::Error;

use august_build::runtime::cli::{run, CLI};
use clap::Parser;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = CLI::parse();
    std::fs::create_dir_all({
        let mut home = dirs::home_dir().unwrap();
        home.push(".august");
        home.push("modules");
        home
    })
    .expect("Failed to create module directory");

    run(cli)
}
