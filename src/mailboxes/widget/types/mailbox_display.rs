use crate::{backend::mailbox::types::SortOrder, mailboxes::state::SelectionType};

#[derive(Debug)]
pub enum MailboxDisplay {
    This,
    Entry {
        selection_type: Option<SelectionType>,
        sort_order: SortOrder,
        name: String,
        unread_mails: usize,
    },
}
