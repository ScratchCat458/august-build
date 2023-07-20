use std::error::Error;

use august_build::runtime::cli::{run, CLI};
use clap::Parser;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

fn main() {
    fn sub_main() -> Result<(), Box<dyn Error>> {
        tracing::subscriber::set_global_default(
            FmtSubscriber::builder()
                .with_max_level(Level::TRACE)
                .finish(),
        )?;

        let cli = CLI::parse();
        std::fs::create_dir_all({
            let mut home = dirs::home_dir().unwrap();
            home.push(".august");
            home.push("modules");
            home
        })?;

        run(cli)?;

        Ok(())
    }

    if let Err(e) = sub_main() {
        eprintln!("{e}")
    }
}
