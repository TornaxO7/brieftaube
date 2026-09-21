use crate::{
    types::{MailDataCore, MailboxData},
    ui::{Loadable, mailfs::MailfsColumn},
};
use ratatui::widgets::TableState;

#[derive(Debug)]
pub struct MailboxColumn {
    pub mailboxes: Loadable<Vec<MailboxData>>,
    pub mails: Loadable<Vec<MailDataCore>>,
    pub state: TableState,
}

impl MailboxColumn {
    pub fn new(mailboxes: Loadable<Vec<MailboxData>>, mails: Loadable<Vec<MailDataCore>>) -> Self {
        Self {
            mailboxes,
            mails,
            state: TableState::new().with_selected(Some(0)),
        }
    }

    pub fn get_selected_entry(&self) {
        todo!();
    }
}

impl MailfsColumn for MailboxColumn {
    fn navigate_up(&mut self) {
        self.state.select_previous();
    }

    fn navigate_down(&mut self) {
        self.state.select_next();
    }

    fn navigate_to_bottom(&mut self) {
        todo!()
    }

    fn navigate_to_top(&mut self) {
        todo!()
    }
}

pub enum MailboxColumnEntry {
    Mailbox(),
    Mail,
    Thread,
}
