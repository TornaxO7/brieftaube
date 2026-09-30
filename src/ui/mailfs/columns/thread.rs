use crate::{types::MailDataCore, ui::mailfs::columns::MailfsColumn};
use ratatui::widgets::TableState;

#[derive(Debug)]
pub struct ThreadColumn {
    pub mails: Vec<MailDataCore>,
    pub state: TableState,
}

impl ThreadColumn {
    pub fn new(mails: Vec<MailDataCore>) -> Self {
        debug_assert!(!mails.is_empty());

        Self {
            mails,
            state: TableState::new().with_selected(Some(0)),
        }
    }

    pub fn set_mails(&mut self, mails: Vec<MailDataCore>) {
        self.mails = mails;
    }

    pub fn get_selected_entry<'a>(&'a self) -> &'a MailDataCore {
        let idx = self.state.selected().unwrap();
        &self.mails[idx]
    }
}

impl MailfsColumn for ThreadColumn {
    fn navigate_up(&mut self) {
        self.state.select_previous();
    }

    fn navigate_down(&mut self) {
        match self.state.selected() {
            Some(current_idx) => {
                if current_idx < self.len() - 1 {
                    self.state.select_next();
                }
            }
            None => self.state.select(Some(0)),
        }
    }

    fn navigate_to_bottom(&mut self) {
        self.state.select(Some(self.len() - 1));
    }

    fn navigate_to_top(&mut self) {
        self.state.select_first();
    }

    fn len(&self) -> usize {
        self.mails.len()
    }
}
