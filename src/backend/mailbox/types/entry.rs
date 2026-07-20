use crate::backend::mailbox::types::MailboxId;

#[derive(Debug, Clone)]
pub enum Entry {
    This,
    Child(MailboxId),
}
