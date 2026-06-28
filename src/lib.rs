use color_eyre::eyre::Result;
use ratatui::{DefaultTerminal, Frame};
use std::sync::Arc;

mod backend;
mod ui;

#[derive(Debug)]
pub enum Action {
    Quit,
}

/// Stores the app state
pub struct App {
    is_running: bool,

    _account: Arc<backend::Account>,
    ui: ui::State,
}

impl App {
    pub async fn new() -> Self {
        let account = Arc::new(backend::Account::new().await);
        let ui = ui::State::new(account.clone()).await;

        Self {
            is_running: true,
            _account: account,
            ui,
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

    fn handle_events(&mut self) -> std::io::Result<()> {
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

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("is_running", &self.is_running)
            .field("ui", &self.ui)
            .finish()
    }
}
