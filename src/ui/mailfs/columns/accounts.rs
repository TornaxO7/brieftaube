use crate::{types::AccountData, ui::mailfs::columns::MailfsColumn};
use ratatui::widgets::TableState;

pub struct AccountsColumn {
    pub accounts: Vec<AccountData>,
    pub state: TableState,
}

impl AccountsColumn {
    pub fn new(accounts: Vec<AccountData>) -> Self {
        debug_assert!(!accounts.is_empty());
        Self {
            accounts,
            state: TableState::new().with_selected(Some(0)),
        }
    }

    pub fn set_accounts(&mut self, accounts: Vec<AccountData>) {
        debug_assert!(!accounts.is_empty());
        self.accounts = accounts;
    }

    pub fn get_selected_entry<'a>(&'a self) -> &'a AccountData {
        let idx = self.state.selected().expect("Must select an entry");
        &self.accounts[idx]
    }
}

impl MailfsColumn for AccountsColumn {
    fn navigate_up(&mut self, offset: u16) {
        let idx = self
            .state
            .selected()
            .unwrap_or(0)
            .saturating_sub(offset as usize);
        self.state.select(Some(idx));
    }

    fn navigate_down(&mut self, offset: u16) {
        self.state.select(
            self.state
                .selected()
                .map(|idx| (idx + offset as usize).min(self.len() - 1)),
        )
    }

    fn navigate_to_bottom(&mut self) {
        self.state.select(Some(self.len()));
    }

    fn navigate_to_top(&mut self) {
        self.state.select(Some(0));
    }

    fn len(&self) -> usize {
        self.accounts.len()
    }
}
