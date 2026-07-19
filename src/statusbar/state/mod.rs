mod counter;

use crate::Screen;
pub use counter::Counter;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use throbber_widgets_tui::ThrobberState;

pub struct State {
    pub(super) screen_name: &'static str,
    pub(super) keypresses: String,

    pub(super) counter: Counter,
    pub(super) show_counter: bool,
    pub(super) throbber_state: Option<ThrobberState>,
}

impl State {
    pub fn new(init_screen: &Screen, counter: Counter) -> Self {
        let mut state = Self {
            screen_name: "",
            counter: counter,

            keypresses: String::new(),
            show_counter: true,
            throbber_state: None,
        };

        state.set_screen(init_screen);

        state
    }

    pub fn set_screen(&mut self, screen: &Screen) {
        self.screen_name = match screen {
            Screen::Mailboxes(_) => "Mailboxes",
            Screen::MailList(_) => "Mail-List",
            Screen::Composer(_) => "Composer",
            Screen::MailViewer(_) => "Mail-Viewer",
            Screen::LogViewer(_) => "Log-Viewer",
        };

        match screen {
            Screen::LogViewer(_) => {
                self.show_counter = false;
            }
            _ => {
                self.show_counter = true;
                self.counter.reset();
            }
        };
    }

    pub fn push_key_press(&mut self, event: KeyEvent) {
        let code = match event.code {
            KeyCode::Char(c) => c,
            _ => '?',
        };

        let s = match event.modifiers {
            KeyModifiers::ALT => format!("<A-{}>", code),
            KeyModifiers::CONTROL => format!("<C-{}>", code),
            _ => code.to_string(),
        };

        self.keypresses.push_str(&s);
    }

    pub fn reset_key_press(&mut self) {
        self.keypresses.clear();
    }

    pub async fn has_changed(&self) {
        if self.throbber_state.is_some() {
            tokio::select! {
                _ = self.counter.has_changed() => { }
                _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {}
            }
        } else {
            self.counter.has_changed().await;
        }
    }

    pub fn tick(&mut self) {
        match self.throbber_state.as_mut() {
            Some(throbber) => throbber.calc_next(),
            None => self.throbber_state = Some(ThrobberState::default()),
        }
    }

    pub fn remove_throbber(&mut self) {
        self.throbber_state = None;
    }
}
