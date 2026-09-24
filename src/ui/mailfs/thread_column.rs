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
            state: TableState::new().with_selected(Some(0)),
        }
    }

    pub fn set_mails(&mut self, thread_mails: color_eyre::Result<Vec<MailDataCore>>) {
        match thread_mails {
            Ok(mails) => self.mails = Loadable::Loaded(mails),
            Err(err) => self.mails = Loadable::Error(err.to_string()),
        }
    }

    pub fn get_selected_entry<'a>(&'a self) -> Option<Loadable<&'a MailDataCore>> {
        let selected_idx = self.state.selected()?;

        Some(self.mails.as_ref().map(|mails| &mails[selected_idx]))
    }
}

impl MailfsColumn for ThreadColumn {
    fn navigate_up(&mut self) {
        self.state.select_previous();
    }

    fn navigate_down(&mut self) {
        self.state.select_next();
    }

    fn navigate_to_bottom(&mut self) {
        self.state.select_last();
    }

    fn navigate_to_top(&mut self) {
        self.state.select_first();
    }
}
