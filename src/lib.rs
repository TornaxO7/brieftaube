use color_eyre::eyre::Result;
use ratatui::{DefaultTerminal, Frame};
use std::io;

mod ui;

#[derive(Debug)]
pub enum Action {
    Quit,
}

/// Stores the app state
#[derive(Debug)]
pub struct App {
    is_running: bool,

    ui: ui::State,
}

impl App {
    pub fn new() -> Self {
        Self {
            is_running: true,

            ui: ui::State::default(),
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while self.is_running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(&mut self.ui, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let Some(key) = crossterm::event::read()?.as_key_press_event() {
            if let Some(action) = self.ui.handle_event(key) {
                match action {
                    Action::Quit => self.is_running = false,
                }
            }
        }

        Ok(())
    }
}
