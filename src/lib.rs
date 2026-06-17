use std::io;

use color_eyre::eyre::Result;
use crossterm::event::{self, KeyCode};
use ratatui::{DefaultTerminal, Frame};

mod ui;

/// Stores the app state
#[derive(Debug)]
pub struct App {
    is_running: bool,

    ui: ui::UiState,
}

impl App {
    pub fn new() -> Self {
        Self {
            is_running: true,

            ui: ui::UiState::default(),
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while self.is_running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(&self.ui, frame.area());
        // frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Char('q') => self.is_running = false,
                _ => {}
            }
        }

        Ok(())
    }
}
