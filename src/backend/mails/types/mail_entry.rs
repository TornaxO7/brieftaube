use super::{MailData, MailId, ThreadId};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MailEntryType {
    Root,
    Child,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MailEntry {
    pub id: MailId,
    pub thread: ThreadId,
    pub ty: MailEntryType,
}

impl MailEntry {
    pub fn new_root(mail: &MailData) -> Self {
        Self {
            id: mail.id.clone(),
            thread: mail.thread_id.clone(),
            ty: MailEntryType::Root,
        }
    }

    pub fn new_thread(mail: &MailData) -> Self {
        Self {
            id: mail.id.clone(),
            thread: mail.thread_id.clone(),
            ty: MailEntryType::Child,
        }
    }
}
