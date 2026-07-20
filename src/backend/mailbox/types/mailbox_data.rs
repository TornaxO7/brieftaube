use super::{MailboxId, SortOrder};
use jmap_client::mailbox::{Property, Role};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MailboxData {
    pub id: MailboxId,
    pub name: String,
    pub role: Role,
    pub sort_order: SortOrder,
    pub unread_mails: usize,
    pub parent_id: Option<MailboxId>,
}

impl MailboxData {
    pub const PROPERTIES: [Property; 6] = [
        Property::Id,
        Property::Name,
        Property::Role,
        Property::SortOrder,
        Property::UnreadEmails,
        Property::ParentId,
    ];
}

impl From<jmap_client::mailbox::Mailbox> for MailboxData {
    fn from(mailbox: jmap_client::mailbox::Mailbox) -> Self {
        Self::from(&mailbox)
    }
}

impl From<&jmap_client::mailbox::Mailbox> for MailboxData {
    fn from(mailbox: &jmap_client::mailbox::Mailbox) -> Self {
        Self {
            id: mailbox.id().unwrap().to_owned(),
            name: mailbox.name().unwrap().to_owned(),
            role: mailbox.role(),
            sort_order: mailbox.sort_order(),
            unread_mails: mailbox.unread_emails(),
            parent_id: mailbox.parent_id().map(|id| id.to_string()),
        }
    }
}
