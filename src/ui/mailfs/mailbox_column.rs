use crate::{
    datasource::types::QueryWindow,
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

    pub fn mails_len(&self) -> usize {
        self.mails.loaded().map(|mails| mails.len()).unwrap_or(1)
    }

    pub fn set_mailboxes(&mut self, children: color_eyre::Result<Vec<MailboxData>>) {
        match children {
            Ok(mailboxes) => {
                self.mailboxes = Loadable::Loaded(mailboxes);
            }
            Err(err) => {
                self.mailboxes = Loadable::Error(err.to_string());
            }
        };

        self.init_selection();
    }

    pub fn set_mails(
        &mut self,
        window: QueryWindow,
        result: color_eyre::Result<(Vec<MailDataCore>, Option<usize>)>,
    ) {
        let window_range = window.as_range();

        match result {
            Ok((mails, total_mails)) => {
                let end = match total_mails {
                    Some(max) => window_range.end.min(max),
                    None => window_range.end,
                };

                match &mut self.mails {
                    Loadable::NotLoaded | Loadable::Loading | Loadable::Error(_) => {
                        let mut mail_entries = vec![Loadable::NotLoaded; end];

                        for (offset, new_mail) in mails.into_iter().enumerate() {
                            let idx = window.start as usize + offset;
                            mail_entries[idx] = Loadable::Loaded(new_mail);
                        }

                        self.mails = Loadable::Loaded(mail_entries);
                    }
                    Loadable::Loaded(current_mails) => {
                        if current_mails.len() < end {
                            current_mails.resize(end, Loadable::NotLoaded);
                        }

                        for (offset, new_mail) in mails.into_iter().enumerate() {
                            let idx = window.start as usize + offset;
                            current_mails[idx] = Loadable::Loaded(new_mail);
                        }
                    }
                }
            }
            Err(err) => match &mut self.mails {
                Loadable::NotLoaded | Loadable::Loading | Loadable::Error(_) => {
                    let mut mail_entries = vec![Loadable::NotLoaded; window_range.end];

                    for idx in window_range {
                        mail_entries[idx] = Loadable::Error(err.to_string());
                    }

                    self.mails = Loadable::Loaded(mail_entries);
                }
                Loadable::Loaded(mails) => {
                    if mails.len() < window_range.end {
                        mails.resize(window_range.end, Loadable::NotLoaded);
                    }
                    for idx in window_range {
                        mails[idx] = Loadable::Error(err.to_string());
                    }
                }
            },
        };

        self.init_selection();
    }
}

impl MailboxColumn {
    fn init_selection(&mut self) {
        let none_selected =
            self.mailbox_state.selected().is_none() && self.mail_state.selected().is_none();

        if none_selected {
            match &self.mailboxes {
                Loadable::NotLoaded | Loadable::Loading | Loadable::Error(_) => {
                    self.mailbox_state.select(Some(0));
                    return;
                }
                Loadable::Loaded(mailboxes) => {
                    if !mailboxes.is_empty() {
                        self.mailbox_state.select(Some(0));
                        return;
                    }
                }
            };

            match &self.mails {
                Loadable::NotLoaded | Loadable::Loading | Loadable::Error(_) => {
                    self.mail_state.select(Some(0));
                    return;
                }
                Loadable::Loaded(mails) => {
                    if !mails.is_empty() {
                        self.mail_state.select(Some(0));
                    }
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
                    match &self.mails {
                        Loadable::NotLoaded | Loadable::Loading | Loadable::Error(_) => {
                            self.mailbox_state.select(None);
                            self.mail_state.select(Some(0));
                        }
                        Loadable::Loaded(mails) => {
                            if !mails.is_empty() {
                                self.mailbox_state.select(None);
                                self.mail_state.select(Some(0));
                            }
                        }
                    }
                } else {
                    self.mailbox_state.select_next();
                }
            }
            (None, Some(idx)) => {
                let last_mail_idx = self.mails_len() - 1;

                if idx < last_mail_idx {
                    self.mail_state.select_next();
                }
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
