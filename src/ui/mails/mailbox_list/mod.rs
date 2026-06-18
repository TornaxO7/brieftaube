use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, List, ListDirection, ListState, StatefulWidget, Widget},
};

#[derive(Debug, Default)]
pub struct State {}

impl State {
    pub fn handle_event(&mut self, event: KeyEvent) -> Option<super::Action> {
        match event.code {
            KeyCode::Char('q') => Some(super::Action::Quit),
            _ => None,
        }
    }
}

impl Widget for &State {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        StatefulWidget::render(
            List::new(["Mailbox 1", "Mailbox 2", "Mailbox 3"])
                .block(Block::bordered().title("Mailboxes"))
                .highlight_style(Style::new().blue())
                .direction(ListDirection::TopToBottom),
            area,
            buf,
            &mut ListState::default().with_selected(Some(0)),
        )
    }
}
