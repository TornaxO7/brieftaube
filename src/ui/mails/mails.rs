use ratatui::{
    layout::Rect,
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
            List::new(["Mail 1", "Mail 2", "Mail 3"])
                .block(Block::bordered().title("Mails"))
                .highlight_symbol(">> ")
                .direction(ListDirection::TopToBottom),
            area,
            buf,
            &mut ListState::default().with_selected(Some(0)),
        )
    }
}
