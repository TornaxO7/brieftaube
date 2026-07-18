mod backend;
mod composer;
mod config;
mod log_viewer;
mod mail_viewer;
mod mailboxes;
mod root_mails;
mod statusbar;
mod thread_mails;
mod utils;

use crate::{
    statusbar::Statusbar,
    utils::{MailboxId, ThreadId, ui::ScreenState},
};
use color_eyre::eyre;
use crossterm::event::Event;
use futures::{FutureExt, StreamExt};
use jmap_client::email::Email;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
};
use std::{fs::OpenOptions, io, path::PathBuf, sync::OnceLock};
use tracing::{error, level_filters::LevelFilter};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use xdg::BaseDirectories;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
static XDG: OnceLock<BaseDirectories> = OnceLock::new();

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let counter = init_logging()?;

    let mut terminal = ratatui::init();
    App::new(counter).await?.run(&mut terminal).await?;
    ratatui::restore();

    Ok(())
}

enum Screen {
    Mailboxes(mailboxes::ui::State),
    RootMails(root_mails::ui::State),
    Composer(composer::ui::State),
    MailViewer(mail_viewer::ui::State),
    LogViewer(log_viewer::ui::State),
    ThreadMails(thread_mails::ui::State),
}

#[derive(Debug)]
pub enum Action {
    OpenRootMails(MailboxId),
    OpenMailViewer(Email),
    OpenLogViewer,
    OpenComposer,
    OpenThread(ThreadId),

    Redraw,
    Back,
    Quit,
}

/// Stores the app state
pub struct App {
    is_running: bool,
    account: backend::Account,
    screens: Vec<Screen>,
    statusbar: statusbar::State,

    needs_full_redraw: bool,
}

impl App {
    pub async fn new(counter: statusbar::Counter) -> eyre::Result<Self> {
        let account = backend::Account::new().await;
        let initial_screen =
            Screen::Mailboxes(mailboxes::ui::State::new(account.mailboxes.clone()));
        let statusbar = statusbar::State::new(&initial_screen, counter);

        Ok(Self {
            is_running: true,
            account: account,
            screens: vec![initial_screen],
            statusbar,
            needs_full_redraw: false,
        })
    }

    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> eyre::Result<()> {
        let mut reader = crossterm::event::EventStream::new();

        while self.is_running {
            self.sync_throbber();
            tokio::select! {
                _ = self.statusbar.has_changed() => { }

                _ = self.account.mailboxes.has_changed(), if self.account.mailboxes.has_tasks_running() => {
                    self.account.mailboxes.pop_task();
                }
                _ = self.account.root_mails.has_changed(), if self.account.root_mails.has_tasks_running() => {
                    self.account.root_mails.pop_task();
                }

                res = self.account.has_changed(), if self.account.has_tasks_running() => {
                    if let Ok(Err(err)) = res.expect("A task finished") {
                        error!("{}", err);
                    }
                }
                maybe_event = reader.next().fuse() => match maybe_event {
                    Some(Ok(event)) => self.handle_event(event),
                    Some(Err(e)) => error!("{}", e),
                    None => {},
                }
            }
            self.sync_throbber();

            self.apply_action();
            if self.needs_full_redraw {
                self.needs_full_redraw = false;
                terminal.clear().unwrap();
            }

            terminal.draw(|frame| self.draw_screen(frame))?;
        }

        Ok(())
    }

    fn draw_screen(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let [statusbar, screen] =
            Layout::vertical([Constraint::Length(3), Constraint::Fill(0)]).areas(area);
        frame.render_stateful_widget(Statusbar::default(), statusbar, &mut self.statusbar);

        match self.screens.last_mut().unwrap() {
            Screen::Mailboxes(state) => {
                frame.render_stateful_widget(mailboxes::ui::Mailboxes::default(), screen, state);
            }
            Screen::RootMails(state) => {
                frame.render_stateful_widget(root_mails::ui::RootMails::default(), screen, state);
            }
            Screen::Composer(state) => {
                frame.render_stateful_widget(composer::ui::Composer::default(), screen, state);
            }
            Screen::MailViewer(state) => {
                frame.render_stateful_widget(mail_viewer::ui::MailViewer::default(), screen, state);
            }
            Screen::LogViewer(state) => {
                frame.render_stateful_widget(log_viewer::ui::LogViewer::default(), screen, state);
            }
            Screen::ThreadMails(state) => {
                frame.render_stateful_widget(
                    thread_mails::ui::ThreadMails::default(),
                    screen,
                    state,
                );
            }
        };
    }

