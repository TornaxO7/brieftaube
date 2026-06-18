use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Widget,
};
use tracing::debug;

mod mail_list;
mod mailbox_list;
mod preview;
mod statusbar;

#[derive(Debug, Default, PartialEq, Eq)]
enum Focus {
    MailboxList,
    #[default]
    Mails,
    Preview,
    CommandPalette,
}

#[derive(Debug, Default)]
pub struct State {
    focus: Focus,

    mailbox_list: mailbox_list::State,
    mail_list: mail_list::State,
    preview: preview::State,

    statusbar: statusbar::State,
    command_palette: super::command_palette::State,
}

impl State {
    pub fn handle_event(&mut self, event: KeyEvent) -> Option<super::Action> {
        match event.code {
            KeyCode::Char(':') => {
                self.focus = Focus::CommandPalette;
                return None;
            }
            KeyCode::Char('q') => {
                return Some(super::Action::Quit);
            }
            _ => {}
        }

        match self.focus {
            Focus::MailboxList => self.mailbox_list.handle_event(event),
            Focus::Mails => self.mail_list.handle_event(event),
            Focus::Preview => self.preview.handle_event(event),
            Focus::CommandPalette => self.command_palette.handle_event(event),
        };

        None
    }
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
        self.mail_list.render(mail_list, buf);
        self.preview.render(preview, buf);
        self.statusbar.render(statusbar, buf);

        if self.focus == Focus::CommandPalette {
            self.command_palette.render(area, buf);
        }
    }
}
