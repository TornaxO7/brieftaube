use crate::utils::layer::{LayerCore, LayerOverlay};
use crossterm::event::{Event, KeyCode};
use ratatui::style::Style;
use ratatui_textarea::TextArea;

pub struct Model {
    pub input: TextArea<'static>,
    pub desc: String,
}

impl Model {
    pub fn new<S: ToString>(desc: S) -> Self {
        let mut input = TextArea::default();
        input.set_cursor_line_style(Style::default());

        Self {
            input,
            desc: desc.to_string(),
        }
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