    fn handle_event(&mut self, event: Event) {
        match self.screens.last_mut().unwrap() {
            Screen::Mailboxes(state) => state.handle_event(event, &mut self.statusbar),
            Screen::RootMails(state) => state.handle_event(event, &mut self.statusbar),
            Screen::Composer(state) => state.handle_event(event, &mut self.statusbar),
            Screen::MailViewer(state) => state.handle_event(event, &mut self.statusbar),
            Screen::LogViewer(state) => state.handle_event(event, &mut self.statusbar),
            Screen::ThreadMails(state) => state.handle_event(event, &mut self.statusbar),
        };
    }

    fn apply_action(&mut self) {
        let actions = {
            let actions = match self.screens.last_mut().unwrap() {
                Screen::Mailboxes(state) => state.get_app_actions(),
                Screen::RootMails(state) => state.get_app_actions(),
                Screen::Composer(state) => state.get_app_actions(),
                Screen::MailViewer(state) => state.get_app_actions(),
                Screen::LogViewer(state) => state.get_app_actions(),
                Screen::ThreadMails(state) => state.get_app_actions(),
            };

            actions.collect::<Vec<Action>>()
        };

        for action in actions {
            match action {
                Action::OpenRootMails(id) => {
                    let client = self.account.client.clone();
                    let backend = self.account.root_mails.get_backend(id, client);
                    let next_screen = Screen::RootMails(root_mails::ui::State::new(backend));

                    self.statusbar.set_screen(&next_screen);
                    self.screens.push(next_screen);
                }
                Action::OpenMailViewer(mail) => {
                    let next_screen = Screen::MailViewer(mail_viewer::ui::State::new(mail));

                    self.statusbar.set_screen(&next_screen);
                    self.screens.push(next_screen);
                }
                Action::OpenLogViewer => {
                    let next_screen = Screen::LogViewer(log_viewer::ui::State::new());

                    self.statusbar.set_screen(&next_screen);
                    self.screens.push(next_screen);
                }
                Action::OpenComposer => {
                    // let next_screen =
                    //     Screen::Composer(composer::ui::State::new(self.account.clone()));
                    todo!()

                    // self.statusbar.set_screen(&next_screen);
                    // self.screens.push(next_screen);
                }
                Action::OpenThread(thread_id) => {
                    todo!()
                    // self.account.init_thread(thread_id.clone());

                    // let next_screen = Screen::ThreadMails(thread_mails::ui::State::new(
                    //     self.account.clone(),
                    //     thread_id,
                    // ));

                    // self.statusbar.set_screen(&next_screen);
                    // self.screens.push(next_screen);
                }
                Action::Redraw => {
                    self.needs_full_redraw = true;
                }
                Action::Back => {
                    self.screens.pop();

                    let screen = self.screens.last().unwrap();
                    self.statusbar.set_screen(screen);
                }
                Action::Quit => {
                    self.is_running = false;
                }
            }
        }
    }

    fn sync_throbber(&mut self) {
        let top_screen_has_tasks_running =
            match self.screens.last().expect("There's at least one screen") {
                Screen::Mailboxes(_) => self.account.mailboxes.has_tasks_running(),
                Screen::RootMails(_) => self.account.root_mails.has_tasks_running(),
                Screen::ThreadMails(_) => todo!(),
                Screen::Composer(_) => todo!(),
                Screen::MailViewer(_) => todo!(),
                Screen::LogViewer(_) => false,
            };

        if top_screen_has_tasks_running {
            self.statusbar.tick();
        } else {
            self.statusbar.remove_throbber();
        }
    }
}

fn init_logging() -> eyre::Result<statusbar::Counter> {
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

    let counter = statusbar::Counter::new();
    tracing_subscriber::registry()
        .with(counter.clone())
        .with(env_filter)
        .with(fmt_layer)
        .with(tui_logger::TuiTracingSubscriberLayer)
        .init();

    tracing::debug!("Debug logging enabled");
    tracing::trace!("Trace logging enabled");

    Ok(counter)
}

fn get_xdg() -> &'static BaseDirectories {
    XDG.get_or_init(|| BaseDirectories::with_prefix(APP_NAME))
}

fn get_log_file_path() -> io::Result<PathBuf> {
    get_xdg().place_state_file(&format!("{}.log", APP_NAME))
}
