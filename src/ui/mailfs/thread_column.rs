use crate::{
    types::MailId,
    ui::{Loadable, mailfs::MailfsColumn},
};
use ratatui::widgets::TableState;

#[derive(Debug)]
pub struct ThreadColumn {
    pub mails: Loadable<Vec<MailId>>,
    pub state: TableState,
}

impl ThreadColumn {
    pub fn loading() -> Self {
        Self {
            mails: Loadable::Loading,
            state: TableState::new(),
        }
    }
}

impl MailfsColumn for ThreadColumn {
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
