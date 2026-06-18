use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, List, ListState, Paragraph, StatefulWidget, Widget},
};

#[derive(Debug)]
pub struct State {
    attachments_state: ListState,
}

impl State {
    pub fn new() -> Self {
        Self {
            attachments_state: ListState::default(),
        }
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> Option<super::Action> {
        match event.code {
            KeyCode::Char('q') => Some(super::Action::Quit),
            _ => None,
        }
    }
}

impl Widget for &mut State {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [content, attachments] = area.layout(&Layout::vertical([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ]));

        Widget::render(
            Paragraph::new("Random shit go brrr").block(Block::bordered().title("Content")),
            content,
            buf,
        );

        StatefulWidget::render(
            List::new(["Attachment 1", "Attachment 2"])
                .block(Block::bordered().title("Attachments")),
            attachments,
            buf,
            &mut self.attachments_state,
        );
    }
}
