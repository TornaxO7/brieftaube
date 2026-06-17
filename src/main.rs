use brieftaube::App;
use color_eyre::eyre::Result;
use std::{fs::OpenOptions, sync::OnceLock};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, util::SubscriberInitExt};
use xdg::BaseDirectories;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
static XDG: OnceLock<BaseDirectories> = OnceLock::new();

fn main() -> Result<()> {
    color_eyre::install()?;
    init_logging()?;

    ratatui::run(|terminal| App::new().run(terminal))
}

fn get_xdg() -> &'static BaseDirectories {
    XDG.get_or_init(|| BaseDirectories::with_prefix(APP_NAME))
}

fn init_logging() -> Result<()> {
    let log_file = {
        let log_file_path = get_xdg().place_state_file(&format!("{}.log", APP_NAME))?;

        OpenOptions::new()
            .append(true)
            .create(true)
            .open(log_file_path)?
    };

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(log_file)
        .without_time()
        .pretty()
        .finish()
        .init();

    tracing::debug!("Debug logging enabled");

    Ok(())
}
