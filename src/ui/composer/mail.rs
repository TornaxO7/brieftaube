use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    style::Style,
    widgets::{Block, Paragraph, Widget},
};

#[derive(Debug)]
pub struct State {
    is_focussed: bool,
}

impl State {
    pub fn new() -> Self {
        Self { is_focussed: true }
    }

    pub fn handle_event(&mut self, event: crossterm::event::KeyEvent) -> Option<super::Action> {
        if event.modifiers.contains(KeyModifiers::CONTROL) && event.code == KeyCode::Char('k') {
            return Some(super::Action::OpenCommandPalette);
        }

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

impl Widget for &mut State {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block = {
            let mut block = Block::bordered().title("Mail content");

            block = if self.is_focussed {
                block.border_style(Style::new().green())
            } else {
                block
            };

            block
        };

        Widget::render(
            Paragraph::new("Random shit go brrr").block(block),
            area,
            buf,
        );
    }
}
