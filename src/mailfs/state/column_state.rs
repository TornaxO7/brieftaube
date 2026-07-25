use std::rc::Rc;

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
use throbber_widgets_tui::ThrobberState;

#[derive(Clone)]
pub enum ColumnState {
    /// The state for loading columns
    Loading { state: ThrobberState },
    /// The state for columns with available data
    Loaded {
        /// The mailbox it represents
        mailbox: Option<MailboxId>,
        entries: Vec<ColumnStateEntry>,
        state: TableState,
    },
}

impl ColumnState {
    pub fn loading() -> Self {
        Self::Loading {
            state: ThrobberState::default(),
        }
    }

    pub fn loaded(parent: ParentMailboxId, entries: Vec<ColumnStateEntry>) -> Self {
        let state = if entries.is_empty() {
            TableState::new()
        } else {
            TableState::new().with_selected(1)
        };

        Self::Loaded {
            mailbox: parent,
            entries,
            state,
        }
    }
}

#[derive(Clone)]
pub enum ColumnStateEntry {
    Mailbox(MailboxId),
    SingleMail(MailId),
    CollapsedThread(ThreadId),
    UncollapsedThread(ThreadId),
}

impl ColumnStateEntry {
    pub fn create_entries(
        parent: ParentMailboxId,
        backend: Rc<Backend>,
    ) -> Result<Vec<Self>, error::BackendNotReady> {
        let mut entries: Vec<ColumnStateEntry> = Vec::new();

        // get mailboxes
        {
            let mailbox_ids = backend
                .get_child_mailboxes(parent.clone())
                .ok_or(error::BackendNotReady)?;
            for mailbox_id in mailbox_ids {
                entries.push(ColumnStateEntry::Mailbox(mailbox_id));
            }
        }

        // get mails
        if let Some(parent_mailbox_id) = parent.as_ref() {
            let collapsed_mails = backend
                .get_collapsed_mails(parent_mailbox_id)
                .ok_or(error::BackendNotReady)?;

            for collapsed_mail in collapsed_mails {
                match collapsed_mail {
                    CollapsedMail::SingleMail(mail_id) => {
                        entries.push(ColumnStateEntry::SingleMail(mail_id))
                    }
                    CollapsedMail::CollapsedThread(thread_id) => {
                        entries.push(ColumnStateEntry::CollapsedThread(thread_id))
                    }
                }
            }
        }

        Ok(entries)
    }
}
