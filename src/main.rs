mod backend;
mod task_manager;
// mod composer;
mod config;
mod log_viewer;
mod mail_viewer;
mod mailfs;
mod palette;
mod prompt;
mod statusbar;
mod utils;

use crate::{
    backend::MailId,
    statusbar::Statusbar,
    task_manager::TaskManager,
    utils::layer::{LayerCore, LayerModel},
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

enum Layer {
    MailViewer(mail_viewer::Model),
    LogViewer(log_viewer::Model),
    Mailfs(mailfs::Model),

    Palette(palette::Model),
    Prompt(prompt::Model),
}

pub enum Action {
    OpenMailViewer(MailId),
    OpenLogViewer,
    OpenPalette { entries: Vec<palette::PaletteEntry> },
    OpenPrompt { description: String },
    Redraw,
    Back,
    Quit,
}

/// Stores the app state
pub struct App {
    is_running: bool,
    backend: Arc<backend::Backend>,
    layers: Vec<Layer>,
    statusbar: statusbar::State,
    task_manager: Rc<TaskManager>,

    needs_full_redraw: bool,
}

impl App {
    pub async fn new(counter: statusbar::Counter) -> eyre::Result<Self> {
        let task_manager = Rc::new(TaskManager::new());
        let backend = Arc::new(backend::Backend::new().await);
        let initial_layer =
            Layer::Mailfs(mailfs::Model::new(backend.clone(), task_manager.clone()));

        let statusbar = statusbar::State::new(&initial_layer, counter);

        Ok(Self {
            is_running: true,
            task_manager,
            backend,
            layers: vec![initial_layer],
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
            terminal.draw(|frame| self.draw_layer(frame))?;
        }

        Ok(())
    }

    fn draw_layer(&mut self, frame: &mut Frame) {
        let area = frame.area();

        if self.needs_full_redraw {
            self.needs_full_redraw = false;
            frame.render_widget(Clear, area);
        }

        let [statusbar, layer] =
            Layout::vertical([Constraint::Length(3), Constraint::Fill(0)]).areas(area);
        frame.render_stateful_widget(Statusbar::default(), statusbar, &mut self.statusbar);

        match self.layers.last_mut().unwrap() {
            Layer::Mailfs(model) => frame.render_stateful_widget(mailfs::Mailfs, layer, model),
            Layer::MailViewer(state) => {
                frame.render_stateful_widget(mail_viewer::MailViewer, layer, state);
            }
            Layer::LogViewer(state) => {
                frame.render_stateful_widget(log_viewer::LogViewer, layer, state);
            }
            Layer::Palette(model) => frame.render_stateful_widget(palette::Palette, layer, model),
            Layer::Prompt(model) => frame.render_stateful_widget(prompt::Prompt, layer, model),
        };
    }

    fn handle_event(&mut self, event: Event) -> Option<Action> {
        match self.layers.last_mut().unwrap() {
            Layer::MailViewer(model) => LayerCore::handle_event(model, event, &mut self.statusbar),
            Layer::Mailfs(model) => LayerCore::handle_event(model, event, &mut self.statusbar),
            Layer::LogViewer(model) => LayerCore::handle_event(model, event, &mut self.statusbar),
            Layer::Palette(model) => LayerCore::handle_event(model, event, &mut self.statusbar),
            Layer::Prompt(model) => LayerCore::handle_event(model, event, &mut self.statusbar),
        }
    }

    fn apply_action(&mut self, action: Action) {
        match action {
            Action::OpenMailViewer(id) => {
                let backend = self.backend.clone();
                let task_manager = self.task_manager.clone();
                let next_layer =
                    Layer::MailViewer(mail_viewer::Model::new(id, backend, task_manager));

                self.statusbar.set_layer(&next_layer);
                self.layers.push(next_layer);
            }
            Action::OpenPalette { entries } => {
                let next_layer = Layer::Palette(palette::Model::new(entries));
                self.layers.push(next_layer);
            }
            Action::OpenPrompt { description } => {
                let next_layer = Layer::Prompt(prompt::Model::new(description));
                self.layers.push(next_layer);
            }
            Action::OpenLogViewer => {
                let next_layer = Layer::LogViewer(log_viewer::Model::new());

                self.statusbar.set_layer(&next_layer);
                self.layers.push(next_layer);
            }
            Action::Redraw => {
                self.needs_full_redraw = true;
            }
            Action::Back => {
                let last_layer = self.layers.pop().unwrap();

                let current_layer = self.layers.last_mut().unwrap();
                self.statusbar.set_layer(current_layer);

                let next_action = match last_layer {
                    Layer::Palette(palette) => match current_layer {
                        Layer::MailViewer(model) => model.handle_overlay(palette),
                        Layer::LogViewer(model) => model.handle_overlay(palette),
                        Layer::Mailfs(model) => model.handle_overlay(palette),
                        Layer::Palette(_) | Layer::Prompt(_) => unreachable!(),
                    },
                    Layer::Prompt(prompt) => match current_layer {
                        Layer::MailViewer(model) => model.handle_overlay(prompt),
                        Layer::LogViewer(model) => model.handle_overlay(prompt),
                        Layer::Mailfs(model) => model.handle_overlay(prompt),
                        Layer::Palette(_) | Layer::Prompt(_) => unreachable!(),
                    },
                    _ => None,
                };

                if let Some(next_action) = next_action {
                    self.apply_action(next_action);
                }
            }
            Action::Quit => {
                self.is_running = false;
            }
        }
    }

    fn sync_throbber(&mut self) {
        let top_screen_has_tasks_running =
            match self.layers.last().expect("There's at least one screen") {
                Layer::MailViewer(_) | Layer::Mailfs(_) => self.task_manager.has_tasks_running(),
                Layer::LogViewer(_) | Layer::Palette(_) | Layer::Prompt(_) => false,
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
