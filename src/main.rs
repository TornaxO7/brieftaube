mod config;
mod datasource;
mod repository;
mod task_manager;
mod types;
mod ui;
mod utils;

use color_eyre::eyre::{self, Context};
use config::Config;
use crossterm::event::Event;
use futures::{FutureExt, StreamExt};
use material_theme_loader::MaterialTheme;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
};
use std::{fs::OpenOptions, io, path::PathBuf, sync::OnceLock};
use tokio::sync::mpsc;
use tracing::{error, level_filters::LevelFilter, warn};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use xdg::BaseDirectories;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const DEFAULT_THEME: &str = include_str!("./default-theme.json");

static XDG: OnceLock<BaseDirectories> = OnceLock::new();
static THEME: OnceLock<MaterialTheme> = OnceLock::new();
static CONFIG: OnceLock<Config> = OnceLock::new();

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    init_logging()?;
    init_theme();
    init_config()?;

    let mut terminal = ratatui::init();
    App::new().run(&mut terminal).await?;
    ratatui::restore();
    Ok(())
}

enum Layer {}

impl Layer {
    pub fn is_overlay(&self) -> bool {
        false
    }
}

pub enum Action {
    Redraw,
    Quit,
}

/// Stores the app state
pub struct App {
    is_running: bool,
    layers: Vec<Layer>,
    needs_full_redraw: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            is_running: true,
            layers: vec![],
            needs_full_redraw: false,
        }
    }

    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> eyre::Result<()> {
        let mut reader = crossterm::event::EventStream::new();

        while self.is_running {
            tokio::select! {
                maybe_event = reader.next().fuse() => match maybe_event {
                    Some(Ok(event)) => if let Some(action) = self.handle_event(event) {
                        self.apply_action(action);
                    }
                    Some(Err(e)) => error!("{}", e),
                    None => {},
                }
            }

            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {}

    fn handle_event(&mut self, event: Event) -> Option<Action> {
        None
    }

    fn apply_action(&mut self, action: Action) {
        match action {
            Action::Redraw => {
                self.needs_full_redraw = true;
            }

            Action::Quit => {
                self.is_running = false;
            }
        }
    }
}

fn init_logging() -> eyre::Result {
    let log_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(get_log_file_path()?)?;

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_writer(log_file)
        .without_time()
        .with_file(true)
        .with_line_number(true)
        .pretty();

    tui_logger::init_logger(tui_logger::LevelFilter::Info)?;

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(tui_logger::TuiTracingSubscriberLayer)
        .init();

    tracing::debug!("Debug logging enabled");
    tracing::trace!("Trace logging enabled");

    Ok(())
}

fn init_theme() {
    let theme: MaterialTheme = match try_load_custom_theme() {
        Ok(theme) => theme,
        Err(err) => {
            warn!("Couldn't load theme: {err}\nFallback to default theme.");
            serde_json::from_str(DEFAULT_THEME).unwrap()
        }
    };

    THEME.set(theme).expect("Set theme");
}

fn init_config() -> eyre::Result<()> {
    let path = get_xdg().place_config_file(config::FILE_NAME)?;
    let content = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(content.as_str())?;

    CONFIG.set(config).unwrap();
    Ok(())
}

fn try_load_custom_theme() -> eyre::Result<MaterialTheme> {
    let path = get_theme_file_path().context("Get path to theme file")?;
    let content = std::fs::read_to_string(path).context("Read the theme")?;
    let theme: MaterialTheme = serde_json::from_str(content.as_str())?;
    Ok(theme)
}

fn get_xdg() -> &'static BaseDirectories {
    XDG.get_or_init(|| BaseDirectories::with_prefix(APP_NAME))
}

fn get_log_file_path() -> io::Result<PathBuf> {
    get_xdg().place_state_file(&format!("{}.log", APP_NAME))
}

fn get_theme_file_path() -> io::Result<PathBuf> {
    get_xdg().place_config_file("theme.json")
}

// fn draw_layer(layer: &mut Layer, frame: &mut Frame, area: Rect) {
//     let [_top, bottom] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

//     layer::statusbar::draw(
//         "Mailfs(normal)",
//         layer::statusbar::StatusMsg {
//             msg: "Houston, we've got a problem",
//             ty: layer::statusbar::StatusMsgType::Error,
//         },
//         "abcdefg",
//         frame,
//         bottom,
//     );
// }
