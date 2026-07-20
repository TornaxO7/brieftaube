use super::{MailboxId, SortOrder};
use jmap_client::mailbox::Role;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MailboxNew {
    pub id: Option<MailboxId>,
    pub name: String,
    pub role: Option<Role>,
    pub sort_order: Option<SortOrder>,
    pub parent_id: Option<MailboxId>,
}
