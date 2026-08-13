use crate::{
    backend::{mailbox::types::MailboxId, mails::types::MailId},
    mailfs::model::ColumnStateEntry,
};

#[derive(Hash, PartialEq, Eq)]
pub enum EntryId {
    Mail(MailId),
    Mailbox(MailboxId),
}

impl From<ColumnStateEntry> for EntryId {
    fn from(entry: ColumnStateEntry) -> Self {
        Self::from(&entry)
    }
}

impl From<&ColumnStateEntry> for EntryId {
    fn from(entry: &ColumnStateEntry) -> Self {
        match entry {
            ColumnStateEntry::Mailbox(mailbox_id) => Self::Mailbox(mailbox_id.clone()),
            ColumnStateEntry::SingleMail(mail_id)
            | ColumnStateEntry::CollapsedThread(mail_id, _)
            | ColumnStateEntry::ThreadStart { mail_id, .. }
            | ColumnStateEntry::ThreadChild(mail_id, _)
            | ColumnStateEntry::ThreadEnd(mail_id, _) => Self::Mail(mail_id.clone()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionType {
    Selected,
    Cut,
}
