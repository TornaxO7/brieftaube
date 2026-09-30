use crate::{CONFIG, config::UserConfig, ui::mailfs::columns::MailfsColumn};
use ratatui::widgets::TableState;

#[derive(Debug, Clone)]
pub struct UserColumn {
    pub users: Vec<UserConfig>,
    pub state: TableState,
}

impl UserColumn {
    pub fn new() -> Self {
        let config = CONFIG.get().unwrap();

        let users: Vec<UserConfig> = config.users.clone();

        let state = if users.is_empty() {
            TableState::new()
        } else {
            TableState::new().with_selected(Some(0))
        };

        tracing::debug!("{:?}", state.selected());

        Self { users, state }
    }

    pub fn get_selected_entry<'a>(&'a self) -> &'a UserConfig {
        let idx = self.state.selected().expect("User is selected");
        &self.users[idx]
    }
}

impl MailfsColumn for UserColumn {
    fn navigate_up(&mut self) {
        self.state.select_previous();
    }

    fn navigate_down(&mut self) {
        self.state.select(
            self.state
                .selected()
                .map(|idx| (idx + 1).min(self.users.len() - 1)),
        )
    }

    fn navigate_to_bottom(&mut self) {
        self.state.select(Some(self.len() - 1));
    }

    fn navigate_to_top(&mut self) {
        self.state.select(Some(0));
    }

    fn len(&self) -> usize {
        self.users.len()
    }
}
