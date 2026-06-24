use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, List, ListDirection, ListState, StatefulWidget},
};

pub struct MailboxListWidget<'a> {
    names: &'a [String],
    block: Option<Block<'a>>,
}

impl<'a> MailboxListWidget<'a> {
    pub fn new(names: &'a [String]) -> Self {
        Self { names, block: None }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> StatefulWidget for MailboxListWidget<'a> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut list = List::new(self.names.iter().map(|name| name.as_str()))
            .highlight_style(Style::new().blue())
            .direction(ListDirection::TopToBottom);

        if let Some(block) = self.block {
            list = list.block(block);
        }

        StatefulWidget::render(list, area, buf, state)
    }
}
