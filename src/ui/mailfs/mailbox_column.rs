use crate::{
    types::{MailId, MailboxData},
    ui::{Loadable, mailfs::MailfsColumn},
};
use ratatui::widgets::TableState;

#[derive(Debug)]
pub struct MailboxColumn {
    pub mailboxes: Loadable<Vec<MailboxData>>,
    pub mails: Loadable<Vec<MailId>>,
    pub state: TableState,
}

impl MailboxColumn {
    pub fn new_root() -> Self {
        Self {
            mailboxes: Loadable::Loading,
            mails: Loadable::Loaded(vec![]),
            state: TableState::new().with_selected(Some(0)),
        }
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
