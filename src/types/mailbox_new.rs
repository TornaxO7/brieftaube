use crate::types::ParentMailboxId;
use jmap_client::mailbox::Role;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MailboxNew {
    pub name: String,
    pub role: Role,
    pub sort_order: u32,
    pub parent_id: ParentMailboxId,
}
