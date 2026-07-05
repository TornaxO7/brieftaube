mod backend;
mod config;
mod ui;

use color_eyre::eyre::Result;
use ratatui::{DefaultTerminal, Frame};
use std::{
    fs::OpenOptions,
    sync::{Arc, OnceLock},
    time::Duration,
};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, util::SubscriberInitExt};
use xdg::BaseDirectories;

const INPUT_TIMEOUT: Duration = Duration::from_millis(250);
const APP_NAME: &str = env!("CARGO_PKG_NAME");
static XDG: OnceLock<BaseDirectories> = OnceLock::new();

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    init_logging()?;

    let mut terminal = ratatui::init();
    App::new().await.run(&mut terminal)?;
    ratatui::restore();

    Ok(())
}

#[derive(Debug)]
pub enum Action {
    Quit,
}

/// Stores the app state
pub struct App {
    is_running: bool,

    _account: Arc<backend::Account>,
    ui: ui::State,
}

impl App {
    pub async fn new() -> Self {
        let account = Arc::new(backend::Account::new().await);
        let ui = ui::State::new(account.clone()).await;

        Self {
            is_running: true,
            _account: account,
            ui,
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while self.is_running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(&mut self.ui, frame.area());
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        if crossterm::event::poll(INPUT_TIMEOUT).expect("Poll for event") {
            if let Some(key) = crossterm::event::read()
                .expect("Read event")
                .as_key_press_event()
            {
                if let Some(action) = self.ui.handle_event(key) {
                    match action {
                        Action::Quit => self.is_running = false,
                    }
                }
            }
        }

        Ok(())
    }
}

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("is_running", &self.is_running)
            .field("ui", &self.ui)
            .finish()
    }
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

fn get_xdg() -> &'static BaseDirectories {
    XDG.get_or_init(|| BaseDirectories::with_prefix(APP_NAME))
}
