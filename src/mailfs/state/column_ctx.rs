use crate::backend::{mailbox::types::MailboxId, mails::types::MailId, threads::types::ThreadId};
use ratatui::widgets::TableState;

pub struct ColumnCtx {
    pub mailbox: Option<MailboxId>,
    pub entries: Vec<ColumnCtxEntry>,
    pub state: TableState,
}

impl ColumnCtx {
    pub fn new(mailbox: Option<MailboxId>) -> Self {
        Self {
            mailbox,
            entries: Vec::new(),
            state: TableState::default(),
        }
    }
}

pub enum ColumnCtxEntry {
    Mailbox(MailboxId),
    SingleMail(MailId),
    CollapsedThread(ThreadId),
    UncollapsedThread(ThreadId),
}
