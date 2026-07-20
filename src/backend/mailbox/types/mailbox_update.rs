use super::{MailboxId, SortOrder};
use jmap_client::mailbox::Role;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MailboxUpdate {
    pub id: MailboxId,
    pub name: Option<String>,
    pub role: Option<Role>,
    pub sort_order: Option<SortOrder>,
    pub parent_id: Option<Option<MailboxId>>,
}
