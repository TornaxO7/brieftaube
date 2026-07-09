mod action;
mod composer;
mod keybindmanager;
mod log_viewer;
mod mail_viewer;
mod mailboxes;
mod mails;
mod palette;

use crate::backend::Account;
use action::Action;
use crossterm::event::KeyEvent;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use std::sync::Arc;

type MailboxId = String;
type MailId = String;

#[derive(Debug, Clone, Copy)]
enum Mode {
    Mails,
    Mailboxes,
    Composer,
    MailViewer,
    LogViewer,
}

pub struct Ui {
    mode: Mode,

    mails: mails::Mails,
    mailboxes: mailboxes::Mailboxes,
    mail_viewer: mail_viewer::MailViewer,
    composer: composer::Composer,
    log_viewer: log_viewer::LogViewer,
}

impl Ui {
    pub async fn new(account: Arc<Account>) -> Self {
        Self {
            mode: Mode::Mailboxes,

            mails: mails::Mails::new(account.clone()).await,
            mailboxes: mailboxes::Mailboxes::new(account.clone()).await,
            mail_viewer: mail_viewer::MailViewer::new(account.clone()),
            composer: composer::Composer::new(account.clone()),
            log_viewer: log_viewer::LogViewer::new(),
        }
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> Option<super::Action> {
        let sub_actions = match self.mode {
            Mode::Mails => self.mails.handle_event(event),
            Mode::Mailboxes => self.mailboxes.handle_event(event),
            Mode::MailViewer => self.mail_viewer.handle_event(event),
            Mode::Composer => self.composer.handle_event(event),
            Mode::LogViewer => self.log_viewer.handle_event(event),
        };

        for action in sub_actions {
            if let Some(mode_action) = self.apply_action(action) {
                return Some(mode_action);
            }
        }

        None
    }

    fn apply_action(&mut self, action: Action) -> Option<super::Action> {
        match action {
            Action::Quit => return Some(super::Action::Quit),
            Action::OpenMailboxList => self.mode = Mode::Mailboxes,
            Action::OpenMailList(mailbox_id) => {
                self.mode = Mode::Mails;
                self.mails.open_mailbox(mailbox_id);
            }
            Action::OpenMailViewer(mail) => {
                self.mode = Mode::MailViewer;
                self.mail_viewer.open_mail(mail);
            }
            Action::OpenComposer => {
                self.mode = Mode::Composer;
            }
            Action::OpenLogs(callback) => {
                self.mode = Mode::LogViewer;
                self.log_viewer.set_callback(callback);
            }

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
            Action::Composer(action) => {
                return self
                    .composer
                    .apply_action(action)
                    .and_then(|action| self.apply_action(action));
            }
            Action::LogViewer(action) => {
                return self
                    .log_viewer
                    .apply_action(action)
                    .and_then(|action| self.apply_action(action));
            }
        }

        None
    }
}

impl Widget for &mut Ui {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        match self.mode {
            Mode::Mails => self.mails.render(area, buf),
            Mode::Mailboxes => self.mailboxes.render(area, buf),
            Mode::MailViewer => self.mail_viewer.render(area, buf),
            Mode::Composer => self.composer.render(area, buf),
            Mode::LogViewer => self.log_viewer.render(area, buf),
        }
    }
}
