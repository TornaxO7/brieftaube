use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, List, ListDirection, ListState, StatefulWidget},
};

pub struct MailListWidget<'a> {
    titles: &'a [String],
}

impl<'a> MailListWidget<'a> {
    pub fn new(titles: &'a [String]) -> Self {
        Self { titles }
    }
}

impl<'a> StatefulWidget for MailListWidget<'a> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        StatefulWidget::render(
            List::new(self.titles.iter().map(|name| name.as_str()))
                .block(Block::bordered().title("Mails"))
                .highlight_style(Style::new().blue())
                .direction(ListDirection::TopToBottom),
            area,
            buf,
            state,
        )
    }
}
