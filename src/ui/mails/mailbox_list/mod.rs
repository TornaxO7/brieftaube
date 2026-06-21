use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, List, ListDirection, ListState, StatefulWidget},
};

pub struct MailboxListWidget<'a> {
    names: &'a [String],
}

impl<'a> MailboxListWidget<'a> {
    pub fn new(names: &'a [String]) -> Self {
        Self { names }
    }
}

impl<'a> StatefulWidget for MailboxListWidget<'a> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        StatefulWidget::render(
            List::new(self.names.iter().map(|name| name.as_str()))
                .block(Block::bordered().title("Mailboxes"))
                .highlight_style(Style::new().blue())
                .direction(ListDirection::TopToBottom),
            area,
            buf,
            state,
        )
    }
}
