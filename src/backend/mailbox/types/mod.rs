mod children;
mod mailbox_data;
mod mailbox_new;
mod mailbox_update;
mod mailbox_validate;

pub use mailbox_data::MailboxData;
pub use mailbox_new::MailboxNew;
pub use mailbox_update::MailboxUpdate;
pub use mailbox_validate::MailboxValidate;

pub type SortOrder = u32;
pub type ParentMailboxId = Option<MailboxId>;
pub const TOP_PARENT_MAILBOX_ID: ParentMailboxId = None;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct MailboxId(pub String);

impl MailboxId {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<MailboxId> for String {
    fn from(id: MailboxId) -> Self {
        id.0
    }
}

impl From<&MailboxId> for String {
    fn from(id: &MailboxId) -> Self {
        id.0.clone()
    }
}
