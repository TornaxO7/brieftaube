use super::{MailboxId, MailboxNew, MailboxUpdate, SortOrder};
use jmap_client::mailbox::Role;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MailboxValidate {
    pub name: Option<String>,
    pub role: Option<Role>,
    pub sort_order: Option<SortOrder>,
    pub parent_id: Option<Option<MailboxId>>,
}

impl From<MailboxUpdate> for MailboxValidate {
    fn from(mailbox: MailboxUpdate) -> Self {
        Self {
            name: mailbox.name,
            role: mailbox.role,
            sort_order: mailbox.sort_order,
            parent_id: mailbox.parent_id,
        }
    }
}

impl From<&MailboxUpdate> for MailboxValidate {
    fn from(mailbox: &MailboxUpdate) -> Self {
        Self::from(mailbox.clone())
    }
}

impl From<MailboxNew> for MailboxValidate {
    fn from(mailbox: MailboxNew) -> Self {
        Self {
            name: Some(mailbox.name),
            role: mailbox.role,
            sort_order: mailbox.sort_order,
            parent_id: Some(mailbox.parent_id),
        }
    }
}

impl From<&MailboxNew> for MailboxValidate {
    fn from(mailbox: &MailboxNew) -> Self {
        Self::from(mailbox.clone())
    }
}
