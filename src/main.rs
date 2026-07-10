mod backend;
mod config;
mod ui;

use color_eyre::eyre;
use ratatui::{DefaultTerminal, Frame};
use std::{
    fs::OpenOptions,
    io,
    path::PathBuf,
    sync::{Arc, OnceLock},
    time::Duration,
};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use xdg::BaseDirectories;

use crate::ui::ScreenState;

const INPUT_TIMEOUT: Duration = Duration::from_millis(250);
const APP_NAME: &str = env!("CARGO_PKG_NAME");
static XDG: OnceLock<BaseDirectories> = OnceLock::new();

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    init_logging()?;

    let mut terminal = ratatui::init();
    App::new().await.run(&mut terminal)?;
    ratatui::restore();

    Ok(())
}

enum Screen {
    Mailboxes(ui::mailboxes::State),
    MailList(ui::mails::State),
    Composer(ui::composer::State),
    MailViewer(ui::mail_viewer::State),
    LogViewer(ui::log_viewer::State),
}

#[derive(Debug)]
pub enum Action {
    Quit,
}

/// Stores the app state
pub struct App {
    is_running: bool,
    _account: Arc<backend::Account>,
    screens: Vec<Screen>,
}

impl App {
    pub async fn new() -> Self {
        let account = Arc::new(backend::Account::new().await);

        Self {
            is_running: true,
            _account: account.clone(),
            screens: vec![Screen::Mailboxes(ui::mailboxes::State::new(account))],
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> eyre::Result<()> {
        while self.is_running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
            self.apply_action();
            self.update_screen();
        }

        Ok(())
    }

    fn update_screen(&mut self) {
        match self.screens.last_mut().unwrap() {
            Screen::Mailboxes(state) => state.update(),
            Screen::MailList(state) => state.update(),
            Screen::Composer(state) => state.update(),
            Screen::MailViewer(state) => state.update(),
            Screen::LogViewer(state) => state.update(),
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        match self.screens.last_mut().unwrap() {
            Screen::Mailboxes(state) => {
                frame.render_stateful_widget(
                    ui::mailboxes::Mailboxes::default(),
                    frame.area(),
                    state,
                );
            }
            Screen::MailList(state) => {
                frame.render_stateful_widget(ui::mails::MailList::default(), frame.area(), state);
            }
            Screen::Composer(state) => {
                frame.render_stateful_widget(
                    ui::composer::Composer::default(),
                    frame.area(),
                    state,
                );
            }
            Screen::MailViewer(state) => {
                frame.render_stateful_widget(
                    ui::mail_viewer::MailViewer::default(),
                    frame.area(),
                    state,
                );
            }
            Screen::LogViewer(state) => {
                frame.render_stateful_widget(
                    ui::log_viewer::LogViewer::default(),
                    frame.area(),
                    state,
                );
            }
        };
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        if crossterm::event::poll(INPUT_TIMEOUT).expect("Poll for event") {
            if let Some(event) = crossterm::event::read()
                .expect("Read event")
                .as_key_press_event()
            {
                match self.screens.last_mut().unwrap() {
                    Screen::Mailboxes(state) => state.handle_event(event),
                    Screen::MailList(state) => state.handle_event(event),
                    Screen::Composer(state) => state.handle_event(event),
                    Screen::MailViewer(state) => state.handle_event(event),
                    Screen::LogViewer(state) => state.handle_event(event),
                };
            }
        }

        Ok(())
    }

    fn apply_action(&mut self) {
        let actions = match self.screens.last_mut().unwrap() {
            Screen::Mailboxes(state) => state.get_app_actions(),
            Screen::MailList(state) => state.get_app_actions(),
            Screen::Composer(state) => state.get_app_actions(),
            Screen::MailViewer(state) => state.get_app_actions(),
            Screen::LogViewer(state) => state.get_app_actions(),
        };

        for action in actions {
            match action {
                Action::Quit => self.is_running = false,
            }
        }
    }
}

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("is_running", &self.is_running)
            .finish()
    }
}

fn init_logging() -> eyre::Result<()> {
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
        .pretty();

    tui_logger::init_logger(tui_logger::LevelFilter::Info)?;

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(tui_logger::TuiTracingSubscriberLayer)
        .init();

    tracing::debug!("Debug logging enabled");
    tracing::info!("Greetings!");

    Ok(())
}

fn get_xdg() -> &'static BaseDirectories {
    XDG.get_or_init(|| BaseDirectories::with_prefix(APP_NAME))
}

fn get_log_file_path() -> io::Result<PathBuf> {
    get_xdg().place_state_file(&format!("{}.log", APP_NAME))
}
