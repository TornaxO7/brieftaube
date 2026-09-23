use crate::{
    types::{MailDataCore, MailboxData},
    ui::{Loadable, mailfs::MailfsColumn},
};
use ratatui::widgets::TableState;

#[derive(Debug)]
pub struct MailboxColumn {
    pub mailboxes: Loadable<Vec<MailboxData>>,
    pub mails: Loadable<Vec<Loadable<MailDataCore>>>,

    pub mailbox_state: TableState,
    pub mail_state: TableState,
}

impl MailboxColumn {
    pub fn new(
        mailboxes: Loadable<Vec<MailboxData>>,
        mails: Loadable<Vec<Loadable<MailDataCore>>>,
    ) -> Self {
        Self {
            mailboxes,
            mails,

            mailbox_state: TableState::new().with_selected(None),
            mail_state: TableState::new().with_selected(None),
        }
    }

    pub fn get_selected_entry<'a>(&'a self) -> Option<Loadable<MailboxColumnEntry<'a>>> {
        match (self.mailbox_state.selected(), self.mail_state.selected()) {
            (Some(idx), None) => Some(
                self.mailboxes
                    .as_ref()
                    .map(|mailboxes| MailboxColumnEntry::Mailbox(&mailboxes[idx])),
            ),
            (None, Some(idx)) => Some(self.mails.as_ref().and_then(|mails| {
                mails[idx]
                    .as_ref()
                    .map(|mail| MailboxColumnEntry::RootMail(mail))
            })),
            (Some(_), Some(_)) => unreachable!(),
            (None, None) => None,
        }
    }

    pub fn mailboxes_len(&self) -> usize {
        self.mailboxes
            .loaded()
            .map(|mailboxes| mailboxes.len())
            .unwrap_or(1)
    }

    pub fn set_mailboxes(&mut self, children: color_eyre::Result<Vec<MailboxData>>) {
        match children {
            Ok(mailboxes) => {
                let none_selected =
                    self.mailbox_state.selected().is_none() && self.mail_state.selected().is_none();
                if none_selected && !mailboxes.is_empty() {
                    self.mailbox_state.select(Some(0));
                }

                self.mailboxes = Loadable::Loaded(mailboxes);
            }
            Err(err) => {
                self.mailboxes = Loadable::Error(err.to_string());

                let none_selected =
                    self.mailbox_state.selected().is_none() && self.mail_state.selected().is_none();

                if none_selected {
                    self.mailbox_state.select(Some(0));
                }
            }
        }
    }
}

impl MailfsColumn for MailboxColumn {
    fn navigate_up(&mut self) {
        match (self.mailbox_state.selected(), self.mail_state.selected()) {
            (Some(_), None) => {
                self.mailbox_state.select_previous();
            }
            (None, Some(idx)) => {
                if idx == 0 {
                    let last_mailbox_idx = self.mailboxes_len() - 1;
                    self.mailbox_state.select(Some(last_mailbox_idx));

                    self.mail_state.select(None);
                } else {
                    self.mail_state.select_previous();
                }
            }
            (None, None) => {}
            (Some(_), Some(_)) => unreachable!(),
        }
    }

    fn navigate_down(&mut self) {
        match (self.mailbox_state.selected(), self.mail_state.selected()) {
            (Some(idx), None) => {
                let last_mailbox_idx = self.mailboxes_len() - 1;

                if last_mailbox_idx == idx {
                    self.mailbox_state.select(None);
                    self.mail_state.select(Some(0));
                } else {
                    self.mailbox_state.select_next();
                }
            }
            (None, Some(_)) => {
                self.mail_state.select_next();
            }
            (None, None) => {}
            (Some(_), Some(_)) => unreachable!(),
        }
    }

    fn navigate_to_bottom(&mut self) {
        todo!("get the oldest mail from the mailbox")
    }

    fn navigate_to_top(&mut self) {
        self.mail_state.select(None);

        match &self.mailboxes {
            Loadable::NotLoaded | Loadable::Loading | Loadable::Error(_) => {
                self.mailbox_state.select(Some(0));
            }
            Loadable::Loaded(mailboxes) => {
                if mailboxes.is_empty() {
                    self.mailbox_state.select(None);
                } else {
                    self.mailbox_state.select(Some(0));
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum MailboxColumnEntry<'a> {
    Mailbox(&'a MailboxData),
    RootMail(&'a MailDataCore),
}
