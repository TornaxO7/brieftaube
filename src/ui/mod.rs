mod renderer;
mod utils;

// pub mod composer;
// pub mod log_viewer;
pub mod mailfs;
pub mod palette;
pub mod prompt;
// pub mod reader;
pub mod statusbar;

use crate::{task_manager::TaskManager, ui::palette::PaletteEntry};
use color_eyre::eyre;
use crossterm::event::Event;
use futures::{FutureExt, StreamExt};
use ratatui::{DefaultTerminal, Frame};
use std::rc::Rc;
use tracing::error;

#[derive(Debug, Clone, Copy)]
enum ActiveLayer {
    Mailfs,
    Palette,
    Prompt,
}

pub enum Message {
    Mailfs(mailfs::Message),
    Palette(palette::Message),
    Prompt(prompt::Message),

    Event(Event),
    OpenPrompt { description: String },
    OpenPalette { entries: Vec<PaletteEntry> },
    Back,
    Redraw,
    Quit,
}

/// Stores the app state
pub struct Ui {
    is_running: bool,
    layers: Vec<ActiveLayer>,
    needs_full_redraw: bool,
    task_manager: Rc<TaskManager>,

    mailfs: mailfs::State,
    palette: palette::State,
    prompt: prompt::State,
}

impl Ui {
    pub fn new() -> Self {
        let task_manager = Rc::new(TaskManager::new());

        let mailfs = mailfs::State::new(task_manager.clone());
        let palette = palette::State::new();
        let prompt = prompt::State::new();

        Self {
            mailfs,
            palette,
            prompt,

            is_running: true,
            layers: vec![ActiveLayer::Mailfs],
            needs_full_redraw: false,
            task_manager,
        }
    }

    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> eyre::Result<()> {
        let mut reader = crossterm::event::EventStream::new();
        terminal.draw(|frame| self.draw(frame))?;

        while self.is_running {
            let mut msg = tokio::select! {
                maybe_event = reader.next().fuse() => match maybe_event {
                    Some(Ok(event)) => Some(Message::Event(event)),
                    Some(Err(e)) => {
                        error!("{}", e);
                        None
                    },
                    None => None,
                }
            };

            while let Some(next_message) = msg {
                msg = self.handle_message(next_message);
            }

            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let is_overlay = match self.layers.last().unwrap() {
            ActiveLayer::Mailfs => false,
            ActiveLayer::Palette | ActiveLayer::Prompt => true,
        };

        if is_overlay {
            match self.layers.iter().rev().skip(1).next().unwrap() {
                ActiveLayer::Mailfs => mailfs::view(&mut self.mailfs, frame, area),
                ActiveLayer::Palette => palette::view(&mut self.palette, frame, area),
                ActiveLayer::Prompt => prompt::view(&mut self.prompt, frame, area),
            }
        }

        match self.layers.last_mut().unwrap() {
            ActiveLayer::Mailfs => mailfs::view(&mut self.mailfs, frame, area),
            ActiveLayer::Palette => palette::view(&mut self.palette, frame, area),
            ActiveLayer::Prompt => prompt::view(&mut self.prompt, frame, area),
        }
    }

    fn handle_message(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::Event(event) => match self.layers.last_mut().unwrap() {
                ActiveLayer::Mailfs => self.mailfs.update(mailfs::Message::Event(event)),
                ActiveLayer::Palette => self.palette.update(palette::Message::Event(event)),
                ActiveLayer::Prompt => self.prompt.update(prompt::Message::Event(event)),
            },

            Message::OpenPrompt { description } => {
                self.prompt.update(prompt::Message::Reset(description));
                self.layers.push(ActiveLayer::Prompt);
                None
            }
            Message::OpenPalette { entries } => {
                self.palette.update(palette::Message::Restart(entries));
                self.layers.push(ActiveLayer::Palette);
                None
            }

            Message::Back => {
                self.layers.pop();
                None
            }

            Message::Redraw => {
                self.needs_full_redraw = true;
                None
            }

            Message::Quit => {
                self.is_running = false;
                None
            }
            Message::Mailfs(message) => self.mailfs.update(message),
            Message::Palette(message) => self.palette.update(message),
            Message::Prompt(message) => self.prompt.update(message),
        }
    }
}

pub trait Layer<LayerMsg, ParentLayerMsg = Message> {
    fn update(&mut self, msg: LayerMsg) -> Option<ParentLayerMsg>;
}
