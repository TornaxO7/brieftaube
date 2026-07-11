use chrono::DateTime;
use jmap_client::email::Email;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, HorizontalAlignment, Layout, Rect},
    style::Style,
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};
use tui_widget_list::{ListBuilder, ListState, ListView};

pub struct MailListWidget<'a> {
    root_mails: &'a [Email],
    block: Option<Block<'a>>,
}

impl<'a> MailListWidget<'a> {
    pub fn new(root_mails: &'a [Email]) -> Self {
        Self {
            root_mails,
            block: None,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> StatefulWidget for MailListWidget<'a> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let entry_builder = ListBuilder::new(|context| {
            const ENTRY_SIZE: u16 = 4;

            let mail = &self.root_mails[context.index];
            let entry = {
                let subject = mail.subject().unwrap();
                let from = {
                    let addresses = mail.from().unwrap();

                    let mut iter = addresses.iter();
                    let mut s = format!("{}", iter.next().unwrap().email());

                    for addr in iter {
                        s.push_str(&format!(", {}", addr.email()));
                    }

                    s
                };

                let date = DateTime::from_timestamp_millis(mail.received_at().unwrap())
                    .unwrap()
                    .format("%b %e, %H:%M")
                    .to_string();

                let style = if context.is_selected {
                    Style::new().green()
                } else {
                    Style::default()
                };

                MailListEntry {
                    subject,
                    from,
                    date,

                    style,
                }
            };

            (entry, ENTRY_SIZE)
        });

        let mut list =
            ListView::new(entry_builder, self.root_mails.len()).infinite_scrolling(false);
        if let Some(block) = self.block {
            list = list.block(block);
        }

        StatefulWidget::render(list, area, buf, state)
    }
}

struct MailListEntry<'a> {
    from: String,
    subject: &'a str,
    date: String,

    style: Style,
}

impl<'a> Widget for MailListEntry<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let entry_block = Block::bordered().style(self.style);
        let entry_area = entry_block.inner(area);

        let [top, bottom] =
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(entry_area);

        let [top_left, top_right] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(top);

        Widget::render(entry_block, area, buf);
        Widget::render(Paragraph::new(self.from), top_left, buf);
        Widget::render(Paragraph::new(self.subject), bottom, buf);
        Widget::render(
            Paragraph::new(self.date).alignment(HorizontalAlignment::Right),
            top_right,
            buf,
        );
    }
}
