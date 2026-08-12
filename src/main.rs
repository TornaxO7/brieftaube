mod backend;
mod task_manager;
// mod composer;
mod config;
mod log_viewer;
// mod mail_viewer;
mod statusbar;
mod utils;

mod mailfs;

use crate::{
    backend::MailId, statusbar::Statusbar, task_manager::TaskManager, utils::ui::ScreenState,
};
use color_eyre::eyre;
use crossterm::event::Event;
use futures::{FutureExt, StreamExt};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    widgets::Clear,
};
use std::{
    fs::OpenOptions,
    io,
    path::PathBuf,
    rc::Rc,
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
    let counter = init_logging()?;

    let mut terminal = ratatui::init();
    App::new(counter).await?.run(&mut terminal).await?;
    ratatui::restore();

    Ok(())
}

enum Screen {
    // Composer(composer::ui::State),
    // MailViewer(mail_viewer::State),
    LogViewer(log_viewer::Model),
    Mailfs(mailfs::Model),
}

#[derive(Debug)]
pub enum Action {
    OpenMailViewer(MailId),
    OpenLogViewer,
    // OpenComposer,
    Redraw,
    Back,
    Quit,
}

/// Stores the app state
pub struct App {
    is_running: bool,
    backend: Arc<backend::Backend>,
    screens: Vec<Screen>,
    statusbar: statusbar::State,
    task_manager: Rc<TaskManager>,

    needs_full_redraw: bool,
}

impl App {
    pub async fn new(counter: statusbar::Counter) -> eyre::Result<Self> {
        let task_manager = Rc::new(TaskManager::new());
        let backend = Arc::new(backend::Backend::new().await);
        let initial_screen =
            Screen::Mailfs(mailfs::Model::new(backend.clone(), task_manager.clone()));

        let statusbar = statusbar::State::new(&initial_screen, counter);

        Ok(Self {
            is_running: true,
            task_manager,
            backend,
            screens: vec![initial_screen],
            statusbar,
            needs_full_redraw: false,
        })
    }

    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> eyre::Result<()> {
        let mut reader = crossterm::event::EventStream::new();
        self.statusbar.tick();

        while self.is_running {
            tokio::select! {
                _ = self.statusbar.has_changed() => { }
                _ = self.task_manager.finish_next_task() => {}
                maybe_event = reader.next().fuse() => match maybe_event {
                    Some(Ok(event)) => if let Some(action) = self.handle_event(event) {
                        self.apply_action(action);
                    }
                    Some(Err(e)) => error!("{}", e),
                    None => {},
                }
            }

            self.sync_throbber();
            terminal.draw(|frame| self.draw_screen(frame))?;
        }

        Ok(())
    }

    fn draw_screen(&mut self, frame: &mut Frame) {
        let area = frame.area();

        if self.needs_full_redraw {
            self.needs_full_redraw = false;
            frame.render_widget(Clear, area);
        }

        let [statusbar, screen] =
            Layout::vertical([Constraint::Length(3), Constraint::Fill(0)]).areas(area);
        frame.render_stateful_widget(Statusbar::default(), statusbar, &mut self.statusbar);

        match self.screens.last_mut().unwrap() {
            Screen::Mailfs(model) => frame.render_stateful_widget(mailfs::Mailfs, screen, model),
            // Screen::Composer(state) => {
            //     frame.render_stateful_widget(composer::ui::Composer::default(), screen, state);
            // }
            // Screen::MailViewer(state) => {
            //     frame.render_stateful_widget(mail_viewer::MailViewer::default(), screen, state);
            // }
            Screen::LogViewer(state) => {
                frame.render_stateful_widget(log_viewer::LogViewer, screen, state);
            }
        };
    }

    fn handle_event(&mut self, event: Event) -> Option<Action> {
        match self.screens.last_mut().unwrap() {
            // Screen::Mailboxes(state) => state.handle_event(event, &mut self.statusbar),
            // Screen::MailList(state) => state.handle_event(event, &mut self.statusbar),
            // Screen::Composer(state) => state.handle_event(event, &mut self.statusbar),
            // Screen::MailViewer(state) => state.handle_event(event, &mut self.statusbar),
            Screen::Mailfs(state) => state.handle_event(event, &mut self.statusbar),
            Screen::LogViewer(state) => state.handle_event(event, &mut self.statusbar),
        }
    }

    fn apply_action(&mut self, action: Action) {
        match action {
            Action::OpenMailViewer(_id) => {
                todo!();
                // let backend = self.backend.clone();
                // let next_screen = Screen::MailViewer(mail_viewer::State::new(id, backend));

                // self.statusbar.set_screen(&next_screen);
                // self.screens.push(next_screen);
            }
            Action::OpenLogViewer => {
                let next_screen = Screen::LogViewer(log_viewer::Model::new());

                self.statusbar.set_screen(&next_screen);
                self.screens.push(next_screen);
            }
            // Action::OpenComposer => {
            //     // let next_screen =
            //     //     Screen::Composer(composer::ui::State::new(self.account.clone()));
            //     todo!()

            //     // self.statusbar.set_screen(&next_screen);
            //     // self.screens.push(next_screen);
            // }
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

    fn sync_throbber(&mut self) {
        let top_screen_has_tasks_running =
            match self.screens.last().expect("There's at least one screen") {
                // Screen::Composer(_) => todo!(),
                // Screen::MailViewer(_) | Screen::Mailfs(_) => self.backend.has_tasks_running(),
                Screen::Mailfs(_) => self.task_manager.has_tasks_running(),
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
