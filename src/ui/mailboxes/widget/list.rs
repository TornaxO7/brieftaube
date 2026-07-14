use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};
use tui_widget_list::{ListBuilder, ListView};

use crate::{backend::mailboxes::MailboxData, ui::MailboxId};

pub struct List<'a> {
    mailboxes: &'a [MailboxData],
    block: Option<Block<'a>>,
}

impl<'a> List<'a> {
    pub fn new(mailboxes: &'a [MailboxData]) -> Self {
        Self {
            mailboxes,
            block: None,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> StatefulWidget for List<'a> {
    type State = tui_widget_list::ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let builder = ListBuilder::new(|context| {
            const ENTRY_SIZE: u16 = 3;

            let mailbox = &self.mailboxes[context.index];

            let name = &mailbox.name;
            let unread_mails = mailbox.unread_mails;
            let sort_order = mailbox.sort_order;

            let style = if context.is_selected {
                Style::default().green()
            } else {
                Style::default()
            };

            let entry = Entry {
                name,
                unread_mails,
                sort_order,
                style,
            };

            (entry, ENTRY_SIZE)
        });

        let mut list = ListView::new(builder, self.mailboxes.len()).infinite_scrolling(false);
        if let Some(block) = self.block {
            list = list.block(block);
        }

        StatefulWidget::render(list, area, buf, state);
    }
}

struct Entry<'a> {
    name: &'a str,
    unread_mails: usize,
    sort_order: u32,

    style: Style,
}

impl<'a> Widget for Entry<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = Block::bordered().style(self.style);
        let block_area = block.inner(area);

        let [left, right] =
            Layout::horizontal([Constraint::Fill(0), Constraint::Fill(0)]).areas(block_area);

        Widget::render(block, area, buf);
        Widget::render(
            Paragraph::new(format!("{} {}", self.sort_order, self.name)),
            left,
            buf,
        );
        Widget::render(
            Paragraph::new(format!("{}", self.unread_mails)).right_aligned(),
            right,
            buf,
        );
    }
}
