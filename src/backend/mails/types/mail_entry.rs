use super::{MailId, ThreadId};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MailEntry {
    Root(MailId),
    Child { mail: MailId, thread: ThreadId },
}
