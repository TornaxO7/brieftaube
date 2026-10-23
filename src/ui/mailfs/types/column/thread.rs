use crate::ui::mailfs::types::{MailEntry, column::Column};
use ratatui::widgets::TableState;

#[derive(Debug)]
pub struct ThreadColumn {
    pub mails: Vec<MailEntry>,
    pub state: TableState,
}

impl ThreadColumn {
    pub fn new(mails: Vec<MailEntry>) -> Self {
        debug_assert!(!mails.is_empty());

        Self {
            mails,
            state: TableState::new().with_selected(Some(0)),
        }
    }

    pub fn set_mails(&mut self, mails: Vec<MailEntry>) {
        self.mails = mails;
    }

    pub fn get_selected_entry<'a>(&'a self) -> &'a MailEntry {
        let idx = self.state.selected().unwrap();
        &self.mails[idx]
    }

    pub fn get_selected_entry_mut<'a>(&'a mut self) -> &'a mut MailEntry {
        let idx = self.state.selected().unwrap();
        &mut self.mails[idx]
    }
}

impl Column for ThreadColumn {
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
