use crate::{
    types::{MailId, MailboxId},
    ui::{Loadable, mailfs::MailfsColumn},
};
use ratatui::widgets::TableState;

#[derive(Debug)]
pub struct MailboxColumn {
    pub mailboxes: Loadable<Vec<MailboxId>>,
    pub mails: Loadable<Vec<MailId>>,
    pub state: TableState,
}

impl MailboxColumn {
    pub fn loading() -> Self {
        Self {
            mailboxes: Loadable::Loading,
            mails: Loadable::Loading,
            state: TableState::new().with_selected(Some(0)),
        }
    }
}

impl MailfsColumn for MailboxColumn {
    fn navigate_up(&mut self) -> Vec<crate::ui::Message> {
        todo!()
    }

    fn navigate_down(&mut self) -> Vec<crate::ui::Message> {
        todo!()
    }

    fn navigate_to_bottom(&mut self) -> Vec<crate::ui::Message> {
        todo!()
    }

    fn navigate_to_top(&mut self) -> Vec<crate::ui::Message> {
        todo!()
    }
}
