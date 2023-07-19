use august_build::runtime::cli::{run, CLI};
use clap::Parser;
use color_eyre::eyre::WrapErr;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

fn main() -> Result<(), color_eyre::Report> {
    color_eyre::install()?;
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
    })
    .wrap_err("Failed to create module directory")?;

    run(cli)?;

    Ok(())
}
