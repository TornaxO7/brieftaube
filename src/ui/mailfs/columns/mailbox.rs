use crate::{
    types::{MailDataCore, MailboxData},
    ui::{Loadable, mailfs::columns::MailfsColumn},
};
use ratatui::widgets::TableState;

#[derive(Debug)]
pub struct MailboxColumn {
    pub child_mailboxes: Vec<MailboxData>,
    pub mails: Vec<Loadable<MailDataCore>>,

    pub mailbox_state: TableState,
    pub mail_state: TableState,
}

impl MailboxColumn {
    pub fn new(
        mut child_mailboxes: Vec<MailboxData>,
        total_threads: usize,
        init_mails: Vec<MailDataCore>,
    ) -> Self {
        debug_assert!(init_mails.len() <= total_threads);

        let (mailbox_state, mail_state) = {
            let mut mailbox_state = TableState::new();
            let mut mail_state = TableState::new();

            if !child_mailboxes.is_empty() {
                mailbox_state = mailbox_state.with_selected(Some(0));
            } else if !init_mails.is_empty() {
                mail_state = mail_state.with_selected(Some(0));
            }

            (mailbox_state, mail_state)
        };

        let mut mails = vec![Loadable::NotLoaded; total_threads];
        mails.splice(
            0..init_mails.len(),
            init_mails.into_iter().map(|mail| Loadable::Loaded(mail)),
        );

        child_mailboxes.sort_by(|a, b| {
            if a.sort_order != b.sort_order {
                a.sort_order.cmp(&b.sort_order)
            } else {
                a.name.cmp(&b.name)
            }
        });

        Self {
            child_mailboxes,
            mails,

            mailbox_state,
            mail_state,
        }
    }

    pub fn get_selected_entry<'a>(&'a self) -> Option<MailboxColumnEntry<'a>> {
        match (self.mailbox_state.selected(), self.mail_state.selected()) {
            (Some(idx), None) => self
                .child_mailboxes
                .get(idx)
                .map(|mailbox| MailboxColumnEntry::Mailbox(mailbox)),
            (None, Some(idx)) => self
                .mails
                .get(idx)
                .map(|mail| MailboxColumnEntry::RootMail(mail)),
            (Some(_), Some(_)) => unreachable!(),
            (None, None) => None,
        }
    }

    // pub fn set_mails(
    //     &mut self,
    //     window: QueryWindow,
    //     result: color_eyre::Result<Vec<MailDataCore>>,
    // ) {
    //     let window_range = window.as_range();

    //     match result {
    //         Ok(mails) => {
    //             self.mails.splice(
    //                 window_range,
    //                 mails.into_iter().map(|mail| Loadable::Loaded(mail)),
    //             );
    //         }
    //         Err(err) => {
    //             self.mails.splice(
    //                 window_range,
    //                 std::iter::repeat(Loadable::Error(err.to_string())),
    //             );
    //         }
    //     };
    // }
}

impl MailfsColumn for MailboxColumn {
    fn navigate_up(&mut self, offset: u16) {
        match (self.mailbox_state.selected(), self.mail_state.selected()) {
            (Some(current_idx), None) => {
                self.mailbox_state
                    .select(Some(current_idx.saturating_sub(offset as usize)));
            }
            (None, Some(current_idx)) => {
                let overflows_to_mailbox = current_idx < offset as usize;

                if overflows_to_mailbox {
                    if self.child_mailboxes.is_empty() {
                        self.mail_state.select(Some(0));
                        return;
                    }

                    let mailbox_idx = self
                        .child_mailboxes
                        .len()
                        .saturating_sub(offset as usize - current_idx);
                    self.mailbox_state.select(Some(mailbox_idx));
                    self.mail_state.select(None);
                } else {
                    let next_idx = current_idx - offset as usize;
                    self.mail_state.select(Some(next_idx));
                }
            }
            (None, None) => {}
            (Some(_), Some(_)) => unreachable!(),
        }
    }

    fn navigate_down(&mut self, offset: u16) {
        match (self.mailbox_state.selected(), self.mail_state.selected()) {
            (Some(current_idx), None) => {
                let overflows_to_mails =
                    (self.child_mailboxes.len() - 1 - current_idx) < offset as usize;

                if overflows_to_mails {
                    if self.mails.is_empty() {
                        self.mailbox_state
                            .select(Some(self.child_mailboxes.len() - 1));
                        return;
                    }

                    let next_mail_idx =
                        offset as usize - (self.child_mailboxes.len() - 1 - current_idx);
                    self.mail_state.select(Some(next_mail_idx));
                    self.mailbox_state.select(None);
                } else {
                    let next_idx = current_idx + offset as usize;
                    self.mailbox_state.select(Some(next_idx));
                }
            }
            (None, Some(current_idx)) => {
                let next_idx = (current_idx + offset as usize).min(self.mails.len() - 1);
                self.mail_state.select(Some(next_idx));
            }
            (None, None) => {}
            (Some(_), Some(_)) => unreachable!(),
        }
    }

    fn navigate_to_bottom(&mut self) {
        if !self.mails.is_empty() {
            self.mail_state.select_last();
            self.mailbox_state.select(None);
            return;
        }

        if !self.child_mailboxes.is_empty() {
            self.mail_state.select(None);
            self.mailbox_state.select_last();
        }

        debug_assert!(
            !(self.mailbox_state.selected().is_some() && self.mail_state.selected().is_some())
        );
    }

    fn navigate_to_top(&mut self) {
        if !self.child_mailboxes.is_empty() {
            self.mail_state.select(None);
            self.mailbox_state.select_first();
            return;
        }

        if !self.mails.is_empty() {
            self.mail_state.select_first();
            self.mailbox_state.select(None);
        }

        debug_assert!(
            !(self.mailbox_state.selected().is_some() && self.mail_state.selected().is_some())
        );
    }

    fn len(&self) -> usize {
        self.child_mailboxes.len() + self.mails.len()
    }
}

#[derive(Debug)]
pub enum MailboxColumnEntry<'a> {
    Mailbox(&'a MailboxData),
    RootMail(&'a Loadable<MailDataCore>),
}
