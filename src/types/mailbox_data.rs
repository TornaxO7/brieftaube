use super::MailboxId;
use crate::types::{MailboxNew, MailboxUpdate};
use jmap_client::mailbox::{Mailbox, MailboxRights, Property, Role};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailboxData {
    pub id: MailboxId,
    pub name: String,
    pub role: Role,
    pub sort_order: u32,
    pub unread_mails: usize,
    pub parent_id: Option<MailboxId>,
    pub total_threads: usize,
    pub my_rights: MailboxRights,
}

impl MailboxData {
    pub const PROPERTIES: [Property; 8] = [
        Property::Id,
        Property::Name,
        Property::Role,
        Property::SortOrder,
        Property::UnreadEmails,
        Property::ParentId,
        Property::TotalThreads,
        Property::MyRights,
    ];

    pub fn from_new(new: MailboxNew, mut server_mailbox: Mailbox) -> Self {
        Self {
            id: server_mailbox.take_id().into(),
            name: new.name,
            role: new.role,
            sort_order: new.sort_order,
            unread_mails: server_mailbox.unread_emails(),
            parent_id: new.parent_id,
            total_threads: server_mailbox.total_threads(),
            my_rights: server_mailbox.my_rights().cloned().unwrap(),
        }
    }

    pub fn from_get_request(mailbox: Mailbox) -> Self {
        Self {
            id: MailboxId(mailbox.id().unwrap().to_owned()),
            name: mailbox.name().unwrap().to_owned(),
            role: mailbox.role(),
            sort_order: mailbox.sort_order(),
            unread_mails: mailbox.unread_emails(),
            parent_id: mailbox.parent_id().map(|id| MailboxId(id.to_string())),
            total_threads: mailbox.total_threads(),
            my_rights: mailbox.my_rights().cloned().unwrap(),
        }
    }

    pub fn update(&mut self, update: MailboxUpdate) {
        if let Some(name) = update.name {
            self.name = name;
        }

        if let Some(role) = update.role {
            self.role = role;
        }

        if let Some(sort_order) = update.sort_order {
            self.sort_order = sort_order;
        }

        if let Some(parent_id) = update.parent_id {
            self.parent_id = parent_id;
        }
    }
}
