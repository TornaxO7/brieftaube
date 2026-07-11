mod backend;
mod config;

mod composer;
mod log_viewer;
mod mail_viewer;
mod mailboxes;
mod mails;
mod utils;

use color_eyre::eyre;
use crossterm::event::{Event, EventStream};
use futures::{FutureExt, StreamExt};
use ratatui::{DefaultTerminal, Frame};
use std::{
    fs::OpenOptions,
    io,
    path::PathBuf,
    sync::{Arc, OnceLock},
};
use tracing::{error, level_filters::LevelFilter};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use utils::ui::ScreenState;
use xdg::BaseDirectories;

use crate::utils::ui::MailboxId;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
static XDG: OnceLock<BaseDirectories> = OnceLock::new();

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    init_logging()?;

    let mut terminal = ratatui::init();
    App::new().await.run(&mut terminal).await?;
    ratatui::restore();

    Ok(())
}

enum Screen {
    Mailboxes(mailboxes::ui::State),
    MailList(mails::ui::State),
    Composer(composer::ui::State),
    MailViewer(mail_viewer::ui::State),
    LogViewer(log_viewer::ui::State),
}

#[derive(Debug)]
pub enum Action {
    OpenMailList(MailboxId),
    OpenMailViewer,
    OpenLogViewer,
    OpenComposer,

    Refresh,
    Back,
    Quit,
}

/// Stores the app state
pub struct App {
    is_running: bool,
    fetcher: Arc<backend::Account>,
    screens: Vec<Screen>,
}

impl App {
    pub async fn new() -> Self {
        let fetcher = Arc::new(backend::Account::new().await);

        Self {
            is_running: true,
            fetcher: fetcher.clone(),
            screens: vec![Screen::Mailboxes(mailboxes::ui::State::new(fetcher))],
        }
    }

    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> eyre::Result<()> {
        let mut reader = EventStream::new();

        while self.is_running {
            tokio::select! {
                // TODO: Check if the backend received any changes from the event source
                // _ => self.account_changed() => {
                //      self.update_screen();
                // }
                maybe_event = reader.next().fuse() => match maybe_event {
                    Some(Ok(event)) => self.handle_event(event),
                    Some(Err(e)) => error!("{}", e),
                    None => {},
                }
            }

            terminal.draw(|frame| self.draw(frame))?;
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
                    mailboxes::ui::Mailboxes::default(),
                    frame.area(),
                    state,
                );
            }
            Screen::MailList(state) => {
                frame.render_stateful_widget(mails::ui::MailList::default(), frame.area(), state);
            }
            Screen::Composer(state) => {
                frame.render_stateful_widget(
                    composer::ui::Composer::default(),
                    frame.area(),
                    state,
                );
            }
            Screen::MailViewer(state) => {
                frame.render_stateful_widget(
                    mail_viewer::ui::MailViewer::default(),
                    frame.area(),
                    state,
                );
            }
            Screen::LogViewer(state) => {
                frame.render_stateful_widget(
                    log_viewer::ui::LogViewer::default(),
                    frame.area(),
                    state,
                );
            }
        };
    }

    fn handle_event(&mut self, event: Event) {
        match self.screens.last_mut().unwrap() {
            Screen::Mailboxes(state) => state.handle_event(event),
            Screen::MailList(state) => state.handle_event(event),
            Screen::Composer(state) => state.handle_event(event),
            Screen::MailViewer(state) => state.handle_event(event),
            Screen::LogViewer(state) => state.handle_event(event),
        };
    }

    fn apply_action(&mut self) {
        let actions = {
            let actions = match self.screens.last_mut().unwrap() {
                Screen::Mailboxes(state) => state.get_app_actions(),
                Screen::MailList(state) => state.get_app_actions(),
                Screen::Composer(state) => state.get_app_actions(),
                Screen::MailViewer(state) => state.get_app_actions(),
                Screen::LogViewer(state) => state.get_app_actions(),
            };

            actions.collect::<Vec<Action>>()
        };

        for action in actions {
            match action {
                Action::OpenMailList(mailbox_id) => {
                    self.screens.push(Screen::MailList(mails::ui::State::new(
                        self.fetcher.clone(),
                        mailbox_id,
                    )));
                }
                Action::OpenMailViewer => {
                    self.screens
                        .push(Screen::MailViewer(mail_viewer::ui::State::new(
                            self.fetcher.clone(),
                        )));
                }
                Action::OpenLogViewer => {
                    self.screens
                        .push(Screen::LogViewer(log_viewer::ui::State::new()));
                }
                Action::OpenComposer => {
                    self.screens.push(Screen::Composer(composer::ui::State::new(
                        self.fetcher.clone(),
                    )));
                }
                Action::Refresh => todo!(),
                Action::Back => {
                    self.screens.pop();
                    self.update_screen();
                }
                Action::Quit => {
                    self.is_running = false;
                }
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
