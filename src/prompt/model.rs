use crate::utils::layer::{LayerCore, LayerOverlay};
use crossterm::event::{Event, KeyCode};
use ratatui_textarea::TextArea;

pub struct Model {
    pub input: TextArea<'static>,
    pub desc: String,
}

impl Model {
    pub fn new<S: ToString>(desc: S) -> Self {
        Self {
            input: TextArea::default(),
            desc: desc.to_string(),
        }
    }

    pub fn input_len(&self, line: usize) -> usize {
        self.input.lines()[line].len()
    }
}

impl LayerCore for Model {
    fn handle_event(
        &mut self,
        event: Event,
        _: &mut crate::statusbar::State,
    ) -> Option<crate::Action> {
        match event {
            Event::Key(event) => match event.code {
                KeyCode::Esc | KeyCode::Enter => Some(crate::Action::Back),
                _ => {
                    self.input.input(event);
                    None
                }
            },
            _ => None,
        }
    }
}

impl LayerOverlay for Model {
    fn into_message(self) -> Option<String> {
        Some(self.input.lines()[0].clone())
    }
}
