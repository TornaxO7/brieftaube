use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    widgets::Widget,
};

mod mailbox_list;
mod mails;
mod preview;
mod statusbar;

#[derive(Debug, Default)]
pub struct State {
    mailbox_list: mailbox_list::State,
    mails: mails::State,
    preview: preview::State,
    statusbar: statusbar::State,
}

impl Widget for &State {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let [content, statusbar] = area.layout(&Layout::vertical([
            Constraint::Fill(0),
            Constraint::Length(3),
        ]));

        let [mail_boxes, mail_list, preview] = content.layout(&Layout::horizontal([
            Constraint::Percentage(10),
            Constraint::Percentage(50),
            Constraint::Percentage(40),
        ]));

        self.mailbox_list.render(mail_boxes, buf);
        self.mails.render(mail_list, buf);
        self.preview.render(preview, buf);
        self.statusbar.render(statusbar, buf);
    }
}
