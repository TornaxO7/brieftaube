use jmap_client::mailbox::Role;

use crate::ui::MailboxId;

#[derive(Debug, Clone)]
pub struct MailboxData {
    pub id: MailboxId,
    pub name: String,
    pub role: Role,
    pub sort_order: u32,
    pub unread_mails: usize,
}

impl From<jmap_client::mailbox::Mailbox> for MailboxData {
    fn from(mailbox: jmap_client::mailbox::Mailbox) -> Self {
        Self {
            id: mailbox.id().unwrap().to_owned(),
            name: mailbox.name().unwrap().to_owned(),
            role: mailbox.role(),
            sort_order: mailbox.sort_order(),
            unread_mails: mailbox.unread_emails(),
        }
    }
}
