use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, List, ListDirection, ListState, StatefulWidget, Widget},
};

#[derive(Debug, Default)]
pub struct State {}

impl Widget for &State {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        StatefulWidget::render(
            List::new(["Mailbox 1", "Mailbox 2", "Mailbox 3"])
                .block(Block::bordered().title("Mailboxes"))
                .highlight_symbol(">> ")
                .direction(ListDirection::TopToBottom),
            area,
            buf,
            &mut ListState::default().with_selected(Some(0)),
        )
    }
}
