use crate::{
    backend::{Backend, MailId, ThreadId},
    mailfs::state::{ColumnState, ColumnStateEntry},
};

/// Tasks which require data from the backend which may not be there yet.
#[derive(Debug)]
pub enum Update {
    UncollapseThread {
        collapsed_mail_id: MailId,
        thread_id: ThreadId,
    },
}

impl Update {
    pub fn apply(&self, column: &mut ColumnState, backend: &Backend) -> bool {
        match self {
            Self::UncollapseThread {
                collapsed_mail_id,
                thread_id,
            } => uncollapse_thread(column, backend, collapsed_mail_id, thread_id),
        }
    }
}

fn uncollapse_thread(
    column: &mut ColumnState,
    backend: &Backend,
    collapsed_mail_id: &MailId,
    thread_id: &ThreadId,
) -> bool {
    let Some(mut thread_mails) = backend.get_or_request_thread_mails(thread_id) else {
        return true;
    };

    debug_assert!(
        thread_mails.len() >= 2,
        "Uncollapseable threads must have at least 2 mails <.<"
    );

    // according to the jmap specs: The thread saves the mails from oldest to latest,
    // but we want the newest mail to be first: So reverse it
    thread_mails.reverse();

    let new_entries = {
        let (first, rest) = thread_mails.split_first().unwrap();
        let (last, inner) = rest.split_last().unwrap();

        let mut new_entries = vec![ColumnStateEntry::ThreadStart {
            mail_id: first.id.clone(),
            thread_id: thread_id.clone(),
            collapsed_mail_id: collapsed_mail_id.clone(),
        }];

        new_entries.extend(
            inner
                .iter()
                .map(|mail| ColumnStateEntry::ThreadChild(mail.id.clone(), thread_id.clone())),
        );

        new_entries.push(ColumnStateEntry::ThreadEnd(
            last.id.clone(),
            thread_id.clone(),
        ));

        new_entries
    };

    match column
        .entries()
        .iter()
        .position(|entry| matches!(entry, ColumnStateEntry::CollapsedThread(_, entry_thread_id) if entry_thread_id == thread_id))
    {
        Some(thread_idx) => {
            column
                .entries_mut()
                .splice(thread_idx..(thread_idx + 1), new_entries);
            false
        },
        None => true
    }
}
