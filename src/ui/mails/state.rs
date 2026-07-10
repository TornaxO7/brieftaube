use crate::{backend, ui::MailboxId};
use jmap_client::email::Email;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;

pub struct State {}

impl State {
    pub fn new(account: Arc<backend::Account>) -> Self {
        Self {
            account,

            selected_mailbox_id: None,

            rx,
            tx: Arc::new(tx),

            mails: HashMap::new(),

            list_state: tui_widget_list::ListState::default(),
        }
    }
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State").finish()
    }
}
