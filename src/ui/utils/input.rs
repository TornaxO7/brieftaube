use crate::ui::ScreenOverlayResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, StatefulWidget, Widget},
};
use ratatui_textarea::TextArea;

pub struct State {
    input: TextArea<'static>,
    desc: String,
}

impl State {
    pub fn new(desc: String) -> Self {
        Self {
            input: TextArea::default(),
            desc,
        }
    }

    pub fn handle_event<P>(&mut self, event: KeyEvent) -> Option<ScreenOverlayResult<P>> {
        match event.code {
            KeyCode::Esc => Some(ScreenOverlayResult::Cancel),
            KeyCode::Enter => Some(ScreenOverlayResult::Input(
                self.input.lines().get(0).unwrap().to_owned(),
            )),
            _ => {
                self.input.input(event);
                None
            }
        }
    }
}

#[derive(Default)]
pub struct Input;

impl StatefulWidget for Input {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        Widget::render(Paragraph::new("Input"), area, buf);
    }
}
