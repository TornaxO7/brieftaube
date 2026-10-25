use super::MailboxId;
use crate::types::MailboxProperty;
use jmap_client::mailbox::{MailboxRights, Role};
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedMailbox {
    pub id: MailboxId,
    pub name: OnceLock<String>,
    pub role: OnceLock<Role>,
    pub sort_order: OnceLock<u32>,
    pub unread_mails: OnceLock<usize>,
    pub parent_id: OnceLock<Option<MailboxId>>,
    pub total_threads: OnceLock<usize>,
    pub my_rights: OnceLock<MailboxRights>,
}

impl CachedMailbox {
    pub const PROPERTIES: [MailboxProperty; 8] = [
        MailboxProperty::Id,
        MailboxProperty::Name,
        MailboxProperty::Role,
        MailboxProperty::SortOrder,
        MailboxProperty::UnreadEmails,
        MailboxProperty::ParentId,
        MailboxProperty::TotalThreads,
        MailboxProperty::MyRights,
    ];

    pub fn merge(&mut self, other: CachedMailbox) {
        self.id = other.id;

        if let Some(name) = other.name.into_inner() {
            self.name = OnceLock::from(name);
        }

        if let Some(role) = other.role.into_inner() {
            self.role = OnceLock::from(role);
        }

        if let Some(sort_order) = other.sort_order.into_inner() {
            self.sort_order = OnceLock::from(sort_order);
        }

        if let Some(unread_mails) = other.unread_mails.into_inner() {
            self.unread_mails = OnceLock::from(unread_mails);
        }

        if let Some(parent_id) = other.parent_id.into_inner() {
            self.parent_id = OnceLock::from(parent_id);
        }

        if let Some(total_threads) = other.total_threads.into_inner() {
            self.total_threads = OnceLock::from(total_threads);
        }

        if let Some(my_rights) = other.my_rights.into_inner() {
            self.my_rights = OnceLock::from(my_rights);
        }
    }
}

impl From<jmap_client::mailbox::Mailbox> for CachedMailbox {
    fn from(mailbox: jmap_client::mailbox::Mailbox) -> Self {
        Self {
            id: mailbox.id().unwrap().into(),
            name: mailbox
                .name()
                .map_or_default(|name| OnceLock::from(name.to_string())),
            role: mailbox.role().map_or_default(OnceLock::from),
            sort_order: mailbox.sort_order().map_or_default(OnceLock::from),
            unread_mails: mailbox.unread_emails().map_or_default(OnceLock::from),
            parent_id: mailbox
                .parent_id()
                .map_or_default(|id| OnceLock::from(id.map(MailboxId::from))),
            total_threads: mailbox.total_threads().map_or_default(OnceLock::from),
            my_rights: mailbox
                .my_rights()
                .map_or_default(|rights| OnceLock::from(rights.clone())),
        }
    }
}
