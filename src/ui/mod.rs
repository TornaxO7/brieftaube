// pub mod composer;
// pub mod log_viewer;
pub mod mailfs;
pub mod palette;
pub mod prompt;
// pub mod reader;
pub mod statusbar;
mod utils;

use crate::ui::palette::PaletteEntry;
use color_eyre::eyre;
use crossterm::event::Event;
use futures::{FutureExt, StreamExt};
use ratatui::{DefaultTerminal, Frame};
use tracing::error;

pub struct LayerMessage(pub String);

impl LayerMessage {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for LayerMessage {
    fn from(msg: String) -> Self {
        Self(msg)
    }
}

enum Layer {
    Mailfs(mailfs::State),

    Palette(palette::State),
    Prompt(prompt::State),
}

impl From<Layer> for Option<LayerMessage> {
    fn from(layer: Layer) -> Self {
        match layer {
            Layer::Mailfs(state) => state.into(),
            Layer::Palette(state) => state.into(),
            Layer::Prompt(state) => state.into(),
        }
    }
}

pub enum Action {
    OpenPrompt { description: String },
    OpenPalette { entries: Vec<PaletteEntry> },
    Back,
    Redraw,
    Quit,
}

/// Stores the app state
pub struct Ui {
    is_running: bool,
    layers: Vec<Layer>,
    needs_full_redraw: bool,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            is_running: true,
            layers: vec![Layer::Mailfs(mailfs::State::new())],
            needs_full_redraw: false,
        }
    }

    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> eyre::Result<()> {
        let mut reader = crossterm::event::EventStream::new();
        terminal.draw(|frame| self.draw(frame))?;

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

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let is_overlay = match self.layers.last().unwrap() {
            Layer::Mailfs(_) => false,
            Layer::Palette(_) | Layer::Prompt(_) => true,
        };

        if is_overlay {
            match self.layers.iter_mut().rev().skip(1).next().unwrap() {
                Layer::Mailfs(state) => mailfs::view(state, frame, area),
                Layer::Palette(state) => palette::view(state, frame, area),
                Layer::Prompt(state) => prompt::view(state, frame, area),
            }
        }

        match self.layers.last_mut().unwrap() {
            Layer::Mailfs(state) => mailfs::view(state, frame, area),
            Layer::Palette(state) => palette::view(state, frame, area),
            Layer::Prompt(state) => prompt::view(state, frame, area),
        }
    }

    fn handle_event(&mut self, event: Event) -> Option<Action> {
        match self.layers.last_mut().unwrap() {
            Layer::Mailfs(state) => state.handle_event(event),
            Layer::Palette(state) => state.handle_event(event),
            Layer::Prompt(state) => state.handle_event(event),
        }
    }

    fn apply_action(&mut self, action: Action) {
        match action {
            Action::OpenPrompt { description } => {
                let state = prompt::State::new(description);
                self.layers.push(Layer::Prompt(state));
            }
            Action::OpenPalette { entries } => {
                let state = palette::State::new(entries);
                self.layers.push(Layer::Palette(state));
            }

            Action::Back => {
                let Some(layer) = self.layers.pop() else {
                    panic!("Layers must never be empty!");
                };

                let action = match self.layers.last_mut().unwrap() {
                    Layer::Mailfs(state) => state.handle_layer_message(layer),
                    Layer::Palette(state) => state.handle_layer_message(layer),
                    Layer::Prompt(state) => state.handle_layer_message(layer),
                };

                if let Some(action) = action {
                    self.apply_action(action);
                }
            }

            Action::Redraw => {
                self.needs_full_redraw = true;
            }

            Action::Quit => {
                self.is_running = false;
            }
        }
    }
}

pub trait LayerCore<ParentAction = Action>: Into<Option<LayerMessage>> {
    fn handle_event(&mut self, event: Event) -> Option<ParentAction>;

    #[must_use]
    fn handle_layer_message<Msg>(&mut self, layer: Msg) -> Option<ParentAction>
    where
        Msg: Into<Option<LayerMessage>>;
}

pub trait LayerState<UserAction, ParentAction = Action>: LayerCore<ParentAction> {
    #[must_use]
    fn apply_action(&mut self, action: UserAction) -> Option<ParentAction>;
}
