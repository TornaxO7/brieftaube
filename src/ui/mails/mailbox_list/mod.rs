use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, List, ListDirection, ListState, StatefulWidget, Widget},
};

#[derive(Debug, Default)]
pub struct State {
    is_focussed: bool,
}

impl State {
    pub fn handle_event(&mut self, event: KeyEvent) -> Option<super::Action> {
        match event.code {
            KeyCode::Char('q') => Some(super::Action::Quit),
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
            let mut block = Block::bordered().title("Mailboxes");

            block = if self.is_focussed {
                block.border_style(Style::new().green())
            } else {
                block
            };

            block
        };

        StatefulWidget::render(
            List::new(["Mailbox 1", "Mailbox 2", "Mailbox 3"])
                .block(block)
                .highlight_style(Style::new().blue())
                .direction(ListDirection::TopToBottom),
            area,
            buf,
            &mut ListState::default().with_selected(Some(0)),
        )
    }
}
