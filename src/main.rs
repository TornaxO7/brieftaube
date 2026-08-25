mod backend;
mod config;
mod frontend;
mod task_manager;
mod utils;

use color_eyre::eyre::{self, Context};
use crossterm::event::Event;
use futures::{FutureExt, StreamExt};
use material_theme_loader::MaterialTheme;
use ratatui::{DefaultTerminal, Frame, layout::Rect, widgets::Clear};
use std::{
    fs::OpenOptions,
    io,
    path::PathBuf,
    sync::{Arc, OnceLock},
};
use tracing::{error, level_filters::LevelFilter, warn};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use xdg::BaseDirectories;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const DEFAULT_THEME: &str = include_str!("./default-theme.json");

static XDG: OnceLock<BaseDirectories> = OnceLock::new();
static THEME: OnceLock<MaterialTheme> = OnceLock::new();

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    init_logging()?;
    init_theme();

    let mut terminal = ratatui::init();
    App::new().await?.run(&mut terminal).await?;
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
    // OpenMailViewer(MailId),
    OpenPalette {
        entries: Vec<frontend::palette::PaletteEntry>,
    },
    OpenPrompt {
        description: String,
    },
    Redraw,
    Back,
    Quit,
}

/// Stores the app state
pub struct App {
    is_running: bool,
    backend: Arc<backend::Backend>,
    layers: Vec<Layer>,
    // task_manager: Rc<TaskManager>,
    needs_full_redraw: bool,
}

impl App {
    pub async fn new() -> eyre::Result<Self> {
        // let task_manager = Rc::new(TaskManager::new());
        let backend = Arc::new(backend::Backend::new().await);
        // let initial_layer =
        //     Layer::Mailfs(mailfs::Model::new(backend.clone(), task_manager.clone()));

        Ok(Self {
            is_running: true,
            // task_manager,
            backend,
            layers: vec![],
            needs_full_redraw: false,
        })
    }

    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> eyre::Result<()> {
        let mut reader = crossterm::event::EventStream::new();

        while self.is_running {
            tokio::select! {
                // _ = self.task_manager.finish_next_task() => {}
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

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        if self.needs_full_redraw {
            self.needs_full_redraw = false;
            frame.render_widget(Clear, area);
        }

        if self.layers.last().unwrap().is_overlay() {
            let len = self.layers.len();
            draw_layer(self.layers.get_mut(len - 2).unwrap(), frame, area);
        }

        draw_layer(self.layers.last_mut().unwrap(), frame, area);
    }

    fn handle_event(&mut self, event: Event) -> Option<Action> {
        // match self.layers.last_mut().unwrap() {
        //     Layer::Reader(model) => LayerCore::handle_event(model, event, &mut self.statusbar),
        //     Layer::Mailfs(model) => LayerCore::handle_event(model, event, &mut self.statusbar),
        //     Layer::LogViewer(model) => LayerCore::handle_event(model, event, &mut self.statusbar),
        //     Layer::Palette(model) => LayerCore::handle_event(model, event, &mut self.statusbar),
        //     Layer::Prompt(model) => LayerCore::handle_event(model, event, &mut self.statusbar),
        // }
        None
    }

    fn apply_action(&mut self, action: Action) {
        match action {
            // Action::OpenMailViewer(id) => {
            //     // let backend = self.backend.clone();
            //     // let task_manager = self.task_manager.clone();
            //     // let next_layer = Layer::Reader(reader::Model::new(id, backend, task_manager));

            //     // self.layers.push(next_layer);
            // }
            Action::OpenPalette { entries } => {
                // let next_layer = Layer::Palette(frontend::palette::Model::new(entries));
                // self.layers.push(next_layer);
            }
            Action::OpenPrompt { description } => {
                // let next_layer = Layer::Prompt(prompt::Model::new(description));
                // self.layers.push(next_layer);
            }

            Action::Redraw => {
                self.needs_full_redraw = true;
            }
            Action::Back => {
                // let last_layer = self.layers.pop().unwrap();

                // let current_layer = self.layers.last_mut().unwrap();
                // self.statusbar.set_layer(current_layer);

                // let next_action = match last_layer {
                //     Layer::Palette(palette) => match current_layer {
                //         Layer::Reader(model) => model.handle_overlay(palette),
                //         Layer::LogViewer(model) => model.handle_overlay(palette),
                //         Layer::Mailfs(model) => model.handle_overlay(palette),
                //         Layer::Palette(_) | Layer::Prompt(_) => unreachable!(),
                //     },
                //     Layer::Prompt(prompt) => match current_layer {
                //         Layer::Reader(model) => model.handle_overlay(prompt),
                //         Layer::LogViewer(model) => model.handle_overlay(prompt),
                //         Layer::Mailfs(model) => model.handle_overlay(prompt),
                //         Layer::Palette(_) | Layer::Prompt(_) => unreachable!(),
                //     },
                //     _ => None,
                // };

                // if let Some(next_action) = next_action {
                //     self.apply_action(next_action);
                // }
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

fn draw_layer(layer: &mut Layer, frame: &mut Frame, area: Rect) {
    // match layer {
    // Layer::Mailfs(model) => frame.render_stateful_widget(mailfs::Mailfs, area, model),
    // Layer::Reader(model) => {
    //     frame.render_stateful_widget(reader::Reader, area, model);
    // }
    // Layer::LogViewer(model) => {
    //     frame.render_stateful_widget(log_viewer::LogViewer, area, model);
    // }
    // Layer::Palette(model) => {
    //     let area = {
    //         let [_top, center, _bottom] =
    //             Layout::vertical([Constraint::Max(4), Constraint::Fill(1), Constraint::Max(4)])
    //                 .areas(area);

    //         let [_left, center, _right] = Layout::horizontal([
    //             Constraint::Max(20),
    //             Constraint::Fill(1),
    //             Constraint::Max(20),
    //         ])
    //         .areas(center);

    //         center
    //     };

    //     frame.render_widget(Clear, area);
    //     frame.render_stateful_widget(palette::Palette, area, model);
    // }
    // Layer::Prompt(model) => {
    //     let area = {
    //         let [_top, center, _bottom] = Layout::vertical([
    //             Constraint::Fill(1),
    //             Constraint::Length(3),
    //             Constraint::Fill(1),
    //         ])
    //         .areas(area);

    //         let [_left, center, _right] = Layout::horizontal([
    //             Constraint::Fill(1),
    //             Constraint::Min(50),
    //             Constraint::Fill(1),
    //         ])
    //         .areas(center);

    //         center
    //     };

    //     frame.render_widget(Clear, area);
    //     frame.render_stateful_widget(prompt::Prompt, area, model);
    // }
    // }
}
