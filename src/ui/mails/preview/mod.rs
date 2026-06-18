use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Paragraph, Widget},
};

#[derive(Debug)]
pub struct State {
    is_focussed: bool,
}

impl State {
    pub fn new() -> Self {
        Self { is_focussed: false }
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> Option<super::Action> {
        match event.code {
            KeyCode::Char('q') => Some(super::Action::Quit),
            KeyCode::Char(':') => Some(super::Action::OpenCommandPalette),
            _ => None,
        }
    }

    pub fn set_focus(&mut self, focussed: bool) {
        self.is_focussed = focussed;
    }
}

impl Widget for &State {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block = {
            let mut block = Block::bordered().title("Preview");

            block = if self.is_focussed {
                block.border_style(Style::new().green())
            } else {
                block
            };

            block
        };

        Widget::render(
            Paragraph::new("This is an example mail").block(block),
            area,
            buf,
        )
    }
}
