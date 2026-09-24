use crate::{
    types::MailDataCore,
    ui::{Loadable, mailfs::MailfsColumn},
};
use ratatui::widgets::TableState;

#[derive(Debug)]
pub struct ThreadColumn {
    pub mails: Loadable<Vec<MailDataCore>>,
    pub state: TableState,
}

impl ThreadColumn {
    pub fn new(mails: Loadable<Vec<MailDataCore>>) -> Self {
        Self {
            mails,
            state: TableState::new(),
        }
    }

    pub fn set_mails(&mut self, thread_mails: color_eyre::Result<Vec<MailDataCore>>) {
        match thread_mails {
            Ok(mails) => self.mails = Loadable::Loaded(mails),
            Err(err) => self.mails = Loadable::Error(err.to_string()),
        }
    }
}

impl MailfsColumn for ThreadColumn {
    fn navigate_up(&mut self) {
        todo!()
    }

    fn navigate_down(&mut self) {
        todo!()
    }

    fn navigate_to_bottom(&mut self) {
        todo!()
    }

    fn navigate_to_top(&mut self) {
        todo!()
    }
}
