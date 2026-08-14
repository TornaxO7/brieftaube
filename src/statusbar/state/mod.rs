mod counter;

use crate::Layer;
pub use counter::Counter;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use throbber_widgets_tui::ThrobberState;

pub struct State {
    pub(super) screen_name: String,
    pub(super) keypresses: String,

    pub(super) counter: Counter,
    pub(super) show_counter: bool,
    pub(super) throbber_state: Option<ThrobberState>,
}

impl State {
    pub fn new(init_layer: &Layer, counter: Counter) -> Self {
        let mut state = Self {
            screen_name: String::new(),
            counter: counter,

            keypresses: String::new(),
            show_counter: true,
            throbber_state: None,
        };

        state.set_layer(init_layer);

        state
    }

    pub fn set_layer(&mut self, layer: &Layer) {
        self.screen_name = match layer {
            Layer::Mailfs(_) => "Mail Filesystem".to_string(),
            Layer::MailViewer(_) => "Mail-Viewer".to_string(),
            Layer::LogViewer(_) => "Log-Viewer".to_string(),
            Layer::Palette(_) => "Palette".to_string(),
            Layer::Prompt(model) => model.desc.clone(),
        };

        match layer {
            Layer::LogViewer(_) => {
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
