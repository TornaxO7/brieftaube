use crate::backend::{
    mailbox::types::MailboxId, mails::types::MailId, threads::types::ThreadId, types::CollapsedMail,
};
use ratatui::widgets::TableState;

/// Internal representation of a column
#[derive(Clone, Debug)]
pub struct Column {
    /// The entries within the column
    entries: Vec<ColumnEntry>,
    /// The table state
    pub state: TableState,
}

impl Column {
    pub fn new(entries: Vec<ColumnEntry>) -> Self {
        let state = if entries.is_empty() {
            TableState::new()
        } else {
            TableState::new().with_selected(0)
        };

        Self { entries, state }
    }

    pub fn selected_idx(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn selected_entry(&self) -> Option<&ColumnEntry> {
        self.state.selected().and_then(|idx| self.entries.get(idx))
    }

    // pub fn selected_entry_mut(&mut self) -> Option<&mut ColumnStateEntry> {
    //     self.state
    //         .selected()
    //         .and_then(|idx| self.entries.get_mut(idx))
    // }

    pub fn entries(&self) -> &[ColumnEntry] {
        &self.entries
    }

    pub fn entries_mut(&mut self) -> &mut Vec<ColumnEntry> {
        &mut self.entries
    }
}

#[derive(Clone, Debug)]
pub enum ColumnEntry {
    Mailbox(MailboxId),
    /// Mails which are the only mail in a thread
    SingleMail(MailId),
    /// Root mail of a thread
    CollapsedThread(MailId, ThreadId),

    ThreadStart {
        mail_id: MailId,
        thread_id: ThreadId,
        // not all mails within a thread are in the given mailbox (a response for example)
        // this attribute should store the original mail in the collapsed-state.
        collapsed_mail_id: MailId,
    },
    ThreadChild(MailId, ThreadId),
    ThreadEnd(MailId, ThreadId),
}

impl From<CollapsedMail> for ColumnEntry {
    fn from(collapsed: CollapsedMail) -> Self {
        match collapsed {
            CollapsedMail::SingleMail(id) => Self::SingleMail(id),
            CollapsedMail::CollapsedThread(mail_id, thread_id) => {
                Self::CollapsedThread(mail_id, thread_id)
            }
        }
    }
}
