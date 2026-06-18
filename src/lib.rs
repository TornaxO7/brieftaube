use std::sync::Arc;

use color_eyre::eyre::Result;
use jmap_client::client::Client;
use ratatui::{DefaultTerminal, Frame};

// mod backend;
mod ui;

#[derive(Debug)]
pub enum Action {
    Quit,
}

/// Stores the app state
pub struct App {
    is_running: bool,

    client: Arc<Client>,
    ui: ui::State,
}

impl App {
    pub async fn new() -> Self {
        let client = {
            let url = std::fs::read_to_string("/tmp/url.txt").unwrap();
            let password = std::fs::read_to_string("/tmp/password.txt").unwrap();
            let allow_redirects = std::fs::read_to_string("/tmp/redirects.txt").unwrap();

            Arc::new(
                Client::new()
                    .credentials(("test", password.as_str()))
                    .follow_redirects([allow_redirects])
                    .connect(&url)
                    .await
                    .unwrap(),
            )
        };

        let ui = ui::State::new(client).await;

        Self {
            is_running: true,
            client,
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
