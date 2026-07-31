use crate::{
    backend::{
        Backend,
        mailbox::types::{MailboxId, ParentMailboxId},
        mails::types::MailId,
        threads::types::ThreadId,
        types::CollapsedMail,
    },
    mailfs::state::error,
};
use ratatui::widgets::TableState;
use std::rc::Rc;

/// Internal representation of a column
#[derive(Clone, Debug)]
pub struct ColumnState {
    /// The mailbox it represents
    mailbox: Option<MailboxId>,
    /// The entries within the column
    entries: Vec<ColumnStateEntry>,
    /// The table state
    pub state: TableState,
}

impl ColumnState {
    pub fn new(mailbox: ParentMailboxId, entries: Vec<ColumnStateEntry>) -> Self {
        let state = if entries.is_empty() {
            TableState::new()
        } else {
            TableState::new().with_selected(0)
        };

        Self {
            mailbox,
            entries,
            state,
        }
    }

    pub fn selected_entry(&self) -> Option<&ColumnStateEntry> {
        self.state.selected().and_then(|idx| self.entries.get(idx))
    }

    pub fn selected_entry_mut(&mut self) -> Option<&mut ColumnStateEntry> {
        self.state
            .selected()
            .and_then(|idx| self.entries.get_mut(idx))
    }

    pub fn entries(&self) -> &[ColumnStateEntry] {
        &self.entries
    }

    pub fn entries_mut(&mut self) -> &mut Vec<ColumnStateEntry> {
        &mut self.entries
    }

    pub fn mailbox(&self) -> &ParentMailboxId {
        &self.mailbox
    }
}

#[derive(Clone, Debug)]
pub enum ColumnStateEntry {
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

impl ColumnStateEntry {
    /// get the entries of the given mailbox
    pub fn create_entries(
        mailbox: ParentMailboxId,
        backend: Rc<Backend>,
    ) -> Result<Vec<Self>, error::BackendNotReady> {
        let mut entries: Vec<ColumnStateEntry> = Vec::new();

        // get mailboxes
        {
            let mailbox_ids = backend
                .mailboxes_get_or_request_children(mailbox.clone())
                .ok_or(error::BackendNotReady)?;

            for mailbox_id in mailbox_ids {
                entries.push(ColumnStateEntry::Mailbox(mailbox_id));
            }
        }

        // get mails
        if let Some(parent_mailbox_id) = mailbox.as_ref() {
            let collapsed_mails = backend
                .mailbox_get_or_request_root_mails(parent_mailbox_id)
                .ok_or(error::BackendNotReady)?;

            for collapsed_mail in collapsed_mails {
                match collapsed_mail {
                    CollapsedMail::SingleMail(mail_id) => {
                        entries.push(ColumnStateEntry::SingleMail(mail_id))
                    }
                    CollapsedMail::CollapsedThread(mail_id, thread_id) => {
                        entries.push(ColumnStateEntry::CollapsedThread(mail_id, thread_id))
                    }
                }
            }
        }

        Ok(entries)
    }
}
