use crate::{
    backend::{ParentMailboxId, mailbox::types::MailboxId, mails::types::MailId},
    mailfs::model::ColumnEntry,
};

#[derive(Hash, PartialEq, Eq)]
pub enum SelectedEntry {
    Mail(MailId),
    Mailbox(MailboxId),
}

impl From<ColumnEntry> for SelectedEntry {
    fn from(entry: ColumnEntry) -> Self {
        Self::from(&entry)
    }
}

impl From<&ColumnEntry> for SelectedEntry {
    // TODO: If selected a collapsed thread => Select all mails from the collapsed thread
    fn from(entry: &ColumnEntry) -> Self {
        match entry {
            ColumnEntry::Mailbox(mailbox_id) => Self::Mailbox(mailbox_id.clone()),
            ColumnEntry::SingleMail(mail_id)
            | ColumnEntry::CollapsedThread(mail_id, _)
            | ColumnEntry::ThreadStart { mail_id, .. }
            | ColumnEntry::ThreadChild(mail_id, _)
            | ColumnEntry::ThreadEnd(mail_id, _) => Self::Mail(mail_id.clone()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Selection {
    pub mailbox: ParentMailboxId,
    pub ty: SelectionType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionType {
    Selected,
    Cut,
}
