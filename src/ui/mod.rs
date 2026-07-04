use crate::backend::Account;
use crossterm::event::KeyEvent;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use std::sync::Arc;

mod command_palette;
mod mailboxes;
mod mails;
// mod composer;
mod mail_viewer;

type MailboxId = String;
type MailId = String;

#[derive(Debug, Clone)]
pub enum Action {
    Quit,

    MailboxList(mailboxes::Action),
    MailList(mails::Action),
    MailViewer(mail_viewer::Action),

    OpenMailList(Option<MailboxId>),
    OpenMailViewer(Option<MailId>),
    OpenMailboxList,
    // OpenComposer,
}

impl From<mails::Action> for Action {
    fn from(action: mails::Action) -> Self {
        Self::MailList(action)
    }
}

impl From<mailboxes::Action> for Action {
    fn from(action: mailboxes::Action) -> Self {
        Self::MailboxList(action)
    }
}

impl From<mail_viewer::Action> for Action {
    fn from(action: mail_viewer::Action) -> Self {
        Self::MailViewer(action)
    }
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Mails,
    Mailboxes,
    // Composer,
    MailViewer,
}

#[derive(Debug)]
pub struct State {
    mode: Mode,

    mails: mails::Mails,
    mailboxes: mailboxes::Mailboxes,
    mail_viewer: mail_viewer::MailViewer,
    // composer: composer::State,
}

impl State {
    pub async fn new(account: Arc<Account>) -> Self {
        Self {
            mode: Mode::Mailboxes,

            mails: mails::Mails::new(account.clone()).await,
            mailboxes: mailboxes::Mailboxes::new(account.clone()).await,
            mail_viewer: mail_viewer::MailViewer::new(account.clone()),
        }
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> Option<super::Action> {
        let sub_actions = match self.mode {
            Mode::Mails => self.mails.handle_event(event),
            Mode::Mailboxes => self.mailboxes.handle_event(event),
            Mode::MailViewer => self.mail_viewer.handle_event(event), // Mode::Composer => self.composer.handle_event(event),
                                                                      // Mode::Pager => self.pager.handle_event(event),
        };

        for action in sub_actions {
            if let Some(app_action) = self.apply_action(action) {
                return Some(app_action);
            }
        }

        None
    }

    fn apply_action(&mut self, action: Action) -> Option<super::Action> {
        match action {
            Action::Quit => return Some(super::Action::Quit),
            Action::OpenMailList(mailbox_id) => {
                self.mode = Mode::Mails;
                self.mails.open_mailbox(mailbox_id);
            }
            Action::OpenMailboxList => self.mode = Mode::Mailboxes,
            Action::MailList(action) => {
                return self
                    .mails
                    .apply_action(action)
                    .and_then(|action| self.apply_action(action));
            }
            Action::MailboxList(action) => {
                return self
                    .mailboxes
                    .apply_action(action)
                    .and_then(|action| self.apply_action(action));
            }
            Action::MailViewer(action) => {
                return self
                    .mail_viewer
                    .apply_action(action)
                    .and_then(|action| self.apply_action(action));
            }
            Action::OpenMailViewer(mail) => {
                self.mode = Mode::MailViewer;
                self.mail_viewer.open_mail(mail);
            }
        }

        None
    }
}

impl Widget for &mut State {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        match self.mode {
            Mode::Mails => self.mails.render(area, buf),
            Mode::Mailboxes => self.mailboxes.render(area, buf),
            Mode::MailViewer => self.mail_viewer.render(area, buf),
            // Mode::Pager => self.pager.render(area, buf),
            // Mode::Composer => self.composer.render(area, buf),
        }
    }
}
