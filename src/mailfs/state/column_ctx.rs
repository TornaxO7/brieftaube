use crate::backend::mailbox::types::MailboxId;
use ratatui::widgets::TableState;

#[derive(Default)]
pub struct ColumnCtx {
    pub mailbox: Option<MailboxId>,
    pub state: TableState,
}

impl ColumnCtx {
    pub fn new(mailbox: Option<MailboxId>) -> Self {
        Self {
            mailbox,
            state: TableState::default(),
        }
    }
}
