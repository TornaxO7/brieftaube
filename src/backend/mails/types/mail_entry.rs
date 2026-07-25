use crate::backend::threads::types::ThreadId;

use super::MailId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MailEntry {
    Root(MailId),
    Child { mail: MailId, thread: ThreadId },
}
