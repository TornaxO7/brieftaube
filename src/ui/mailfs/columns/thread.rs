use crate::ui::mailfs::{columns::MailfsColumn, types::MailColumnEntry};
use ratatui::widgets::TableState;

#[derive(Debug)]
pub struct ThreadColumn {
    pub mails: Vec<MailColumnEntry>,
    pub state: TableState,
}

impl ThreadColumn {
    pub fn new(mails: Vec<MailColumnEntry>) -> Self {
        debug_assert!(!mails.is_empty());

        Self {
            mails,
            state: TableState::new().with_selected(Some(0)),
        }
    }

    pub fn set_mails(&mut self, mails: Vec<MailColumnEntry>) {
        self.mails = mails;
    }

    pub fn get_selected_entry<'a>(&'a self) -> &'a MailColumnEntry {
        let idx = self.state.selected().unwrap();
        &self.mails[idx]
    }
}

impl MailfsColumn for ThreadColumn {
    fn navigate_up(&mut self, offset: u16) {
        let idx = self
            .state
            .selected()
            .unwrap_or(0)
            .saturating_sub(offset as usize);

        self.state.select(Some(idx));
    }

    fn navigate_down(&mut self, offset: u16) {
        let idx = self.state.selected().unwrap_or(0) + offset as usize;
        self.state.select(Some(idx.min(self.len() - 1)));
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
