mod backend;
mod config;
mod ui;

use crate::ui::{MailboxId, ScreenState, ThreadId};
use color_eyre::eyre;
use crossterm::event::Event;
use futures::{FutureExt, StreamExt};
use jmap_client::email::Email;
use ratatui::{DefaultTerminal, Frame};
use std::{
    fs::OpenOptions,
    io,
    path::PathBuf,
    sync::{Arc, OnceLock},
};
use tracing::{error, level_filters::LevelFilter};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use xdg::BaseDirectories;

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
    Mailboxes(ui::mailboxes::State),
    MailList(ui::root_mails::State),
    Composer(ui::composer::State),
    MailViewer(ui::mail_viewer::State),
    LogViewer(ui::log_viewer::State),
    ThreadMails(ui::thread_mails::State),
}

#[derive(Debug)]
pub enum Action {
    OpenRootMails(MailboxId),
    OpenMailViewer(Email),
    OpenLogViewer,
    OpenComposer,
    OpenThread(ThreadId),

    Refresh,
    Back,
    Quit,
}

/// Stores the app state
pub struct App {
    is_running: bool,
    account: Arc<backend::Account>,
    screens: Vec<Screen>,
}

impl App {
    pub async fn new() -> Self {
        let account = Arc::new(backend::Account::new().await);
        account.init_mailboxes();

        Self {
            is_running: true,
            account: account.clone(),
            screens: vec![Screen::Mailboxes(ui::mailboxes::State::new(account))],
        }
    }

    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> eyre::Result<()> {
        let mut reader = crossterm::event::EventStream::new();

        while self.is_running {
            tokio::select! {
                res = self.account.has_changed(), if self.account.has_tasks_running() => {
                    debug_assert!(res.is_some(), "Eeeeh, this should only return if a task finished and not if there are no tasks o.O");

                    if let Err(err) = res.unwrap() {
                        error!("{}", err);
                    }
                }
                maybe_event = reader.next().fuse() => match maybe_event {
                    Some(Ok(event)) => self.handle_event(event),
                    Some(Err(e)) => error!("{}", e),
                    None => {},
                }
            }

            self.apply_action();
            self.update_state_of_active_screen();
            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    fn update_state_of_active_screen(&mut self) {
        match self.screens.last_mut().unwrap() {
            Screen::Mailboxes(state) => state.update(),
            Screen::MailList(state) => state.update(),
            Screen::Composer(state) => state.update(),
            Screen::MailViewer(state) => state.update(),
            Screen::LogViewer(state) => state.update(),
            Screen::ThreadMails(state) => state.update(),
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        match self.screens.last_mut().unwrap() {
            Screen::Mailboxes(state) => {
                frame.render_stateful_widget(ui::mailboxes::Mailboxes::default(), area, state);
            }
            Screen::MailList(state) => {
                frame.render_stateful_widget(ui::root_mails::RootMails::default(), area, state);
            }
            Screen::Composer(state) => {
                frame.render_stateful_widget(ui::composer::Composer::default(), area, state);
            }
            Screen::MailViewer(state) => {
                frame.render_stateful_widget(ui::mail_viewer::MailViewer::default(), area, state);
            }
            Screen::LogViewer(state) => {
                frame.render_stateful_widget(ui::log_viewer::LogViewer::default(), area, state);
            }
            Screen::ThreadMails(state) => {
                frame.render_stateful_widget(ui::thread_mails::ThreadMails::default(), area, state);
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
            Screen::ThreadMails(state) => state.handle_event(event),
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
                Screen::ThreadMails(state) => state.get_app_actions(),
            };

            actions.collect::<Vec<Action>>()
        };

        for action in actions {
            match action {
                Action::OpenRootMails(mailbox_id) => {
                    self.account.init_root_mails(mailbox_id.clone());

                    self.screens
                        .push(Screen::MailList(ui::root_mails::State::new(
                            self.account.clone(),
                            mailbox_id,
                        )));
                }
                Action::OpenMailViewer(mail) => {
                    self.screens
                        .push(Screen::MailViewer(ui::mail_viewer::State::new(mail)));
                }
                Action::OpenLogViewer => {
                    self.screens
                        .push(Screen::LogViewer(ui::log_viewer::State::new()));
                }
                Action::OpenComposer => {
                    self.screens.push(Screen::Composer(ui::composer::State::new(
                        self.account.clone(),
                    )));
                }
                Action::OpenThread(thread_id) => {
                    self.account.init_thread(thread_id.clone());

                    self.screens
                        .push(Screen::ThreadMails(ui::thread_mails::State::new(
                            self.account.clone(),
                            thread_id,
                        )));
                }
                Action::Refresh => self.update_state_of_active_screen(),
                Action::Back => {
                    self.screens.pop();
                }
                Action::Quit => {
                    self.is_running = false;
                }
            }
        }

        self.update_state_of_active_screen();
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

    tracing::info!("Greetings!");
    tracing::debug!("Debug logging enabled");
    tracing::trace!("Trace logging enabled");

    Ok(())
}

fn get_xdg() -> &'static BaseDirectories {
    XDG.get_or_init(|| BaseDirectories::with_prefix(APP_NAME))
}

fn get_log_file_path() -> io::Result<PathBuf> {
    get_xdg().place_state_file(&format!("{}.log", APP_NAME))
}
